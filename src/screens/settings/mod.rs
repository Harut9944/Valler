//! The Settings window: sidebar navigation and one file per page.

use crate::app::{SettingsPage, ValleApp};
use crate::core::features::FEATURES;
use crate::ui::{self, caption};
use egui::{Color32, CornerRadius, Margin, RichText, Sense, Ui, vec2};
use egui_phosphor::regular as ph;

mod about;
mod clipboard;
mod general;
mod hub;
mod keep_awake;
mod monitor;
mod snippets;
mod url_clean;

use about::*;
use clipboard::*;
use general::*;
use hub::*;
use keep_awake::*;
use monitor::*;
use snippets::*;
use url_clean::*;

/// One entry in the Settings sidebar.
struct PageLink {
    page: SettingsPage,
    icon: &'static str,
    name: &'static str,
}

/// A titled group of sidebar entries.
struct Section {
    title: &'static str,
    pages: &'static [PageLink],
}

const fn link(page: SettingsPage, icon: &'static str, name: &'static str) -> PageLink {
    PageLink { page, icon, name }
}

const SECTIONS: &[Section] = &[
    Section {
        title: "Essentials",
        pages: &[
            link(SettingsPage::General, ph::GEAR, "General"),
            link(SettingsPage::Features, ph::SQUARES_FOUR, "Features"),
            link(SettingsPage::Monitor, ph::CHART_LINE, "Monitor"),
            link(SettingsPage::KeepAwake, ph::MOON_STARS, "Keep awake"),
        ],
    },
    Section {
        title: "Files",
        pages: &[
            link(SettingsPage::Clipboard, ph::CLIPBOARD_TEXT, "Clipboard"),
            link(SettingsPage::Snippets, ph::TEXT_AA, "Snippets"),
            link(SettingsPage::UrlClean, ph::LINK, "Clean URL"),
        ],
    },
    Section { title: "Valler", pages: &[link(SettingsPage::About, ph::INFO, "About")] },
];

pub fn show(app: &mut ValleApp, ui: &mut Ui, _ctx: &egui::Context) {
    // The whole window, measured before any panel takes its share.
    let window = ui.max_rect();
    let r = ui::WINDOW_RADIUS;
    let side_fill = Color32::from_rgb(0x22, 0x22, 0x25);
    egui::Panel::left("settings-sidebar").show_separator_line(false)
        .exact_size(230.0)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(side_fill)
                .corner_radius(CornerRadius { nw: r, sw: r, ne: 0, se: 0 })
                .inner_margin(Margin { left: 12, right: 12, top: 14, bottom: 14 }),
        )
        .show(ui, |ui| {
            // The top strip of the sidebar moves the window too.
            let strip = egui::Rect::from_min_size(ui.max_rect().min - vec2(12.0, 14.0), vec2(230.0, 12.0));
            ui::drag_area(ui, strip, "settings-drag-side", true);
            sidebar(app, ui)
        });
    // Title bar: drag area, title and the window buttons (no system frame).
    let mut close = false;
    egui::Panel::top("settings-titlebar")
        .show_separator_line(false)
        .exact_size(40.0)
        .frame(egui::Frame::new().inner_margin(Margin { left: 28, right: 12, top: 8, bottom: 0 }))
        .show(ui, |ui| {
            // Span the title bar from the sidebar to the real right edge of the window,
            // so the window buttons sit inside the rounded border.
            let top = ui.max_rect();
            let bar = egui::Rect::from_min_max(egui::pos2(top.left() - 28.0, window.top()), egui::pos2(window.right(), top.bottom()));
            ui::drag_area(ui, bar, "settings-drag", true);
            ui.label(RichText::new("Valler Settings").font(ui::bold(13.5)).color(ui::MUTED));
            close = ui::window_buttons(ui, bar.shrink2(vec2(6.0, 0.0)).translate(vec2(-2.0, 2.0)), "settings");
        });
    if close {
        app.settings_open = false;
    }
    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(Margin { left: 28, right: 28, top: 4, bottom: 18 }))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let w = ui.available_width().min(860.0);
                ui.set_max_width(w);
                ui.spacing_mut().item_spacing.y = 10.0;
                match app.settings_page {
                    SettingsPage::General => general(app, ui),
                    SettingsPage::Features => hub(app, ui),
                    SettingsPage::Monitor => monitor(app, ui),
                    SettingsPage::KeepAwake => keep_awake(app, ui),
                    SettingsPage::Clipboard => clipboard(app, ui),
                    SettingsPage::Snippets => snippets(app, ui),
                    SettingsPage::UrlClean => url_clean(app, ui),
                    SettingsPage::About => about(app, ui),
                }
                ui.add_space(20.0);
            });
        });
}

