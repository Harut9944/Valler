//! The main window: the panel with its tab bar, the tab pages, the footer,
//! the Settings window, saving, and the `--screenshot` mode.

use crate::LaunchOptions;
use crate::cleaner;
use crate::clipboard::{WatchConfig, Watcher};
use crate::features::Tab;
use crate::keepawake_ctl::KeepAwake;
use crate::mixer::{self, MixerState};
use crate::monitor::{self, Snapshot};
use crate::ports::PortEntry;
use crate::speedtest;
use crate::store::{ClipEntry, Store};
use crate::ui;
use egui::{Color32, CornerRadius, Margin, RichText, Sense, Stroke, Ui, vec2};
use egui_phosphor::regular as ph;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sub {
    Clipboard,
    Snippets,
    Scratchpad,
    UrlClean,
    QuitApps,
    Ports,
    Cleaner,
    CommandBar,
    QuickToggles,
    CleaningMode,
    Timer,
    Packages,
    Uninstaller,
    AppUpdates,
    Media,
    Shelf,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsPage {
    General,
    Features,
    Monitor,
    Clipboard,
    Snippets,
    UrlClean,
    KeepAwake,
    About,
}

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

    pub alerts: crate::alerts::Alerts,

    pub cmd_query: String,
    pub cmd_selected: usize,
    pub cmd_focus: bool,

    pub qt_outcome: crate::quick_toggles::Outcome,
    pub qt_dark: Option<bool>,
    pub qt_trash: Option<usize>,
    pub qt_checked: Option<Instant>,
    pub qt_confirm_trash: bool,

    pub cleaning: bool,
    pub cleaning_presses: Vec<Instant>,

    pub apps: crate::apps::Shared,
    pub apps_scanned: bool,

    pub timer_end: Option<Instant>,
    pub timer_paused: Option<Duration>,
    pub timer_total: Duration,
    pub timer_minutes: u32,
    pub timer_label: String,

    pub launch_at_login: Option<bool>,

    pub pkg: crate::packages::Shared,
    pub pkg_tab: u8,
    pub pkg_tab_forced: bool,
    pub pkg_query: String,
    pub pkg_filter: String,
    pub pkg_confirm: Option<(crate::packages::Package, crate::packages::Op)>,

    pub media: crate::media::Shared,
    pub media_files: Vec<std::path::PathBuf>,
    pub media_format: crate::media::Format,
    pub media_max: u32,
    pub media_quality: u8,

    pub shelf_selected: HashSet<u64>,
    pub shelf_floating: bool,
    pub shelf_note: String,
    pub shelf_thumbs: std::collections::HashMap<u64, egui::TextureHandle>,
    pub shelf_thumb_queue: crate::shelf_ui::ThumbQueue,
    pub shelf_thumb_requested: HashSet<u64>,

    pub tray: Option<crate::tray::Tray>,
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
            media_format: crate::media::Format::Jpeg,
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
            crate::desktop_entry::ensure();
            if app.store.tray_enabled {
                app.tray = crate::tray::Tray::new(&ctx);
            }
        }
        app
    }

    pub fn mark_dirty(&mut self) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(Instant::now());
        }
        if let Some(w) = &self.clip {
            if let Ok(mut c) = w.config.lock() {
                *c = watch_config(&self.store);
            }
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

    fn poll_mixer(&mut self, ctx: &egui::Context) {
        if self.demo {
            if let Ok(mut m) = self.mixer.lock() {
                if !m.available && self.mixer_last.is_none() {
                    let real = mixer::read();
                    *m = if real.available && !real.streams.is_empty() { real } else { mixer::demo() };
                }
            }
            self.mixer_last = Some(Instant::now());
            return;
        }
        let due = self.mixer_last.is_none_or(|t| t.elapsed() > Duration::from_millis(1500));
        let holding = self.mixer_hold.is_some_and(|t| t.elapsed() < Duration::from_millis(1200));
        let busy = self.mixer_busy.lock().map(|b| *b).unwrap_or(true);
        if due && !holding && !busy {
            self.mixer_last = Some(Instant::now());
            let out = self.mixer.clone();
            let flag = self.mixer_busy.clone();
            let ctx = ctx.clone();
            if let Ok(mut b) = flag.lock() {
                *b = true;
            }
            std::thread::spawn(move || {
                let st = mixer::read();
                if let Ok(mut m) = out.lock() {
                    *m = st;
                }
                if let Ok(mut b) = flag.lock() {
                    *b = false;
                }
                ctx.request_repaint();
            });
        }
        ctx.request_repaint_after(Duration::from_millis(1500));
    }

    fn drain_clipboard(&mut self) {
        let mut changed = false;
        if let Some(w) = &self.clip {
            while let Ok(text) = w.rx.try_recv() {
                if !self.store.is_on("clipboard") {
                    continue;
                }
                let h = &mut self.store.clip_history;
                let pinned = h.iter().position(|e| e.text == text).map(|i| h.remove(i).pinned).unwrap_or(false);
                h.insert(0, ClipEntry { text, pinned, at: chrono::Utc::now().timestamp() });
                let max = self.store.clip_max_items.max(10);
                let mut unpinned = 0;
                h.retain(|e| {
                    if e.pinned {
                        return true;
                    }
                    unpinned += 1;
                    unpinned <= max
                });
                changed = true;
            }
        }
        if changed {
            self.mark_dirty();
        }
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.opts.screenshot.clone() else { return };
        ctx.request_repaint();
        self.frames += 1;
        if self.frames >= 120 && !self.shot_sent {
            self.shot_sent = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let [w, h] = img.size;
            let mut buf = Vec::with_capacity(w * h * 4);
            for p in &img.pixels {
                buf.extend_from_slice(&p.to_srgba_unmultiplied());
            }
            match image::save_buffer(&path, &buf, w as u32, h as u32, image::ExtendedColorType::Rgba8) {
                Ok(_) => println!("Saved screenshot to {}", path.display()),
                Err(e) => eprintln!("Could not save screenshot: {e}"),
            }
            std::process::exit(0);
        }
        if self.frames > 600 {
            eprintln!("Screenshot was not delivered; exiting");
            std::process::exit(1);
        }
    }

    // ----------------------------------------------------------------- panel

    fn tab_bar(&mut self, ui: &mut Ui) {
        let tabs = self.visible_tabs();
        if tabs.is_empty() {
            return;
        }
        let w = ui.available_width();
        let h = 48.0;
        let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        ui.painter().rect(
            rect,
            CornerRadius::same(14),
            ui::CARD,
            Stroke::new(1.0, ui::BORDER),
            egui::StrokeKind::Inside,
        );
        let inner = rect.shrink(5.0);
        let slot = inner.width() / tabs.len() as f32;
        for (i, t) in tabs.iter().enumerate() {
            let r = egui::Rect::from_min_size(inner.min + vec2(slot * i as f32, 0.0), vec2(slot, inner.height()));
            let id = ui.id().with(("tab", t.id()));
            let resp = ui.interact(r, id, Sense::click()).on_hover_text(t.title());
            let selected = self.tab == *t && self.sub.is_none() || self.tab == *t;
            let p = ui.painter();
            if selected {
                p.rect_filled(r.shrink2(vec2(3.0, 0.0)), CornerRadius::same(10), ui::ACCENT_SOFT);
            } else if resp.hovered() {
                p.rect_filled(r.shrink2(vec2(3.0, 0.0)), CornerRadius::same(10), ui::CARD_HOVER);
            }
            let color = if selected { ui::ACCENT } else { ui::MUTED };
            p.text(r.center(), egui::Align2::CENTER_CENTER, t.icon(), egui::FontId::proportional(21.0), color);
            if resp.clicked() {
                self.tab = *t;
                self.sub = None;
            }
        }
    }

    fn panel_title(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if let Some(sub) = self.sub {
                if ui
                    .add(egui::Button::new(RichText::new(format!("{}  Back", ph::CARET_LEFT)).color(ui::ACCENT)).frame(false))
                    .clicked()
                {
                    self.sub = None;
                }
                ui.label(RichText::new(sub_title(sub).to_uppercase()).font(ui::bold(13.0)).color(ui::MUTED).extra_letter_spacing(1.2));
            } else {
                ui::section_title(ui, self.tab.title());
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let r = ui
                    .add(egui::Button::new(RichText::new(ph::SLIDERS).size(17.0).color(ui::MUTED)).frame(false))
                    .on_hover_text("Settings for this section");
                if r.clicked() {
                    self.settings_open = true;
                    self.settings_page = match (self.tab, self.sub) {
                        (_, Some(Sub::Clipboard)) => SettingsPage::Clipboard,
                        (_, Some(Sub::Snippets)) => SettingsPage::Snippets,
                        (_, Some(Sub::UrlClean)) => SettingsPage::UrlClean,
                        (Tab::KeepAwake, _) => SettingsPage::KeepAwake,
                        (Tab::System | Tab::Network | Tab::Disk | Tab::Power, _) => SettingsPage::Monitor,
                        _ => SettingsPage::Features,
                    };
                }
            });
        });
    }

    fn footer(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 10.0) / 2.0;
            if ui::footer_button(ui, ph::GEAR, "Settings", w).clicked() {
                self.settings_open = true;
            }
            ui.add_space(2.0);
            if ui::footer_button(ui, ph::POWER, "Quit", w).clicked() {
                self.quit(ctx);
            }
        });
        ui.add_space(4.0);
    }

    pub(crate) fn panel(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let margin = Margin { left: 16, right: 16, top: 10, bottom: 8 };
        let mut close = false;
        egui::Panel::top("header").show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(margin))
            .show(ui, |ui| {
                // The whole header moves the window (there is no system title bar).
                let head = ui.max_rect().expand2(vec2(16.0, 10.0));
                let drag_rect = egui::Rect::from_min_max(head.min, egui::pos2(head.max.x, head.min.y + 104.0));
                ui::drag_area(ui, drag_rect, "main-drag", false);
                close = ui::window_buttons(ui, head, "main");
                ui::header(ui);
                ui.add_space(8.0);
                self.tab_bar(ui);
                ui.add_space(8.0);
                self.panel_title(ui);
            });
        if close {
            self.close_window(ctx);
        }
        egui::Panel::bottom("footer").show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(Margin::symmetric(16, 6)))
            .show(ui, |ui| {
                if let Some((msg, at)) = &self.toast {
                    if at.elapsed() < Duration::from_millis(2500) {
                        ui.vertical_centered(|ui| {
                            egui::Frame::new()
                                .fill(ui::ACCENT_SOFT)
                                .corner_radius(CornerRadius::same(10))
                                .inner_margin(Margin::symmetric(12, 4))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(format!("{}  {msg}", ph::CHECK_CIRCLE)).color(ui::TEXT));
                                });
                        });
                        ctx.request_repaint_after(Duration::from_millis(300));
                    }
                }
                self.footer(ui, ctx);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(Margin { left: 16, right: 16, top: 2, bottom: 4 }))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    if self.visible_tabs().is_empty() {
                        ui::card(ui, |ui| {
                            ui::empty_note(ui, ph::PACKAGE, "No features are installed. Open Settings → Features to install some.");
                        });
                        return;
                    }
                    match self.sub {
                        Some(sub) => crate::tools_ui::sub_view(self, ui, ctx, sub),
                        None => match self.tab {
                            Tab::KeepAwake => crate::tabs::keep_awake(self, ui),
                            Tab::Mixer => crate::tabs::mixer(self, ui),
                            Tab::System => crate::tabs::system(self, ui),
                            Tab::Network => crate::tabs::network(self, ui, ctx),
                            Tab::Disk => crate::tabs::disk(self, ui),
                            Tab::Power => crate::tabs::power(self, ui),
                            Tab::Utilities => crate::tools_ui::utilities(self, ui),
                            Tab::Controls => crate::tabs::controls(self, ui),
                        },
                    }
                    ui.add_space(6.0);
                });
            });
    }
}

