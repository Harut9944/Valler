//! Disks tab: activity and free space per drive.

use crate::app::ValleApp;
use crate::ui::{self, bar, caption, card, fmt_bytes, fmt_rate, sparkline, title};
use egui::{RichText, Ui, vec2};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- disks

pub fn disk(app: &mut ValleApp, ui: &mut Ui) {
    let s = app.snap.clone();
    card(ui, |ui| {
        ui.label(RichText::new("Activity").font(ui::bold(15.0)).color(ui::MUTED));
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 8.0) / 2.0;
            for (label, rate, col) in [("Read", s.disk_read_rate, ui::CYAN), ("Write", s.disk_write_rate, ui::ORANGE)] {
                ui.allocate_ui(vec2(w, 48.0), |ui| {
                    ui.set_min_width(w);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new(label).color(col).size(13.0));
                        ui.label(RichText::new(fmt_rate(rate)).font(ui::bold(20.0)));
                    });
                });
            }
        });
        let r: Vec<f32> = s.disk_read_hist.iter().copied().collect();
        let w: Vec<f32> = s.disk_write_hist.iter().copied().collect();
        let max = r.iter().chain(w.iter()).copied().fold(64.0 * 1024.0, f32::max);
        let top = ui.cursor().min;
        sparkline(ui, &r, max, ui::CYAN, 40.0);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(top, vec2(ui.available_width(), 40.0))));
        sparkline(&mut child, &w, max, ui::ORANGE, 40.0);
    });
    if s.disks.is_empty() && s.ready {
        card(ui, |ui| ui::empty_note(ui, ph::HARD_DRIVES, "No drives were reported by the system."));
    }
    for d in &s.disks {
        card(ui, |ui| {
            let used = d.total.saturating_sub(d.avail);
            let frac = used as f32 / d.total.max(1) as f32;
            ui.horizontal(|ui| {
                ui.label(ui::icon(if d.removable { ph::USB } else { ph::HARD_DRIVE }, 24.0, ui::ACCENT));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let name = if d.mount == "/" { "System drive".to_string() } else { d.mount.clone() };
                    ui.label(title(name));
                    ui.label(caption(format!("{} · {}{}", d.name, d.fs, if d.kind == "Unknown(-1)" || d.kind.starts_with("Unknown") { String::new() } else { format!(" · {}", d.kind) })));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:.0}%", frac * 100.0)).font(ui::bold(15.0)));
                });
            });
            let col = if frac > 0.9 { ui::RED } else if frac > 0.8 { ui::ORANGE } else { ui::ACCENT };
            bar(ui, ui.available_width(), frac, col);
            ui.label(caption(format!("{} free of {}", fmt_bytes(d.avail), fmt_bytes(d.total))));
        });
    }
}
