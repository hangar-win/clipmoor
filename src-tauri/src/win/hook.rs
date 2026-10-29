use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU8, AtomicU32, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LWIN, VK_MENU, VK_RWIN};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::PanelCtl;
use crate::event;
use crate::keys::{HookAction, HookState, KeyEvent, PanelKey, Release, Shortcuts};
use crate::placement::{self, Align, Method, Rect};
use crate::settings::PanelPosition;
use crate::win::caret::{self, CaretWorker};
use crate::win::{clipboard, panel_window, paste, text_input};

const CARET_BUDGET: Duration = Duration::from_millis(30);
const WM_WATCH: u32 = WM_APP + 1;
const WM_UNWATCH: u32 = WM_APP + 2;

static STATE: OnceLock<Mutex<HookState>> = OnceLock::new();
static JOBS: OnceLock<Mutex<Sender<Job>>> = OnceLock::new();
static HOOK_THREAD: AtomicU32 = AtomicU32::new(0);
static PANEL: AtomicIsize = AtomicIsize::new(0);
static MODS: AtomicU8 = AtomicU8::new(0);
static HOLD: AtomicU8 = AtomicU8::new(0);
static UNLOCKING: AtomicBool = AtomicBool::new(false);
static POSITION: Mutex<PanelPosition> = Mutex::new(PanelPosition::Caret);
static AT_MOUSE: AtomicBool = AtomicBool::new(false);
static ALIGN: Mutex<Align> = Mutex::new(Align::Start);

thread_local! {
    static WATCH: Cell<Option<(HHOOK, HWINEVENTHOOK)>> = const { Cell::new(None) };
}

enum Job {
    Open(Instant),
    Key(PanelKey),
    Paste,
    Click(POINT),
    Foreground(isize),
    OpenSettings,
}

pub struct HookHandle;

impl HookHandle {
    pub fn set_released(&self, reason: Release, on: bool) {
        state().set_released(reason, on);
    }

    pub fn set_takeover(&self, on: bool) {
        state().set_takeover(on);
    }

    pub fn set_shortcuts(&self, s: Shortcuts) {
        state().set_shortcuts(s);
    }

    pub fn hold_open(&self, reason: Hold, on: bool) {
        hold_open(reason, on);
    }

    pub fn set_position(&self, p: PanelPosition) {
        *POSITION.lock().unwrap_or_else(|e| e.into_inner()) = p;
    }

