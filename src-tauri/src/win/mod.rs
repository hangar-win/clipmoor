pub mod caret;
pub mod clipboard;
pub mod hello;
pub mod hook;
pub mod keywrap;
pub mod panel_window;
pub mod paste;
pub mod paste_flow;
pub mod session;
pub mod source_app;
pub mod text_input;
pub mod tray;

use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
use windows::core::{HSTRING, w};

pub fn fatal(message: &str) -> ! {
    eprintln!("startup failed: {message}");
    let text = HSTRING::from(format!("Clipmoor başlatılamadı / could not start:\n\n{message}"));
    unsafe { MessageBoxW(None, &text, w!("Clipmoor"), MB_OK | MB_ICONERROR) };
    std::process::exit(1)
}

pub fn purge_keys() -> i32 {
    let tpm = keywrap::delete_tpm_key();
    let hello = hello::delete_credential();
    if let Err(e) = &tpm {
        eprintln!("tpm key: {e}");
    }
    if let Err(e) = &hello {
        eprintln!("hello credential: {e}");
    }
    i32::from(tpm.is_err() || hello.is_err())
}
