mod app;
pub mod capture;
pub mod classify;
pub mod clip;
mod commands;
pub mod crypto;
mod keys;
pub mod search;
pub mod lock;
pub mod placement;
mod settings;
pub mod store;
mod update;
#[cfg(windows)]
mod win;

use std::sync::Mutex;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Instant;
use tauri::Manager;
use windows::Win32::Foundation::HWND;

pub mod event {
    pub const ITEMS_CHANGED: &str = "items-changed";
    pub const SETTINGS_CHANGED: &str = "settings-changed";
    pub const LOCK_ERROR: &str = "lock-error";
    pub const PANEL_KEY: &str = "panel-key";
    pub const PANEL_SHOWN: &str = "panel-shown";
    pub const PANEL_HIDDEN: &str = "panel-hidden";
    pub const PREVIEW_ITEM: &str = "preview-item";
    pub const UPDATE_CHANGED: &str = "update-changed";
}

pub struct PanelCtl {
    pub hwnd: isize,
    pub target: AtomicIsize,
    pub opened_at: Mutex<Option<Instant>>,
    pub preview: isize,
    pub preview_id: Mutex<Option<i64>>,
}

impl PanelCtl {
    pub fn panel(&self) -> HWND {
        HWND(self.hwnd as _)
    }

    pub fn preview(&self) -> HWND {
        HWND(self.preview as _)
    }

    pub fn target(&self) -> HWND {
        HWND(self.target.load(Ordering::Relaxed) as _)
    }

    pub fn set_target(&self, hwnd: HWND) {
        self.target.store(hwnd.0 as isize, Ordering::Relaxed);
    }
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let panel = app.get_webview_window("panel").ok_or("panel window missing")?;
    let hwnd = panel.hwnd()?.0 as isize;
    win::panel_window::make_noactivate(HWND(hwnd as _))?;
    win::panel_window::pin_in_place(HWND(hwnd as _))?;
    let preview_window = app.get_webview_window("preview").ok_or("preview window missing")?;
    preview_window.set_ignore_cursor_events(true)?;
    let preview = preview_window.hwnd()?.0 as isize;
    win::panel_window::make_noactivate(HWND(preview as _))?;
    app.manage(PanelCtl { hwnd, target: AtomicIsize::new(0), opened_at: Mutex::new(None), preview, preview_id: Mutex::new(None) });
    app::init(app)?;
    update::init(app.handle());
    let (takeover, position, shortcuts) = {
        let core = app.state::<Mutex<app::Core>>();
        let core = app::lock(&core);
        (core.settings.takeover, core.settings.panel_position, core.settings.shortcuts)
    };
    let hook = win::hook::spawn(app.handle().clone(), takeover, shortcuts);
    hook.set_position(position);
    app.manage(hook);
    Ok(())
}

pub fn run() {
    if std::env::args().any(|a| a == "--purge-keys") {
        std::process::exit(win::purge_keys());
    }
    let ran = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| win::tray::open_settings(app)))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            if let Err(e) = setup(app) {
                win::fatal(&e.to_string());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::list_items, commands::activate, commands::item_image, commands::hide_panel, commands::panel_painted, commands::drag_panel, commands::set_panel_geometry, commands::capture_panel_keys, commands::item_text, commands::toggle_pin, commands::pin_items, commands::list_groups, commands::create_group, commands::rename_group, commands::delete_group, commands::move_to_group, commands::panel_input, commands::preview_item, commands::place_preview, commands::set_locked, commands::get_settings, commands::save_settings, commands::set_keep_open,commands::delete_item, commands::delete_history, commands::delete_all, commands::recent_sources, commands::app_info, commands::set_autostart, commands::open_settings, commands::open_windows_clipboard_settings, commands::set_recording, update::update_state, update::check_update, update::install_update])
        .run(tauri::generate_context!());
    if let Err(e) = ran {
        win::fatal(&e.to_string());
    }
}