impl ValleApp {
    pub fn quit(&mut self, ctx: &egui::Context) {
        self.quitting = true;
        self.store.save();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// The window's close button: tuck Valler into the tray when there is one.
    pub fn close_window(&mut self, ctx: &egui::Context) {
        if self.tray.is_some() && self.store.close_to_tray {
            self.hide_window(ctx);
        } else {
            self.quit(ctx);
        }
    }

    pub fn hide_window(&mut self, ctx: &egui::Context) {
        self.store.save();
        // Wayland compositors don't let apps hide their own windows; minimize instead.
        if cfg!(target_os = "linux") && std::env::var_os("WAYLAND_DISPLAY").is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
        self.window_hidden = true;
    }

    pub fn show_window(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        self.window_hidden = false;
    }

    /// Handles tray clicks and menu items, and refreshes the tray readout.
    fn tray_tick(&mut self, ctx: &egui::Context) {
        let Some(tray) = self.tray.as_ref() else { return };
        let cmds: Vec<crate::tray::TrayCmd> = tray.rx.try_iter().collect();
        for c in cmds {
            use crate::tray::TrayCmd::*;
            match c {
                Toggle => {
                    let minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
                    if self.window_hidden || minimized {
                        self.show_window(ctx);
                    } else {
                        self.hide_window(ctx);
                    }
                }
                Show => self.show_window(ctx),
                KeepAwake => {
                    if self.awake.active() {
                        self.awake.stop();
                    } else {
                        self.awake.start(self.store.keep_awake_minutes, self.store.keep_display_on);
                    }
                }
                CommandBar => {
                    self.tab = Tab::Utilities;
                    self.sub = Some(Sub::CommandBar);
                    self.cmd_focus = true;
                    self.show_window(ctx);
                }
                System => {
                    self.tab = Tab::System;
                    self.sub = None;
                    self.show_window(ctx);
                }
                Settings => {
                    self.settings_open = true;
                    self.show_window(ctx);
                }
                Shelf => {
                    if self.store.is_installed("shelf") {
                        self.shelf_floating = !self.shelf_floating;
                        ctx.request_repaint();
                    }
                }
                Quit => self.quit(ctx),
            }
        }
        // Closing the window from the system (Alt+F4, Dock, taskbar) also goes to the tray.
        if !self.quitting && self.store.close_to_tray && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.hide_window(ctx);
        }
        let awake = self.awake.active();
        let s = &self.snap;
        use crate::tray::Readout;
        let level = |f: f32| if f > 0.85 { ui::RED } else if f > 0.6 { ui::ORANGE } else { ui::GREEN };
        let (text, gauge) = if !self.store.is_installed("tray_readouts") || !s.ready {
            (None, None)
        } else {
            match self.store.tray_readout {
                Readout::Off => (None, None),
                Readout::Cpu => (Some(format!("CPU {:.0}%", s.cpu_total)), Some((s.cpu_total / 100.0, level(s.cpu_total / 100.0)))),
                Readout::Memory => (Some(format!("Mem {:.0}%", s.mem_frac() * 100.0)), Some((s.mem_frac(), level(s.mem_frac())))),
                Readout::Battery => match &s.battery {
                    Some(b) => (Some(format!("{:.0}%", b.percent)), Some((b.percent / 100.0, if b.percent < 20.0 { ui::RED } else { ui::GREEN }))),
                    None => (None, None),
                },
                Readout::Network => (Some(format!("↓{} ↑{}", ui::fmt_rate(s.rx_rate), ui::fmt_rate(s.tx_rate))), None),
            }
        };
        if let Some(tray) = self.tray.as_mut() {
            tray.set_keep_awake(awake);
            tray.set_readout(text, gauge);
        }
    }
}

