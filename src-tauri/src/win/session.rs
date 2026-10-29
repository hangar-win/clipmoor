use std::sync::OnceLock;

use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::Power::RegisterSuspendResumeNotification;
use windows::Win32::System::RemoteDesktop::{NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification};
use windows::Win32::UI::WindowsAndMessaging::{DEVICE_NOTIFY_WINDOW_HANDLE, PBT_APMSUSPEND, WM_POWERBROADCAST, WM_WTSSESSION_CHANGE, WTS_CONSOLE_DISCONNECT, WTS_REMOTE_DISCONNECT, WTS_SESSION_LOCK, WTS_SESSION_LOGOFF, WTS_SESSION_UNLOCK};

use crate::win::hook;

static ON_EVENT: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

pub fn register(hwnd: HWND, on_event: impl Fn() + Send + Sync + 'static) {
    let _ = ON_EVENT.set(Box::new(on_event));
    if let Err(e) = unsafe { WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) } {
        eprintln!("session notification failed: {e}");
    }
    if let Err(e) = unsafe { RegisterSuspendResumeNotification(HANDLE(hwnd.0), DEVICE_NOTIFY_WINDOW_HANDLE) } {
        eprintln!("suspend notification failed: {e}");
    }
}

fn relocks(msg: u32, wparam: usize) -> bool {
    match msg {
        WM_WTSSESSION_CHANGE => matches!(wparam as u32, WTS_SESSION_LOCK | WTS_SESSION_LOGOFF | WTS_CONSOLE_DISCONNECT | WTS_REMOTE_DISCONNECT),
        WM_POWERBROADCAST => wparam as u32 == PBT_APMSUSPEND,
        _ => false,
    }
}

fn resets_keys(msg: u32, wparam: usize) -> bool {
    msg == WM_WTSSESSION_CHANGE && matches!(wparam as u32, WTS_SESSION_LOCK | WTS_SESSION_UNLOCK)
}

pub fn handle(msg: u32, wparam: usize) {
    if resets_keys(msg, wparam) {
        hook::reset_keys();
    }
    if relocks(msg, wparam) && let Some(f) = ON_EVENT.get() {
        f();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{PBT_APMRESUMEAUTOMATIC, WM_CLIPBOARDUPDATE};

    #[test]
    fn lock_logoff_switch_and_sleep_relock() {
        for w in [WTS_SESSION_LOCK, WTS_SESSION_LOGOFF, WTS_CONSOLE_DISCONNECT, WTS_REMOTE_DISCONNECT] {
            assert!(relocks(WM_WTSSESSION_CHANGE, w as usize));
        }
        assert!(relocks(WM_POWERBROADCAST, PBT_APMSUSPEND as usize));
    }

    #[test]
    fn unlock_resume_and_other_messages_do_not() {
        assert!(!relocks(WM_WTSSESSION_CHANGE, WTS_SESSION_UNLOCK as usize));
        assert!(!relocks(WM_POWERBROADCAST, PBT_APMRESUMEAUTOMATIC as usize));
        assert!(!relocks(WM_CLIPBOARDUPDATE, WTS_SESSION_LOCK as usize));
    }

    #[test]
    fn lock_and_unlock_reset_tracked_keys() {
        assert!(resets_keys(WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK as usize));
        assert!(resets_keys(WM_WTSSESSION_CHANGE, WTS_SESSION_UNLOCK as usize));
        assert!(!resets_keys(WM_POWERBROADCAST, PBT_APMSUSPEND as usize));
        assert!(!resets_keys(WM_CLIPBOARDUPDATE, WTS_SESSION_UNLOCK as usize));
    }
}
