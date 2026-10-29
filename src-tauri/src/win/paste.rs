use std::mem::size_of;

use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub const OUR_EXTRA_INFO: usize = 0x4B41_5254;

fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let flags = if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) };
    INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: OUR_EXTRA_INFO } } }
}

fn send(inputs: &[INPUT]) -> windows::core::Result<()> {
    let sent = unsafe { SendInput(inputs, size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() { Ok(()) } else { Err(windows::core::Error::from_thread()) }
}

fn ctrl_v_keys(ctrl_down: bool, shifts_down: &[VIRTUAL_KEY]) -> Vec<(VIRTUAL_KEY, bool)> {
    let mut keys: Vec<_> = shifts_down.iter().map(|&s| (s, true)).collect();
    if !ctrl_down {
        keys.push((VK_CONTROL, false));
    }
    keys.extend([(VK_V, false), (VK_V, true)]);
    if !ctrl_down {
        keys.push((VK_CONTROL, true));
    }
    keys.extend(shifts_down.iter().map(|&s| (s, false)));
    keys
}

fn system_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000 != 0 }
}

pub fn send_ctrl_v() -> windows::core::Result<()> {
    let shifts: Vec<_> = [VK_LSHIFT, VK_RSHIFT].into_iter().filter(|&s| system_down(s)).collect();
    let keys = ctrl_v_keys(system_down(VK_CONTROL), &shifts);
    send(&keys.into_iter().map(|(vk, up)| key(vk, up)).collect::<Vec<_>>())
}

pub fn send_start_mask() {
    let _ = send(&[key(VIRTUAL_KEY(0xE8), false), key(VIRTUAL_KEY(0xE8), true)]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_full_ctrl_v_when_nothing_is_held() {
        assert_eq!(ctrl_v_keys(false, &[]), vec![(VK_CONTROL, false), (VK_V, false), (VK_V, true), (VK_CONTROL, true)]);
    }

    #[test]
    fn held_ctrl_gets_only_v() {
        assert_eq!(ctrl_v_keys(true, &[]), vec![(VK_V, false), (VK_V, true)]);
    }

    #[test]
    fn held_shift_is_lifted_around_v_and_restored() {
        assert_eq!(ctrl_v_keys(true, &[VK_RSHIFT]), vec![(VK_RSHIFT, true), (VK_V, false), (VK_V, true), (VK_RSHIFT, false)]);
    }
}
