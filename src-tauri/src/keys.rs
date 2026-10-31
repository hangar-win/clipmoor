use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PasteMode {
    Paste,
    PlainPaste,
    Copy,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PanelKey {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Activate { mode: PasteMode },
    Esc,
    Quick { n: u8 },
    Pin,
    Text { s: String },
    Backspace,
    DeleteWord,
    SelectAll,
    Tab { back: bool },
    TogglePreview,
    SourceMenu,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Mods {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
}

impl Mods {
    const NONE: Mods = Mods { ctrl: false, shift: false, alt: false, win: false };
    const CTRL: Mods = Mods { ctrl: true, ..Mods::NONE };
    const SHIFT: Mods = Mods { shift: true, ..Mods::NONE };
    const WIN: Mods = Mods { win: true, ..Mods::NONE };
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Chord {
    #[serde(flatten)]
    pub mods: Mods,
    pub key: u32,
}

const WIN_V: Chord = Chord { mods: Mods::WIN, key: VK_V };

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct Shortcuts {
    pub open: Chord,
    pub paste: Chord,
    pub copy: Chord,
    pub plain_paste: Chord,
    pub pin: Chord,
    pub preview: Chord,
    pub close: Chord,
    pub source: Chord,
    pub quick: Mods,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            open: WIN_V,
            paste: Chord { mods: Mods::NONE, key: VK_RETURN },
            copy: Chord { mods: Mods::SHIFT, key: VK_RETURN },
            plain_paste: Chord { mods: Mods { ctrl: true, shift: true, ..Mods::NONE }, key: VK_RETURN },
            pin: Chord { mods: Mods::CTRL, key: VK_P },
            preview: Chord { mods: Mods::CTRL, key: VK_SPACE },
            close: Chord { mods: Mods::NONE, key: VK_ESCAPE },
            source: Chord { mods: Mods::CTRL, key: VK_K },
            quick: Mods::CTRL,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub vk: u32,
    pub down: bool,
    pub ours: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HookAction {
    Pass,
    Swallow,
    Open,
    PassWithStartMask,
    Forward(PanelKey),
    Translate,
    PasteText,
}

const VK_BACK: u32 = 0x08;
const VK_TAB: u32 = 0x09;
const VK_RETURN: u32 = 0x0D;
const VK_SHIFT: u32 = 0x10;
const VK_CONTROL: u32 = 0x11;
const VK_MENU: u32 = 0x12;
const VK_CAPITAL: u32 = 0x14;
const VK_ESCAPE: u32 = 0x1B;
const VK_SPACE: u32 = 0x20;
const VK_PRIOR: u32 = 0x21;
const VK_NEXT: u32 = 0x22;
const VK_END: u32 = 0x23;
const VK_HOME: u32 = 0x24;
const VK_LEFT: u32 = 0x25;
const VK_UP: u32 = 0x26;
const VK_RIGHT: u32 = 0x27;
const VK_DOWN: u32 = 0x28;
const VK_DELETE: u32 = 0x2E;
const VK_A: u32 = 0x41;
const VK_K: u32 = 0x4B;
const VK_V: u32 = 0x56;
const VK_P: u32 = 0x50;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const MODIFIERS: [u32; 11] = [VK_SHIFT, VK_CONTROL, VK_MENU, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, VK_LWIN, VK_RWIN];

pub fn mode_for(ctrl: bool, shift: bool) -> PasteMode {
    match (ctrl, shift) {
        (true, true) => PasteMode::PlainPaste,
        (false, true) => PasteMode::Copy,
        _ => PasteMode::Paste,
    }
}

fn generic_modifier(vk: u32) -> Option<(u32, u32, u32)> {
    match vk {
        0xA0 | 0xA1 => Some((VK_SHIFT, 0xA0, 0xA1)),
        0xA2 | 0xA3 => Some((VK_CONTROL, 0xA2, 0xA3)),
        0xA4 | 0xA5 => Some((VK_MENU, 0xA4, 0xA5)),
        _ => None,
    }
}

fn is_typable(vk: u32) -> bool {
    matches!(vk, VK_SPACE | 0x30..=0x39 | 0x41..=0x5A | 0x60..=0x6F | 0xBA..=0xC0 | 0xDB..=0xDF | 0xE2)
}

#[derive(Clone, Copy, Debug)]
pub enum Release {
    Dialog = 1,
    Prompt = 2,
}

pub struct HookState {
    takeover: bool,
    panel_open: bool,
    released: u8,
    win_down: bool,
    win_consumed: bool,
    held: HashSet<u32>,
    state: [u8; 256],
    probe: Option<fn(u32) -> bool>,
    shortcuts: Shortcuts,
    recording: bool,
}

impl HookState {
    pub fn new(takeover: bool) -> Self {
        Self { takeover, panel_open: false, released: 0, win_down: false, win_consumed: false, held: HashSet::new(), state: [0; 256], probe: None, shortcuts: Shortcuts::default(), recording: false }
    }

    pub fn with_probe(takeover: bool, probe: fn(u32) -> bool) -> Self {
        Self { probe: Some(probe), ..Self::new(takeover) }
    }

    pub fn reset_keys(&mut self) {
        for s in &mut self.state {
            *s &= !0x80;
        }
        self.win_down = false;
        self.win_consumed = false;
        self.held.clear();
    }

    fn resync_modifiers(&mut self, probe: fn(u32) -> bool) {
        for vk in MODIFIERS {
            let i = vk as usize;
            self.state[i] = (self.state[i] & !0x80) | if probe(vk) { 0x80 } else { 0 };
        }
    }

    fn held_mods(&self) -> Mods {
        Mods { ctrl: self.pressed(VK_CONTROL), shift: self.pressed(VK_SHIFT), alt: self.pressed(VK_MENU), win: self.win_down }
    }

    fn opens(&mut self, vk: u32) -> bool {
        let open = self.shortcuts.open;
        let chords = [(open != WIN_V).then_some(open), self.takeover.then_some(WIN_V)];
        let wanted: Vec<Mods> = chords.into_iter().flatten().filter(|c| c.key == vk).map(|c| c.mods).collect();
        if wanted.is_empty() {
            return false;
        }
        if let Some(probe) = self.probe {
            self.resync_modifiers(probe);
            self.win_down = self.pressed(VK_LWIN) || self.pressed(VK_RWIN);
        }
        wanted.contains(&self.held_mods())
    }

    pub fn set_takeover(&mut self, on: bool) {
        self.takeover = on;
    }

    pub fn set_shortcuts(&mut self, s: Shortcuts) {
        self.shortcuts = s;
    }

    pub fn set_recording(&mut self, on: bool) {
        self.recording = on;
    }

    pub fn set_panel_open(&mut self, open: bool) {
        self.panel_open = open;
    }

    pub fn panel_open(&self) -> bool {
        self.panel_open
    }

    pub fn set_released(&mut self, reason: Release, on: bool) {
        if on { self.released |= reason as u8 } else { self.released &= !(reason as u8) }
    }

    pub fn key_state(&self) -> &[u8; 256] {
        &self.state
    }

    pub fn mods(&self) -> (bool, bool) {
        (self.pressed(VK_CONTROL), self.pressed(VK_SHIFT))
    }

    fn pressed(&self, vk: u32) -> bool {
        self.state[(vk & 0xFF) as usize] & 0x80 != 0
    }

    fn track(&mut self, ev: KeyEvent) {
        let i = (ev.vk & 0xFF) as usize;
        if ev.down {
            if ev.vk == VK_CAPITAL && self.state[i] & 0x80 == 0 {
                self.state[i] ^= 0x01;
            }
            self.state[i] |= 0x80;
        } else {
            self.state[i] &= !0x80;
        }
        if let Some((generic, left, right)) = generic_modifier(ev.vk) {
            let down = (self.state[left as usize] | self.state[right as usize]) & 0x80;
            self.state[generic as usize] = (self.state[generic as usize] & !0x80) | down;
        }
    }

    pub fn decide(&mut self, ev: KeyEvent) -> HookAction {
        if ev.ours {
            return HookAction::Pass;
        }
        let repeat = ev.down && self.pressed(ev.vk);
        self.track(ev);

        if ev.vk == VK_LWIN || ev.vk == VK_RWIN {
            self.win_down = ev.down;
            if !ev.down && self.win_consumed {
                self.win_consumed = false;
                return HookAction::PassWithStartMask;
            }
            return HookAction::Pass;
        }

        if !ev.down {
            return if self.held.remove(&ev.vk) { HookAction::Swallow } else { HookAction::Pass };
        }
        if repeat {
            if !self.held.contains(&ev.vk) {
                return HookAction::Pass;
            }
            if self.panel_open && self.released == 0 && !self.win_down {
                let action = self.panel_action(ev.vk);
                if matches!(action, HookAction::Forward(PanelKey::Up | PanelKey::Down | PanelKey::Left | PanelKey::Right | PanelKey::Backspace | PanelKey::DeleteWord)) {
                    return action;
                }
            }
            return HookAction::Swallow;
        }

        if self.recording {
            return HookAction::Pass;
        }
        if self.opens(ev.vk) {
            self.win_consumed |= self.win_down;
            return if self.held.insert(ev.vk) { HookAction::Open } else { HookAction::Swallow };
        }

        if !self.panel_open || self.released != 0 || self.win_down {
            return HookAction::Pass;
        }
        let action = self.panel_action(ev.vk);
        if action != HookAction::Pass {
            self.held.insert(ev.vk);
        }
        action
    }

    fn bound(&self, vk: u32, held: Mods) -> Option<PanelKey> {
        let s = &self.shortcuts;
        let digit = match vk {
            0x31..=0x39 => Some(vk - 0x30),
            0x61..=0x69 => Some(vk - 0x60),
            _ => None,
        };
        let quick = digit.map(|n| (Chord { mods: s.quick, key: vk }, PanelKey::Quick { n: n as u8 }));
        let activate = |mode| PanelKey::Activate { mode };
        let chords = [
            (s.paste, activate(PasteMode::Paste)),
            (s.copy, activate(PasteMode::Copy)),
            (s.plain_paste, activate(PasteMode::PlainPaste)),
            (s.pin, PanelKey::Pin),
            (s.preview, PanelKey::TogglePreview),
            (s.close, PanelKey::Esc),
            (s.source, PanelKey::SourceMenu),
        ];
        let weight = |m: Mods| m.ctrl as u8 + m.shift as u8 + m.alt as u8;
        chords.into_iter().chain(quick).filter(|(c, _)| c.key == vk && covers(c.mods, held)).rev().max_by_key(|(c, _)| weight(c.mods)).map(|(_, k)| k)
    }

    fn panel_action(&self, vk: u32) -> HookAction {
        let held = self.held_mods();
        let (ctrl, shift, alt) = (held.ctrl, held.shift, held.alt);
        if vk == VK_ESCAPE && ctrl {
            return HookAction::Pass;
        }
        if let Some(key) = self.bound(vk, held) {
            return HookAction::Forward(key);
        }
        if alt && !ctrl {
            return HookAction::Pass;
        }
        let command = ctrl && !alt;
        let key = match vk {
            VK_UP => PanelKey::Up,
            VK_DOWN => PanelKey::Down,
            VK_HOME => PanelKey::Home,
            VK_END => PanelKey::End,
            VK_BACK if command => PanelKey::DeleteWord,
            VK_BACK => PanelKey::Backspace,
            VK_A if command => PanelKey::SelectAll,
            VK_V if command => return HookAction::PasteText,
            VK_TAB => PanelKey::Tab { back: shift },
            VK_LEFT => PanelKey::Left,
            VK_RIGHT => PanelKey::Right,
            _ if !command && is_typable(vk) => return HookAction::Translate,
            VK_MENU | 0xA4 | 0xA5 | VK_DELETE | VK_PRIOR | VK_NEXT => return HookAction::Swallow,
            _ => return HookAction::Pass,
        };
        HookAction::Forward(key)
    }
}

fn covers(bound: Mods, held: Mods) -> bool {
    let extra_ctrl = held.ctrl && !held.alt && !bound.ctrl;
    !bound.win && !held.win && bound.alt == held.alt && (bound.ctrl == held.ctrl || extra_ctrl) && (held.shift || !bound.shift)
}

#[cfg(test)]
mod tests {
    use super::*;
    const CTRL_SHIFT: Mods = Mods { ctrl: true, shift: true, alt: false, win: false };
    fn chord(key: u32, mods: Mods) -> Chord { Chord { mods, key } }
    fn d(vk: u32) -> KeyEvent { KeyEvent { vk, down: true, ours: false } }
    fn u(vk: u32) -> KeyEvent { KeyEvent { vk, down: false, ours: false } }

    #[test]
    fn win_v_opens_and_masks_start() {
        let mut h = HookState::new(true);
        assert_eq!(h.decide(d(0x5B)), HookAction::Pass);
        assert_eq!(h.decide(d(0x56)), HookAction::Open);
        h.set_panel_open(true);
        assert_eq!(h.decide(u(0x56)), HookAction::Swallow);
        assert_eq!(h.decide(u(0x5B)), HookAction::PassWithStartMask);
    }

    #[test]
    fn takeover_off_passes_win_v() {
        let mut h = HookState::new(false);
        assert_eq!(h.decide(d(0x5B)), HookAction::Pass);
        assert_eq!(h.decide(d(0x56)), HookAction::Pass);
        assert_eq!(h.decide(u(0x5B)), HookAction::Pass);
    }

    #[test]
    fn other_win_shortcuts_untouched() {
        let mut h = HookState::new(true);
        for ev in [d(0x5B), d(0x45), u(0x45), u(0x5B)] {
            assert_eq!(h.decide(ev), HookAction::Pass);
        }
    }

    #[test]
    fn enter_modes() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::Paste }));
        h.decide(u(0x0D));
        assert_eq!(h.decide(d(0xA0)), HookAction::Pass);
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::Copy }));
        h.decide(u(0x0D));
        assert_eq!(h.decide(d(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::PlainPaste }));
    }

    #[test]
    fn panel_navigation_and_shortcuts() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
        assert_eq!(h.decide(u(0x28)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x1B)), HookAction::Forward(PanelKey::Esc));
        assert_eq!(h.decide(d(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x33)), HookAction::Forward(PanelKey::Quick { n: 3 }));
        assert_eq!(h.decide(d(0x50)), HookAction::Forward(PanelKey::Pin));
        assert_eq!(h.decide(u(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x41)), HookAction::Translate);
        assert_eq!(h.decide(d(0x08)), HookAction::Forward(PanelKey::Backspace));
    }

    #[test]
    fn our_injected_events_pass() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        assert_eq!(h.decide(KeyEvent { vk: 0x56, down: true, ours: true }), HookAction::Pass);
    }

    #[test]
    fn releases_after_close_are_swallowed() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        h.decide(d(0xA2));
        h.decide(d(0x33));
        h.set_panel_open(false);
        assert_eq!(h.decide(d(0x33)), HookAction::Swallow);
        assert_eq!(h.decide(u(0x33)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x41)), HookAction::Pass);
    }

    #[test]
    fn key_state_tracks_modifiers() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        h.decide(d(0xA0));
        assert_ne!(h.key_state()[0x10] & 0x80, 0);
        assert_eq!(h.mods(), (false, true));
        h.decide(u(0xA0));
        assert_eq!(h.key_state()[0x10] & 0x80, 0);
    }

    fn nothing_down(_: u32) -> bool { false }

    #[test]
    fn stale_win_down_does_not_turn_plain_v_into_open() {
        let mut h = HookState::with_probe(true, nothing_down);
        assert_eq!(h.decide(d(0x5B)), HookAction::Pass);
        assert_eq!(h.decide(d(0x56)), HookAction::Pass);
        assert_eq!(h.decide(u(0x56)), HookAction::Pass);
        assert_eq!(h.decide(d(0x56)), HookAction::Pass);
    }

    #[test]
    fn confirmed_win_v_opens_and_resyncs_modifiers() {
        let mut h = HookState::with_probe(true, |vk| vk == 0x5B);
        h.decide(d(0xA2));
        h.decide(d(0x5B));
        assert_eq!(h.decide(d(0x56)), HookAction::Open);
        assert_eq!(h.mods(), (false, false));
        h.set_panel_open(true);
        h.decide(u(0x56));
        h.decide(u(0x5B));
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::Paste }));
    }

    #[test]
    fn reset_forgets_keys_lost_to_the_secure_desktop() {
        let mut h = open_panel();
        h.decide(d(0x5B));
        h.decide(d(0xA2));
        h.decide(d(0x28));
        h.reset_keys();
        assert_eq!(h.mods(), (false, false));
        assert_eq!(h.decide(u(0x28)), HookAction::Pass);
        assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
        assert_eq!(h.decide(d(0x56)), HookAction::Translate);
    }

    #[test]
    fn out_of_range_vk_does_not_panic() {
        let mut h = open_panel();
        assert_eq!(h.decide(d(0x1FF)), HookAction::Pass);
        assert_eq!(h.decide(d(0x1FF)), HookAction::Pass);
    }

    #[test]
    fn click_mode_matches_enter() {
        assert_eq!(mode_for(false, false), PasteMode::Paste);
        assert_eq!(mode_for(false, true), PasteMode::Copy);
        assert_eq!(mode_for(true, true), PasteMode::PlainPaste);
        assert_eq!(mode_for(true, false), PasteMode::Paste);
    }

    #[test]
    fn altgr_char_translates_instead_of_shortcut() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        h.decide(d(0xA2));
        h.decide(d(0xA5));
        assert_eq!(h.decide(d(0x51)), HookAction::Translate);
        assert_eq!(h.decide(d(0x50)), HookAction::Translate);
    }

    #[test]
    fn release_of_key_pressed_before_open_passes() {
        let mut h = HookState::new(true);
        assert_eq!(h.decide(d(0xA0)), HookAction::Pass);
        h.set_panel_open(true);
        assert_eq!(h.decide(u(0xA0)), HookAction::Pass);
    }

    #[test]
    fn caps_lock_toggles_low_bit_once_per_press() {
        let mut h = HookState::new(true);
        h.decide(d(0x14));
        h.decide(d(0x14));
        assert_eq!(h.key_state()[0x14] & 0x01, 1);
        h.decide(u(0x14));
        h.decide(d(0x14));
        assert_eq!(h.key_state()[0x14] & 0x01, 0);
    }

    fn passes_untouched(h: &mut HookState, chord: &[u32]) {
        for &vk in chord {
            assert_eq!(h.decide(d(vk)), HookAction::Pass, "down {vk:#x}");
        }
        for &vk in chord.iter().rev() {
            assert_eq!(h.decide(u(vk)), HookAction::Pass, "up {vk:#x}");
        }
    }

    const SYSTEM_CHORDS: [&[u32]; 5] = [&[0x5B, 0xA0, 0x53], &[0x5B, 0x45], &[0x5B, 0x44], &[0xA4, 0x09], &[0xA2, 0x43]];

    fn win_v_cycle(h: &mut HookState) {
        assert_eq!(h.decide(d(0x5B)), HookAction::Pass);
        assert_eq!(h.decide(d(0x56)), HookAction::Open);
        h.set_panel_open(true);
        assert_eq!(h.decide(u(0x56)), HookAction::Swallow);
        assert_eq!(h.decide(u(0x5B)), HookAction::PassWithStartMask);
        h.set_panel_open(false);
    }

    #[test]
    fn system_shortcuts_pass_before_any_win_v() {
        let mut h = HookState::new(true);
        for chord in SYSTEM_CHORDS {
            passes_untouched(&mut h, chord);
        }
    }

    #[test]
    fn system_shortcuts_pass_after_open_close_cycle() {
        let mut h = HookState::new(true);
        win_v_cycle(&mut h);
        for chord in SYSTEM_CHORDS {
            passes_untouched(&mut h, chord);
        }
        win_v_cycle(&mut h);
        passes_untouched(&mut h, &[0x5B, 0xA0, 0x53]);
    }

    #[test]
    fn win_chords_pass_while_panel_open() {
        let mut h = HookState::new(true);
        win_v_cycle(&mut h);
        h.set_panel_open(true);
        passes_untouched(&mut h, &[0x5B, 0xA0, 0x53]);
        passes_untouched(&mut h, &[0x5C, 0x44]);
        assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
    }

    #[test]
    fn released_hook_passes_keys_until_every_reason_clears() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        h.set_released(Release::Dialog, true);
        h.set_released(Release::Prompt, true);
        passes_untouched(&mut h, &[0x41, 0x1B, 0x28]);
        h.set_released(Release::Prompt, false);
        assert_eq!(h.decide(d(0x41)), HookAction::Pass);
        h.set_released(Release::Dialog, false);
        assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
    }

    #[test]
    fn release_survives_panel_close_and_reopen() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        h.set_released(Release::Dialog, true);
        h.set_panel_open(false);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0x41)), HookAction::Pass);
        h.decide(u(0x41));
        h.set_released(Release::Dialog, false);
        assert_eq!(h.decide(d(0x41)), HookAction::Translate);
    }

    #[test]
    fn custom_open_chord_opens_and_win_v_goes_back_to_windows() {
        let mut h = HookState::new(false);
        h.set_shortcuts(Shortcuts { open: chord(0x56, CTRL_SHIFT), ..Shortcuts::default() });
        passes_untouched(&mut h, &[0x5B, 0x56]);
        assert_eq!(h.decide(d(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0xA0)), HookAction::Pass);
        assert_eq!(h.decide(d(0x56)), HookAction::Open);
        assert_eq!(h.decide(u(0x56)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA0)), HookAction::Pass);
        assert_eq!(h.decide(u(0xA2)), HookAction::Pass);
        passes_untouched(&mut h, &[0xA2, 0x56]);
    }

    #[test]
    fn takeover_also_catches_win_v_next_to_a_custom_open_chord() {
        let mut h = HookState::new(true);
        h.set_shortcuts(Shortcuts { open: chord(0x56, CTRL_SHIFT), ..Shortcuts::default() });
        win_v_cycle(&mut h);
        h.decide(d(0xA2));
        h.decide(d(0xA0));
        assert_eq!(h.decide(d(0x56)), HookAction::Open);
    }

    #[test]
    fn win_open_chord_masks_start_and_needs_exact_modifiers() {
        let mut h = HookState::new(false);
        h.set_shortcuts(Shortcuts { open: chord(0x43, Mods { win: true, shift: true, ..Mods::default() }), ..Shortcuts::default() });
        passes_untouched(&mut h, &[0x5B, 0x43]);
        assert_eq!(h.decide(d(0x5B)), HookAction::Pass);
        assert_eq!(h.decide(d(0xA0)), HookAction::Pass);
        assert_eq!(h.decide(d(0x43)), HookAction::Open);
        assert_eq!(h.decide(u(0x43)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA0)), HookAction::Pass);
        assert_eq!(h.decide(u(0x5B)), HookAction::PassWithStartMask);
    }

    #[test]
    fn alt_open_chord_toggles_while_panel_open() {
        let mut h = open_panel();
        h.set_shortcuts(Shortcuts { open: chord(0x43, Mods { alt: true, ..Mods::default() }), ..Shortcuts::default() });
        h.decide(d(0xA4));
        assert_eq!(h.decide(d(0x43)), HookAction::Open);
        assert_eq!(h.decide(u(0x43)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA4)), HookAction::Pass);
    }

    #[test]
    fn configured_panel_chords_replace_the_defaults() {
        let mut h = open_panel();
        h.set_shortcuts(Shortcuts { pin: chord(0x50, CTRL_SHIFT), close: chord(0x57, Mods::CTRL), paste: chord(0x71, Mods::NONE), ..Shortcuts::default() });
        h.decide(d(0xA2));
        assert_eq!(h.decide(d(0x50)), HookAction::Pass);
        h.decide(u(0x50));
        assert_eq!(h.decide(d(0x57)), HookAction::Forward(PanelKey::Esc));
        h.decide(u(0x57));
        h.decide(d(0xA0));
        assert_eq!(h.decide(d(0x50)), HookAction::Forward(PanelKey::Pin));
        h.decide(u(0x50));
        h.decide(u(0xA0));
        h.decide(u(0xA2));
        assert_eq!(h.decide(d(0x71)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::Paste }));
        h.decide(u(0x71));
        assert_eq!(h.decide(d(0x25)), HookAction::Forward(PanelKey::Left));
        assert_eq!(h.decide(u(0x25)), HookAction::Swallow);
        passes_untouched(&mut h, &[0x1B]);
        passes_untouched(&mut h, &[0x0D]);
        assert_eq!(h.decide(d(0xA0)), HookAction::Pass);
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::Copy }));
    }

    #[test]
    fn extra_ctrl_or_shift_falls_back_to_the_closest_binding() {
        let mut h = open_panel();
        h.decide(d(0xA2));
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::Paste }));
        h.decide(u(0x0D));
        h.decide(u(0xA2));
        h.decide(d(0xA0));
        assert_eq!(h.decide(d(0x1B)), HookAction::Forward(PanelKey::Esc));
    }

    #[test]
    fn alt_quick_number_modifier() {
        let mut h = open_panel();
        h.set_shortcuts(Shortcuts { quick: Mods { alt: true, ..Mods::default() }, ..Shortcuts::default() });
        h.decide(d(0xA4));
        assert_eq!(h.decide(d(0x33)), HookAction::Forward(PanelKey::Quick { n: 3 }));
        assert_eq!(h.decide(u(0x33)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x09)), HookAction::Pass);
        h.decide(u(0x09));
        h.decide(u(0xA4));
        passes_untouched(&mut h, &[0xA2, 0x33]);
        h.decide(d(0xA2));
        h.decide(d(0xA5));
        assert_eq!(h.decide(d(0x33)), HookAction::Translate);
    }

    #[test]
    fn numpad_digits_are_quick_numbers_too() {
        let mut h = open_panel();
        h.decide(d(0xA2));
        assert_eq!(h.decide(d(0x61)), HookAction::Forward(PanelKey::Quick { n: 1 }));
        assert_eq!(h.decide(u(0x61)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x69)), HookAction::Forward(PanelKey::Quick { n: 9 }));
        h.decide(u(0x69));
        assert_eq!(h.decide(d(0x60)), HookAction::Pass);
        h.decide(u(0x60));
        h.decide(u(0xA2));
        assert_eq!(h.decide(d(0x61)), HookAction::Translate);
    }

    #[test]
    fn recording_passes_every_new_press_but_keeps_swallowed_releases() {
        let mut h = open_panel();
        h.decide(d(0x28));
        h.set_recording(true);
        assert_eq!(h.decide(u(0x28)), HookAction::Swallow);
        passes_untouched(&mut h, &[0x5B, 0x56]);
        passes_untouched(&mut h, &[0x0D]);
        passes_untouched(&mut h, &[0x41]);
        h.set_recording(false);
        assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
    }

    fn only_shift_down(vk: u32) -> bool { matches!(vk, 0x10 | 0xA0) }

    #[test]
    fn stale_ctrl_does_not_turn_shift_v_into_open() {
        let mut h = HookState::with_probe(false, only_shift_down);
        h.set_shortcuts(Shortcuts { open: chord(0x56, CTRL_SHIFT), ..Shortcuts::default() });
        h.decide(d(0xA2));
        h.decide(d(0xA0));
        assert_eq!(h.decide(d(0x56)), HookAction::Pass);
    }

    #[test]
    fn released_hook_still_toggles_on_win_v() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        h.set_released(Release::Dialog, true);
        assert_eq!(h.decide(d(0x5B)), HookAction::Pass);
        assert_eq!(h.decide(d(0x56)), HookAction::Open);
    }

    #[test]
    fn alt_chords_pass_while_panel_open() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        passes_untouched(&mut h, &[0xA4, 0x09]);
        passes_untouched(&mut h, &[0xA4, 0xA0, 0x09]);
        passes_untouched(&mut h, &[0xA4, 0x1B]);
        passes_untouched(&mut h, &[0xA4, 0x73]);
        passes_untouched(&mut h, &[0xA5, 0x09]);
        assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
    }

    #[test]
    fn alt_tab_repeats_pass_while_alt_held() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0xA4)), HookAction::Pass);
        for _ in 0..3 {
            assert_eq!(h.decide(d(0x09)), HookAction::Pass);
            assert_eq!(h.decide(u(0x09)), HookAction::Pass);
        }
        assert_eq!(h.decide(u(0xA4)), HookAction::Pass);
    }

    #[test]
    fn altgr_still_captured_with_alt_passthrough() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0xA5)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x51)), HookAction::Translate);
        assert_eq!(h.decide(u(0x51)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA5)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x41)), HookAction::Translate);
    }

    #[test]
    fn passed_alt_release_never_swallowed_across_open_close() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0xA4)), HookAction::Pass);
        h.set_panel_open(false);
        assert_eq!(h.decide(u(0xA4)), HookAction::Pass);
        assert_eq!(h.decide(d(0xA4)), HookAction::Pass);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0x09)), HookAction::Pass);
        assert_eq!(h.decide(u(0x09)), HookAction::Pass);
        assert_eq!(h.decide(u(0xA4)), HookAction::Pass);
        win_v_cycle(&mut h);
        passes_untouched(&mut h, &[0xA4, 0x09]);
    }

    fn open_panel() -> HookState {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        h
    }

    #[test]
    fn print_screen_and_variants_pass_with_panel_open() {
        let mut h = open_panel();
        for chord in [&[0x2C][..], &[0xA4, 0x2C], &[0x5B, 0x2C], &[0xA2, 0x2C], &[0xA0, 0x2C]] {
            passes_untouched(&mut h, chord);
        }
        assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
    }

    #[test]
    fn function_media_and_lock_keys_pass_with_panel_open() {
        let mut h = open_panel();
        let keys = [0x70, 0x74, 0x7B, 0x87, 0xAD, 0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB3, 0xA6, 0xAC, 0xB4, 0xB7, 0x5F, 0x13, 0x91, 0x90, 0x5D, 0x2D, 0x88, 0x03];
        for vk in keys {
            passes_untouched(&mut h, &[vk]);
        }
        passes_untouched(&mut h, &[0xA2, 0x74]);
        passes_untouched(&mut h, &[0xA0, 0x79]);
    }

    #[test]
    fn caps_lock_passes_with_panel_open_and_toggle_is_tracked() {
        let mut h = open_panel();
        passes_untouched(&mut h, &[0x14]);
        assert_eq!(h.key_state()[0x14] & 0x01, 1);
        passes_untouched(&mut h, &[0x14]);
        assert_eq!(h.key_state()[0x14] & 0x01, 0);
    }

    #[test]
    fn typing_still_translates_with_panel_open() {
        let mut h = open_panel();
        for vk in [0x41, 0x5A, 0x30, 0x20, 0xBA, 0xDE, 0xE2, 0x60, 0x69, 0x6A, 0x6B, 0x6D, 0x6E, 0x6F] {
            assert_eq!(h.decide(d(vk)), HookAction::Translate, "{vk:#x}");
            assert_eq!(h.decide(u(vk)), HookAction::Swallow, "{vk:#x}");
        }
        h.decide(d(0xA0));
        assert_eq!(h.decide(d(0x41)), HookAction::Translate);
    }

    #[test]
    fn panel_keys_still_captured_with_panel_open() {
        let mut h = open_panel();
        let forwarded = [(0x26, PanelKey::Up), (0x28, PanelKey::Down), (0x24, PanelKey::Home), (0x23, PanelKey::End), (0x1B, PanelKey::Esc), (0x08, PanelKey::Backspace), (0x09, PanelKey::Tab { back: false }), (0x25, PanelKey::Left), (0x27, PanelKey::Right)];
        for (vk, key) in forwarded {
            assert_eq!(h.decide(d(vk)), HookAction::Forward(key));
            assert_eq!(h.decide(u(vk)), HookAction::Swallow);
        }
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::Paste }));
        h.decide(u(0x0D));
        for vk in [0x2E, 0x21, 0x22] {
            assert_eq!(h.decide(d(vk)), HookAction::Swallow, "{vk:#x}");
            assert_eq!(h.decide(u(vk)), HookAction::Swallow, "{vk:#x}");
        }
        h.decide(d(0xA0));
        assert_eq!(h.decide(d(0x09)), HookAction::Forward(PanelKey::Tab { back: true }));
        h.decide(d(0xA2));
        assert_eq!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { mode: PasteMode::PlainPaste }));
    }

    #[test]
    fn unused_ctrl_chords_pass_with_panel_open() {
        let mut h = open_panel();
        passes_untouched(&mut h, &[0xA2, 0x43]);
        passes_untouched(&mut h, &[0xA2, 0xA0, 0x1B]);
        passes_untouched(&mut h, &[0xA2, 0x1B]);
    }

    #[test]
    fn repeats_keep_the_verdict_of_their_first_press() {
        let mut h = HookState::new(true);
        assert_eq!(h.decide(d(0x28)), HookAction::Pass);
        h.set_panel_open(true);
        assert_eq!(h.decide(d(0x28)), HookAction::Pass);
        assert_eq!(h.decide(u(0x28)), HookAction::Pass);
        assert_eq!(h.decide(d(0x41)), HookAction::Translate);
        h.set_panel_open(false);
        assert_eq!(h.decide(d(0x41)), HookAction::Swallow);
        assert_eq!(h.decide(u(0x41)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x41)), HookAction::Pass);
    }

    #[test]
    fn held_navigation_keys_repeat_but_actions_do_not() {
        let mut h = HookState::new(true);
        h.set_panel_open(true);
        for _ in 0..3 {
            assert_eq!(h.decide(d(0x28)), HookAction::Forward(PanelKey::Down));
        }
        assert_eq!(h.decide(u(0x28)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x08)), HookAction::Forward(PanelKey::Backspace));
        assert_eq!(h.decide(d(0x08)), HookAction::Forward(PanelKey::Backspace));
        assert_eq!(h.decide(u(0x08)), HookAction::Swallow);
        assert!(matches!(h.decide(d(0x0D)), HookAction::Forward(PanelKey::Activate { .. })));
        assert_eq!(h.decide(d(0x0D)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x41)), HookAction::Translate);
        assert_eq!(h.decide(d(0x41)), HookAction::Swallow);
        h.set_panel_open(false);
        assert_eq!(h.decide(d(0x0D)), HookAction::Swallow);
    }

    #[test]
    fn search_edit_chords_are_captured_with_panel_open() {
        let mut h = open_panel();
        assert_eq!(h.decide(d(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x41)), HookAction::Forward(PanelKey::SelectAll));
        assert_eq!(h.decide(u(0x41)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x08)), HookAction::Forward(PanelKey::DeleteWord));
        assert_eq!(h.decide(u(0x08)), HookAction::Swallow);
        assert_eq!(h.decide(d(0x56)), HookAction::PasteText);
        assert_eq!(h.decide(u(0x56)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x08)), HookAction::Forward(PanelKey::Backspace));
    }

    #[test]
    fn ctrl_space_toggles_preview_and_swallows_space_only() {
        let mut h = open_panel();
        assert_eq!(h.decide(d(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x20)), HookAction::Forward(PanelKey::TogglePreview));
        assert_eq!(h.decide(d(0x20)), HookAction::Swallow);
        assert_eq!(h.decide(u(0x20)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x20)), HookAction::Translate);
    }

    #[test]
    fn ctrl_space_passes_when_panel_closed() {
        let mut h = HookState::new(true);
        passes_untouched(&mut h, &[0xA2, 0x20]);
    }

    #[test]
    fn other_ctrl_letters_still_pass() {
        let mut h = open_panel();
        for vk in [0x43, 0x58, 0x5A, 0x46] {
            passes_untouched(&mut h, &[0xA2, vk]);
        }
    }

    #[test]
    fn altgr_letters_are_not_search_edits() {
        let mut h = open_panel();
        h.decide(d(0xA2));
        h.decide(d(0xA5));
        for vk in [0x41, 0x56, 0x51] {
            assert_eq!(h.decide(d(vk)), HookAction::Translate, "{vk:#x}");
            h.decide(u(vk));
        }
    }

    #[test]
    fn search_edit_chords_pass_when_panel_closed_or_released() {
        let mut h = HookState::new(true);
        passes_untouched(&mut h, &[0xA2, 0x41]);
        passes_untouched(&mut h, &[0xA2, 0x56]);
        h.set_panel_open(true);
        h.set_released(Release::Dialog, true);
        passes_untouched(&mut h, &[0xA2, 0x41]);
        passes_untouched(&mut h, &[0xA2, 0x08]);
        passes_untouched(&mut h, &[0xA2, 0x56]);
    }
    #[test]
    fn ctrl_comma_is_not_bound() {
        let mut h = open_panel();
        h.decide(d(0xA2));
        assert_eq!(h.decide(d(0xBC)), HookAction::Pass);
    }

    #[test]
    fn ctrl_k_opens_the_source_menu_by_default() {
        assert_eq!(Shortcuts::default().source, chord(0x4B, Mods::CTRL));
        let mut h = open_panel();
        assert_eq!(h.decide(d(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x4B)), HookAction::Forward(PanelKey::SourceMenu));
        assert_eq!(h.decide(d(0x4B)), HookAction::Swallow);
        assert_eq!(h.decide(u(0x4B)), HookAction::Swallow);
        assert_eq!(h.decide(u(0xA2)), HookAction::Pass);
        assert_eq!(h.decide(d(0x4B)), HookAction::Translate);
    }

    #[test]
    fn ctrl_k_passes_when_panel_closed_or_released() {
        let mut h = HookState::new(true);
        passes_untouched(&mut h, &[0xA2, 0x4B]);
        h.set_panel_open(true);
        h.set_released(Release::Dialog, true);
        passes_untouched(&mut h, &[0xA2, 0x4B]);
    }

    #[test]
    fn configured_source_chord_replaces_the_default() {
        let mut h = open_panel();
        h.set_shortcuts(Shortcuts { source: chord(0x4F, Mods { alt: true, ..Mods::default() }), ..Shortcuts::default() });
        passes_untouched(&mut h, &[0xA2, 0x4B]);
        h.decide(d(0xA4));
        assert_eq!(h.decide(d(0x4F)), HookAction::Forward(PanelKey::SourceMenu));
    }

    #[test]
    fn default_panel_chords_are_all_distinct() {
        let s = Shortcuts::default();
        let chords = [s.paste, s.copy, s.plain_paste, s.pin, s.preview, s.close, s.source];
        for (i, a) in chords.iter().enumerate() {
            assert!(chords[i + 1..].iter().all(|b| b != a), "{a:?} is bound twice");
            assert!(!(a.mods == s.quick && (0x31..=0x39).contains(&a.key)));
        }
    }
}