//! Small pieces the tab pages share: collapsible rows, usage rows, tiles.

use crate::app::ValleApp;
use crate::ui::{self, bar};
use egui::{Color32, CornerRadius, RichText, Sense, Ui};
use egui_phosphor::regular as ph;

pub(crate) fn chevron_row(app: &mut ValleApp, ui: &mut Ui, key: &'static str, label: &str) -> bool {
    let open = app.expanded.contains(key);
    let r = ui
        .horizontal(|ui| {
            ui.label(RichText::new(if open { ph::CARET_DOWN } else { ph::CARET_RIGHT }).color(ui::MUTED));
            ui.label(RichText::new(label).size(15.0).color(ui::TEXT));
        })
        .response;
    let r = ui.interact(r.rect, ui.id().with(key), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    if r.clicked() {
        if open {
            app.expanded.remove(key);
        } else {
            app.expanded.insert(key);
        }
    }
    open
}

pub(crate) fn usage_row(app: &mut ValleApp, ui: &mut Ui, key: &'static str, label: &str, frac: f32, color: Color32, value: String) -> bool {
    let mut open = false;
    ui.horizontal(|ui| {
        let left = ui.cursor().min.x;
        open = chevron_row(app, ui, key, label);
        let used = ui.cursor().min.x - left;
        let pad = (110.0 - used).max(4.0);
        ui.add_space(pad);
        let w = ui.available_width() - 64.0;
        bar(ui, w, frac, color);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).font(ui::bold(15.0)));
        });
    });
    open
}

pub(crate) fn temp_text(app: &ValleApp, c: Option<f32>) -> String {
    match c {
        Some(c) if app.store.temperature_fahrenheit => format!("{:.0} °F", c * 9.0 / 5.0 + 32.0),
        Some(c) => format!("{c:.0} °C"),
        None => "—".into(),
    }
}

pub(crate) fn temp_tile(ui: &mut Ui, w: f32, glyph: &str, label: &str, value: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(0x31, 0x31, 0x35))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(6, 8))
        .show(ui, |ui| {
            ui.set_width(w - 12.0);
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(RichText::new(format!("{glyph} {label}")).size(13.0).color(ui::MUTED));
                ui.label(RichText::new(value).font(ui::bold(21.0)));
            });
        });
}

pub(crate) fn grid_row(ui: &mut Ui, k: &str, v: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(k).color(ui::MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(v).size(14.0));
        });
    });
}

pub(crate) fn short(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    }
}

pub fn fmt_secs(s: u32) -> String {
    if s < 60 { format!("{s} seconds") } else if s == 60 { "1 minute".into() } else { format!("{} minutes", s / 60) }
}
