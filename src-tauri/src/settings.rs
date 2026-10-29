use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::keys::Shortcuts;
use crate::lock::UnlockDuration;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    #[default]
    System,
    Tr,
    En,
}

impl Language {
    pub fn is_turkish(self, system_locale: &str) -> bool {
        match self {
            Language::System => system_locale.to_ascii_lowercase().starts_with("tr"),
            Language::Tr => true,
            Language::En => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PanelPosition {
    #[default]
    Caret,
    Mouse,
    Center,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CustomColors {
    pub enabled: bool,
    pub dark: BTreeMap<String, String>,
    pub light: BTreeMap<String, String>,
    pub preset: Option<String>,
}

const COLOR_KEYS: [&str; 9] = ["ground", "card", "text", "selection", "typeText", "typeImage", "typeLink", "typeCode", "typeLocked"];

fn renamed(key: &str) -> &str {
    match key {
        "chip" => "card",
        "signal" => "selection",
        k => k,
    }
}

fn migrate_set(set: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    set.iter().filter_map(|(k, v)| {
        let key = renamed(k);
        (COLOR_KEYS.contains(&key) && (key == k || !set.contains_key(key))).then(|| (key.to_string(), v.clone()))
    }).collect()
}

impl CustomColors {
    fn migrate(&mut self) {
        self.dark = migrate_set(&self.dark);
        self.light = migrate_set(&self.light);
        if self.preset.as_deref() == Some("haki") {
            self.preset = Some("haki-gri".into());
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub history_max_items: u32,
    pub history_max_days: u32,
    pub takeover: bool,
    pub language: Language,
    pub close_on_paste: bool,
    pub unlock_duration: UnlockDuration,
    pub relock_on_session_change: bool,
    pub lock_all_history: bool,
    pub panel_position: PanelPosition,
    pub theme: Theme,
    pub excluded_apps: Vec<String>,
    pub keep_panel_open: bool,
    pub shortcuts: Shortcuts,
    pub custom_colors: CustomColors,
    pub layout: String,
    pub compact: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            history_max_items: 500,
            history_max_days: 30,
            takeover: true,
            language: Language::System,
            close_on_paste: true,
            unlock_duration: UnlockDuration::WhileRunning,
            relock_on_session_change: true,
            lock_all_history: false,
            panel_position: PanelPosition::Caret,
            theme: Theme::System,
            excluded_apps: Vec::new(),
            keep_panel_open: false,
            shortcuts: Shortcuts::default(),
            custom_colors: CustomColors::default(),
            layout: "list".into(),
            compact: false,
        }
    }
}

pub fn file(dir: &Path) -> PathBuf {
    dir.join("settings.json")
}

pub fn load(path: &Path) -> (Settings, bool) {
    let Ok(text) = std::fs::read_to_string(path) else { return (Settings::default(), false) };
    match serde_json::from_str::<Settings>(&text) {
        Ok(mut s) => {
            s.custom_colors.migrate();
            (s, false)
        }
        Err(e) => {
            eprintln!("settings unreadable ({:?}), using defaults", e.classify());
            if let Err(e) = std::fs::write(path.with_extension("json.bak"), &text) {
                eprintln!("settings backup failed: {e}");
            }
            (Settings::default(), true)
        }
    }
}

pub fn save(path: &Path, s: &Settings) -> std::io::Result<()> {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!("json.{}-{}.tmp", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&tmp, serde_json::to_vec_pretty(s)?)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(load(&d.path().join("settings.json")), (Settings::default(), false));
    }

    #[test]
    fn corrupt_file_gives_defaults_flags_reset_and_keeps_a_backup() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, "{ bozuk").unwrap();
        assert_eq!(load(&p), (Settings::default(), true));
        assert_eq!(std::fs::read_to_string(p.with_extension("json.bak")).unwrap(), "{ bozuk");
    }

    #[test]
    fn concurrent_saves_never_collide_on_the_temp_file() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        let saves: Vec<_> = (1..=16u32)
            .map(|n| {
                let p = p.clone();
                std::thread::spawn(move || (0..20).try_for_each(|_| save(&p, &Settings { history_max_items: n, ..Settings::default() })))
            })
            .collect();
        for s in saves {
            s.join().unwrap().unwrap();
        }
        assert!(!load(&p).1);
    }

    #[test]
    fn partial_file_fills_defaults() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        std::fs::write(&p, r#"{"historyMaxItems":100,"theme":"dark"}"#).unwrap();
        let (s, _) = load(&p);
        assert_eq!((s.history_max_items, s.theme, s.history_max_days, s.close_on_paste), (100, Theme::Dark, 30, true));
    }

