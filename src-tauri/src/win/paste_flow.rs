use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::app::{self, Core, lock, now};
use crate::clip::write_plan;
use crate::event;
use crate::keys::{PasteMode, mode_for};
use crate::settings::Settings;
use crate::win::{clipboard, hook, panel_window, paste};

fn resolve_mode(mode: Option<PasteMode>, mods: impl FnOnce() -> (bool, bool)) -> PasteMode {
    mode.unwrap_or_else(|| {
        let (ctrl, shift) = mods();
        mode_for(ctrl, shift)
    })
}

fn closes_panel(s: &Settings) -> bool {
    s.close_on_paste && !s.keep_panel_open
}

fn sends_paste(mode: PasteMode) -> bool {
    mode != PasteMode::Copy
}

pub fn activate(app: &AppHandle, id: i64, mode: Option<PasteMode>) -> Result<(), String> {
    let mode = resolve_mode(mode, || app.state::<hook::HookHandle>().mods());
    let (formats, locked) = app::with_unlock(app, |c| Ok((c.store.formats(&c.keys, id)?, c.items.iter().any(|i| i.id == id && i.locked))))?;
    let plan = write_plan(&formats, mode, locked);
    let state = app.state::<Mutex<Core>>();
    lock(&state).note_own_write(&formats);
    let seq = clipboard::write(&plan).map_err(|e| e.to_string())?;
    let close = {
        let mut core = lock(&state);
        core.own_seq = Some(seq);
        if let Err(e) = core.store.touch(id, now()).and_then(|_| core.reload()) {
            eprintln!("touch failed: {e}");
        }
        closes_panel(&core.settings)
    };
    let _ = app.emit(event::ITEMS_CHANGED, ());
    if close {
        hook::close(app);
    } else {
        app::after_use(app);
    }
    if sends_paste(mode) {
        panel_window::restore_foreground(app.state::<crate::PanelCtl>().target());
        paste::send_ctrl_v().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_mode_wins_over_held_modifiers() {
        assert_eq!(resolve_mode(Some(PasteMode::Copy), || (true, true)), PasteMode::Copy);
        assert_eq!(resolve_mode(None, || (true, true)), PasteMode::PlainPaste);
        assert_eq!(resolve_mode(None, || (false, true)), PasteMode::Copy);
        assert_eq!(resolve_mode(None, || (false, false)), PasteMode::Paste);
    }

    #[test]
    fn panel_closes_only_when_close_on_paste_and_not_kept_open() {
        let s = |close_on_paste, keep_panel_open| Settings { close_on_paste, keep_panel_open, ..Settings::default() };
        assert!(closes_panel(&s(true, false)));
        assert!(!closes_panel(&s(true, true)));
        assert!(!closes_panel(&s(false, false)));
        assert!(!closes_panel(&s(false, true)));
    }

    #[test]
    fn copy_mode_never_sends_ctrl_v() {
        assert!(!sends_paste(PasteMode::Copy));
        assert!(sends_paste(PasteMode::Paste));
        assert!(sends_paste(PasteMode::PlainPaste));
    }
}
