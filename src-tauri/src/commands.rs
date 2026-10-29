use crate::app::{Core, core, now, with_unlock};
use crate::classify::Kind;
use crate::event;
use crate::keys::{PasteMode, Release};
use crate::placement::Align;
use crate::search::{self, Query};
use crate::settings::Settings;
use crate::store::{self, Group, Item, LockSource};
use crate::win::{hook, panel_window, paste_flow, tray};
use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    pub id: i64,
    pub kind: Option<Kind>,
    pub source: Option<String>,
    pub size: Option<i64>,
    pub preview: Option<String>,
    pub last_used: i64,
    pub pinned: bool,
    pub locked: bool,
    pub locked_by: Option<LockSource>,
    pub group: Option<i64>,
    pub has_image: bool,
}

impl From<&Item> for ItemView {
    fn from(i: &Item) -> Self {
        let m = i.meta.as_ref();
        ItemView {
            id: i.id,
            kind: m.map(|m| m.kind),
            source: m.map(|m| m.source.clone()),
            size: m.map(|m| m.size as i64),
            preview: m.map(|m| m.preview.clone()),
            last_used: i.last_used,
            pinned: i.pinned,
            locked: i.locked,
            locked_by: i.locked_by,
            group: m.and_then(|m| m.group),
            has_image: m.is_some_and(|m| m.has_image),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct GroupView {
    pub id: i64,
    pub name: String,
    pub locked: bool,
}

impl From<Group> for GroupView {
    fn from(g: Group) -> Self {
        GroupView { id: g.id, name: g.name, locked: g.locked }
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LockTarget {
    Item { id: i64 },
    Group { id: i64 },
}


fn mutate<T>(app: &AppHandle, f: impl Fn(&mut Core) -> store::Result<T>) -> Result<T, String> {
    let out = with_unlock(app, |c| {
        let out = f(c)?;
        c.reload()?;
        Ok(out)
    })?;
    let _ = app.emit(event::ITEMS_CHANGED, ());
    Ok(out)
}

fn clean(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() { Err("empty group name".into()) } else { Ok(name) }
}

#[tauri::command(async)]
pub fn toggle_pin(app: AppHandle, id: i64) -> Result<bool, String> {
    mutate(&app, |c| {
        let on = !c.items.iter().find(|i| i.id == id).is_some_and(|i| i.pinned);
        c.store.set_pinned(&c.keys, id, on).map(|_| on)
    })
}

#[tauri::command(async)]
pub fn pin_items(app: AppHandle, ids: Vec<i64>) -> Result<(), String> {
    mutate(&app, |c| c.store.pin_items(&ids))
}

#[tauri::command(async)]
pub fn list_groups(app: AppHandle) -> Result<Vec<GroupView>, String> {
    let core = core(&app);
    Ok(core.store.list_groups(&core.keys).map_err(|e| e.to_string())?.into_iter().map(GroupView::from).collect())
}

#[tauri::command(async)]
pub fn create_group(app: AppHandle, name: String) -> Result<i64, String> {
    let name = clean(&name)?;
    mutate(&app, |c| c.store.create_group(&c.keys, name))
}

#[tauri::command(async)]
pub fn rename_group(app: AppHandle, id: i64, name: String) -> Result<(), String> {
    let name = clean(&name)?;
    mutate(&app, |c| c.store.rename_group(&c.keys, id, name))
}

#[tauri::command(async)]
pub fn delete_group(app: AppHandle, id: i64) -> Result<(), String> {
    mutate(&app, |c| c.store.delete_group(&c.keys, id))
}

#[tauri::command(async)]
pub fn move_to_group(app: AppHandle, id: i64, group: Option<i64>) -> Result<(), String> {
    mutate(&app, |c| c.store.set_group(&c.keys, id, group))
}

#[tauri::command(async)]
pub fn set_locked(app: AppHandle, target: LockTarget, locked: bool) -> Result<(), String> {
    mutate(&app, |c| match target {
        LockTarget::Item { id } => c.store.set_item_locked(&c.keys, id, locked),
        LockTarget::Group { id } => c.store.set_group_locked(&c.keys, id, locked),
    })
}

#[tauri::command]
pub fn preview_item(app: AppHandle, id: Option<i64>) {
    let ctl = app.state::<crate::PanelCtl>();
    let mut want = ctl.preview_id.lock().unwrap_or_else(|e| e.into_inner());
    *want = id;
    if id.is_none() {
        panel_window::hide(ctl.preview());
    }
    let _ = app.emit_to("preview", event::PREVIEW_ITEM, id);
}

#[tauri::command]
pub fn place_preview(app: AppHandle, id: i64, width: f64, height: f64) {
    let ctl = app.state::<crate::PanelCtl>();
    let want = ctl.preview_id.lock().unwrap_or_else(|e| e.into_inner());
    if *want == Some(id) && panel_window::is_visible(ctl.panel()) {
        panel_window::show_centered(ctl.preview(), ctl.panel(), (width, height));
    }
}

#[tauri::command]
pub fn panel_input(app: AppHandle, on: bool) -> Result<(), String> {
    let hook = app.state::<hook::HookHandle>();
    let ctl = app.state::<crate::PanelCtl>();
    if !on {
        hook.set_released(Release::Dialog, false);
        panel_window::give_back_focus(ctl.panel(), ctl.target());
        return Ok(());
    }
    if !panel_window::is_visible(ctl.panel()) || !panel_window::take_focus(ctl.panel()) {
        return Err("panel could not take focus".into());
    }
    hook.set_released(Release::Dialog, true);
    let webview = app.get_webview_window("panel").ok_or("panel window missing")?;
    webview.as_ref().set_focus().map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn list_items(app: AppHandle, query: Query) -> Vec<ItemView> {
    search::filter(&core(&app).items, &query).into_iter().map(ItemView::from).collect()
}

#[tauri::command(async)]
pub fn activate(app: AppHandle, id: i64, mode: Option<PasteMode>) -> Result<(), String> {
    paste_flow::activate(&app, id, mode)
}
#[tauri::command(async)]
pub fn item_image(app: AppHandle, id: i64, thumb: bool) -> Result<tauri::ipc::Response, String> {
    let core = core(&app);
    let small = if thumb { core.store.thumb(&core.keys, id).map_err(|e| e.to_string())? } else { None };
    let png = match small {
        Some(png) => png,
        None => core.store.formats(&core.keys, id).map_err(|e| e.to_string())?.png.ok_or("item has no image")?,
    };
    Ok(tauri::ipc::Response::new(png))
}

const TEXT_MAX: usize = 20_000;

#[tauri::command(async)]
pub fn item_text(app: AppHandle, id: i64) -> Result<String, String> {
    let core = core(&app);
    let plain = core.store.formats(&core.keys, id).map_err(|e| e.to_string())?.plain.unwrap_or_default();
    Ok(plain.chars().take(TEXT_MAX).collect())
}

const GEOMETRY_MAX: f64 = 4096.0;

#[tauri::command]
pub fn set_panel_geometry(ctl: tauri::State<crate::PanelCtl>, width: f64, height: f64, align: Align) -> Result<(), String> {
    if !(1.0..=GEOMETRY_MAX).contains(&width) || !(1.0..=GEOMETRY_MAX).contains(&height) {
        return Err(format!("panel size out of range: {width}x{height}"));
    }
    hook::set_align(align);
    panel_window::resize(ctl.panel(), (width, height), align);
    Ok(())
}

#[tauri::command]
pub fn capture_panel_keys(on: bool) {
    hook::capture_keys(on);
}

#[tauri::command(async)]
pub fn hide_panel(app: AppHandle) {
    hook::close(&app);
}

#[tauri::command]
pub fn panel_painted(ctl: tauri::State<crate::PanelCtl>) {
    if let Some(at) = ctl.opened_at.lock().unwrap().take()
        && cfg!(debug_assertions)
    {
        eprintln!("win+v->paint {} ms", at.elapsed().as_millis());
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub enum DragPhase {
    Grab,
    Move,
    Release,
}

#[tauri::command]
pub fn drag_panel(ctl: tauri::State<crate::PanelCtl>, phase: DragPhase) {
    match phase {
        DragPhase::Grab => panel_window::grab(ctl.panel()),
        DragPhase::Move => panel_window::drag(ctl.panel()),
        DragPhase::Release => panel_window::release(ctl.panel()),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    key_method: &'static str,
    windows_history: bool,
    autostart: bool,
    lock_key_missing: bool,
    lock_pending: bool,
    settings_reset: bool,
    unreadable: usize,
}

#[tauri::command(async)]
pub fn get_settings(app: AppHandle) -> Settings {
    core(&app).settings.clone()
}

#[tauri::command(async)]
pub fn save_settings(app: AppHandle, s: Settings) -> Result<(), String> {
    crate::app::apply_settings(&app, |cur| *cur = s)
}

#[tauri::command(async)]
pub fn set_keep_open(app: AppHandle, on: bool) -> Result<(), String> {
    crate::app::apply_settings(&app, |s| s.keep_panel_open = on)
}
#[tauri::command(async)]
pub fn delete_item(app: AppHandle, id: i64) -> Result<(), String> {
    mutate(&app, |c| c.store.delete_item(id))
}

#[tauri::command(async)]
pub fn delete_history(app: AppHandle) -> Result<(), String> {
    mutate(&app, |c| {
        c.store.delete_history()?;
        c.store.vacuum(now())
    })
}

#[tauri::command(async)]
pub fn delete_all(app: AppHandle) -> Result<(), String> {
    mutate(&app, |c| {
        c.key_missing = false;
        c.store.delete_all()?;
        c.store.vacuum(now())
    })
}

#[tauri::command(async)]
pub fn recent_sources(app: AppHandle) -> Vec<String> {
    core(&app).recent_sources()
}

fn autolaunch(app: &AppHandle) -> Result<AutoLaunch, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    AutoLaunchBuilder::new().set_app_name(&app.package_info().name).set_app_path(&format!("\"{}\"", exe.display())).set_windows_enable_mode(WindowsEnableMode::CurrentUser).build().map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn app_info(app: AppHandle) -> AppInfo {
    let (key_method, lock_key_missing, lock_pending, settings_reset, unreadable) = {
        let core = core(&app);
        (core.key_method, core.key_missing, core.store.has_unsealed().unwrap_or(false), core.settings_reset, core.unreadable)
    };
    let autostart = autolaunch(&app).and_then(|a| a.is_enabled().map_err(|e| e.to_string())).unwrap_or(false);
    AppInfo { key_method, windows_history: tray::windows_history_enabled(), autostart, lock_key_missing, lock_pending, settings_reset, unreadable }
}

#[tauri::command(async)]
pub fn set_autostart(app: AppHandle, on: bool) -> Result<(), String> {
    let launch = autolaunch(&app)?;
    if on { launch.enable() } else { launch.disable() }.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_recording(on: bool) {
    hook::set_recording(on);
}

#[tauri::command]
pub fn open_settings() {
    hook::open_settings();
}

#[tauri::command]
pub fn open_windows_clipboard_settings() {
    tray::open_windows_clipboard_settings();
}
