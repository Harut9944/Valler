//! Keep awake: holds a system sleep inhibitor (Windows power request,
//! macOS IOPMAssertion, freedesktop/GNOME/KDE inhibit on Linux) for a chosen
//! time or until stopped.

use std::time::{Duration, Instant};

pub struct KeepAwake {
    handle: Option<keepawake::KeepAwake>,
    pub until: Option<Instant>,
    pub started: Option<Instant>,
    pub display: bool,
    pub error: Option<String>,
}

pub const DURATIONS: &[(u32, &str)] = &[
    (15, "15 min"),
    (30, "30 min"),
    (60, "1 hour"),
    (120, "2 hours"),
    (300, "5 hours"),
    (0, "Until I stop it"),
];

impl KeepAwake {
    pub fn new() -> Self {
        Self { handle: None, until: None, started: None, display: true, error: None }
    }

    pub fn active(&self) -> bool {
        self.handle.is_some()
    }

    pub fn start(&mut self, minutes: u32, display: bool) {
        self.stop();
        let res = keepawake::Builder::default()
            .display(display)
            .idle(true)
            .sleep(true)
            .reason("Keep awake is on in Valler")
            .app_name("Valler")
            .app_reverse_domain("app.valler.Valler")
            .create();
        match res {
            Ok(h) => {
                self.handle = Some(h);
                self.display = display;
                self.started = Some(Instant::now());
                self.until = (minutes > 0).then(|| Instant::now() + Duration::from_secs(minutes as u64 * 60));
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Could not keep the system awake: {e}")),
        }
    }

    /// Pretend to be active without touching the system (screenshot mode).
    pub fn demo(&mut self, minutes: u32) {
        self.started = Some(Instant::now());
        self.until = Some(Instant::now() + Duration::from_secs(minutes as u64 * 60 - 1));
    }

    pub fn demo_active(&self) -> bool {
        self.handle.is_none() && self.started.is_some()
    }

    pub fn stop(&mut self) {
        self.handle = None;
        self.until = None;
        self.started = None;
    }

    /// Call every frame; ends the session when its time is up.
    pub fn tick(&mut self) {
        if let Some(u) = self.until {
            if Instant::now() >= u && self.handle.is_some() {
                self.stop();
            }
        }
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.until.map(|u| u.saturating_duration_since(Instant::now()))
    }
}