pub(crate) fn sidebar(app: &mut ValleApp, ui: &mut Ui) {
    ui.add(
        egui::TextEdit::singleline(&mut app.settings_search)
            .hint_text(format!("{}  Search settings", ph::MAGNIFYING_GLASS))
            .desired_width(f32::INFINITY)
            .margin(vec2(10.0, 7.0)),
    );
    ui.add_space(8.0);
    let q = app.settings_search.to_lowercase();
    for Section { title: section, pages } in SECTIONS {
        let visible: Vec<_> = pages.iter().filter(|l| q.is_empty() || l.name.to_lowercase().contains(&q)).collect();
        if visible.is_empty() {
            continue;
        }
        ui.add_space(6.0);
        ui.label(RichText::new(*section).font(ui::bold(12.5)).color(ui::FAINT));
        for PageLink { page, icon: glyph, name } in visible {
            let sel = app.settings_page == *page;
            let w = ui.available_width();
            let (r, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
            let p = ui.painter();
            if sel {
                p.rect_filled(r, CornerRadius::same(8), ui::ACCENT);
            } else if resp.hovered() {
                p.rect_filled(r, CornerRadius::same(8), ui::CARD_HOVER);
            }
            let ic = if sel { Color32::WHITE } else { ui::ACCENT };
            p.text(r.left_center() + vec2(14.0, 0.0), egui::Align2::LEFT_CENTER, *glyph, egui::FontId::proportional(17.0), ic);
            p.text(r.left_center() + vec2(42.0, 0.0), egui::Align2::LEFT_CENTER, *name, egui::FontId::proportional(15.0), if sel { Color32::WHITE } else { ui::TEXT });
            if resp.clicked() {
                app.settings_page = *page;
            }
        }
    }
    // Features matching a search jump straight to the hub.
    if !q.is_empty() {
        let hits: Vec<_> = FEATURES.iter().filter(|f| f.name.to_lowercase().contains(&q) || f.desc.to_lowercase().contains(&q)).take(6).collect();
        if !hits.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new("Features").font(ui::bold(12.5)).color(ui::FAINT));
            for f in hits {
                if ui.add(egui::Button::new(RichText::new(format!("{}  {}", f.icon, f.name))).frame(false)).clicked() {
                    app.settings_page = SettingsPage::Features;
                }
            }
        }
    }
}

pub(crate) fn page_title(ui: &mut Ui, t: &str, sub: &str) {
    ui.label(RichText::new(t).font(ui::bold(24.0)));
    if !sub.is_empty() {
        ui.label(caption(sub));
    }
    ui.add_space(4.0);
}

pub(crate) fn setting_row(ui: &mut Ui, name: &str, desc: &str, right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 200.0).max(200.0));
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new(name).size(14.5));
            if !desc.is_empty() {
                ui.add(egui::Label::new(caption(desc)).wrap());
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
    });
}

// ------------------------------------------------------------- features hub

pub(crate) fn segmented(ui: &mut Ui, left: &str, right: &str, right_selected: &mut bool) {
    egui::Frame::new()
        .fill(ui::CONTROL)
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (i, t) in [left, right].iter().enumerate() {
                    let sel = (i == 1) == *right_selected;
                    let b = egui::Button::new(RichText::new(*t).size(15.0).color(if sel { Color32::WHITE } else { ui::TEXT }))
                        .fill(if sel { ui::ACCENT } else { Color32::TRANSPARENT })
                        .corner_radius(CornerRadius::same(7))
                        .min_size(vec2(130.0, 32.0));
                    if ui.add(b).clicked() {
                        *right_selected = i == 1;
                    }
                }
            });
        });
}

impl ValleApp {
    /// Pages edit the store directly and set `dirty_flag`; this folds it into the save timer.
    pub fn mark_dirty_if_changed(&mut self) {
        if self.store.dirty_flag {
            self.store.dirty_flag = false;
            self.mark_dirty();
        }
    }
}
