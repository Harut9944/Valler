//! Clipboard history and Paste as plain text.

use crate::app::ValleApp;
use crate::app::forget_images;
use crate::services::clipboard::history::{self, ClipEntry, ClipKind};
use crate::ui::{self, caption, card};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- clipboard

pub(crate) fn clipboard(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if !app.store.is_on("clipboard") {
        card(ui, |ui| {
            ui.label(caption("Clipboard history is off. Turn it on to start saving what you copy."));
            if ui::primary_button(ui, "Turn on clipboard history").clicked() {
                app.store.set_on("clipboard", true);
                app.mark_dirty();
            }
        });
    }
    search(ui, &mut app.clip_search, "Search clipboard history");
    let order = history::search(&app.store.clip_history, &app.clip_search);
    if order.is_empty() {
        let empty = app.clip_search.trim().is_empty();
        card(ui, |ui| ui::empty_note(ui, ph::CLIPBOARD, if empty { "Nothing copied yet. Copy text, images or files and they show up here." } else { "No matches." }));
    }
    let mut action: Option<(usize, u8)> = None; // 0 copy, 1 pin, 2 delete
    for i in order.into_iter().take(150) {
        let e = app.store.clip_history[i].clone();
        let r = ui::clickable_card(ui, |ui| {
            ui.horizontal(|ui| {
                entry_preview(app, ui, &e);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(ph::TRASH).color(ui::MUTED)).frame(false)).on_hover_text("Delete").clicked() {
                        action = Some((i, 2));
                    }
                    let pin = if e.pinned { ph::PUSH_PIN_SLASH } else { ph::PUSH_PIN };
                    if ui.add(egui::Button::new(RichText::new(pin).color(if e.pinned { ui::ACCENT } else { ui::MUTED })).frame(false)).on_hover_text(if e.pinned { "Unpin" } else { "Pin" }).clicked() {
                        action = Some((i, 1));
                    }
                });
            });
        });
        if r.clicked() && action.is_none() {
            action = Some((i, 0));
        }
    }
    if let Some((i, a)) = action {
        match a {
            0 => {
                let e = app.store.clip_history[i].clone();
                copy_entry(app, ctx, &e);
            }
            1 => {
                app.store.clip_history[i].pinned ^= true;
                app.mark_dirty();
            }
            _ => {
                forget_images(history::remove(&mut app.store.clip_history, i));
                app.mark_dirty();
            }
        }
    }
    ui.horizontal(|ui| {
        if ui::soft_button(ui, "Clear history (keep pinned)").clicked() {
            forget_images(history::clear_unpinned(&mut app.store.clip_history));
            app.mark_dirty();
        }
        if ui::soft_button(ui, "Empty system clipboard").clicked() {
            if let Some(w) = &app.clip {
                w.clear();
            }
            app.toast("Clipboard emptied");
        }
    });
    if app.store.is_installed("paste_plain") && ui::soft_button(ui, format!("{} Make clipboard plain text", ph::TEXT_T)).on_hover_text("Removes formatting so the next paste is plain text").clicked() {
        make_plain(app, ctx);
    }
}

/// The left part of a history row: text preview, image thumbnail or file names.
fn entry_preview(app: &mut ValleApp, ui: &mut Ui, e: &ClipEntry) {
    let meta = |extra: String| format!("{}{} · {extra}", if e.pinned { "📌 " } else { "" }, ago(e.at));
    match e.kind {
        ClipKind::Text => {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() - 70.0);
                let preview: String = e.text.lines().take(3).collect::<Vec<_>>().join("\n");
                let preview = if preview.chars().count() > 220 { format!("{}…", preview.chars().take(220).collect::<String>()) } else { preview };
                ui.add(egui::Label::new(RichText::new(preview).size(13.5)).wrap());
                ui.label(RichText::new(meta(format!("{} characters", e.text.chars().count()))).size(11.5).color(ui::FAINT));
            });
        }
        ClipKind::Image => {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(72.0, 54.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, egui::CornerRadius::same(6), ui::CONTROL);
            let shown = e.image.as_ref().is_some_and(|p| app.thumbs.paint(ui, p, rect.shrink(2.0)));
            if !shown {
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, ph::IMAGE, egui::FontId::proportional(22.0), ui::MUTED);
            }
            ui.vertical(|ui| {
                ui.label(RichText::new(&e.text).size(13.5));
                ui.label(RichText::new(meta("image".into())).size(11.5).color(ui::FAINT));
            });
        }
        ClipKind::Files => {
            ui.label(ui::icon(if e.files.len() > 1 { ph::FILES } else { ph::FILE }, 24.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() - 70.0);
                ui.add(egui::Label::new(RichText::new(&e.text).size(13.5)).truncate());
                let folder = e.files.first().and_then(|f| f.parent()).map(|p| p.display().to_string()).unwrap_or_default();
                ui.add(egui::Label::new(RichText::new(meta(folder)).size(11.5).color(ui::FAINT)).truncate());
            });
        }
    }
}

/// Puts a history entry back on the system clipboard as what it was.
pub(crate) fn copy_entry(app: &mut ValleApp, ctx: &egui::Context, e: &ClipEntry) {
    match (e.kind, &app.clip) {
        (ClipKind::Image, Some(w)) => match &e.image {
            Some(p) if p.exists() => {
                w.copy_image(p.clone());
                app.toast("Image copied");
            }
            _ => app.toast("That image is no longer saved"),
        },
        (ClipKind::Files, Some(w)) => {
            w.copy_files(e.files.clone());
            app.toast("Files copied — paste them in any folder");
        }
        _ => app.copy_text(ctx, e.text.clone()),
    }
}

/// Re-puts the clipboard text without formatting (Paste as plain text).
pub fn make_plain(app: &mut ValleApp, ctx: &egui::Context) {
    match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
        Ok(t) if !t.is_empty() => {
            app.copy_text(ctx, t);
            app.toast("Clipboard is now plain text");
        }
        _ => app.toast("There's no text on the clipboard"),
    }
}