pub fn sub_from_id(id: &str) -> Option<Sub> {
    Some(match id {
        "clipboard" => Sub::Clipboard,
        "snippets" => Sub::Snippets,
        "scratchpad" => Sub::Scratchpad,
        "cleanurl" => Sub::UrlClean,
        "quitapps" => Sub::QuitApps,
        "ports" => Sub::Ports,
        "cleaner" => Sub::Cleaner,
        "commandbar" => Sub::CommandBar,
        "toggles" => Sub::QuickToggles,
        "cleaning" => Sub::CleaningMode,
        "timer" => Sub::Timer,
        "packages" => Sub::Packages,
        "updates" => Sub::AppUpdates,
        "uninstaller" => Sub::Uninstaller,
        "media" => Sub::Media,
        "shelf" => Sub::Shelf,
        _ => return None,
    })
}

pub fn sub_title(s: Sub) -> &'static str {
    match s {
        Sub::CommandBar => "Command bar",
        Sub::QuickToggles => "Quick toggles",
        Sub::CleaningMode => "Cleaning mode",
        Sub::Timer => "Timer",
        Sub::Packages => "Package manager",
        Sub::Uninstaller => "Uninstaller",
        Sub::AppUpdates => "App updates",
        Sub::Media => "Media",
        Sub::Shelf => "Shelf",
        Sub::Clipboard => "Clipboard",
        Sub::Snippets => "Snippets",
        Sub::Scratchpad => "Scratchpad",
        Sub::UrlClean => "Clean URL",
        Sub::QuitApps => "Quit apps",
        Sub::Ports => "Port manager",
        Sub::Cleaner => "Cleaner",
    }
}

