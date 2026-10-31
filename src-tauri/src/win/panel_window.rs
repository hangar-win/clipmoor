use std::sync::Mutex;

use crate::placement::{self, Align, Rect};
use crate::win::caret;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn make_noactivate(hwnd: HWND) -> windows::core::Result<()> {
    let want = (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW).0 as isize;
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | want);
        if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & want != want {
            return Err(windows::core::Error::from_thread());
        }
    }
    Ok(())
}

pub fn pin_in_place(hwnd: HWND) -> windows::core::Result<()> {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(hwnd, GWL_STYLE, style & !((WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX).0 as isize));
        SetWindowSubclass(hwnd, Some(stay_put), 1, 0).ok()
    }
}

unsafe extern "system" fn stay_put(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _: usize, _: usize) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGING {
        let pos = unsafe { &mut *(lparam.0 as *mut WINDOWPOS) };
        if pos.flags & SWP_SHOWWINDOW == SET_WINDOW_POS_FLAGS(0) && is_visible(hwnd) {
            pos.flags |= SWP_NOMOVE;
        }
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

fn set_noactivate(hwnd: HWND, on: bool) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let bit = WS_EX_NOACTIVATE.0 as isize;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, if on { ex | bit } else { ex & !bit });
    }
}

pub fn take_focus(hwnd: HWND) -> bool {
    set_noactivate(hwnd, false);
    let ok = unsafe {
        let me = GetCurrentThreadId();
        let fg = GetWindowThreadProcessId(GetForegroundWindow(), None);
        let attached = fg != 0 && fg != me && AttachThreadInput(me, fg, true).as_bool();
        let _ = SetForegroundWindow(hwnd);
        if attached {
            let _ = AttachThreadInput(me, fg, false);
        }
        GetForegroundWindow() == hwnd
    };
    if !ok {
        set_noactivate(hwnd, true);
    }
    ok
}

pub fn give_back_focus(hwnd: HWND, target: HWND) {
    set_noactivate(hwnd, true);
    unsafe {
        if GetForegroundWindow() == hwnd {
            let _ = SetForegroundWindow(target);
        }
    }
}

pub fn show_at(hwnd: HWND, x: i32, y: i32) -> windows::core::Result<()> {
    unsafe { SetWindowPos(hwnd, Some(HWND_TOPMOST), x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW) }
}

const PREVIEW_MARGIN: f64 = 40.0;

