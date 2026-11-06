//! Scratchpad notes in tabs.

use crate::app::ValleApp;
use crate::core::store::NoteTab;
use crate::ui::{self, caption, card};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- scratchpad

pub(crate) fn scratchpad(app: &mut ValleApp, ui: &mut Ui) {
    if app.store.notes.is_empty() {
        app.store.notes.push(NoteTab { title: "Notes".into(), body: String::new() });
    }
    app.store.note_active = app.store.note_active.min(app.store.notes.len() - 1);
    ui.horizontal_wrapped(|ui| {
        for i in 0..app.store.notes.len() {
            let sel = i == app.store.note_active;
            let t = app.store.notes[i].title.clone();
            let b = egui::Button::new(RichText::new(if t.is_empty() { "Untitled".into() } else { t }).size(13.0).color(if sel { Color32::WHITE } else { ui::TEXT }))
                .fill(if sel { ui::ACCENT } else { ui::CONTROL });
            if ui.add(b).clicked() {
                app.store.note_active = i;
                app.mark_dirty();
            }
        }
        if ui.add(egui::Button::new(RichText::new(ph::PLUS)).fill(ui::CONTROL)).on_hover_text("New tab").clicked() {
            app.store.notes.push(NoteTab { title: format!("Note {}", app.store.notes.len() + 1), body: String::new() });
            app.store.note_active = app.store.notes.len() - 1;
            app.mark_dirty();
        }
    });
    let idx = app.store.note_active;
    let mut changed = false;
    // Edit / Preview, like the original's Markdown preview.
    ui.horizontal(|ui| {
        for (preview, label, glyph) in [(false, "Edit", ph::PENCIL_SIMPLE), (true, "Preview", ph::EYE)] {
            let on = app.note_preview == preview;
            let b = egui::Button::new(RichText::new(format!("{glyph} {label}")).size(13.0).color(if on { Color32::WHITE } else { ui::TEXT })).fill(if on { ui::ACCENT } else { ui::CONTROL });
            if ui.add(b).clicked() {
                app.note_preview = preview;
            }
        }
        ui.label(caption("Markdown: # headings, - lists, - [ ] tasks, **bold**, `code`"));
    });
    card(ui, |ui| {
        let n = &mut app.store.notes[idx];
        changed |= ui.add(egui::TextEdit::singleline(&mut n.title).font(ui::bold(15.0)).frame(egui::Frame::NONE).desired_width(f32::INFINITY)).changed();
        ui::separator(ui);
        if app.note_preview {
            let blocks = crate::services::markdown::parse(&n.body);
            if blocks.is_empty() {
                ui.label(caption("Nothing to preview yet."));
            }
            ui::markdown(ui, &blocks);
        } else {
            changed |= ui
                .add(egui::TextEdit::multiline(&mut n.body).hint_text("Write anything. It saves by itself.").frame(egui::Frame::NONE).desired_rows(16).desired_width(f32::INFINITY))
                .changed();
        }
        ui.label(caption(format!("{} words · {} characters", n.body.split_whitespace().count(), n.body.chars().count())));
    });
    if changed {
        app.mark_dirty();
    }
    ui.horizontal(|ui| {
        if ui::soft_button(ui, format!("{} Export to Documents", ph::EXPORT)).clicked() {
            let n = &app.store.notes[idx];
            let dir = dirs::document_dir().or_else(dirs::home_dir).unwrap_or_default();
            let safe: String = n.title.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
            let path = dir.join(format!("{}.md", if safe.trim().is_empty() { "Note".to_string() } else { safe.trim().to_string() }));
            match std::fs::write(&path, &n.body) {
                Ok(_) => app.toast(format!("Saved {}", path.display())),
                Err(e) => app.toast(format!("Could not save: {e}")),
            }
        }
        if app.store.notes.len() > 1 && ui::soft_button(ui, format!("{} Delete tab", ph::TRASH)).clicked() {
            app.store.notes.remove(idx);
            app.store.note_active = 0;
            app.mark_dirty();
        }
    });
}
