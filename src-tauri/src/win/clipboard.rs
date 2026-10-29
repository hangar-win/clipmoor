use std::mem::size_of;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BI_BITFIELDS, BITMAPV5HEADER};
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GHND, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{CF_DIB, CF_DIBV5, CF_UNICODETEXT};
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, HWND_MESSAGE, MSG, RegisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLIPBOARDUPDATE, WNDCLASSW};
use windows::core::{Error, PCWSTR, Result, w};

use image::ImageDecoder;

use crate::capture::{self, ClipMarkers, RawClip};
use crate::clip::{ClipFormats, OutFormat};
use crate::win::{session, source_app};

fn registered(name: PCWSTR) -> Result<u32> {
    match unsafe { RegisterClipboardFormatW(name) } {
        0 => Err(Error::from_thread()),
        id => Ok(id),
    }
}

fn put(format: u32, bytes: &[u8]) -> Result<()> {
    unsafe {
        let mem = GlobalAlloc(GHND, bytes.len().max(1))?;
        let ptr = GlobalLock(mem) as *mut u8;
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return Err(Error::from_thread());
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        let _ = GlobalUnlock(mem);
        if let Err(e) = SetClipboardData(format, Some(HANDLE(mem.0))) {
            let _ = GlobalFree(Some(mem));
            return Err(e);
        }
    }
    Ok(())
}

fn dibv5(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let header = BITMAPV5HEADER {
        bV5Size: size_of::<BITMAPV5HEADER>() as u32,
        bV5Width: width as i32,
        bV5Height: height as i32,
        bV5Planes: 1,
        bV5BitCount: 32,
        bV5Compression: BI_BITFIELDS,
        bV5SizeImage: width * height * 4,
        bV5RedMask: 0x00FF_0000,
        bV5GreenMask: 0x0000_FF00,
        bV5BlueMask: 0x0000_00FF,
        bV5AlphaMask: 0xFF00_0000,
        bV5CSType: 0x7352_4742,
        bV5Intent: 4,
        ..Default::default()
    };
    let mut out = unsafe { std::slice::from_raw_parts(&header as *const _ as *const u8, size_of::<BITMAPV5HEADER>()) }.to_vec();
    for row in rgba.chunks_exact(width as usize * 4).rev() {
        out.extend(row.as_chunks::<4>().0.iter().flat_map(|&[r, g, b, a]| [b, g, r, a]));
    }
    out
}

const OPEN_BUDGET: Duration = Duration::from_secs(1);

fn prepare(plan: &[OutFormat]) -> Result<Vec<(u32, Vec<u8>)>> {
    let mut out = Vec::new();
    for f in plan {
        match f {
            OutFormat::Plain(s) => out.push((CF_UNICODETEXT.0 as u32, s.encode_utf16().chain([0]).flat_map(u16::to_le_bytes).collect())),
            OutFormat::Html(b) => out.push((registered(w!("HTML Format"))?, b.clone())),
            OutFormat::Rtf(b) => out.push((registered(w!("Rich Text Format"))?, b.clone())),
            OutFormat::Image(png) => {
                out.push((registered(w!("PNG"))?, png.clone()));
                if let Some(img) = capture::decode_png(png) {
                    let img = img.to_rgba8();
                    out.push((CF_DIBV5.0 as u32, dibv5(&img, img.width(), img.height())));
                }
            }
            OutFormat::ExcludeMonitor => out.push((registered(w!("ExcludeClipboardContentFromMonitorProcessing"))?, Vec::new())),
            OutFormat::CanIncludeInHistory(v) => out.push((registered(w!("CanIncludeInClipboardHistory"))?, v.to_le_bytes().to_vec())),
        }
    }
    Ok(out)
}

fn backoff(attempt: u32) -> Duration {
    Duration::from_millis((5u64 << attempt.min(5)).min(100))
}

fn open() -> Result<()> {
    let started = Instant::now();
    let mut attempt = 0;
    loop {
        match unsafe { OpenClipboard(None) } {
            Ok(()) => return Ok(()),
            Err(e) if started.elapsed() >= OPEN_BUDGET => return Err(e),
            Err(_) => std::thread::sleep(backoff(attempt)),
        }
        attempt += 1;
    }
}

