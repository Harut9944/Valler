//! What the Shelf does with its items: add, copy, move, remove, paste and drops.

use crate::app::{Sub, ValleApp};
use crate::services::shelf::{self, Item};
use crate::ui::{self};
use egui::{CornerRadius, Stroke, StrokeKind, vec2};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- shared actions

pub(crate) fn selected_items(app: &ValleApp) -> Vec<Item> {
    let items = &app.store.shelf_items;
    let sel: Vec<Item> = items.iter().filter(|i| app.shelf_selected.contains(&i.id)).cloned().collect();
    if sel.is_empty() { items.clone() } else { sel }
}

/// After items were used: unpinned ones leave the shelf when that option is on.
pub(crate) fn consume(app: &mut ValleApp, used: &[u64]) {
    if app.store.shelf_remove_after_use {
        app.store.shelf_items.retain(|i| i.pinned || !used.contains(&i.id));
        app.shelf_selected.retain(|id| !used.contains(id));
    }
    app.mark_dirty();
}

pub(crate) fn do_copy(app: &mut ValleApp) {
    let items = selected_items(app);
    let refs: Vec<&Item> = items.iter().collect();
    match shelf::copy(&refs) {
        Ok(m) => app.toast(m),
        Err(e) => app.toast(e),
    }
}

pub(crate) fn do_transfer(app: &mut ValleApp, moving: bool) {
    let items = selected_items(app);
    if items.is_empty() {
        return;
    }
    let Some(dir) = rfd::FileDialog::new().set_title(if moving { "Move to folder" } else { "Copy to folder" }).pick_folder() else { return };
    let refs: Vec<&Item> = items.iter().collect();
    let (done, failed, moved) = shelf::transfer(&refs, &dir, moving);
    // Moved files live somewhere new now; keep pinned items pointing at them.
    for (id, p) in moved {
        if let Some(it) = app.store.shelf_items.iter_mut().find(|i| i.id == id) {
            it.path = Some(p);
        }
    }
    let ids: Vec<u64> = items.iter().map(|i| i.id).collect();
    consume(app, &ids);
    let verb = if moving { "Moved" } else { "Copied" };
    app.toast(if failed == 0 { format!("{verb} {done} item{} to {}", if done == 1 { "" } else { "s" }, dir.display()) } else { format!("{verb} {done}, {failed} failed") });
}

pub(crate) fn do_paste(app: &mut ValleApp) {
    match shelf::add_from_clipboard(&mut app.store.shelf_items) {
        Ok(m) | Err(m) => app.toast(m),
    }
    app.mark_dirty();
}

pub(crate) fn do_add_files(app: &mut ValleApp) {
    if let Some(files) = rfd::FileDialog::new().set_title("Add to the Shelf").pick_files() {
        let n = shelf::add_paths(&mut app.store.shelf_items, files);
        if n > 0 {
            app.toast(format!("Added {n} item{}", if n == 1 { "" } else { "s" }));
            app.mark_dirty();
        }
    }
}

pub(crate) fn remove(app: &mut ValleApp, ids: &[u64]) {
    app.store.shelf_items.retain(|i| !ids.contains(&i.id));
    app.shelf_selected.retain(|id| !ids.contains(id));
    app.mark_dirty();
}

pub(crate) fn clear(app: &mut ValleApp) {
    app.store.shelf_items.retain(|i| i.pinned);
    app.shelf_selected.clear();
    app.mark_dirty();
}

/// Ctrl+V (Cmd+V) while the Shelf is showing and no text field is focused.
pub(crate) fn handle_paste_key(app: &mut ValleApp, ctx: &egui::Context) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let pasted = ctx.input(|i| {
        i.events.iter().any(|e| matches!(e, egui::Event::Paste(_)))
            || i.events.iter().any(|e| matches!(e, egui::Event::Key { key: egui::Key::V, pressed: true, modifiers, .. } if modifiers.command))
    });
    if pasted && !app.demo {
        do_paste(app);
    }
}

pub(crate) fn take_drops(app: &mut ValleApp, ctx: &egui::Context) -> usize {
    let paths: Vec<std::path::PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
    if paths.is_empty() {
        return 0;
    }
    let n = shelf::add_paths(&mut app.store.shelf_items, paths);
    if n > 0 {
        app.mark_dirty();
    }
    n
}

/// Files dropped on the main window go to the Shelf (except on the Media page,
/// which takes images for conversion). Draws a hint while files hover.
pub fn main_window_drops(app: &mut ValleApp, ctx: &egui::Context) {
    if !app.store.is_installed("shelf") || app.sub == Some(Sub::Media) {
        return;
    }
    let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
    if hovering {
        drop_overlay(ctx, "Drop to add to the Shelf");
    }
    let n = take_drops(app, ctx);
    if n > 0 {
        app.toast(format!("Added {n} item{} to the Shelf", if n == 1 { "" } else { "s" }));
    }
}

pub(crate) fn drop_overlay(ctx: &egui::Context, text: &str) {
    let rect = ctx.content_rect();
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("shelf-drop")));
    p.rect(rect.shrink(6.0), CornerRadius::same(ui::WINDOW_RADIUS), ui::ACCENT.gamma_multiply(0.18), Stroke::new(2.0, ui::ACCENT), StrokeKind::Inside);
    p.text(rect.center() - vec2(0.0, 14.0), egui::Align2::CENTER_CENTER, ph::TRAY_ARROW_DOWN, egui::FontId::proportional(40.0), ui::ACCENT);
    p.text(rect.center() + vec2(0.0, 22.0), egui::Align2::CENTER_CENTER, text, ui::bold(16.0), ui::TEXT);
}
