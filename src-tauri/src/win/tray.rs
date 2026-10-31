use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent, Wry};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Globalization::GetUserDefaultUILanguage;
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

use crate::app::{Core, lock};
use crate::settings::Language;
use crate::win::hook;

const LANG_TURKISH: u16 = 0x1F;

struct Text {
    open: &'static str,
    settings: &'static str,
    pause: &'static str,
    quit: &'static str,
    update: &'static str,
    busy: &'static str,
    window: &'static str,
}

const TR: Text = Text { open: "Aç", settings: "Ayarlar", pause: "Duraklat", quit: "Çıkış", update: "Güncelle", busy: "Clipmoor: şifreleme anahtarı hazırlanıyor…", window: "Clipmoor Ayarlar" };
const EN: Text = Text { open: "Open", settings: "Settings", pause: "Pause", quit: "Quit", update: "Update", busy: "Clipmoor: preparing the encryption key…", window: "Clipmoor Settings" };

pub struct Tray {
    icon: TrayIcon,
    open: MenuItem<Wry>,
    settings: MenuItem<Wry>,
    pause: CheckMenuItem<Wry>,
    quit: MenuItem<Wry>,
    menu: Menu<Wry>,
    update: MenuItem<Wry>,
    version: Mutex<Option<String>>,
    turkish: AtomicBool,
}

impl Tray {
    fn text(&self) -> &'static Text {
        if self.turkish.load(Ordering::Relaxed) { &TR } else { &EN }
    }

    fn version(&self) -> std::sync::MutexGuard<'_, Option<String>> {
        self.version.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn update_text(&self, version: &str) -> String {
        format!("{} (v{version})", self.text().update)
    }
}

fn system_locale() -> &'static str {
    if unsafe { GetUserDefaultUILanguage() } & 0x3FF == LANG_TURKISH { "tr" } else { "en" }
}

pub fn is_turkish(lang: Language) -> bool {
    lang.is_turkish(system_locale())
}

pub fn create(app: &AppHandle, lang: Language) -> tauri::Result<()> {
    let turkish = is_turkish(lang);
    let t = if turkish { &TR } else { &EN };
    let open = MenuItem::with_id(app, "open", t.open, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", t.settings, true, None::<&str>)?;
    let pause = CheckMenuItem::with_id(app, "pause", t.pause, true, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", t.quit, true, None::<&str>)?;
    let update = MenuItem::with_id(app, "update", t.update, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &settings, &pause, &PredefinedMenuItem::separator(app)?, &quit])?;
    let mut builder = TrayIconBuilder::with_id("main").tooltip("Clipmoor").menu(&menu).on_menu_event(on_menu);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    let icon = builder.build(app)?;
    app.manage(Tray { icon, open, settings, pause, quit, menu, update, version: Mutex::new(None), turkish: AtomicBool::new(turkish) });
    Ok(())
}

fn on_menu(app: &AppHandle, e: MenuEvent) {
    match e.id().as_ref() {
        "open" => hook::open_at_mouse(),
        "settings" => open_settings(app),
        "pause" => {
            let state = app.state::<Mutex<Core>>();
            let mut core = lock(&state);
            core.paused = !core.paused;
            let _ = app.state::<Tray>().pause.set_checked(core.paused);
        }
        "update" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move { crate::update::install(&app).await });
        }
        "quit" => app.exit(0),
        _ => {}
    }
}

pub fn set_language(app: &AppHandle, lang: Language) {
    let Some(tray) = app.try_state::<Tray>() else { return };
    tray.turkish.store(is_turkish(lang), Ordering::Relaxed);
    let t = tray.text();
    let _ = tray.open.set_text(t.open);
    let _ = tray.settings.set_text(t.settings);
    let _ = tray.pause.set_text(t.pause);
    let _ = tray.quit.set_text(t.quit);
    if let Some(v) = tray.version().as_deref() {
        let _ = tray.update.set_text(tray.update_text(v));
    }
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_title(t.window);
    }
}

pub fn set_update(app: &AppHandle, version: Option<&str>) {
    let Some(tray) = app.try_state::<Tray>() else { return };
    *tray.version() = version.map(str::to_owned);
    let shown = tray.menu.get("update").is_some();
    match version {
        Some(v) => {
            let _ = tray.update.set_text(tray.update_text(v));
            if !shown {
                let _ = tray.menu.insert(&tray.update, 0);
            }
        }
        None if shown => {
            let _ = tray.menu.remove(&tray.update);
        }
        None => {}
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(tray) = app.try_state::<Tray>() {
        let _ = tray.icon.set_visible(false);
    }
}

pub fn set_busy(app: &AppHandle, busy: bool) {
    if let Some(tray) = app.try_state::<Tray>() {
        let _ = tray.icon.set_tooltip(Some(if busy { tray.text().busy } else { "Clipmoor" }));
    }
}

pub fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let title = app.try_state::<Tray>().map_or(TR.window, |t| t.text().window);
    let built = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into())).title(title).inner_size(1080.0, 680.0).min_inner_size(760.0, 520.0).resizable(true).center().focused(true).build();
    match built {
        Ok(w) => w.on_window_event(|e| {
            if matches!(e, WindowEvent::Focused(false) | WindowEvent::Destroyed) {
                hook::set_recording(false);
            }
        }),
        Err(e) => eprintln!("settings window failed: {e}"),
    }
}

pub fn windows_history_enabled() -> bool {
    let mut value = 0u32;
    let mut size = size_of::<u32>() as u32;
    let read = unsafe { RegGetValueW(HKEY_CURRENT_USER, w!("Software\\Microsoft\\Clipboard"), w!("EnableClipboardHistory"), RRF_RT_REG_DWORD, None, Some(&mut value as *mut u32 as *mut _), Some(&mut size)) };
    read == ERROR_SUCCESS && value == 1
}

pub fn open_windows_clipboard_settings() {
    unsafe {
        ShellExecuteW(None, w!("open"), w!("ms-settings:clipboard"), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
    }
}
