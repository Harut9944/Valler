//! Everything the person chooses is saved as JSON in the platform's config
//! folder (~/.config/Valler, %APPDATA%\Valler, ~/Library/Application Support/Valler).

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Snippet {
    pub trigger: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClipEntry {
    pub text: String,
    pub pinned: bool,
    /// Unix seconds when copied.
    pub at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NoteTab {
    pub title: String,
    pub body: String,
}

/// Monitor alert choices; defaults follow the original (all off, 90 % CPU,
/// 90 °C CPU, 40 °C battery, 10 % disk free, 15 % battery, 15 min cooldown).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AlertPrefs {
    pub cpu: bool,
    pub cpu_temp: bool,
    pub memory: bool,
    pub disk: bool,
    pub battery: bool,
    pub battery_temp: bool,
    pub cpu_threshold: u32,
    pub cpu_temp_threshold: u32,
    pub battery_temp_threshold: u32,
    pub disk_free_percent: u32,
    pub battery_percent: u32,
    pub cooldown_minutes: u32,
}

impl Default for AlertPrefs {
    fn default() -> Self {
        Self {
            cpu: false,
            cpu_temp: false,
            memory: false,
            disk: false,
            battery: false,
            battery_temp: false,
            cpu_threshold: 90,
            cpu_temp_threshold: 90,
            battery_temp_threshold: 40,
            disk_free_percent: 10,
            battery_percent: 15,
            cooldown_minutes: 15,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Store {
    /// Installed (available) features by id; uninstalled ones vanish from the app.
    pub installed: BTreeSet<String>,
    /// Feature on/off switches (by feature id).
    pub enabled: BTreeSet<String>,
    pub first_run_done: bool,
    pub compact: bool,
    pub temperature_fahrenheit: bool,
    pub launch_at_login: bool,

    // Keep awake
    pub keep_display_on: bool,
    pub keep_awake_minutes: u32,
    pub keep_awake_auto: crate::services::keepawake::automation::Rules,

    // Clipboard
    pub clip_history: Vec<ClipEntry>,
    pub clip_max_items: usize,
    pub clip_auto_clear_secs: u32,

    // Snippets
    pub snippets: Vec<Snippet>,

    // Clean URL
    pub url_extra_params: String,
    pub url_auto_clean: bool,

    // Scratchpad
    pub notes: Vec<NoteTab>,
    pub note_active: usize,

    // Network
    pub speedtest_url: String,

    // Mixer
    pub mixer_boost: bool,

    // Panel tab visibility (tab ids hidden by the person)
    pub hidden_tabs: BTreeSet<String>,

    // Monitor graph length in samples
    pub history_len: usize,

    pub alerts: AlertPrefs,

    // Cleaning mode: keep a visible indicator instead of a black screen.
    pub cleaning_keep_visible: bool,

    // Shelf
    pub shelf_items: Vec<crate::services::shelf::Item>,
    pub shelf_remove_after_use: bool,
    /// Linux: run through XWayland so files can be dropped on Valler.
    pub linux_x11: bool,

    // Tray / menu bar icon
    pub tray_enabled: bool,
    pub close_to_tray: bool,
    pub tray_readout: crate::services::tray::Readout,
    pub cleaning_presses: u32,

    /// Set by settings pages when they edit the store directly (never saved).
    #[serde(skip)]
    pub dirty_flag: bool,
}

impl Default for Store {
    fn default() -> Self {
        let installed = crate::core::features::default_installed();
        let enabled = crate::core::features::default_enabled();
        Self {
            installed,
            enabled,
            first_run_done: false,
            compact: false,
            temperature_fahrenheit: false,
            launch_at_login: false,
            keep_display_on: true,
            keep_awake_minutes: 60,
            keep_awake_auto: Default::default(),
            clip_history: Vec::new(),
            clip_max_items: 200,
            clip_auto_clear_secs: 0,
            snippets: vec![
                Snippet { trigger: ";date".into(), text: "{{date}}".into() },
                Snippet { trigger: ";sig".into(), text: "Best regards,\n{{name}}".into() },
                Snippet { trigger: ";now".into(), text: "{{datetime}}".into() },
            ],
            url_extra_params: String::new(),
            url_auto_clean: false,
            notes: vec![NoteTab { title: "Notes".into(), body: String::new() }],
            note_active: 0,
            speedtest_url: "https://speed.cloudflare.com/__down?bytes=25000000".into(),
            mixer_boost: true,
            hidden_tabs: BTreeSet::new(),
            history_len: 90,
            alerts: AlertPrefs::default(),
            cleaning_keep_visible: false,
            shelf_items: Vec::new(),
            shelf_remove_after_use: true,
            linux_x11: false,
            tray_enabled: true,
            close_to_tray: true,
            tray_readout: crate::services::tray::Readout::Off,
            cleaning_presses: 3,
            dirty_flag: false,
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Valler")
}

fn path() -> PathBuf {
    config_dir().join("settings.json")
}

impl Store {
    /// In screenshot mode we never read or write the person's real settings.
    pub fn load(ephemeral: bool) -> Self {
        if ephemeral {
            return Self::demo();
        }
        std::fs::read_to_string(path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let tmp = dir.join("settings.json.tmp");
            if std::fs::write(&tmp, s).is_ok() {
                let _ = std::fs::rename(tmp, path());
            }
        }
    }

    fn demo() -> Self {
        let mut s = Self::default();
        let now = chrono::Utc::now().timestamp();
        s.clip_history = vec![
            ClipEntry { text: "https://example.com/article?id=42".into(), pinned: true, at: now - 300 },
            ClipEntry { text: "Meeting moved to 3 pm".into(), pinned: false, at: now - 120 },
        ];
        s.notes = vec![NoteTab {
            title: "Plan".into(),
            body: "# This week\n\n- [x] Ship the **Shelf**\n- [ ] Write tests\n- Read `README.md`\n\n> Small steps, every day.\n\nMore at [example.com](https://example.com).".into(),
        }];
        s
    }

    pub fn is_installed(&self, id: &str) -> bool {
        self.installed.contains(id)
    }

    pub fn is_on(&self, id: &str) -> bool {
        self.installed.contains(id) && self.enabled.contains(id)
    }

    pub fn set_on(&mut self, id: &str, on: bool) {
        if on {
            self.enabled.insert(id.to_string());
        } else {
            self.enabled.remove(id);
        }
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;

    #[test]
    fn defaults_install_working_features_and_sensible_options() {
        let s = Store::default();
        assert!(s.is_installed("monitor_cpu"));
        assert!(!s.is_installed("switcher"), "features coming later are not installed");
        assert!(s.is_on("clipboard"));
        assert!(s.tray_enabled && s.close_to_tray && s.shelf_remove_after_use);
        assert!(!any_alert(&s));
        assert_eq!(s.alerts.cpu_threshold, 90);
        assert_eq!(s.alerts.cooldown_minutes, 15);
    }

    fn any_alert(s: &Store) -> bool {
        crate::services::alerts::any_enabled(s)
    }

    #[test]
    fn switches_only_count_for_installed_features() {
        let mut s = Store::default();
        s.set_on("url_clean", true);
        assert!(s.is_on("url_clean"));
        s.installed.remove("url_clean");
        assert!(!s.is_on("url_clean"));
        s.set_on("clipboard", false);
        assert!(!s.is_on("clipboard"));
    }

    #[test]
    fn saved_settings_round_trip() {
        let mut s = Store::default();
        s.snippets.push(Snippet { trigger: ";x".into(), text: "y".into() });
        s.tray_readout = crate::services::tray::Readout::Memory;
        crate::services::shelf::add_text(&mut s.shelf_items, "note");
        s.dirty_flag = true;
        let json = serde_json::to_string(&s).unwrap();
        let back: Store = serde_json::from_str(&json).unwrap();
        assert_eq!(back.snippets.len(), s.snippets.len());
        assert_eq!(back.tray_readout, crate::services::tray::Readout::Memory);
        assert_eq!(back.shelf_items.len(), 1);
        assert!(!back.dirty_flag, "the unsaved-changes flag is never saved");
    }

    #[test]
    fn old_settings_files_still_load() {
        // A file from an earlier version, missing everything added since.
        let back: Store = serde_json::from_str(r#"{"keep_awake_minutes": 30, "compact": true}"#).unwrap();
        assert_eq!(back.keep_awake_minutes, 30);
        assert!(back.tray_enabled);
        assert_eq!(back.alerts.battery_percent, 15);
        assert!(back.is_installed("monitor_cpu"));
    }

    #[test]
    fn screenshot_mode_uses_sample_data() {
        let s = Store::load(true);
        assert_eq!(s.clip_history.len(), 2);
    }
}
