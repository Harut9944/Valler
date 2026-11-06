//! Cleaner.

use crate::app::ValleApp;
use crate::services::cleaner::{self, ScanState};
use crate::ui::{self, caption, card, fmt_bytes, title};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- cleaner

pub(crate) fn cleaner_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if app.demo
        && let Ok(mut s) = app.cleaner.lock()
            && matches!(*s, ScanState::Idle) {
                *s = ScanState::Done(cleaner::demo());
            }
    let state = app.cleaner.lock().map(|s| s.clone()).unwrap_or_default();
    match state {
        ScanState::Idle => {
            card(ui, |ui| {
                ui::empty_note(ui, ph::SPARKLE, "Look for caches, logs and temporary files you can safely remove. Everything goes to the trash, so you can put it back.");
                if ui::primary_button(ui, "Scan").clicked() {
                    cleaner::scan(app.cleaner.clone(), ctx.clone());
                }
            });
        }
        ScanState::Scanning => {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Working…");
                });
            });
        }
        ScanState::Cleaned { freed, failed } => {
            card(ui, |ui| {
                ui.label(title(format!("{} Moved {} to the trash", ph::CHECK_CIRCLE, fmt_bytes(freed))));
                if failed > 0 {
                    ui.label(caption(format!("{failed} items were in use or protected and were skipped.")));
                }
                if ui::soft_button(ui, "Scan again").clicked() {
                    cleaner::scan(app.cleaner.clone(), ctx.clone());
                }
            });
        }
        ScanState::Done(mut items) => {
            let mut changed = false;
            let total: u64 = items.iter().filter(|i| i.selected).map(|i| i.size).sum();
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(title(format!("{} selected", fmt_bytes(total))));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(total > 0, egui::Button::new(RichText::new("Move to trash").color(Color32::WHITE)).fill(ui::ACCENT)).clicked() {
                            cleaner::clean(app.cleaner.clone(), items.clone(), ctx.clone());
                        }
                        if ui::soft_button(ui, "Rescan").clicked() {
                            cleaner::scan(app.cleaner.clone(), ctx.clone());
                        }
                    });
                });
                if items.is_empty() {
                    ui::empty_note(ui, ph::CHECK_CIRCLE, "Nothing worth cleaning.");
                }
                let mut all = !items.is_empty() && items.iter().all(|i| i.selected);
                if !items.is_empty() && ui.checkbox(&mut all, "Select all").changed() {
                    for i in items.iter_mut() {
                        i.selected = all;
                    }
                    changed = true;
                }
                for it in items.iter_mut().take(200) {
                    ui.horizontal(|ui| {
                        changed |= ui.checkbox(&mut it.selected, "").changed();
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 80.0);
                            ui.spacing_mut().item_spacing.y = 0.0;
                            let name = it.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                            ui.add(egui::Label::new(RichText::new(name).size(14.0)).truncate());
                            ui.add(egui::Label::new(RichText::new(format!("{} · {}", it.category, it.path.display())).size(11.5).color(ui::FAINT)).truncate());
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(fmt_bytes(it.size)).color(ui::MUTED));
                        });
                    });
                }
            });
            if changed
                && let Ok(mut s) = app.cleaner.lock() {
                    *s = ScanState::Done(items);
                }
        }
    }
}
