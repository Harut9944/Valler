//! Media tools: image conversion.

use crate::app::ValleApp;
use crate::services::media::{self, Format};
use crate::ui::{self, caption, card, fmt_bytes, title};
use egui::{Color32, CornerRadius, RichText, Ui};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- media tools

pub fn media_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    let prog = app.media.lock().map(|p| p.clone()).unwrap_or_default();
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ui::icon(ph::IMAGES, 30.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.label(title("Convert and compress images"));
                ui.add(egui::Label::new(caption("Change the format, shrink large photos and reduce file size. Everything happens on this computer; originals are kept.")).wrap());
            });
        });
        ui::separator(ui);
        ui.horizontal(|ui| {
            if ui::soft_button(ui, format!("{} Choose images…", ph::FOLDER_OPEN)).clicked() && !app.demo {
                let picked = media::pick_files();
                if !picked.is_empty() {
                    app.media_files = picked;
                }
            }
            if !app.media_files.is_empty() && ui.add(egui::Button::new(RichText::new("Clear").size(12.5)).frame(false)).clicked() {
                app.media_files.clear();
            }
        });
        // Files dropped on the window are added too.
        let dropped: Vec<std::path::PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        for p in dropped {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if media::EXTENSIONS.contains(&ext.as_str()) && !app.media_files.contains(&p) {
                app.media_files.push(p);
            }
        }
        if app.media_files.is_empty() {
            ui.label(caption("Or drop images onto this window."));
        } else {
            let total: u64 = app.media_files.iter().filter_map(|f| std::fs::metadata(f).ok()).map(|m| m.len()).sum();
            ui.label(RichText::new(format!("{} image{} · {}", app.media_files.len(), if app.media_files.len() == 1 { "" } else { "s" }, fmt_bytes(total))).font(ui::bold(14.0)));
            for f in app.media_files.iter().take(6) {
                ui.add(egui::Label::new(RichText::new(f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()).size(12.0).color(ui::MUTED)).truncate());
            }
            if app.media_files.len() > 6 {
                ui.label(caption(format!("and {} more", app.media_files.len() - 6)));
            }
        }
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label("Format");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("media-format").selected_text(app.media_format.label()).show_ui(ui, |ui| {
                    for f in Format::ALL {
                        ui.selectable_value(&mut app.media_format, f, f.label());
                    }
                });
            });
        });
        ui.horizontal(|ui| {
            ui.label("Largest side");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = |v: u32| if v == 0 { "Keep size".to_string() } else { format!("{v} px") };
                egui::ComboBox::from_id_salt("media-size").selected_text(label(app.media_max)).show_ui(ui, |ui| {
                    for v in [0u32, 3840, 2560, 1920, 1280, 800] {
                        ui.selectable_value(&mut app.media_max, v, label(v));
                    }
                });
            });
        });
        if app.media_format == Format::Jpeg {
            ui.horizontal(|ui| {
                ui.label("Quality");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::Slider::new(&mut app.media_quality, 30..=100).suffix(" %"));
                });
            });
        }
        ui.add_space(4.0);
        let w = ui.available_width();
        let can = !app.media_files.is_empty() && !prog.running;
        let b = egui::Button::new(RichText::new(format!("{}  Convert", ph::ARROWS_CLOCKWISE)).font(ui::bold(14.5)).color(Color32::WHITE))
            .fill(ui::ACCENT)
            .corner_radius(CornerRadius::same(10))
            .min_size(egui::vec2(w, 36.0));
        if ui.add_enabled(can, b).clicked() {
            media::start(
                app.media.clone(),
                ctx.clone(),
                media::Job { files: app.media_files.clone(), format: app.media_format, max_side: app.media_max, quality: app.media_quality },
            );
        }
    });
    if prog.total > 0 {
        card(ui, |ui| {
            if prog.running {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Converting {} of {}…", prog.done + 1, prog.total));
                });
                ui::bar(ui, ui.available_width(), prog.done as f32 / prog.total.max(1) as f32, ui::ACCENT);
            } else {
                let saved = if prog.saved_bytes >= 0 { format!("{} smaller", fmt_bytes(prog.saved_bytes as u64)) } else { format!("{} larger", fmt_bytes((-prog.saved_bytes) as u64)) };
                ui.label(title(format!("{} Converted {} image{}", ph::CHECK_CIRCLE, prog.outputs.len(), if prog.outputs.len() == 1 { "" } else { "s" })));
                ui.label(caption(format!("Saved next to the originals · {saved} in total")));
            }
            for e in prog.errors.iter().take(5) {
                ui.label(RichText::new(e).size(12.0).color(ui::ORANGE));
            }
        });
    }
}