    pub fn mods(&self) -> (bool, bool) {
        let m = MODS.load(Ordering::Relaxed);
        (m & 1 != 0, m & 2 != 0)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Hold {
    Prompt = 1,
    KeepOpen = 2,
}

pub fn hold_open(reason: Hold, on: bool) {
    if on {
        HOLD.fetch_or(reason as u8, Ordering::Relaxed);
    } else {
        HOLD.fetch_and(!(reason as u8), Ordering::Relaxed);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Outside {
    Ignore,
    Release,
    Close,
}

fn outside(hold: u8) -> Outside {
    if hold & Hold::Prompt as u8 != 0 {
        Outside::Ignore
    } else if hold & Hold::KeepOpen as u8 != 0 {
        Outside::Release
    } else {
        Outside::Close
    }
}

fn state() -> MutexGuard<'static, HookState> {
    STATE.get().expect("hook not spawned").lock().unwrap_or_else(|e| e.into_inner())
}

pub fn reset_keys() {
    if let Some(s) = STATE.get() {
        s.lock().unwrap_or_else(|e| e.into_inner()).reset_keys();
    }
}

pub fn set_align(align: Align) {
    *ALIGN.lock().unwrap_or_else(|e| e.into_inner()) = align;
}

pub fn capture_keys(on: bool) {
    if let Some(s) = STATE.get() {
        s.lock().unwrap_or_else(|e| e.into_inner()).set_panel_open(on && panel_window::is_visible(panel()));
    }
}

pub fn set_recording(on: bool) {
    if let Some(s) = STATE.get() {
        s.lock().unwrap_or_else(|e| e.into_inner()).set_recording(on);
    }
}

fn system_down(vk: u32) -> bool {
    unsafe { GetAsyncKeyState(vk as i32) as u16 & 0x8000 != 0 }
}

pub fn spawn(app: AppHandle, takeover: bool, shortcuts: Shortcuts) -> HookHandle {
    STATE.get_or_init(|| {
        let mut s = HookState::with_probe(takeover, system_down);
        s.set_shortcuts(shortcuts);
        Mutex::new(s)
    });
    PANEL.store(app.state::<PanelCtl>().hwnd, Ordering::Relaxed);
    let (tx, rx) = mpsc::channel();
    let _ = JOBS.set(Mutex::new(tx));
    std::thread::spawn(move || {
        let caret = CaretWorker::spawn();
        for job in rx {
            run(&app, job, &caret);
        }
    });
    std::thread::spawn(|| unsafe {
        HOOK_THREAD.store(GetCurrentThreadId(), Ordering::SeqCst);
        let module = GetModuleHandleW(None).ok().map(Into::into);
        if let Err(e) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), module, 0) {
            eprintln!("keyboard hook failed: {e}");
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            match msg.message {
                WM_WATCH => watch(),
                WM_UNWATCH if !panel_window::is_visible(panel()) => unwatch(),
                _ => {}
            }
        }
    });
    HookHandle
}

fn panel() -> HWND {
    HWND(PANEL.load(Ordering::Relaxed) as _)
}

fn key_event(wparam: WPARAM, kb: &KBDLLHOOKSTRUCT) -> KeyEvent {
    let down = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let ours = kb.flags.contains(LLKHF_INJECTED) && kb.dwExtraInfo == paste::OUR_EXTRA_INFO;
    KeyEvent { vk: kb.vkCode, down, ours }
}

fn post(job: Job) {
    if let Some(tx) = JOBS.get() {
        let _ = tx.lock().unwrap_or_else(|e| e.into_inner()).send(job);
    }
}

fn post_hook(msg: u32) {
    let _ = unsafe { PostThreadMessageW(HOOK_THREAD.load(Ordering::SeqCst), msg, WPARAM(0), LPARAM(0)) };
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let ev = key_event(wparam, kb);
        let (action, keys) = {
            let mut s = state();
            let action = s.decide(ev);
            let (ctrl, shift) = s.mods();
            MODS.store(ctrl as u8 | (shift as u8) << 1, Ordering::Relaxed);
            (action, *s.key_state())
        };
        match action {
            HookAction::Pass | HookAction::PassWithStartMask => {}
            HookAction::Swallow => return LRESULT(1),
            HookAction::Translate => {
                if let Some(s) = text_input::translate(kb.vkCode, kb.scanCode, &keys) {
                    post(Job::Key(PanelKey::Text { s }));
                }
                return LRESULT(1);
            }
            HookAction::Open => {
                text_input::reset();
                post(Job::Open(Instant::now()));
                return LRESULT(1);
            }
            HookAction::PasteText => {
                post(Job::Paste);
                return LRESULT(1);
            }
            HookAction::Forward(k) => {
                text_input::reset();
                post(Job::Key(k));
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn run(app: &AppHandle, job: Job, caret: &CaretWorker) {
    match job {
        Job::Open(at) => {
            if [VK_LWIN, VK_RWIN, VK_MENU].iter().any(|vk| system_down(vk.0 as u32)) {
                paste::send_start_mask();
            }
            if !panel_window::is_visible(panel()) && crate::app::panel_locked() {
                unlock_then_open(app);
                return;
            }
            if let Err(e) = toggle(app, at, caret) {
                eprintln!("panel show failed: {e}");
            }
        }
        Job::OpenSettings => {
            if outside(HOLD.load(Ordering::Relaxed)) == Outside::Close {
                close(app);
            } else {
                state().set_panel_open(false);
            }
            crate::win::tray::open_settings(app);
        }
        Job::Key(k) => {
            let _ = app.emit_to("panel", event::PANEL_KEY, k);
        }
        Job::Paste => {
            let app = app.clone();
            std::thread::spawn(move || {
                if let Some(s) = clipboard::text_for_search() {
                    let _ = app.emit_to("panel", event::PANEL_KEY, PanelKey::Text { s });
                }
            });
        }
        Job::Click(pt) => {
            let act = outside(HOLD.load(Ordering::Relaxed));
            if act != Outside::Ignore && panel_window::is_visible(panel()) && !contains(panel_window::rect(panel()), pt) {
                if act == Outside::Close { close(app) } else { state().set_panel_open(false) }
            }
        }
        Job::Foreground(fg) => {
            let act = outside(HOLD.load(Ordering::Relaxed));
            let ctl = app.state::<PanelCtl>();
            let fg = HWND(fg as _);
            if act != Outside::Ignore && panel_window::is_visible(panel()) && foreign_foreground(fg, ctl.panel(), ctl.target()) {
                if act == Outside::Close {
                    close(app);
                } else {
                    ctl.set_target(fg);
                    state().set_panel_open(false);
                }
            }
        }
    }
}

fn unlock_then_open(app: &AppHandle) {
    if UNLOCKING.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let unlocked = crate::app::unlock(&app);
        UNLOCKING.store(false, Ordering::SeqCst);
        match unlocked {
            Ok(()) => post(Job::Open(Instant::now())),
            Err(e) => eprintln!("unlock failed: {e}"),
        }
    });
}

fn button_down(msg: u32) -> bool {
    matches!(msg, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN)
}

fn contains(r: RECT, pt: POINT) -> bool {
    pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom
}

fn foreign_foreground(fg: HWND, panel: HWND, target: HWND) -> bool {
    !fg.0.is_null() && fg != panel && fg != target
}

fn watch() {
    if WATCH.get().is_some() {
        return;
    }
    let module = unsafe { GetModuleHandleW(None) }.ok().map(Into::into);
    let mouse = match unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0) } {
        Ok(h) => h,
        Err(e) => {
            eprintln!("mouse hook failed: {e}");
            return;
        }
    };
    let fg = unsafe { SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, None, Some(foreground_proc), 0, 0, WINEVENT_OUTOFCONTEXT) };
    WATCH.set(Some((mouse, fg)));
}

