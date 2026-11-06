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
    pub cleaning_presses: u32,

    /// Set by settings pages when they edit the store directly (never saved).
    #[serde(skip)]
    pub dirty_flag: bool,
}

impl Default for Store {
    fn default() -> Self {
        let installed = crate::features::default_installed();
        let enabled = crate::features::default_enabled();
        Self {
            installed,
            enabled,
            first_run_done: false,
            compact: false,
            temperature_fahrenheit: false,
            launch_at_login: false,
            keep_display_on: true,
            keep_awake_minutes: 60,
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
