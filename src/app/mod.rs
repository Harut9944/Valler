//! The application state (`ValleApp`) and how it runs: navigation,
//! background polling, the panel frame, window/tray behaviour and the
//! screenshot mode each live in their own file.

mod automation;
mod background;
mod lifecycle;
mod nav;
mod panel;
mod screenshot;
mod window;

pub use nav::*;
use background::watch_config;

use crate::LaunchOptions;
use crate::services::cleaner;
use crate::services::clipboard::Watcher;
use crate::core::features::Tab;
use crate::services::keepawake::KeepAwake;
use crate::services::mixer::MixerState;
use crate::services::monitor::{self, Snapshot};
use crate::services::ports::PortEntry;
use crate::services::speedtest;
use crate::core::store::Store;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct ValleApp {
    pub store: Store,
    dirty_since: Option<Instant>,
    pub opts: LaunchOptions,
    pub demo: bool,
    frames: u32,
    shot_sent: bool,

    pub tab: Tab,
    pub sub: Option<Sub>,
    pub expanded: HashSet<&'static str>,

    pub mon: monitor::Shared,
    pub snap: Snapshot,

    pub awake: KeepAwake,
    pub note_preview: bool,
    awake_checked: Option<Instant>,

    pub mixer: Arc<Mutex<MixerState>>,
    mixer_busy: Arc<Mutex<bool>>,
    pub mixer_last: Option<Instant>,
    pub mixer_hold: Option<Instant>,

    pub clip: Option<Watcher>,
    pub clip_search: String,

    pub url_input: String,
    pub url_result: Option<(String, Vec<String>)>,
    pub url_error: Option<String>,

    pub snip_trigger: String,
    pub snip_text: String,
    pub snip_editing: Option<usize>,

    pub proc_filter: String,
    pub proc_sort_mem: bool,
    pub ports: Option<Result<Vec<PortEntry>, String>>,
    pub port_filter: String,

    pub cleaner: cleaner::Shared,
    pub speed: speedtest::Shared,

    pub settings_open: bool,
    pub settings_page: SettingsPage,
    pub settings_search: String,
    pub hub_tab_support: bool,

    pub toast: Option<(String, Instant)>,

    pub alerts: crate::services::alerts::Alerts,

    pub cmd_query: String,
    pub cmd_selected: usize,
    pub cmd_focus: bool,

    pub qt_outcome: crate::services::quick_toggles::Outcome,
    pub qt_dark: Option<bool>,
    pub qt_trash: Option<usize>,
    pub qt_checked: Option<Instant>,
    pub qt_confirm_trash: bool,

    pub cleaning: bool,
    pub cleaning_presses: Vec<Instant>,

    pub apps: crate::services::apps::Shared,
    pub apps_scanned: bool,

    pub timer_end: Option<Instant>,
    pub timer_paused: Option<Duration>,
    pub timer_total: Duration,
    pub timer_minutes: u32,
    pub timer_label: String,

    pub launch_at_login: Option<bool>,

    pub pkg: crate::services::packages::Shared,
    pub pkg_tab: u8,
    pub pkg_tab_forced: bool,
    pub pkg_query: String,
    pub pkg_filter: String,
    pub pkg_confirm: Option<(crate::services::packages::Package, crate::services::packages::Op)>,

    pub media: crate::services::media::Shared,
    pub media_files: Vec<std::path::PathBuf>,
    pub media_format: crate::services::media::Format,
    pub media_max: u32,
    pub media_quality: u8,

    pub shelf_selected: HashSet<u64>,
    pub shelf_floating: bool,
    pub shelf_note: String,
    pub shelf_thumbs: std::collections::HashMap<u64, egui::TextureHandle>,
    pub shelf_thumb_queue: crate::screens::tools::shelf::ThumbQueue,
    pub shelf_thumb_requested: HashSet<u64>,

    pub tray: Option<crate::services::tray::Tray>,
    pub window_hidden: bool,
    pub quitting: bool,
}

