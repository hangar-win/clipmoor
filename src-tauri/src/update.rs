use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::event;
use crate::win::tray;

const EVERY: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Idle,
    Checking,
    Current,
    Available,
    Failed,
    Installing,
    InstallFailed,
}

#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    pub current: String,
    pub available: Option<String>,
    pub status: Status,
}

impl UpdateView {
    fn busy(&self) -> bool {
        matches!(self.status, Status::Checking | Status::Installing)
    }

    fn checked(&mut self, found: Result<Option<String>, ()>) {
        match found {
            Ok(v) => {
                self.status = if v.is_some() { Status::Available } else { Status::Current };
                self.available = v;
            }
            Err(()) => self.status = if self.available.is_some() { Status::Available } else { Status::Failed },
        }
    }

    fn offered(&self) -> Option<&str> {
        self.available.as_deref().filter(|_| self.status != Status::Installing)
    }
}

struct State {
    view: UpdateView,
    pending: Option<Update>,
}

fn state(app: &AppHandle) -> MutexGuard<'_, State> {
    app.state::<Mutex<State>>().inner().lock().unwrap_or_else(|e| e.into_inner())
}

fn publish(app: &AppHandle, f: impl FnOnce(&mut State)) -> UpdateView {
    let view = {
        let mut s = state(app);
        f(&mut s);
        s.view.clone()
    };
    tray::set_update(app, view.offered());
    let _ = app.emit(event::UPDATE_CHANGED, &view);
    view
}

pub fn init(app: &AppHandle) {
    app.manage(Mutex::new(State { view: UpdateView { current: app.package_info().version.to_string(), available: None, status: Status::Idle }, pending: None }));
    let app = app.clone();
    std::thread::spawn(move || {
        loop {
            tauri::async_runtime::block_on(check(&app));
            std::thread::sleep(EVERY);
        }
    });
}

async fn check(app: &AppHandle) -> UpdateView {
    let mut go = false;
    let view = publish(app, |s| {
        if !s.view.busy() {
            s.view.status = Status::Checking;
            go = true;
        }
    });
    if !go {
        return view;
    }
    let exit = app.clone();
    let found = match app.updater_builder().on_before_exit(move || tray::hide(&exit)).build() {
        Ok(u) => u.check().await,
        Err(e) => Err(e),
    };
    publish(app, |s| match found {
        Ok(u) => {
            s.view.checked(Ok(u.as_ref().map(|u| u.version.clone())));
            s.pending = u;
        }
        Err(e) => {
            eprintln!("update check failed: {e}");
            s.view.checked(Err(()));
        }
    })
}

pub async fn install(app: &AppHandle) -> Result<(), String> {
    let mut pending = None;
    publish(app, |s| {
        if !s.view.busy() {
            pending = s.pending.clone();
            if pending.is_some() {
                s.view.status = Status::Installing;
            }
        }
    });
    let Some(update) = pending else { return Err("noUpdate".into()) };
    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => app.restart(),
        Err(e) => {
            publish(app, |s| s.view.status = Status::InstallFailed);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn update_state(app: AppHandle) -> UpdateView {
    state(&app).view.clone()
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> UpdateView {
    check(&app).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    install(&app).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(available: Option<&str>, status: Status) -> UpdateView {
        UpdateView { current: "0.1.0".into(), available: available.map(Into::into), status }
    }

    #[test]
    fn check_result_sets_status() {
        let mut v = view(None, Status::Checking);
        v.checked(Ok(Some("0.2.0".into())));
        assert_eq!(v, view(Some("0.2.0"), Status::Available));
        v.checked(Ok(None));
        assert_eq!(v, view(None, Status::Current));
        v.checked(Err(()));
        assert_eq!(v, view(None, Status::Failed));
    }

    #[test]
    fn failed_check_keeps_known_update() {
        let mut v = view(Some("0.2.0"), Status::Checking);
        v.checked(Err(()));
        assert_eq!(v, view(Some("0.2.0"), Status::Available));
    }

    #[test]
    fn installing_hides_offer() {
        assert_eq!(view(Some("0.2.0"), Status::Installing).offered(), None);
        assert_eq!(view(Some("0.2.0"), Status::Checking).offered(), Some("0.2.0"));
        assert_eq!(view(Some("0.2.0"), Status::InstallFailed).offered(), Some("0.2.0"));
    }
}
