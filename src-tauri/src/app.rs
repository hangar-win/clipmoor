use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::HWND;
use zeroize::Zeroizing;

use crate::capture::{self, CaptureContext, CaptureDecision, RawClip};
use crate::classify::classify;
use crate::clip::ClipFormats;
use crate::crypto::{self, CryptoError, Keys, LockRoot, lock_root};
use crate::event;
use crate::keys::Release;
use crate::lock::UnlockWindow;
use crate::settings::{self, Settings};
use crate::store::{self, Item, NewClip, Store, StoreError};
use crate::win::hook::{self, Hold, HookHandle};
use crate::win::{clipboard, hello, keywrap, panel_window, tray};

const RETENTION_EVERY: u32 = 50;
const OWN_ECHO: Duration = Duration::from_secs(2);
const RELOCK_TICK: Duration = Duration::from_secs(5);
const PROMPT_TIMEOUT: Duration = Duration::from_secs(120);

static APP: OnceLock<AppHandle> = OnceLock::new();
static PANEL_LOCKED: AtomicBool = AtomicBool::new(false);

pub struct Core {
    pub store: Store,
    pub keys: Keys,
    pub items: Vec<Item>,
    pub own_seq: Option<u32>,
    own_write: Option<([u8; 32], Instant)>,
    pub paused: bool,
    pub settings: Settings,
    pub unlock: UnlockWindow<LockRoot>,
    pub key_method: &'static str,
    pub key_missing: bool,
    pub settings_reset: bool,
    pub unreadable: usize,
}

impl Core {
    pub fn new(mut store: Store, keys: Keys, settings: Settings, now: i64) -> store::Result<Self> {
        store.lock_all = settings.lock_all_history;
        let unlock = UnlockWindow::new(settings.unlock_duration);
        let mut core = Core { store, keys, items: Vec::new(), own_seq: None, own_write: None, paused: false, settings, unlock, key_method: "dpapi", key_missing: false, settings_reset: false, unreadable: 0 };
        core.retain(now)?;
        core.reload()?;
        Ok(core)
    }

    pub fn reload(&mut self) -> store::Result<()> {
        let loaded = self.store.load(&self.keys, &self.items)?;
        self.items = loaded.items;
        self.unreadable = loaded.unreadable;
        Ok(())
    }

    pub fn refresh(&mut self) -> bool {
        let root = self.unlock.current(Instant::now()).cloned();
        let changed = root.is_some() != self.keys.lock_root.is_some();
        self.keys.lock_root = root;
        if changed && let Err(e) = self.reload() {
            eprintln!("reload failed: {e}");
        }
        changed
    }

    pub fn needs_panel_unlock(&self) -> bool {
        self.settings.lock_all_history && self.keys.lock_root.is_none()
    }

    pub fn unlock_with(&mut self, signature: &[u8], now: Instant) -> store::Result<()> {
        let root = lock_root(&self.keys.master, signature);
        self.keys.lock_root = Some(root.clone());
        let checked = self.store.load(&self.keys, &[]).and_then(|l| if l.locked_bad > 0 && l.locked_ok == 0 { Err(StoreError::Crypto(CryptoError::Decrypt)) } else { Ok(()) }).and_then(|_| self.store.reconcile_all(&self.keys));
        if let Err(e) = checked {
            self.keys.lock_root = None;
            return Err(e);
        }
        self.unlock.set(root, now);
        self.reload()
    }

    pub fn apply_settings(&mut self, s: Settings, now: i64) -> store::Result<()> {
        let relock = self.store.lock_all != s.lock_all_history;
        self.store.lock_all = s.lock_all_history;
        self.unlock.set_duration(s.unlock_duration);
        self.settings = s;
        self.settings_reset = false;
        if relock && self.keys.lock_root.is_some() {
            self.store.reconcile_all(&self.keys)?;
        }
        self.retain(now)?;
        self.reload()
    }

    pub fn recent_sources(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for s in self.items.iter().filter_map(|i| i.meta.as_ref()).map(|m| &m.source) {
            if !s.is_empty() && !out.iter().any(|o| o.eq_ignore_ascii_case(s)) {
                out.push(s.clone());
            }
        }
        out
    }

    pub fn retain(&mut self, now: i64) -> store::Result<usize> {
        self.store.apply_retention(self.settings.history_max_items, self.settings.history_max_days, now)
    }

