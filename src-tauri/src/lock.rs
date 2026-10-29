use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

const FIVE_MINUTES: Duration = Duration::from_secs(300);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum UnlockDuration {
    EveryTime,
    FiveMinutes,
    #[default]
    WhileRunning,
}

pub struct UnlockWindow<R> {
    duration: UnlockDuration,
    root: Option<(R, Instant)>,
}

impl<R> UnlockWindow<R> {
    pub fn new(d: UnlockDuration) -> Self {
        Self { duration: d, root: None }
    }

    pub fn set(&mut self, root: R, now: Instant) {
        self.root = Some((root, now));
    }

    pub fn current(&mut self, now: Instant) -> Option<&R> {
        if self.duration == UnlockDuration::FiveMinutes && self.root.as_ref().is_some_and(|(_, at)| now.saturating_duration_since(*at) >= FIVE_MINUTES) {
            self.root = None;
        }
        self.root.as_ref().map(|(r, _)| r)
    }

    pub fn after_use(&mut self) {
        if self.duration == UnlockDuration::EveryTime {
            self.root = None;
        }
    }

    pub fn set_duration(&mut self, d: UnlockDuration) {
        self.duration = d;
    }

    pub fn on_session_event(&mut self, relock: bool) {
        if relock {
            self.root = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    fn root() -> [u8; 64] { [1u8; 64] }

    #[test]
    fn five_minutes_expires() {
        let t0 = Instant::now();
        let mut w = UnlockWindow::new(UnlockDuration::FiveMinutes);
        w.set(root(), t0);
        assert!(w.current(t0 + Duration::from_secs(299)).is_some());
        assert!(w.current(t0 + Duration::from_secs(301)).is_none());
    }

    #[test]
    fn while_running_persists_and_every_time_clears_after_use() {
        let t0 = Instant::now();
        let mut w = UnlockWindow::new(UnlockDuration::WhileRunning);
        w.set(root(), t0);
        w.after_use();
        assert!(w.current(t0 + Duration::from_secs(86_400)).is_some());
        let mut e = UnlockWindow::new(UnlockDuration::EveryTime);
        e.set(root(), t0);
        assert!(e.current(t0).is_some());
        e.after_use();
        assert!(e.current(t0).is_none());
    }

    #[test]
    fn session_event_relocks_only_when_enabled() {
        let t0 = Instant::now();
        let mut w = UnlockWindow::new(UnlockDuration::WhileRunning);
        w.set(root(), t0);
        w.on_session_event(false);
        assert!(w.current(t0).is_some());
        w.on_session_event(true);
        assert!(w.current(t0).is_none());
    }

    #[test]
    fn changed_duration_applies_to_open_window() {
        let t0 = Instant::now();
        let mut w = UnlockWindow::new(UnlockDuration::WhileRunning);
        w.set(root(), t0);
        w.set_duration(UnlockDuration::FiveMinutes);
        assert!(w.current(t0 + Duration::from_secs(301)).is_none());
    }
}