pub fn show_centered(preview: HWND, panel: HWND, (w, h): (f64, f64), min: f64) {
    let scale = unsafe { GetDpiForWindow(panel) }.max(96) as f64 / 96.0;
    let px = |v: f64| (v * scale).round() as i32;
    let p = rect(panel);
    let panel = Rect { left: p.left, top: p.top, right: p.right, bottom: p.bottom };
    let r = placement::centered((px(w), px(h)), px(min), px(PREVIEW_MARGIN), panel, caret::work_area(panel));
    let want = RECT { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
    for _ in 0..2 {
        unsafe {
            let _ = SetWindowPos(preview, Some(HWND_TOPMOST), r.left, r.top, r.right - r.left, r.bottom - r.top, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        }
        if rect(preview) == want {
            break;
        }
    }
}

pub fn resize(hwnd: HWND, size: (f64, f64), align: Align) {
    let (w, h) = placement::scaled(size, unsafe { GetDpiForWindow(hwnd) });
    if !is_visible(hwnd) {
        unsafe {
            let _ = SetWindowPos(hwnd, None, 0, 0, w, h, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        }
        return;
    }
    let r = rect(hwnd);
    let (x, y) = placement::resized(Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom }, (w, h), align);
    unsafe {
        let _ = SetWindowPos(hwnd, None, x, y, w, h, SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING);
    }
    release(hwnd);
}

pub fn move_to(hwnd: HWND, x: i32, y: i32) {
    unsafe {
        let _ = SetWindowPos(hwnd, None, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

pub fn drag_to(hwnd: HWND, x: i32, y: i32) {
    unsafe {
        let _ = SetWindowPos(hwnd, None, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING);
    }
}

static GRAB: Mutex<(i32, i32)> = Mutex::new((0, 0));

pub fn grab(hwnd: HWND) {
    if let Some(c) = caret::cursor() {
        let r = rect(hwnd);
        *GRAB.lock().unwrap_or_else(|e| e.into_inner()) = (c.left - r.left, c.top - r.top);
    }
}

pub fn drag(hwnd: HWND) {
    if let Some(c) = caret::cursor() {
        let (dx, dy) = *GRAB.lock().unwrap_or_else(|e| e.into_inner());
        drag_to(hwnd, c.left - dx, c.top - dy);
    }
}

pub fn release(hwnd: HWND) {
    let r = rect(hwnd);
    let at = Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
    let (x, y) = placement::clamp((r.left, r.top), (r.right - r.left, r.bottom - r.top), caret::work_area(at));
    if (x, y) != (r.left, r.top) {
        drag_to(hwnd, x, y);
    }
}

pub fn hide(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn is_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd) }.as_bool()
}

pub fn foreground() -> HWND {
    unsafe { GetForegroundWindow() }
}

pub fn restore_foreground(target: HWND) {
    unsafe {
        if GetForegroundWindow() != target {
            let _ = SetForegroundWindow(target);
        }
    }
}

pub fn rect(hwnd: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe {
        let _ = GetWindowRect(hwnd, &mut r);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetActiveWindow, GetFocus};
    use windows::core::w;

    #[test]
    fn make_noactivate_adds_noactivate_and_toolwindow() {
        unsafe {
            let hwnd = CreateWindowExW(WINDOW_EX_STYLE(0), w!("STATIC"), w!(""), WS_POPUP, 0, 0, 10, 10, None, None, None, None).unwrap();
            make_noactivate(hwnd).unwrap();
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            DestroyWindow(hwnd).unwrap();
            assert_ne!(ex & WS_EX_NOACTIVATE.0, 0);
            assert_ne!(ex & WS_EX_TOOLWINDOW.0, 0);
        }
    }

    #[test]
    fn pinned_window_moves_only_while_hidden_when_shown_or_dragged() {
        unsafe {
            let hwnd = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, w!("STATIC"), w!(""), WS_POPUP | WS_SYSMENU | WS_MAXIMIZEBOX, -30000, -30000, 4, 4, None, None, None, None).unwrap();
            pin_in_place(hwnd).unwrap();
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            move_to(hwnd, -30010, -30010);
            let hidden = rect(hwnd);
            show_at(hwnd, -30020, -30020).unwrap();
            let shown = rect(hwnd);
            move_to(hwnd, -30030, -30030);
            let moved = rect(hwnd);
            drag_to(hwnd, -30040, -30040);
            let dragged = rect(hwnd);
            let active = GetActiveWindow();
            DestroyWindow(hwnd).unwrap();
            assert_eq!(style & (WS_SYSMENU | WS_MAXIMIZEBOX).0, 0);
            assert_eq!((hidden.left, hidden.top), (-30010, -30010));
            assert_eq!((shown.left, shown.top), (-30020, -30020));
            assert_eq!((moved.left, moved.top), (-30020, -30020));
            assert_eq!((dragged.left, dragged.top), (-30040, -30040));
            assert_ne!(active, hwnd);
        }
    }

    #[test]
    fn resize_sets_the_dpi_scaled_size_and_keeps_a_hidden_window_in_place() {
        unsafe {
            let hwnd = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, w!("STATIC"), w!(""), WS_POPUP, -30000, -30000, 4, 4, None, None, None, None).unwrap();
            pin_in_place(hwnd).unwrap();
            resize(hwnd, (920.0, 260.0), Align::Center);
            let r = rect(hwnd);
            let want = placement::scaled((920.0, 260.0), GetDpiForWindow(hwnd));
            let active = GetActiveWindow();
            DestroyWindow(hwnd).unwrap();
            assert_eq!((r.left, r.top, r.right - r.left, r.bottom - r.top), (-30000, -30000, want.0, want.1));
            assert_ne!(active, hwnd);
        }
    }

    #[test]
    fn grab_and_drag_keep_the_panel_under_a_still_cursor_and_release_clamps_it_on_screen() {
        unsafe {
            let hwnd = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, w!("STATIC"), w!(""), WS_POPUP, -30000, -30000, 40, 30, None, None, None, None).unwrap();
            pin_in_place(hwnd).unwrap();
            grab(hwnd);
            drag(hwnd);
            let held = rect(hwnd);
            release(hwnd);
            let r = rect(hwnd);
            DestroyWindow(hwnd).unwrap();
            let work = caret::work_area(Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom });
            assert_eq!((held.left, held.top), (-30000, -30000));
            assert!(r.left >= work.left && r.top >= work.top && r.right <= work.right && r.bottom <= work.bottom, "{r:?} outside {work:?}");
        }
    }

    #[test]
    #[ignore = "briefly takes the real foreground"]
    fn noactivate_panel_takes_focus_and_gives_it_back() {
        unsafe {
            let prev = GetForegroundWindow();
            let hwnd = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST, w!("STATIC"), w!(""), WS_POPUP, 0, 0, 1, 1, None, None, None, None).unwrap();
            show_at(hwnd, 0, 0).unwrap();
            let before = GetForegroundWindow();
            let took = take_focus(hwnd);
            let (fg, focus) = (GetForegroundWindow(), GetFocus());
            give_back_focus(hwnd, prev);
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            let mut back = GetForegroundWindow();
            for _ in 0..50 {
                if back == prev {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
                back = GetForegroundWindow();
            }
            DestroyWindow(hwnd).unwrap();
            assert_eq!(before, prev, "showing must not activate");
            assert!(took, "take_focus failed, fg={fg:?} prev={prev:?}");
            assert_eq!((fg, focus), (hwnd, hwnd));
            assert_eq!(back, prev);
            assert_ne!(ex & WS_EX_NOACTIVATE.0, 0);
        }
    }
}
