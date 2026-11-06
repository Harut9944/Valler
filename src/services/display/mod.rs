//! Displays: list screens with adjustable brightness and change it.
//! Built-in laptop screens use the system backlight (Linux, Windows);
//! external monitors use DDC/CI through `ddcutil` on Linux. Where nothing can
//! be controlled (macOS, most desktops on Windows) the Displays tab offers
//! software dimming instead (see `app::dimmer`).
//!
//! Hardware calls are slow (DDC/CI takes up to a second), so changes go
//! through a worker thread that only applies the latest value per screen.

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod imp {
    use super::Screen;
    pub fn list() -> Vec<Screen> {
        Vec::new()
    }
    pub fn set(_: &Screen, _: f32) -> Result<(), String> {
        Err("Brightness control isn't available on this system".into())
    }
}

use std::collections::HashMap;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A laptop's built-in screen.
    BuiltIn,
    /// An external monitor controlled over DDC/CI.
    External,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Screen {
    /// Backend-specific id (backlight device, ddcutil display number, …).
    pub id: String,
    pub name: String,
    pub kind: Kind,
    /// 0.0 ..= 1.0
    pub level: f32,
}

pub fn list() -> Vec<Screen> {
    imp::list()
}

pub fn demo() -> Vec<Screen> {
    vec![
        Screen { id: "builtin".into(), name: "Built-in display".into(), kind: Kind::BuiltIn, level: 0.7 },
        Screen { id: "1".into(), name: "External monitor".into(), kind: Kind::External, level: 0.45 },
    ]
}

#[derive(Default, Clone)]
pub struct State {
    pub screens: Option<Vec<Screen>>,
    pub error: Option<String>,
}

pub type Shared = Arc<Mutex<State>>;

/// Reads the screens on a background thread.
pub fn refresh(state: Shared, ctx: egui::Context) {
    std::thread::spawn(move || {
        let screens = list();
        if let Ok(mut s) = state.lock() {
            s.screens = Some(screens);
        }
        ctx.request_repaint();
    });
}

/// Applies brightness changes in the background; the latest value wins.
pub struct Controller {
    tx: Sender<(Screen, f32)>,
}

impl Controller {
    pub fn spawn(state: Shared) -> Self {
        let (tx, rx) = channel::<(Screen, f32)>();
        std::thread::Builder::new()
            .name("valler-displays".into())
            .spawn(move || {
                while let Ok(first) = rx.recv() {
                    // Collect what was queued meanwhile; keep the newest per screen.
                    let mut latest: HashMap<String, (Screen, f32)> = HashMap::new();
                    latest.insert(first.0.id.clone(), first);
                    std::thread::sleep(Duration::from_millis(120));
                    while let Ok((s, l)) = rx.try_recv() {
                        latest.insert(s.id.clone(), (s, l));
                    }
                    for (screen, level) in latest.into_values() {
                        let r = imp::set(&screen, level);
                        if let Ok(mut st) = state.lock() {
                            st.error = r.err();
                        }
                    }
                }
            })
            .ok();
        Self { tx }
    }

    pub fn set(&self, screen: &Screen, level: f32) {
        let _ = self.tx.send((screen.clone(), level.clamp(0.0, 1.0)));
    }
}

/// Pure helpers for the platform files, kept here so they can be tested.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub mod parse {
    /// `ddcutil detect --brief` → (display number, monitor name).
    pub fn ddc_detect(text: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut current: Option<String> = None;
        for line in text.lines() {
            let t = line.trim();
            if let Some(n) = t.strip_prefix("Display ") {
                current = Some(n.trim().to_string());
            } else if let (Some(n), Some(m)) = (current.as_ref(), t.strip_prefix("Monitor:")) {
                // "VSC:VP2468a:WA623…" is maker code, model and serial: show the model.
                let parts: Vec<&str> = m.trim().split(':').collect();
                let name = parts.get(1).filter(|p| !p.is_empty()).or(parts.first()).unwrap_or(&"Monitor").to_string();
                out.push((n.clone(), name));
                current = None;
            } else if t.starts_with("Invalid display") {
                current = None; // listed but not controllable
            }
        }
        out
    }

    /// `ddcutil getvcp 10 --brief` → (current, max). Line: "VCP 10 C 95 100".
    pub fn ddc_vcp(text: &str) -> Option<(u32, u32)> {
        let line = text.lines().find(|l| l.starts_with("VCP 10 C"))?;
        let mut it = line.split_whitespace().skip(3);
        let cur = it.next()?.parse().ok()?;
        let max: u32 = it.next()?.parse().ok()?;
        (max > 0).then_some((cur, max))
    }

    /// A 0..=1 level as a raw value on a 0..=max scale, never below `min_raw`
    /// (a backlight at 0 can turn the panel fully dark).
    pub fn raw_level(level: f32, max: u32, min_raw: u32) -> u32 {
        ((level.clamp(0.0, 1.0) * max as f32).round() as u32).clamp(min_raw.min(max), max)
    }
}

#[cfg(test)]
mod tests {
    use super::parse::*;

    #[test]
    fn reads_ddcutil_output() {
        let t = "Display 1\n   I2C bus:          /dev/i2c-8\n   DRM connector:    card1-HDMI-A-1\n   Monitor:          VSC:VP2468a:WA6230400173\n\nInvalid display\n   I2C bus:          /dev/i2c-9\n   Monitor:          ABC:Old:1\n\nDisplay 2\n   Monitor:          DEL::123\n";
        assert_eq!(ddc_detect(t), vec![("1".to_string(), "VP2468a".to_string()), ("2".to_string(), "DEL".to_string())]);
        assert_eq!(ddc_vcp("VCP 10 C 95 100\n"), Some((95, 100)));
        assert_eq!(ddc_vcp("VCP 10 ERR\n"), None);
        assert_eq!(ddc_vcp("VCP 10 C 5 0\n"), None);
    }

    #[test]
    fn converts_levels_to_raw_values() {
        assert_eq!(raw_level(0.5, 100, 0), 50);
        assert_eq!(raw_level(0.0, 937, 1), 1, "a backlight never goes fully dark");
        assert_eq!(raw_level(1.5, 255, 1), 255);
    }

    #[test]
    fn demo_has_both_kinds() {
        let d = super::demo();
        assert!(d.iter().any(|s| s.kind == super::Kind::BuiltIn));
        assert!(d.iter().any(|s| s.kind == super::Kind::External));
    }

    /// Reads the real screens (can take a few seconds with external monitors).
    #[test]
    #[ignore]
    fn lists_this_computers_screens() {
        for s in super::list() {
            assert!((0.0..=1.0).contains(&s.level), "{s:?}");
        }
    }
}
