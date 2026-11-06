//! Clipboard history and Paste as plain text.

use crate::app::ValleApp;
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
    let q = app.clip_search.to_lowercase();
    let mut order: Vec<usize> = (0..app.store.clip_history.len())
        .filter(|i| q.is_empty() || app.store.clip_history[*i].text.to_lowercase().contains(&q))
        .collect();
    order.sort_by_key(|i| !app.store.clip_history[*i].pinned);
    if order.is_empty() {
        card(ui, |ui| ui::empty_note(ui, ph::CLIPBOARD, if q.is_empty() { "Nothing copied yet. Copy some text and it shows up here." } else { "No matches." }));
    }
    let mut action: Option<(usize, u8)> = None; // 0 copy, 1 pin, 2 delete
    for i in order.into_iter().take(150) {
        let e = &app.store.clip_history[i];
        let r = ui::clickable_card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 70.0);
                    let preview: String = e.text.lines().take(3).collect::<Vec<_>>().join("\n");
                    let preview = if preview.chars().count() > 220 { format!("{}…", preview.chars().take(220).collect::<String>()) } else { preview };
                    ui.add(egui::Label::new(RichText::new(preview).size(13.5)).wrap());
                    ui.label(RichText::new(format!("{}{} · {} characters", if e.pinned { "📌 " } else { "" }, ago(e.at), e.text.chars().count())).size(11.5).color(ui::FAINT));
                });
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
                let t = app.store.clip_history[i].text.clone();
                app.copy_text(ctx, t);
            }
            1 => {
                app.store.clip_history[i].pinned ^= true;
                app.mark_dirty();
            }
            _ => {
                app.store.clip_history.remove(i);
                app.mark_dirty();
            }
        }
    }
    ui.horizontal(|ui| {
        if ui::soft_button(ui, "Clear history (keep pinned)").clicked() {
            app.store.clip_history.retain(|e| e.pinned);
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