    fn content_tag(&self, f: &ClipFormats) -> Option<[u8; 32]> {
        let data = f.plain.as_deref().map(str::as_bytes).or(f.png.as_deref())?;
        Some(crypto::dedup_tag(&self.keys.dedup, data))
    }

    pub fn note_own_write(&mut self, f: &ClipFormats) {
        self.own_write = self.content_tag(f).map(|t| (t, Instant::now()));
    }

    fn is_own_echo(&self, f: &ClipFormats) -> bool {
        self.own_write.as_ref().is_some_and(|(tag, at)| at.elapsed() < OWN_ECHO && self.content_tag(f).as_ref() == Some(tag))
    }

    pub fn thumb_for(raw: &RawClip) -> Option<Vec<u8>> {
        raw.formats.png.as_deref().filter(|p| p.len() <= capture::IMAGE_MAX).and_then(capture::thumbnail)
    }

    pub fn store_clip(&mut self, raw: RawClip, thumb: Option<Vec<u8>>, now: i64) -> store::Result<bool> {
        let source = raw.source_exe.clone();
        let ctx = CaptureContext { excluded_apps: &self.settings.excluded_apps, paused: self.paused, own_seq: self.own_seq };
        let CaptureDecision::Store(formats) = capture::decide_capture(raw, &ctx) else { return Ok(false) };
        if self.is_own_echo(&formats) {
            return Ok(false);
        }
        let kind = classify(formats.plain.as_deref(), formats.png.is_some());
        let thumb = thumb.filter(|_| formats.png.is_some());
        self.store.upsert_clip(&self.keys, &NewClip { formats, kind, source, thumb }, now)?;
        self.reload()?;
        Ok(true)
    }

    pub fn vacuum_if_due(&self) {
        let now = now();
        if self.store.vacuum_due(now)
            && let Err(e) = self.store.vacuum(now)
        {
            eprintln!("vacuum failed: {e}");
        }
    }
}

pub struct CoreGuard<'a>(MutexGuard<'a, Core>);

impl Deref for CoreGuard<'_> {
    type Target = Core;

    fn deref(&self) -> &Core {
        &self.0
    }
}

impl DerefMut for CoreGuard<'_> {
    fn deref_mut(&mut self) -> &mut Core {
        &mut self.0
    }
}

impl Drop for CoreGuard<'_> {
    fn drop(&mut self) {
        PANEL_LOCKED.store(self.0.needs_panel_unlock(), Ordering::Relaxed);
    }
}

pub fn lock(core: &Mutex<Core>) -> CoreGuard<'_> {
    let mut core = CoreGuard(core.lock().unwrap_or_else(|e| e.into_inner()));
    if core.refresh()
        && let Some(app) = APP.get()
    {
        let _ = app.emit(event::ITEMS_CHANGED, ());
    }
    core
}

pub fn core(app: &AppHandle) -> CoreGuard<'_> {
    lock(app.state::<Mutex<Core>>().inner())
}

pub fn panel_locked() -> bool {
    PANEL_LOCKED.load(Ordering::Relaxed)
}

pub fn with_unlock<T>(app: &AppHandle, f: impl Fn(&mut Core) -> store::Result<T>) -> Result<T, String> {
    let state = app.state::<Mutex<Core>>();
    let first = f(&mut lock(&state));
    match first {
        Err(StoreError::Locked) => {
            unlock(app)?;
            f(&mut lock(&state)).map_err(|e| e.to_string())
        }
        r => r.map_err(|e| e.to_string()),
    }
}

pub fn apply_settings(app: &AppHandle, change: impl FnOnce(&mut Settings)) -> Result<(), String> {
    let path = settings::file(&app.path().app_local_data_dir().map_err(|e| e.to_string())?);
    let (s, relock) = {
        let state = app.state::<Mutex<Core>>();
        let mut core = lock(&state);
        let mut s = core.settings.clone();
        change(&mut s);
        settings::save(&path, &s).map_err(|e| e.to_string())?;
        let changed = core.settings.lock_all_history != s.lock_all_history;
        core.apply_settings(s.clone(), now()).map_err(|e| e.to_string())?;
        let relock = changed && (s.lock_all_history || core.store.has_locked().unwrap_or(false));
        (s, relock)
    };
    let hook = app.state::<HookHandle>();
    hook.set_takeover(s.takeover);
    hook.set_shortcuts(s.shortcuts);
    hook.set_position(s.panel_position);
    hook.hold_open(Hold::KeepOpen, s.keep_panel_open);
    tray::set_language(app, s.language);
    let _ = app.emit(event::SETTINGS_CHANGED, &s);
    let _ = app.emit(event::ITEMS_CHANGED, ());
    if relock {
        unlock(app)?;
    }
    Ok(())
}

