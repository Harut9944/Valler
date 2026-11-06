//! Snippet library.

use crate::app::ValleApp;
use crate::core::store::Snippet;
use crate::ui::{self, caption, card, title};
use crate::services::snippets;
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- snippets

pub(crate) fn snippets_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    card(ui, |ui| {
        ui.label(caption("Variables: {{date}}, {{time}}, {{datetime}}, {{clipboard}}, {{name}}. A format after a colon picks the look, like {{date:%d.%m.%Y}}."));
    });
    let clip_now = app.store.clip_history.first().map(|e| e.text.clone()).unwrap_or_default();
    let mut action: Option<(usize, u8)> = None;
    for (i, s) in app.store.snippets.iter().enumerate() {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui::key_chip(ui, &s.trigger);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(ph::TRASH).color(ui::MUTED)).frame(false)).on_hover_text("Delete").clicked() {
                        action = Some((i, 2));
                    }
                    if ui.add(egui::Button::new(RichText::new(ph::PENCIL_SIMPLE).color(ui::MUTED)).frame(false)).on_hover_text("Edit").clicked() {
                        action = Some((i, 1));
                    }
                    if ui::primary_button(ui, format!("{} Copy", ph::COPY)).clicked() {
                        action = Some((i, 0));
                    }
                });
            });
            let preview = snippets::expand(&s.text, &clip_now);
            ui.add(egui::Label::new(RichText::new(preview).size(13.5)).wrap());
        });
    }
    if let Some((i, a)) = action {
        match a {
            0 => {
                let t = snippets::expand(&app.store.snippets[i].text, &clip_now);
                app.copy_text(ctx, t);
            }
            1 => {
                app.snip_trigger = app.store.snippets[i].trigger.clone();
                app.snip_text = app.store.snippets[i].text.clone();
                app.snip_editing = Some(i);
            }
            _ => {
                app.store.snippets.remove(i);
                app.snip_editing = None;
                app.mark_dirty();
            }
        }
    }
    card(ui, |ui| {
        ui.label(title(if app.snip_editing.is_some() { "Edit snippet" } else { "New snippet" }));
        ui.add(egui::TextEdit::singleline(&mut app.snip_trigger).hint_text("Name or trigger, like ;addr").desired_width(f32::INFINITY));
        ui.add(egui::TextEdit::multiline(&mut app.snip_text).hint_text("Text to insert").desired_rows(3).desired_width(f32::INFINITY));
        ui.horizontal(|ui| {
            let ok = !app.snip_trigger.trim().is_empty() && !app.snip_text.is_empty();
            if ui.add_enabled(ok, egui::Button::new(RichText::new("Save").color(Color32::WHITE)).fill(ui::ACCENT)).clicked() {
                let sn = Snippet { trigger: app.snip_trigger.trim().to_string(), text: app.snip_text.clone() };
                match app.snip_editing.take() {
                    Some(i) if i < app.store.snippets.len() => app.store.snippets[i] = sn,
                    _ => app.store.snippets.push(sn),
                }
                app.snip_trigger.clear();
                app.snip_text.clear();
                app.mark_dirty();
            }
            if app.snip_editing.is_some() && ui::soft_button(ui, "Cancel").clicked() {
                app.snip_editing = None;
                app.snip_trigger.clear();
                app.snip_text.clear();
            }
        });
    });
}