fn unwatch() {
    if let Some((mouse, fg)) = WATCH.take() {
        unsafe {
            let _ = UnhookWindowsHookEx(mouse);
            if !fg.is_invalid() {
                let _ = UnhookWinEvent(fg);
            }
        }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && button_down(wparam.0 as u32) {
        post(Job::Click(unsafe { (*(lparam.0 as *const MSLLHOOKSTRUCT)).pt }));
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

unsafe extern "system" fn foreground_proc(_: HWINEVENTHOOK, _: u32, hwnd: HWND, _: i32, _: i32, _: u32, _: u32) {
    post(Job::Foreground(hwnd.0 as isize));
}

pub fn open_settings() {
    post(Job::OpenSettings);
}

pub fn open_at_mouse() {
    AT_MOUSE.store(true, Ordering::Relaxed);
    post(Job::Open(Instant::now()));
}

fn toggle(app: &AppHandle, at: Instant, caret: &CaretWorker) -> windows::core::Result<()> {
    let mode = if AT_MOUSE.swap(false, Ordering::Relaxed) { PanelPosition::Mouse } else { *POSITION.lock().unwrap_or_else(|e| e.into_inner()) };
    let ctl = app.state::<PanelCtl>();
    let hwnd = ctl.panel();
    if panel_window::is_visible(hwnd) {
        if state().panel_open() {
            close(app);
        } else {
            ctl.set_target(panel_window::foreground());
            state().set_panel_open(true);
            let _ = app.emit_to("panel", event::PANEL_SHOWN, ());
        }
        return Ok(());
    }
    *ctl.opened_at.lock().unwrap() = Some(at);
    let fg = panel_window::foreground();
    ctl.set_target(fg);
    state().set_panel_open(true);
    let (x, y) = position(hwnd, fg, caret, mode);
    if let Err(e) = panel_window::show_at(hwnd, x, y) {
        state().set_panel_open(false);
        return Err(e);
    }
    post_hook(WM_WATCH);
    let _ = app.emit_to("panel", event::PANEL_SHOWN, ());
    Ok(())
}

fn position(panel: HWND, fg: HWND, caret: &CaretWorker, mode: PanelPosition) -> (i32, i32) {
    let started = Instant::now();
    let r = panel_window::rect(fg);
    let target = Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
    let screen = caret::work_area(target);
    let dpi = unsafe { GetDpiForWindow(fg) }.max(96);
    let (focus, anchor) = match mode {
        PanelPosition::Caret => {
            let (focus, found) = caret.lookup(fg, CARET_BUDGET);
            (focus, placement::anchor(found, caret::cursor(), target, dpi, screen))
        }
        PanelPosition::Mouse => (fg, caret::cursor().map_or((Method::Center, screen), |c| (Method::Mouse, c))),
        PanelPosition::Center => (fg, (Method::Center, screen)),
    };
    let work = caret::work_area(anchor.1);
    let size = panel_size(panel);
    let align = *ALIGN.lock().unwrap_or_else(|e| e.into_inner());
    let (x, y) = placement::spot(anchor, size, work, align);
    panel_window::move_to(panel, x, y);
    let resized = panel_size(panel);
    let at = if resized == size { (x, y) } else { placement::spot(anchor, resized, work, align) };
    if cfg!(debug_assertions) {
        let (m, a) = anchor;
        eprintln!(
            "caret: method={} rect=({},{},{},{}) fg_class={} focus_class={} dpi={dpi} ms={:.1} -> panel=({},{})",
            format!("{m:?}").to_lowercase(), a.left, a.top, a.right, a.bottom, caret::class(fg), caret::class(focus), started.elapsed().as_secs_f64() * 1000.0, at.0, at.1
        );
    }
    at
}

fn panel_size(panel: HWND) -> (i32, i32) {
    let r = panel_window::rect(panel);
    (r.right - r.left, r.bottom - r.top)
}

pub fn close(app: &AppHandle) {
    let ctl = app.state::<PanelCtl>();
    panel_window::give_back_focus(ctl.panel(), ctl.target());
    panel_window::hide(ctl.panel());
    crate::commands::preview_item(app.clone(), None);
    {
        let mut s = state();
        s.set_panel_open(false);
        s.set_released(Release::Dialog, false);
    }
    post_hook(WM_UNWATCH);
    let _ = app.emit_to("panel", event::PANEL_HIDDEN, ());
    crate::app::after_use(app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::win::paste::OUR_EXTRA_INFO;

    fn kb(vk: u32, flags: KBDLLHOOKSTRUCT_FLAGS, extra: usize) -> KBDLLHOOKSTRUCT {
        KBDLLHOOKSTRUCT { vkCode: vk, flags, dwExtraInfo: extra, ..Default::default() }
    }

    #[test]
    fn key_event_marks_only_our_injected_input() {
        let e = key_event(WPARAM(WM_KEYDOWN as usize), &kb(0x56, LLKHF_INJECTED, OUR_EXTRA_INFO));
        assert_eq!((e.vk, e.down, e.ours), (0x56, true, true));
        assert!(!key_event(WPARAM(WM_KEYDOWN as usize), &kb(0x56, LLKHF_INJECTED, 0)).ours);
        assert!(!key_event(WPARAM(WM_KEYDOWN as usize), &kb(0x56, KBDLLHOOKSTRUCT_FLAGS(0), OUR_EXTRA_INFO)).ours);
    }

    const PANEL_RECT: RECT = RECT { left: 100, top: 100, right: 548, bottom: 740 };

    #[test]
    fn hold_reasons_combine_and_release_independently() {
        let now = || outside(HOLD.load(Ordering::Relaxed));
        hold_open(Hold::KeepOpen, true);
        hold_open(Hold::Prompt, true);
        let both = now();
        hold_open(Hold::Prompt, false);
        let kept = now();
        hold_open(Hold::Prompt, true);
        hold_open(Hold::KeepOpen, false);
        let prompting = now();
        hold_open(Hold::Prompt, false);
        assert_eq!([both, kept, prompting, now()], [Outside::Ignore, Outside::Release, Outside::Ignore, Outside::Close]);
    }

    #[test]
    fn only_button_downs_count_as_clicks() {
        for msg in [WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN] {
            assert!(button_down(msg));
        }
        for msg in [WM_MOUSEMOVE, WM_LBUTTONUP, WM_MOUSEWHEEL] {
            assert!(!button_down(msg));
        }
    }

    #[test]
    fn click_outside_panel_rect_is_detected() {
        assert!(!contains(PANEL_RECT, POINT { x: 50, y: 300 }));
        assert!(!contains(PANEL_RECT, POINT { x: 548, y: 300 }));
        assert!(contains(PANEL_RECT, POINT { x: 100, y: 100 }));
        assert!(contains(PANEL_RECT, POINT { x: 547, y: 739 }));
    }

    #[test]
    fn foreground_to_other_app_closes() {
        let (fg, panel, target) = (HWND(3 as _), HWND(1 as _), HWND(2 as _));
        assert!(foreign_foreground(fg, panel, target));
        assert!(!foreign_foreground(panel, panel, target));
        assert!(!foreign_foreground(target, panel, target));
        assert!(!foreign_foreground(HWND(std::ptr::null_mut()), panel, target));
    }

    #[test]
    fn key_event_treats_syskeys_like_keys() {
        assert!(key_event(WPARAM(WM_SYSKEYDOWN as usize), &kb(0x41, KBDLLHOOKSTRUCT_FLAGS(0), 0)).down);
        assert!(!key_event(WPARAM(WM_SYSKEYUP as usize), &kb(0x41, LLKHF_UP, 0)).down);
        assert!(!key_event(WPARAM(WM_KEYUP as usize), &kb(0x41, LLKHF_UP, 0)).down);
    }
}