impl ValleApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: LaunchOptions) -> Self {
        Self::with_context(cc.egui_ctx.clone(), opts)
    }

    /// Builds the app for an egui context (tests use this without a window).
    /// With `opts.screenshot` set it runs on sample data and never touches the
    /// person's settings, clipboard or tray.
    pub fn with_context(ctx: egui::Context, opts: LaunchOptions) -> Self {
        let demo = opts.screenshot.is_some();
        let store = Store::load(demo);
        let mon = monitor::spawn(ctx.clone());
        let clip = (!demo).then(|| Watcher::spawn(ctx.clone(), watch_config(&store)));
        let tab = opts
            .tab
            .as_deref()
            .and_then(Tab::from_id)
            .unwrap_or(if demo { Tab::Utilities } else { Tab::System });
        let mut awake = KeepAwake::new();
        if demo && tab == Tab::KeepAwake {
            awake.demo(store.keep_awake_minutes);
        }
        let mut app = Self {
            dirty_since: None,
            demo,
            frames: 0,
            shot_sent: false,
            tab,
            sub: None,
            expanded: HashSet::new(),
            mon,
            snap: Snapshot::default(),
            awake,
            note_preview: false,
            awake_checked: None,
            mixer: Arc::new(Mutex::new(MixerState::default())),
            mixer_busy: Arc::new(Mutex::new(false)),
            mixer_last: None,
            mixer_hold: None,
            clip,
            clip_search: String::new(),
            url_input: String::new(),
            url_result: None,
            url_error: None,
            snip_trigger: String::new(),
            snip_text: String::new(),
            snip_editing: None,
            proc_filter: String::new(),
            proc_sort_mem: false,
            ports: None,
            port_filter: String::new(),
            cleaner: Arc::new(Mutex::new(cleaner::ScanState::Idle)),
            speed: Arc::new(Mutex::new(speedtest::SpeedState::Idle)),
            settings_open: opts.settings,
            settings_page: SettingsPage::Features,
            settings_search: String::new(),
            hub_tab_support: false,
            toast: None,
            alerts: Default::default(),
            cmd_query: String::new(),
            cmd_selected: 0,
            cmd_focus: false,
            qt_outcome: Arc::new(Mutex::new(None)),
            qt_dark: None,
            qt_trash: None,
            qt_checked: None,
            qt_confirm_trash: false,
            cleaning: false,
            cleaning_presses: Vec::new(),
            apps: Arc::new(Mutex::new(None)),
            apps_scanned: false,
            timer_end: None,
            timer_paused: None,
            timer_total: Duration::ZERO,
            timer_minutes: 25,
            timer_label: String::new(),
            launch_at_login: None,
            pkg: Arc::new(Mutex::new(Default::default())),
            pkg_tab: 0,
            pkg_tab_forced: false,
            pkg_query: String::new(),
            pkg_filter: String::new(),
            pkg_confirm: None,
            media: Arc::new(Mutex::new(Default::default())),
            media_files: Vec::new(),
            media_format: crate::services::media::Format::Jpeg,
            media_max: 0,
            media_quality: 82,
            shelf_selected: HashSet::new(),
            shelf_floating: false,
            shelf_note: String::new(),
            shelf_thumbs: Default::default(),
            shelf_thumb_queue: Default::default(),
            shelf_thumb_requested: HashSet::new(),
            tray: None,
            window_hidden: false,
            quitting: false,
            store,
            opts,
        };
        if app.opts.settings {
            app.settings_page = match app.opts.tab.as_deref() {
                Some("general") => SettingsPage::General,
                Some("monitor") => SettingsPage::Monitor,
                Some("clipboard") => SettingsPage::Clipboard,
                Some("snippets") => SettingsPage::Snippets,
                Some("cleanurl") => SettingsPage::UrlClean,
                Some("keepawake") => SettingsPage::KeepAwake,
                Some("about") => SettingsPage::About,
                _ => SettingsPage::Features,
            };
        }
        if let Some(sub) = app.opts.tab.as_deref().and_then(sub_from_id) {
            app.tab = Tab::Utilities;
            app.sub = Some(sub);
            if app.opts.tab.as_deref() == Some("updates") {
                app.pkg_tab = 1;
            }
            if sub == Sub::Timer && app.demo {
                app.timer_total = Duration::from_secs(25 * 60);
                app.timer_end = Some(Instant::now() + Duration::from_secs(18 * 60 + 42));
                app.timer_label = "Focus".into();
            }
            if sub == Sub::CommandBar && app.demo {
                app.cmd_query = "12*(4+3)".into();
            }
        }
        if !app.tab_visible(app.tab) {
            app.tab = app.visible_tabs().first().copied().unwrap_or(Tab::Controls);
        }
        app.store.first_run_done = true;
        if !app.demo {
            crate::services::desktop_entry::ensure();
            if app.store.tray_enabled {
                app.tray = crate::services::tray::Tray::new(&ctx);
            }
        }
        app
    }

    pub fn mark_dirty(&mut self) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(Instant::now());
        }
        if let Some(w) = &self.clip
            && let Ok(mut c) = w.config.lock() {
                *c = watch_config(&self.store);
            }
    }

    pub fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    pub fn copy_text(&mut self, ctx: &egui::Context, text: String) {
        match &self.clip {
            Some(w) => w.copy(text),
            None => ctx.copy_text(text),
        }
        self.toast("Copied");
    }

    pub fn tab_visible(&self, t: Tab) -> bool {
        !self.store.hidden_tabs.contains(t.id())
            && t.features().iter().any(|f| self.store.is_installed(f))
    }

    pub fn visible_tabs(&self) -> Vec<Tab> {
        Tab::ALL.into_iter().filter(|t| self.tab_visible(*t)).collect()
    }
}