static PROMPT: Mutex<()> = Mutex::new(());

pub fn unlock(app: &AppHandle) -> Result<(), String> {
    let out = try_unlock(app);
    let missing = out.as_ref().err().is_some_and(|e| e == hello::KEY_MISSING);
    if missing || out.is_ok() {
        lock(&app.state::<Mutex<Core>>()).key_missing = missing;
    }
    if missing {
        let _ = app.emit(event::LOCK_ERROR, hello::KEY_MISSING);
        if !panel_window::is_visible(app.state::<crate::PanelCtl>().panel()) {
            tray::open_settings(app);
        }
    }
    out
}

fn try_unlock(app: &AppHandle) -> Result<(), String> {
    let _one = PROMPT.lock().unwrap_or_else(|e| e.into_inner());
    let state = app.state::<Mutex<Core>>();
    let create = {
        let core = lock(&state);
        if core.keys.lock_root.is_some() {
            return Ok(());
        }
        !core.store.has_locked().map_err(|e| e.to_string())?
    };
    let sig = Zeroizing::new(prompt(app, create)?);
    lock(&state).unlock_with(&sig, Instant::now()).map_err(|e| match e {
        StoreError::Crypto(CryptoError::Decrypt) => hello::KEY_MISSING.to_string(),
        e => e.to_string(),
    })?;
    let _ = app.emit(event::ITEMS_CHANGED, ());
    Ok(())
}

fn prompt(app: &AppHandle, create: bool) -> Result<Vec<u8>, String> {
    let hook = app.state::<HookHandle>();
    let ctl = app.state::<crate::PanelCtl>();
    let target = if panel_window::is_visible(ctl.panel()) { ctl.target() } else { panel_window::foreground() };
    let target = target.0 as isize;
    hook.hold_open(Hold::Prompt, true);
    hook.set_released(Release::Prompt, true);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(hello::sign(HWND(target as _), create));
    });
    let sig = rx.recv_timeout(PROMPT_TIMEOUT).unwrap_or_else(|_| Err(hello::CANCELED.to_string()));
    hook.set_released(Release::Prompt, false);
    hook.hold_open(Hold::Prompt, false);
    sig
}

pub fn after_use(app: &AppHandle) {
    if let Some(state) = app.try_state::<Mutex<Core>>() {
        let mut core = lock(&state);
        core.unlock.after_use();
        if core.refresh() {
            let _ = app.emit(event::ITEMS_CHANGED, ());
        }
    }
}

fn relock_on_session_event(app: &AppHandle) {
    {
        let state = app.state::<Mutex<Core>>();
        let mut core = lock(&state);
        let relock = core.settings.relock_on_session_change;
        core.unlock.on_session_event(relock);
        core.refresh();
    }
    let _ = app.emit(event::ITEMS_CHANGED, ());
}

pub fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let _ = APP.set(app.handle().clone());
    let dir = app.path().app_local_data_dir()?;
    let (settings, settings_reset) = settings::load(&settings::file(&dir));
    tray::create(app.handle(), settings.language)?;
    let handle = app.handle().clone();
    let (master, method) = keywrap::load_or_create(&dir.join("master.key"), || tray::set_busy(&handle, true))?;
    tray::set_busy(app.handle(), false);
    let mut core = open_core(&dir, Keys::new(master), settings)?;
    core.key_method = match method {
        keywrap::WrapMethod::Tpm => "tpm",
        keywrap::WrapMethod::Dpapi => "dpapi",
    };
    core.settings_reset = settings_reset;
    hook::hold_open(Hold::KeepOpen, core.settings.keep_panel_open);
    PANEL_LOCKED.store(core.needs_panel_unlock(), Ordering::Relaxed);
    app.manage(Mutex::new(core));
    if settings_reset {
        tray::open_settings(app.handle());
    }
    let (tx, rx) = mpsc::channel();
    let handle = app.handle().clone();
    clipboard::start_listener(tx, move || relock_on_session_event(&handle));
    let handle = app.handle().clone();
    std::thread::spawn(move || capture_loop(handle, rx));
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(RELOCK_TICK);
            drop(lock(&handle.state::<Mutex<Core>>()));
        }
    });
    Ok(())
}

