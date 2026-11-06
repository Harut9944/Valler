//! Keep-awake automation, checked once a second from the frame logic (which
//! also runs while the window is hidden).

use super::ValleApp;
use crate::services::keepawake::automation::World;
use std::time::{Duration, Instant};

impl ValleApp {
    pub(super) fn automate_keep_awake(&mut self) {
        if !self.snap.ready || !self.store.is_installed("keep_awake") {
            return;
        }
        if self.awake_checked.is_some_and(|t| t.elapsed() < Duration::from_secs(1)) {
            return;
        }
        self.awake_checked = Some(Instant::now());
        let running: Vec<String> = self.snap.procs.iter().map(|p| p.name.clone()).collect();
        let on_battery = self.snap.battery.as_ref().map(|b| b.state == "On battery");
        let world = World { running: &running, on_battery };
        self.awake.automate(&self.store.keep_awake_auto, &world, self.store.keep_display_on);
    }
}