    #[test]
    fn old_file_without_shortcuts_gets_the_defaults() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r#"{"takeover":false,"theme":"dark"}"#).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset && !s.takeover);
        assert_eq!(s.shortcuts, Shortcuts::default());
    }

    #[test]
    fn partial_shortcuts_fill_defaults_and_modifiers() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r#"{"shortcuts":{"open":{"ctrl":true,"shift":true,"key":86},"quick":{"alt":true}}}"#).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset);
        let sc = s.shortcuts;
        assert_eq!((sc.open.key, sc.open.mods.ctrl, sc.open.mods.shift, sc.open.mods.win), (86, true, true, false));
        assert!(sc.quick.alt && !sc.quick.ctrl);
        assert_eq!(sc.pin, Shortcuts::default().pin);
    }

    #[test]
    fn old_file_with_the_removed_settings_shortcut_still_loads() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r#"{"takeover":false,"shortcuts":{"pin":{"ctrl":true,"key":81},"settings":{"ctrl":true,"key":188}}}"#).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset && !s.takeover);
        assert_eq!(s.shortcuts.pin.key, 81);
    }

    #[test]
    fn roundtrip() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        let mut shortcuts = Shortcuts::default();
        shortcuts.close.key = 0x57;
        let custom_colors = CustomColors { enabled: true, dark: [("selection".into(), "#00C2FF".into())].into(), light: [("ground".into(), "#FAFAFA".into())].into(), preset: Some("kum".into()) };
        let s = Settings { excluded_apps: vec!["KeePassXC.exe".into()], unlock_duration: UnlockDuration::FiveMinutes, shortcuts, custom_colors, layout: "shelf".into(), compact: true, ..Settings::default() };
        save(&p, &s).unwrap();
        assert_eq!(load(&p), (s, false));
    }

    #[test]
    fn old_file_without_custom_colors_loads_them_off_and_empty() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r#"{"theme":"dark"}"#).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset && !s.custom_colors.enabled);
        assert!(s.custom_colors.dark.is_empty() && s.custom_colors.light.is_empty());
        assert_eq!(s.custom_colors.preset, None);
    }

    #[test]
    fn custom_colors_without_a_preset_id_still_load() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r##"{"customColors":{"enabled":true,"dark":{"ground":"#101820"}}}"##).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset && s.custom_colors.enabled && s.custom_colors.preset.is_none());
        std::fs::write(&p, r##"{"customColors":{"enabled":true,"preset":"kum"}}"##).unwrap();
        assert_eq!(load(&p).0.custom_colors.preset.as_deref(), Some("kum"));
    }

    #[test]
    fn old_colour_names_load_under_the_new_ones() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r##"{"customColors":{"enabled":true,"dark":{"chip":"#222222","signal":"#00C2FF","typeColor":"#00FF00","ground":"#101820"},"light":{"card":"#FFFFFF","chip":"#EEEEEE"},"preset":"haki"}}"##).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset);
        let set = |pairs: &[(&str, &str)]| pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<BTreeMap<_, _>>();
        assert_eq!(s.custom_colors, CustomColors { enabled: true, dark: set(&[("card", "#222222"), ("selection", "#00C2FF"), ("ground", "#101820")]), light: set(&[("card", "#FFFFFF")]), preset: Some("haki-gri".into()) });
    }

    #[test]
    fn colour_migration_leaves_new_files_and_empty_presets_alone() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r##"{"customColors":{"enabled":false,"dark":{"selection":"#00C2FF"},"preset":null}}"##).unwrap();
        let c = load(&p).0.custom_colors;
        assert_eq!(c.dark.get("selection").map(String::as_str), Some("#00C2FF"));
        assert_eq!(c.preset, None);
    }

    #[test]
    fn custom_colors_load_from_camel_case_and_fill_missing_sets() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r##"{"customColors":{"enabled":true,"dark":{"ground":"#101820","typeLink":"#FF8800"}}}"##).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset && s.custom_colors.enabled);
        assert_eq!(s.custom_colors.dark.get("ground").map(String::as_str), Some("#101820"));
        assert_eq!(s.custom_colors.dark.get("typeLink").map(String::as_str), Some("#FF8800"));
        assert!(s.custom_colors.light.is_empty());
    }

    #[test]
    fn system_language_follows_locale_and_explicit_choice_wins() {
        assert!(Language::System.is_turkish("tr-TR"));
        assert!(!Language::System.is_turkish("en-US"));
        assert!(!Language::System.is_turkish(""));
        assert!(Language::Tr.is_turkish("en-US"));
        assert!(!Language::En.is_turkish("tr-TR"));
    }

    #[test]
    fn old_file_without_layout_opens_the_list_at_normal_density() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r#"{"theme":"dark","keepPanelOpen":true}"#).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset && s.keep_panel_open);
        assert_eq!((s.layout.as_str(), s.compact), ("list", false));
    }

    #[test]
    fn unknown_layout_ids_load_untouched_for_the_frontend_to_resolve() {
        let d = tempfile::tempdir().unwrap();
        let p = file(d.path());
        std::fs::write(&p, r#"{"layout":"grid","compact":true}"#).unwrap();
        let (s, reset) = load(&p);
        assert!(!reset && s.compact);
        assert_eq!(s.layout, "grid");
    }

    #[test]
    fn defaults_match_design() {
        let s = Settings::default();
        assert_eq!((s.history_max_items, s.history_max_days, s.takeover, s.close_on_paste, s.relock_on_session_change), (500, 30, true, true, true));
        assert!(!s.lock_all_history);
        assert_eq!((s.unlock_duration, s.panel_position, s.theme), (UnlockDuration::WhileRunning, PanelPosition::Caret, Theme::System));
        assert_eq!((s.layout.as_str(), s.compact), ("list", false));
    }
}