pub fn write(plan: &[OutFormat]) -> Result<u32> {
    let data = prepare(plan)?;
    open()?;
    let written = unsafe { EmptyClipboard() }.and_then(|_| data.iter().try_for_each(|(format, bytes)| put(*format, bytes)));
    let seq = unsafe { GetClipboardSequenceNumber() };
    let _ = unsafe { CloseClipboard() };
    written?;
    Ok(seq)
}

fn dib_to_png(dib: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = image::codecs::bmp::BmpDecoder::new_without_file_header(std::io::Cursor::new(dib)).ok()?;
    decoder.set_limits(capture::image_limits()).ok()?;
    let mut img = image::DynamicImage::from_decoder(decoder).ok()?.to_rgba8();
    if img.pixels().all(|p| p[3] == 0) {
        img.pixels_mut().for_each(|p| p[3] = 255);
    }
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

fn get(format: u32) -> Option<Vec<u8>> {
    unsafe {
        let mem = HGLOBAL(GetClipboardData(format).ok()?.0);
        let ptr = GlobalLock(mem) as *const u8;
        if ptr.is_null() {
            return None;
        }
        let bytes = std::slice::from_raw_parts(ptr, GlobalSize(mem)).to_vec();
        let _ = GlobalUnlock(mem);
        Some(bytes)
    }
}

fn has(name: PCWSTR) -> bool {
    registered(name).is_ok_and(|f| unsafe { IsClipboardFormatAvailable(f) }.is_ok())
}

fn get_named(name: PCWSTR) -> Option<Vec<u8>> {
    registered(name).ok().and_then(get)
}

fn trim_nul(mut b: Vec<u8>) -> Vec<u8> {
    let end = b.iter().rposition(|&c| c != 0).map_or(0, |i| i + 1);
    b.truncate(end);
    b
}

fn utf16(b: &[u8]) -> String {
    let units: Vec<u16> = b.as_chunks::<2>().0.iter().map(|&c| u16::from_le_bytes(c)).take_while(|&u| u != 0).collect();
    String::from_utf16_lossy(&units)
}

pub fn search_text(s: &str) -> Option<String> {
    let line = s.lines().map(str::trim).find(|l| !l.is_empty())?;
    Some(line.chars().take(256).collect())
}

pub fn text_for_search() -> Option<String> {
    open().ok()?;
    let plain = get(CF_UNICODETEXT.0 as u32);
    let _ = unsafe { CloseClipboard() };
    search_text(&utf16(&plain?))
}

fn read_open() -> (ClipMarkers, ClipFormats, Option<Vec<u8>>) {
    let markers = ClipMarkers {
        exclude_monitor: has(w!("ExcludeClipboardContentFromMonitorProcessing")),
        viewer_ignore: has(w!("Clipboard Viewer Ignore")),
        can_include_in_history: get_named(w!("CanIncludeInClipboardHistory")).and_then(|b| Some(u32::from_le_bytes(b.get(..4)?.try_into().ok()?))),
    };
    if markers.exclude_monitor || markers.viewer_ignore || markers.can_include_in_history == Some(0) {
        return (markers, ClipFormats::default(), None);
    }
    let plain = get(CF_UNICODETEXT.0 as u32).map(|b| utf16(&b));
    let png = get_named(w!("PNG"));
    let dib = if png.is_none() { get(CF_DIB.0 as u32) } else { None };
    let formats = ClipFormats { plain, html: get_named(w!("HTML Format")).map(trim_nul), rtf: get_named(w!("Rich Text Format")).map(trim_nul), png };
    (markers, formats, dib)
}

pub fn read() -> Result<RawClip> {
    let source_exe = source_app::foreground_exe();
    open()?;
    let seq = unsafe { GetClipboardSequenceNumber() };
    let (markers, mut formats, dib) = read_open();
    let _ = unsafe { CloseClipboard() };
    if let Some(dib) = dib {
        formats.png = dib_to_png(&dib);
    }
    Ok(RawClip { formats, markers, seq, source_exe })
}

thread_local! {
    static LISTENER: std::cell::OnceCell<Sender<RawClip>> = const { std::cell::OnceCell::new() };
}

unsafe extern "system" fn listener_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        match read() {
            Ok(raw) => LISTENER.with(|tx| {
                if let Some(tx) = tx.get() {
                    let _ = tx.send(raw);
                }
            }),
            Err(e) => eprintln!("clipboard read failed: {e}"),
        }
        return LRESULT(0);
    }
    session::handle(msg, wparam.0);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn start_listener(tx: Sender<RawClip>, on_session: impl Fn() + Send + Sync + 'static) {
    std::thread::spawn(move || unsafe {
        LISTENER.with(|l| l.set(tx).ok());
        let class = w!("ClipmoorClipboardListener");
        let instance = GetModuleHandleW(None).map(Into::into).unwrap_or_default();
        RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(listener_proc), hInstance: instance, lpszClassName: class, ..Default::default() });
        let hwnd = match CreateWindowExW(WINDOW_EX_STYLE(0), class, PCWSTR::null(), WINDOW_STYLE(0), 0, 0, 0, 0, Some(HWND_MESSAGE), None, Some(instance), None) {
            Ok(h) => h,
            Err(e) => return eprintln!("clipboard listener window failed: {e}"),
        };
        session::register(hwnd, on_session);
        if let Err(e) = AddClipboardFormatListener(hwnd) {
            return eprintln!("clipboard listener failed: {e}");
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_text_is_first_trimmed_line_capped() {
        assert_eq!(search_text("  merhaba dünya  \r\nikinci"), Some("merhaba dünya".into()));
        assert_eq!(search_text("\n\nüçüncü"), Some("üçüncü".into()));
        assert_eq!(search_text("   "), None);
        assert_eq!(search_text(&"ş".repeat(300)).map(|s| s.chars().count()), Some(256));
        assert_eq!(search_text("a\tb"), Some("a\tb".into()));
    }

    #[test]
    fn open_backoff_grows_and_caps() {
        assert_eq!([0, 1, 2, 4, 5, 9].map(|a| backoff(a).as_millis()), [5, 10, 20, 80, 100, 100]);
        assert!((0..20).map(backoff).sum::<Duration>() >= OPEN_BUDGET);
    }

    #[test]
    fn dibv5_is_bottom_up_bgra_after_header() {
        let rgba = [1, 2, 3, 4, 5, 6, 7, 8];
        let dib = dibv5(&rgba, 1, 2);
        assert_eq!(&dib[..4], &124u32.to_le_bytes());
        assert_eq!(&dib[8..12], &2i32.to_le_bytes());
        assert_eq!(&dib[124..], &[7, 6, 5, 8, 3, 2, 1, 4]);
    }

    fn decoded(png: Vec<u8>) -> Vec<u8> {
        image::load_from_memory_with_format(&png, image::ImageFormat::Png).unwrap().to_rgba8().into_raw()
    }

    fn info_header(compression: u32) -> Vec<u8> {
        let mut dib = Vec::new();
        for v in [40u32, 1, 1] {
            dib.extend(v.to_le_bytes());
        }
        dib.extend(1u16.to_le_bytes());
        dib.extend(32u16.to_le_bytes());
        dib.extend(compression.to_le_bytes());
        dib.extend([0u8; 20]);
        dib
    }

    #[test]
    fn bitfields_dib_becomes_png() {
        let mut dib = info_header(3);
        for m in [0x00FF_0000u32, 0x0000_FF00, 0x0000_00FF] {
            dib.extend(m.to_le_bytes());
        }
        dib.extend([30, 20, 10, 0]);
        assert_eq!(decoded(dib_to_png(&dib).unwrap()), [10, 20, 30, 255]);
    }

    #[test]
    fn plain_32bit_dib_with_zero_alpha_is_opaque() {
        let mut dib = info_header(0);
        dib.extend([30, 20, 10, 0]);
        assert_eq!(decoded(dib_to_png(&dib).unwrap()), [10, 20, 30, 255]);
        assert!(dib_to_png(b"junk").is_none());
    }

    #[test]
    fn oversized_dib_is_rejected_before_decoding() {
        let mut dib = info_header(0);
        dib[4..8].copy_from_slice(&40_000i32.to_le_bytes());
        dib[8..12].copy_from_slice(&40_000i32.to_le_bytes());
        dib.extend([0u8; 64]);
        assert!(dib_to_png(&dib).is_none());
    }
}
