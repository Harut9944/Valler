//! The Utilities list and the router that shows the open tool.

use crate::app::{Sub, ValleApp};
use crate::ui::{self};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use super::*;

pub struct Tool {
    pub sub: Sub,
    pub feature: &'static str,
    pub icon: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

pub const TOOLS: &[Tool] = &[
    Tool { sub: Sub::CommandBar, feature: "command_bar", icon: ph::MAGNIFYING_GLASS, name: "Command bar", desc: "Search tools, clipboard, snippets and notes. Calculate and convert units." },
    Tool { sub: Sub::QuickToggles, feature: "quick_toggles", icon: ph::TOGGLE_LEFT, name: "Quick toggles", desc: "Dark mode, lock screen, display off, screen saver and empty the trash." },
    Tool { sub: Sub::Shelf, feature: "shelf", icon: ph::TRAY, name: "Shelf", desc: "Park files, text and links for a moment, then copy or move them where they belong." },
    Tool { sub: Sub::Clipboard, feature: "clipboard", icon: ph::CLIPBOARD_TEXT, name: "Clipboard", desc: "Keeps copied text, images and files to reuse later. Everything stays local and can be cleared anytime." },
    Tool { sub: Sub::Snippets, feature: "snippets", icon: ph::TEXT_AA, name: "Snippets", desc: "Saved text with date, time and clipboard variables, one click to copy." },
    Tool { sub: Sub::Scratchpad, feature: "scratchpad", icon: ph::NOTE_PENCIL, name: "Scratchpad", desc: "Autosaved notes in tabs." },
    Tool { sub: Sub::UrlClean, feature: "url_clean", icon: ph::LINK, name: "Clean URL", desc: "Removes tracking parameters from copied links." },
    Tool { sub: Sub::QuitApps, feature: "quit_apps", icon: ph::X_SQUARE, name: "Quit apps", desc: "See what is running and quit or force quit it." },
    Tool { sub: Sub::Ports, feature: "ports", icon: ph::PLUGS_CONNECTED, name: "Port manager", desc: "Find programs listening on network ports and stop them." },
    Tool { sub: Sub::Cleaner, feature: "cleaner", icon: ph::SPARKLE, name: "Cleaner", desc: "Caches, logs and temporary files." },
    Tool { sub: Sub::Packages, feature: "packages", icon: ph::PACKAGE, name: "Package manager", desc: "Search, install, update and remove apps without a terminal." },
    Tool { sub: Sub::Media, feature: "media", icon: ph::IMAGES, name: "Media", desc: "Convert, resize and compress images locally." },
    Tool { sub: Sub::AppUpdates, feature: "app_updates", icon: ph::ARROW_CIRCLE_UP, name: "App updates", desc: "Check apps from your stores and update them together." },
    Tool { sub: Sub::Uninstaller, feature: "uninstaller", icon: ph::TRASH, name: "Uninstaller", desc: "Removes apps you installed from an app store or package manager." },
    Tool { sub: Sub::Timer, feature: "timer", icon: ph::TIMER, name: "Timer", desc: "Countdown with a notification when time is up." },
    Tool { sub: Sub::CleaningMode, feature: "cleaning_mode", icon: ph::KEYBOARD, name: "Cleaning mode", desc: "Locks the keyboard so you can clean safely." },
];

pub fn utilities(app: &mut ValleApp, ui: &mut Ui) {
    for t in TOOLS {
        if !app.store.is_installed(t.feature) {
            continue;
        }
        let r = ui::clickable_card(ui, |ui| {
            ui::feature_row(ui, t.icon, t.name, t.desc, |ui| {
                ui.label(RichText::new(ph::CARET_RIGHT).color(ui::MUTED));
                if t.sub == Sub::CommandBar {
                    ui::key_chip(ui, if cfg!(target_os = "macos") { "⌘K" } else { "Ctrl+K" });
                }
                if t.sub == Sub::Timer
                    && let Some(left) = app.timer_end.map(|e| e.saturating_duration_since(std::time::Instant::now())).or(app.timer_paused) {
                        ui.label(RichText::new(crate::screens::tools::fmt_clock(left)).font(ui::bold(13.0)).color(ui::ACCENT));
                    }
                if t.sub == Sub::Shelf && !app.store.shelf_items.is_empty() {
                    ui.label(RichText::new(format!("{}", app.store.shelf_items.len())).font(ui::bold(12.5)).color(ui::ACCENT));
                }
                if t.sub == Sub::Clipboard {
                    ui.label(RichText::new(format!("{} saved", app.store.clip_history.len())).size(12.0).color(ui::FAINT));
                }
            });
        });
        if r.clicked() {
            app.sub = Some(t.sub);
            app.pkg_tab_forced = false;
            if t.sub == Sub::CommandBar {
                app.cmd_focus = true;
            }
            if t.sub == Sub::Ports && app.ports.is_none() {
                refresh_ports(app);
            }
        }
    }
}

pub fn sub_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context, sub: Sub) {
    match sub {
        Sub::Clipboard => clipboard(app, ui, ctx),
        Sub::Snippets => snippets_view(app, ui, ctx),
        Sub::Scratchpad => scratchpad(app, ui),
        Sub::UrlClean => url_clean(app, ui, ctx),
        Sub::QuitApps => quit_apps(app, ui),
        Sub::Ports => ports_view(app, ui),
        Sub::Cleaner => cleaner_view(app, ui, ctx),
        Sub::CommandBar => crate::screens::tools::command_bar_view(app, ui, ctx),
        Sub::QuickToggles => crate::screens::tools::quick_toggles_view(app, ui, ctx),
        Sub::CleaningMode => crate::screens::tools::cleaning_view(app, ui),
        Sub::Timer => crate::screens::tools::timer_view(app, ui),
        Sub::Packages => crate::screens::tools::packages_view(app, ui, ctx, None),
        Sub::Uninstaller => crate::screens::tools::packages_view(app, ui, ctx, Some(0)),
        Sub::AppUpdates => crate::screens::tools::packages_view(app, ui, ctx, Some(1)),
        Sub::Media => crate::screens::tools::media_view(app, ui, ctx),
        Sub::Shelf => crate::screens::tools::shelf::shelf_view(app, ui, ctx),
    }
}

pub(crate) fn search(ui: &mut Ui, text: &mut String, hint: &str) {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(format!("{}  {hint}", ph::MAGNIFYING_GLASS))
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(10.0, 7.0)),
    );
}

pub fn ago(ts: i64) -> String {
    let d = (chrono::Utc::now().timestamp() - ts).max(0);
    match d {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", d / 60),
        3600..=86_399 => format!("{} h ago", d / 3600),
        _ => format!("{} d ago", d / 86_400),
    }
}