fn open_core(dir: &Path, keys: Keys, settings: Settings) -> store::Result<Core> {
    let store = Store::open(&dir.join("clipmoor.db"))?;
    Core::new(store, keys, settings, now())
}

fn capture_loop(app: AppHandle, rx: Receiver<RawClip>) {
    let state = app.state::<Mutex<Core>>();
    lock(&state).vacuum_if_due();
    let mut since_retention = 0;
    for raw in rx {
        let thumb = Core::thumb_for(&raw);
        let mut core = lock(&state);
        match core.store_clip(raw, thumb, now()) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(e) => {
                eprintln!("capture failed: {e}");
                continue;
            }
        }
        since_retention += 1;
        if since_retention >= RETENTION_EVERY {
            since_retention = 0;
            if let Err(e) = core.retain(now()).and_then(|_| core.reload()) {
                eprintln!("retention failed: {e}");
            }
            core.vacuum_if_due();
        }
        drop(core);
        let _ = app.emit(event::ITEMS_CHANGED, ());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::ClipMarkers;
    use crate::classify::Kind;
    use crate::crypto::MasterKey;
    use crate::lock::UnlockDuration;

    impl Core {
        fn capture(&mut self, raw: RawClip, now: i64) -> store::Result<bool> {
            let thumb = Self::thumb_for(&raw);
            self.store_clip(raw, thumb, now)
        }
    }

    fn core() -> Core {
        Core::new(Store::open_in_memory().unwrap(), Keys::new(MasterKey::generate()), Settings::default(), 0).unwrap()
    }

    fn raw(text: &str, seq: u32) -> RawClip {
        RawClip { formats: ClipFormats { plain: Some(text.into()), ..Default::default() }, markers: ClipMarkers::default(), seq, source_exe: "notepad.exe".into() }
    }

    fn texts(c: &Core) -> Vec<&str> {
        c.items.iter().map(|i| i.search.as_deref().unwrap()).collect()
    }

    #[test]
    fn recopy_moves_to_top_and_own_write_is_skipped() {
        let mut c = core();
        assert!(c.capture(raw("a", 1), 100).unwrap());
        assert!(c.capture(raw("b", 2), 200).unwrap());
        assert!(c.capture(raw("a", 3), 300).unwrap());
        assert_eq!(texts(&c), ["a", "b"]);
        c.own_seq = Some(4);
        assert!(!c.capture(raw("c", 4), 400).unwrap());
        assert_eq!(texts(&c), ["a", "b"]);
    }

    #[test]
    fn own_write_echo_is_skipped_even_with_another_sequence() {
        let mut c = core();
        let mut rich = raw("a", 1);
        rich.formats.html = Some(b"<b>a</b>".to_vec());
        c.capture(rich, 100).unwrap();
        c.capture(raw("b", 2), 150).unwrap();
        c.note_own_write(&ClipFormats { plain: Some("a".into()), ..Default::default() });
        assert!(!c.capture(raw("a", 9), 200).unwrap());
        assert_eq!(texts(&c), ["b", "a"]);
        assert!(c.capture(raw("c", 10), 300).unwrap());
    }

    fn relock(c: &mut Core) {
        c.unlock.on_session_event(true);
        c.refresh();
    }

    #[test]
    fn lock_all_hides_history_until_unlocked_and_seals_pending_copies() {
        let mut c = Core::new(Store::open_in_memory().unwrap(), Keys::new(MasterKey::generate()), Settings { lock_all_history: true, ..Settings::default() }, 0).unwrap();
        assert!(c.capture(raw("a", 1), 100).unwrap());
        assert!(c.needs_panel_unlock());
        c.unlock_with(b"imza", Instant::now()).unwrap();
        assert!(!c.needs_panel_unlock());
        assert!(c.capture(raw("b", 2), 200).unwrap());
        assert_eq!(texts(&c), ["b", "a"]);
        assert!(c.items.iter().all(|i| i.locked));
        relock(&mut c);
        assert!(c.needs_panel_unlock());
        assert!(c.items.iter().all(|i| i.search.is_none()));
    }

    #[test]
    fn wrong_signature_is_rejected_and_items_stay_hidden() {
        let mut c = core();
        c.capture(raw("a", 1), 100).unwrap();
        c.unlock_with(b"imza", Instant::now()).unwrap();
        c.store.set_item_locked(&c.keys, c.items[0].id, true).unwrap();
        relock(&mut c);
        assert!(c.unlock_with(b"baska", Instant::now()).is_err());
        assert!(c.keys.lock_root.is_none() && c.items[0].search.is_none());
        c.unlock_with(b"imza", Instant::now()).unwrap();
        assert_eq!(texts(&c), ["a"]);
    }

    #[test]
    fn relock_reports_a_change_once() {
        let mut c = core();
        c.unlock_with(b"imza", Instant::now()).unwrap();
        c.unlock.on_session_event(true);
        assert!(c.refresh());
        assert!(!c.refresh());
    }

    #[test]
    fn every_time_relocks_after_use() {
        let mut c = Core::new(Store::open_in_memory().unwrap(), Keys::new(MasterKey::generate()), Settings { unlock_duration: UnlockDuration::EveryTime, ..Settings::default() }, 0).unwrap();
        c.unlock_with(b"imza", Instant::now()).unwrap();
        c.unlock.after_use();
        c.refresh();
        assert!(c.keys.lock_root.is_none());
    }

    #[test]
    fn applied_settings_take_effect_without_restart() {
        let mut c = core();
        for (i, t) in ["a", "b", "c"].into_iter().enumerate() {
            c.capture(raw(t, i as u32 + 1), 100 + i as i64).unwrap();
        }
        c.unlock_with(b"imza", Instant::now()).unwrap();
        c.apply_settings(Settings { history_max_items: 2, lock_all_history: true, unlock_duration: UnlockDuration::EveryTime, ..Settings::default() }, 200).unwrap();
        assert_eq!(texts(&c), ["c", "b"]);
        assert!(c.items.iter().all(|i| i.locked));
        c.unlock.after_use();
        c.refresh();
        assert!(c.needs_panel_unlock());
        c.unlock_with(b"imza", Instant::now()).unwrap();
        c.apply_settings(Settings::default(), 200).unwrap();
        assert!(c.items.iter().all(|i| !i.locked));
    }

    #[test]
    fn recent_sources_are_distinct_newest_first() {
        let mut c = core();
        let mut r = raw("a", 1);
        r.source_exe = "Code.exe".into();
        c.capture(r, 100).unwrap();
        c.capture(raw("b", 2), 200).unwrap();
        c.capture(raw("c", 3), 300).unwrap();
        assert_eq!(c.recent_sources(), ["notepad.exe", "Code.exe"]);
    }

    #[test]
    fn recent_sources_fold_case_and_skip_hidden_locked_items() {
        let mut c = core();
        for (i, (text, exe)) in [("a", "Code.exe"), ("b", "notepad.exe"), ("c", "CODE.EXE"), ("d", "KeePassXC.exe")].into_iter().enumerate() {
            let mut r = raw(text, i as u32 + 1);
            r.source_exe = exe.into();
            c.capture(r, 100 + i as i64).unwrap();
        }
        assert_eq!(c.recent_sources(), ["KeePassXC.exe", "CODE.EXE", "notepad.exe"]);
        c.unlock_with(b"imza", Instant::now()).unwrap();
        c.store.set_item_locked(&c.keys, c.items[0].id, true).unwrap();
        relock(&mut c);
        assert_eq!(c.recent_sources(), ["CODE.EXE", "notepad.exe"]);
    }

    #[test]
    fn image_clip_is_classified_and_thumbnailed() {
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::new(300, 300).write_to(&mut png, image::ImageFormat::Png).unwrap();
        let mut c = core();
        let mut r = raw("", 1);
        r.formats.png = Some(png.into_inner());
        assert!(c.capture(r, 100).unwrap());
        let item = &c.items[0];
        assert_eq!(item.meta.as_ref().unwrap().kind, Kind::Image);
        let thumb = image::load_from_memory(&c.store.thumb(&c.keys, item.id).unwrap().unwrap()).unwrap();
        assert_eq!(thumb.height(), 96);
    }
}
