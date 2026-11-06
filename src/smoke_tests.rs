//! Interface tests: build Valler on sample data (no window, no real settings,
//! clipboard or tray) and draw every tab, tool, Settings page and the floating
//! Shelf off-screen, checking that each one renders and behaves.

use crate::app::{SettingsPage, Sub, ValleApp};
use crate::features::Tab;
use egui::{Context, Pos2, RawInput, Rect, Ui, Vec2};

/// A Valler on sample data, as the --screenshot mode runs it.
pub(crate) fn demo_app() -> (Context, ValleApp) {
    let ctx = Context::default();
    crate::fonts::install(&ctx);
    crate::ui::install_style(&ctx);
    let opts = crate::LaunchOptions { screenshot: Some("unused.png".into()), tab: None, settings: false };
    let app = ValleApp::with_context(ctx.clone(), opts);
    (ctx, app)
}

/// Runs three frames (egui lays out over a couple of passes) and returns how
/// many shapes the last one painted.
pub(crate) fn draw(ctx: &Context, app: &mut ValleApp, size: Vec2, mut f: impl FnMut(&mut ValleApp, &mut Ui, &Context)) -> usize {
    let mut shapes = 0;
    for _ in 0..3 {
        let raw = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)), ..Default::default() };
        let mut out = ctx.run_ui(raw, |ui| {
            let c = ui.ctx().clone();
            crate::ui::window_surface(ui);
            f(app, ui, &c);
        });
        shapes = out.shapes.len();
        // No GPU here: acknowledge the font/texture uploads a renderer would apply.
        out.textures_delta.clear();
    }
    shapes
}

const PANEL: Vec2 = Vec2::new(400.0, 820.0);
const SETTINGS: Vec2 = Vec2::new(1000.0, 700.0);

const ALL_SUBS: [Sub; 16] = [
    Sub::Clipboard,
    Sub::Snippets,
    Sub::Scratchpad,
    Sub::UrlClean,
    Sub::QuitApps,
    Sub::Ports,
    Sub::Cleaner,
    Sub::CommandBar,
    Sub::QuickToggles,
    Sub::CleaningMode,
    Sub::Timer,
    Sub::Packages,
    Sub::Uninstaller,
    Sub::AppUpdates,
    Sub::Media,
    Sub::Shelf,
];

#[test]
fn every_panel_tab_draws() {
    let (ctx, mut app) = demo_app();
    *app.mixer.lock().unwrap() = crate::mixer::demo();
    for t in Tab::ALL {
        app.tab = t;
        app.sub = None;
        let n = draw(&ctx, &mut app, PANEL, |a, ui, c| a.panel(ui, c));
        assert!(n > 20, "tab {} painted only {n} shapes", t.id());
    }
}

#[test]
fn every_utilities_tool_draws() {
    let (ctx, mut app) = demo_app();
    app.ports = Some(Ok(crate::ports::demo()));
    for s in ALL_SUBS {
        app.tab = Tab::Utilities;
        app.sub = Some(s);
        let n = draw(&ctx, &mut app, PANEL, |a, ui, c| a.panel(ui, c));
        assert!(n > 20, "{s:?} painted only {n} shapes");
        assert!(!crate::app::sub_title(s).is_empty());
    }
}

#[test]
fn every_tool_card_is_installable_and_reachable_by_id() {
    for t in crate::tools_ui::TOOLS {
        assert!(crate::features::get(t.feature).is_some(), "{} has no catalog entry", t.name);
    }
    for id in ["clipboard", "snippets", "scratchpad", "cleanurl", "quitapps", "ports", "cleaner", "commandbar", "toggles", "cleaning", "timer", "packages", "updates", "uninstaller", "media", "shelf"] {
        assert!(crate::app::sub_from_id(id).is_some(), "--tab {id} should open a tool");
    }
    assert!(crate::app::sub_from_id("nope").is_none());
}

#[test]
fn every_settings_page_draws() {
    let (ctx, mut app) = demo_app();
    for p in [
        SettingsPage::General,
        SettingsPage::Features,
        SettingsPage::Monitor,
        SettingsPage::Clipboard,
        SettingsPage::Snippets,
        SettingsPage::UrlClean,
        SettingsPage::KeepAwake,
        SettingsPage::About,
    ] {
        app.settings_page = p;
        let n = draw(&ctx, &mut app, SETTINGS, |a, ui, c| crate::settings_ui::show(a, ui, c));
        assert!(n > 20, "{p:?} painted only {n} shapes");
    }
    app.settings_page = SettingsPage::Features;
    app.hub_tab_support = true;
    assert!(draw(&ctx, &mut app, SETTINGS, |a, ui, c| crate::settings_ui::show(a, ui, c)) > 50);
}

#[test]
fn floating_shelf_draws_with_sample_items() {
    let (ctx, mut app) = demo_app();
    let n = draw(&ctx, &mut app, Vec2::new(372.0, 330.0), |a, ui, c| crate::shelf_ui::floating_contents(a, ui, c));
    assert!(n > 20);
    assert_eq!(app.store.shelf_items.len(), 5);
}

#[test]
fn uninstalling_features_hides_their_tabs() {
    let (ctx, mut app) = demo_app();
    assert_eq!(app.visible_tabs().len(), Tab::ALL.len());
    for f in Tab::Network.features() {
        app.store.installed.remove(*f);
    }
    assert!(!app.tab_visible(Tab::Network));
    app.store.hidden_tabs.insert("disk".into());
    assert!(!app.visible_tabs().contains(&Tab::Disk));
    // With nothing installed at all, the panel still draws (an empty note).
    app.store.installed.clear();
    assert!(app.visible_tabs().is_empty());
    assert!(draw(&ctx, &mut app, PANEL, |a, ui, c| a.panel(ui, c)) > 5);
}

#[test]
fn applying_a_bundle_installs_exactly_its_features() {
    let (_ctx, mut app) = demo_app();
    let preset = &crate::features::PRESETS[2]; // Battery and quiet
    app.store.installed = preset.features.iter().map(|s| s.to_string()).collect();
    let tabs = app.visible_tabs();
    assert!(tabs.contains(&Tab::System) && tabs.contains(&Tab::Power));
    assert!(!tabs.contains(&Tab::Mixer));
}

#[test]
fn closing_without_a_tray_quits() {
    let (ctx, mut app) = demo_app();
    assert!(app.tray.is_none(), "screenshot mode never creates a tray");
    app.close_window(&ctx);
    assert!(app.quitting);
}

#[test]
fn copying_shows_a_confirmation() {
    let (ctx, mut app) = demo_app();
    app.copy_text(&ctx, "hello".into());
    assert_eq!(app.toast.as_ref().map(|t| t.0.as_str()), Some("Copied"));
}

#[test]
fn launch_options_pick_the_starting_screen() {
    let ctx = Context::default();
    let opts = crate::LaunchOptions { screenshot: Some("x.png".into()), tab: Some("timer".into()), settings: false };
    let app = ValleApp::with_context(ctx.clone(), opts);
    assert_eq!(app.sub, Some(Sub::Timer));
    assert!(app.timer_end.is_some(), "the sample timer is running");
    let opts = crate::LaunchOptions { screenshot: Some("x.png".into()), tab: Some("network".into()), settings: false };
    assert_eq!(ValleApp::with_context(ctx.clone(), opts).tab, Tab::Network);
    let opts = crate::LaunchOptions { screenshot: Some("x.png".into()), tab: Some("monitor".into()), settings: true };
    assert_eq!(ValleApp::with_context(ctx, opts).settings_page, SettingsPage::Monitor);
}