fn watch_config(s: &Store) -> WatchConfig {
    WatchConfig {
        watch: s.is_on("clipboard"),
        auto_clean_urls: s.is_on("url_clean"),
        extra_params: s.url_extra_params.clone(),
        auto_clear_secs: if s.is_on("clip_autoclear") { s.clip_auto_clear_secs.max(5) } else { 0 },
    }
}

impl eframe::App for ValleApp {
    /// Runs even while the window is hidden, so alerts keep working.
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(s) = self.mon.lock() {
            self.snap = s.clone();
        }
        self.tray_tick(_ctx);
        if !self.demo {
            self.alerts.evaluate(&self.snap, &self.store);
        }
        // Timer finishes even while the window is minimized.
        if let Some(end) = self.timer_end {
            if Instant::now() >= end {
                self.timer_end = None;
                let label = if self.timer_label.trim().is_empty() { "Timer".to_string() } else { self.timer_label.trim().to_string() };
                crate::alerts::notify(&format!("{label} is done"), &format!("{} minutes are up.", self.timer_total.as_secs() / 60));
                self.toast(format!("{label} is done"));
            } else {
                _ctx.request_repaint_after(Duration::from_millis(500));
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Ctrl+K (Cmd+K on macOS) opens the command bar from anywhere in the panel.
        if self.store.is_installed("command_bar")
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::K))
        {
            self.tab = Tab::Utilities;
            self.sub = Some(Sub::CommandBar);
            self.cmd_focus = true;
        }
        if self.sub == Some(Sub::CommandBar) && !self.apps_scanned {
            self.apps_scanned = true;
            crate::apps::scan(self.apps.clone(), ctx.clone());
        }
        if let Some(msg) = self.qt_outcome.lock().ok().and_then(|mut o| o.take()) {
            self.toast(msg);
            self.qt_checked = None;
        }
        if self.cleaning {
            crate::tools_extra::cleaning_viewport(self, &ctx);
        }
        if self.shelf_floating && self.store.is_installed("shelf") {
            crate::shelf_ui::floating(self, &ctx);
        }
        self.awake.tick();
        self.drain_clipboard();
        if self.tab == Tab::Mixer && self.sub.is_none() {
            self.poll_mixer(&ctx);
        }
        if self.awake.active() || self.awake.demo_active() {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        let settings_inline = self.demo && self.opts.settings;
        ui::window_surface(ui);
        if self.demo && self.opts.tab.as_deref() == Some("shelffloat") {
            crate::shelf_ui::floating_contents(self, ui, &ctx);
        } else if settings_inline {
            crate::settings_ui::show(self, ui, &ctx);
        } else {
            self.panel(ui, &ctx);
            ui::resize_grip(ui, "main-grip");
            if self.settings_open {
                let vid = egui::ViewportId::from_hash_of("valler-settings");
                let builder = egui::ViewportBuilder::default()
                    .with_title("Valler Settings")
                    .with_app_id("valler")
                    .with_inner_size([980.0, 680.0])
                    .with_min_inner_size([720.0, 480.0])
                    .with_decorations(false)
                    .with_transparent(true);
                ctx.show_viewport_immediate(vid, builder, |ui, _class| {
                    let c = ui.ctx().clone();
                    ui::window_surface(ui);
                    crate::settings_ui::show(self, ui, &c);
                    ui::resize_grip(ui, "settings-grip");
                    if c.input(|i| i.viewport().close_requested()) {
                        self.settings_open = false;
                    }
                });
            }
        }

        if let Some(t) = self.dirty_since {
            if t.elapsed() > Duration::from_millis(800) && !self.demo {
                self.store.save();
                self.dirty_since = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(900));
            }
        }
        crate::shelf_ui::main_window_drops(self, &ctx);
        self.handle_screenshot(&ctx);
    }

    fn on_exit(&mut self) {
        if !self.demo {
            self.store.save();
        }
    }

    /// Fully transparent: the rounded surface painted in `ui` is the whole window.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

#[allow(dead_code)]
pub fn color_for_frac(f: f32) -> Color32 {
    if f > 0.9 {
        ui::RED
    } else if f > 0.75 {
        ui::ORANGE
    } else {
        ui::ACCENT
    }
}
