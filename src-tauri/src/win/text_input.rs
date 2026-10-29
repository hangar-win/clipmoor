use std::sync::{Mutex, MutexGuard};

use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, HKL, ToUnicodeEx};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

const KEEP_KEYBOARD_STATE: u32 = 0x4;

type DeadKey = (u32, u32, [u8; 256]);
static DEAD: Mutex<Option<DeadKey>> = Mutex::new(None);

fn dead() -> MutexGuard<'static, Option<DeadKey>> {
    DEAD.lock().unwrap_or_else(|e| e.into_inner())
}

fn layout() -> HKL {
    unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
}

pub fn reset() {
    *dead() = None;
}

pub fn translate(vk: u32, scan: u32, state: &[u8; 256]) -> Option<String> {
    let hkl = Some(layout());
    let mut buf = [0u16; 8];
    let pending = dead().take();
    let n = match &pending {
        None => unsafe { ToUnicodeEx(vk, scan, state, &mut buf, KEEP_KEYBOARD_STATE, hkl) },
        Some((dvk, dscan, dstate)) => unsafe {
            ToUnicodeEx(*dvk, *dscan, dstate, &mut buf, 0, hkl);
            ToUnicodeEx(vk, scan, state, &mut buf, 0, hkl)
        },
    };
    if n < 0 {
        if pending.is_some() {
            unsafe { ToUnicodeEx(vk, scan, state, &mut buf, 0, hkl) };
        }
        *dead() = Some((vk, scan, *state));
        return None;
    }
    let s = String::from_utf16_lossy(&buf[..n as usize]);
    (!s.is_empty() && !s.chars().any(char::is_control)).then_some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_follow_shift_and_controls_are_dropped() {
        let mut state = [0u8; 256];
        assert_eq!(translate(0x41, 0x1E, &state).as_deref(), Some("a"));
        state[0x10] = 0x80;
        assert_eq!(translate(0x41, 0x1E, &state).as_deref(), Some("A"));
        assert_eq!(translate(0x0D, 0x1C, &[0; 256]), None);
    }
}
