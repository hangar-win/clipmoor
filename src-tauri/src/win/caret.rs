use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, SyncSender, channel, sync_channel};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{ClientToScreen, GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromRect};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, SAFEARRAY};
use windows::Win32::System::Ole::{SafeArrayDestroy, SafeArrayGetUBound};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::Interface;

use crate::placement::{Method, Rect};

struct Job {
    focus: isize,
    uia: bool,
    reply: SyncSender<Option<(Method, Rect)>>,
}

pub struct CaretWorker {
    tx: Sender<Job>,
    busy: Arc<AtomicBool>,
}

impl CaretWorker {
    pub fn spawn() -> Self {
        let (tx, rx) = channel::<Job>();
        let busy = Arc::new(AtomicBool::new(false));
        let flag = busy.clone();
        std::thread::spawn(move || unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let uia = automation();
            for job in rx {
                let r = com_lookup(HWND(job.focus as _), if job.uia { uia.as_ref() } else { None });
                flag.store(false, Ordering::SeqCst);
                let _ = job.reply.send(r);
            }
        });
        CaretWorker { tx, busy }
    }

    pub fn lookup(&self, fg: HWND, budget: Duration) -> (HWND, Option<(Method, Rect)>) {
        let deadline = Instant::now() + budget;
        let mut gti = GUITHREADINFO { cbSize: size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        let tid = unsafe { GetWindowThreadProcessId(fg, None) };
        if tid != 0
            && unsafe { GetGUIThreadInfo(tid, &mut gti) }.is_ok()
            && let Some(r) = system_caret(&gti)
        {
            return (gti.hwndFocus, Some((Method::Gti, r)));
        }
        let focus = if gti.hwndFocus.is_invalid() { fg } else { gti.hwndFocus };
        if focus.is_invalid() || self.busy.swap(true, Ordering::SeqCst) {
            return (focus, None);
        }
        let (reply, rx) = sync_channel(1);
        if self.tx.send(Job { focus: focus.0 as isize, uia: !is_chromium(focus), reply }).is_err() {
            self.busy.store(false, Ordering::SeqCst);
            return (focus, None);
        }
        (focus, rx.recv_timeout(deadline.saturating_duration_since(Instant::now())).ok().flatten())
    }
}

pub fn cursor() -> Option<Rect> {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p) }.ok()?;
    Some(Rect { left: p.x, top: p.y, right: p.x + 1, bottom: p.y + 1 })
}

pub fn work_area(r: Rect) -> Rect {
    let rc = RECT { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
    let mut mi = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    unsafe {
        let mon = MonitorFromRect(&rc, MONITOR_DEFAULTTONEAREST);
        let _ = GetMonitorInfoW(mon, &mut mi);
    }
    let w = mi.rcWork;
    Rect { left: w.left, top: w.top, right: w.right, bottom: w.bottom }
}

fn system_caret(gti: &GUITHREADINFO) -> Option<Rect> {
    let rc = gti.rcCaret;
    if gti.hwndCaret.is_invalid() || rc.bottom <= rc.top {
        return None;
    }
    let mut a = POINT { x: rc.left, y: rc.top };
    let mut b = POINT { x: rc.right, y: rc.bottom };
    unsafe {
        if !ClientToScreen(gti.hwndCaret, &mut a).as_bool() || !ClientToScreen(gti.hwndCaret, &mut b).as_bool() {
            return None;
        }
    }
    Some(Rect { left: a.x, top: a.y, right: b.x.max(a.x + 1), bottom: b.y })
}

pub fn class(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

fn is_chromium(hwnd: HWND) -> bool {
    let class = class(hwnd);
    class.starts_with("Chrome_WidgetWin_") || class == "Chrome_RenderWidgetHostHWND"
}

unsafe fn automation() -> Option<IUIAutomation2> {
    let uia: IUIAutomation2 = unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }.ok()?;
    unsafe {
        let _ = uia.SetConnectionTimeout(50);
        let _ = uia.SetTransactionTimeout(100);
    }
    Some(uia)
}

unsafe fn com_lookup(focus: HWND, uia: Option<&IUIAutomation2>) -> Option<(Method, Rect)> {
    let alive = unsafe { SendMessageTimeoutW(focus, WM_NULL, WPARAM(0), LPARAM(0), SMTO_ABORTIFHUNG | SMTO_BLOCK, 20, None) };
    if alive.0 == 0 {
        return None;
    }
    unsafe {
        msaa_caret(focus).map(|r| (Method::Msaa, r)).or_else(|| uia.and_then(|u| uia_caret(u)).map(|r| (Method::Uia, r)))
    }
}

fn child(id: i32) -> VARIANT {
    let mut v = VARIANT::default();
    unsafe {
        let inner = &mut *v.Anonymous.Anonymous;
        inner.vt = VT_I4;
        inner.Anonymous.lVal = id;
    }
    v
}

unsafe fn accessible(hwnd: HWND, id: OBJECT_IDENTIFIER) -> Option<IAccessible> {
    let mut p = std::ptr::null_mut();
    unsafe {
        AccessibleObjectFromWindow(hwnd, id.0 as u32, &IAccessible::IID, &mut p).ok()?;
        (!p.is_null()).then(|| IAccessible::from_raw(p))
    }
}

unsafe fn location(acc: &IAccessible, id: i32) -> Option<Rect> {
    let (mut l, mut t, mut w, mut h) = (0, 0, 0, 0);
    unsafe { acc.accLocation(&mut l, &mut t, &mut w, &mut h, &child(id)) }.ok()?;
    (w > 0 && h > 0).then_some(Rect { left: l, top: t, right: l + w, bottom: t + h })
}

unsafe fn msaa_caret(focus: HWND) -> Option<Rect> {
    unsafe { location(&accessible(focus, OBJID_CARET)?, CHILDID_SELF as i32) }
}

unsafe fn uia_caret(uia: &IUIAutomation2) -> Option<Rect> {
    unsafe {
        let el = uia.GetFocusedElement().ok()?;
        let text: IUIAutomationTextPattern = el.GetCurrentPatternAs(UIA_TextPatternId).ok()?;
        let sel = text.GetSelection().ok()?;
        if sel.Length().ok()? < 1 {
            return None;
        }
        let range = sel.GetElement(0).ok()?;
        if let Some(r) = first_rect(range.GetBoundingRectangles().ok()?) {
            return Some(r);
        }
        range.ExpandToEnclosingUnit(TextUnit_Character).ok()?;
        first_rect(range.GetBoundingRectangles().ok()?)
    }
}

unsafe fn first_rect(sa: *mut SAFEARRAY) -> Option<Rect> {
    if sa.is_null() {
        return None;
    }
    let r = unsafe {
        let n = SafeArrayGetUBound(sa, 1).map(|u| u + 1).unwrap_or(0);
        if n >= 4 {
            let d = std::slice::from_raw_parts((*sa).pvData as *const f64, 4);
            let (l, t, w, h) = (d[0] as i32, d[1] as i32, d[2] as i32, d[3] as i32);
            (h > 0 && w >= 0).then_some(Rect { left: l, top: t, right: l + 1, bottom: t + h })
        } else {
            None
        }
    };
    let _ = unsafe { SafeArrayDestroy(sa) };
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_within_budget() {
        let worker = CaretWorker::spawn();
        let fg = unsafe { GetForegroundWindow() };
        for _ in 0..3 {
            let t = Instant::now();
            let r = worker.lookup(fg, Duration::from_millis(30));
            let ms = t.elapsed().as_millis();
            println!("caret {r:?} in {ms} ms");
            assert!(ms < 60);
        }
    }
}

