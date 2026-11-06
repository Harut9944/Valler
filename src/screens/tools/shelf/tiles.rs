//! The tile grid (with image thumbnails and the right-click menu) and the action bar.

use crate::app::ValleApp;
use crate::services::shelf::{self, Kind};
use crate::ui::{self, fmt_bytes};
use egui::{Color32, CornerRadius, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- thumbnails

// ------------------------------------------------------------- tiles

enum TileAction {
    Open(u64),
    Reveal(u64),
    CopyOne(u64),
    Pin(u64),
    Remove(u64),
}

pub(crate) fn tiles(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context, tile: f32) {
    let items = app.store.shelf_items.clone();
    if items.is_empty() {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), tile + 30.0), Sense::hover());
        let p = ui.painter();
        p.rect(rect, CornerRadius::same(12), Color32::TRANSPARENT, Stroke::new(1.5, ui::BORDER), StrokeKind::Inside);
        p.text(rect.center() - vec2(0.0, 14.0), egui::Align2::CENTER_CENTER, ph::TRAY, egui::FontId::proportional(30.0), ui::FAINT);
        let hint = if cfg!(target_os = "macos") { "Drop files here, or press ⌘V" } else { "Drop files here, or press Ctrl+V" };
        p.text(rect.center() + vec2(0.0, 18.0), egui::Align2::CENTER_CENTER, hint, egui::FontId::proportional(13.0), ui::MUTED);
        return;
    }
    let gap = 8.0;
    let cols = (((ui.available_width() + gap) / (tile + gap)).floor() as usize).max(1);
    let label_h = 34.0;
    let mut action: Option<TileAction> = None;
    let mut click: Option<(u64, bool)> = None;
    for row in items.chunks(cols) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for it in row {
                let (rect, resp) = ui.allocate_exact_size(vec2(tile, tile + label_h), Sense::click());
                let selected = app.shelf_selected.contains(&it.id);
                let exists = it.exists() || app.demo;
                let p = ui.painter().clone();
                let fill = if selected { ui::ACCENT_SOFT } else if resp.hovered() { ui::CARD_HOVER } else { Color32::from_rgb(0x30, 0x30, 0x34) };
                p.rect(rect, CornerRadius::same(10), fill, Stroke::new(if selected { 1.5 } else { 1.0 }, if selected { ui::ACCENT } else { ui::BORDER }), StrokeKind::Inside);
                let art = egui::Rect::from_min_size(rect.min + vec2(8.0, 8.0), vec2(tile - 16.0, tile - 16.0));
                let tex = match (&it.path, exists && it.kind == Kind::Image) {
                    (Some(p), true) => app.thumbs.get(ctx, p),
                    _ => None,
                };
                if let Some(t) = tex {
                    let size = t.size_vec2();
                    let scale = (art.width() / size.x).min(art.height() / size.y);
                    let r = egui::Rect::from_center_size(art.center(), size * scale);
                    p.image(t.id(), r, egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                } else {
                    let col = match it.kind {
                        Kind::Folder => Color32::from_rgb(0x5a, 0xa9, 0xff),
                        Kind::Link => ui::CYAN,
                        Kind::Text => ui::YELLOW,
                        Kind::Image => ui::PURPLE,
                        Kind::File => ui::TEXT,
                    };
                    let col = if exists { col } else { ui::FAINT };
                    p.text(art.center(), egui::Align2::CENTER_CENTER, it.icon(), egui::FontId::proportional(tile * 0.42), col);
                    if let Some(ext) = it.path.as_ref().and_then(|p| p.extension()).map(|e| e.to_string_lossy().to_uppercase())
                        && it.kind == Kind::File && ext.len() <= 5 {
                            p.text(art.center_bottom() - vec2(0.0, 4.0), egui::Align2::CENTER_BOTTOM, ext, ui::bold(10.0), ui::MUTED);
                        }
                }
                // Name, up to two lines.
                let name = it.title();
                let mut job = egui::text::LayoutJob::simple(name, egui::FontId::proportional(11.5), if exists { ui::TEXT } else { ui::FAINT }, tile - 8.0);
                job.wrap.max_rows = 2;
                job.halign = egui::Align::Center;
                let galley = ui.fonts_mut(|f| f.layout_job(job));
                p.galley(pos2(rect.center().x, rect.bottom() - label_h + 2.0), galley, ui::TEXT);
                if it.pinned {
                    p.text(rect.right_top() + vec2(-8.0, 8.0), egui::Align2::RIGHT_TOP, ph::PUSH_PIN, egui::FontId::proportional(13.0), ui::ACCENT);
                }
                if !exists {
                    p.text(rect.left_top() + vec2(8.0, 8.0), egui::Align2::LEFT_TOP, ph::WARNING, egui::FontId::proportional(13.0), ui::ORANGE);
                }
                let tip = match (&it.path, exists) {
                    (Some(path), true) => format!("{}{}", path.display(), it.size().map(|s| format!(" · {}", fmt_bytes(s))).unwrap_or_default()),
                    (Some(path), false) => format!("{} (moved or deleted)", path.display()),
                    (None, _) => it.text.chars().take(400).collect(),
                };
                let resp = resp.on_hover_text(tip);
                if resp.double_clicked() {
                    action = Some(TileAction::Open(it.id));
                } else if resp.clicked() {
                    click = Some((it.id, ui.input(|i| i.modifiers.command || i.modifiers.shift)));
                }
                let id = it.id;
                let (is_path, pinned) = (it.path.is_some(), it.pinned);
                resp.context_menu(|ui| {
                    if (is_path || it.kind == Kind::Link) && ui.button(format!("{}  Open", ph::ARROW_SQUARE_OUT)).clicked() {
                        action = Some(TileAction::Open(id));
                        ui.close();
                    }
                    if is_path && ui.button(format!("{}  Show in folder", ph::FOLDER_OPEN)).clicked() {
                        action = Some(TileAction::Reveal(id));
                        ui.close();
                    }
                    if ui.button(format!("{}  Copy", ph::COPY)).clicked() {
                        action = Some(TileAction::CopyOne(id));
                        ui.close();
                    }
                    if ui.button(format!("{}  {}", ph::PUSH_PIN, if pinned { "Unpin" } else { "Pin (keep after use)" })).clicked() {
                        action = Some(TileAction::Pin(id));
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(RichText::new(format!("{}  Remove from Shelf", ph::X)).color(ui::RED)).clicked() {
                        action = Some(TileAction::Remove(id));
                        ui.close();
                    }
                });
            }
        });
        ui.add_space(gap - ui.spacing().item_spacing.y);
    }
    if let Some((id, multi)) = click {
        if multi {
            if !app.shelf_selected.remove(&id) {
                app.shelf_selected.insert(id);
            }
        } else if app.shelf_selected.len() == 1 && app.shelf_selected.contains(&id) {
            app.shelf_selected.clear();
        } else {
            app.shelf_selected.clear();
            app.shelf_selected.insert(id);
        }
    }
    if let Some(a) = action {
        if app.demo {
            return;
        }
        let find = |id: u64| app.store.shelf_items.iter().find(|i| i.id == id).cloned();
        match a {
            TileAction::Open(id) => {
                if let Some(it) = find(id) {
                    shelf::open(&it);
                }
            }
            TileAction::Reveal(id) => {
                if let Some(it) = find(id) {
                    shelf::reveal(&it);
                }
            }
            TileAction::CopyOne(id) => {
                if let Some(it) = find(id) {
                    match shelf::copy(&[&it]) {
                        Ok(m) | Err(m) => app.toast(m),
                    }
                }
            }
            TileAction::Pin(id) => {
                if let Some(it) = app.store.shelf_items.iter_mut().find(|i| i.id == id) {
                    it.pinned = !it.pinned;
                }
                app.mark_dirty();
            }
            TileAction::Remove(id) => remove(app, &[id]),
        }
    }
}

pub(crate) fn action_bar(app: &mut ValleApp, ui: &mut Ui) {
    let any = !app.store.shelf_items.is_empty();
    let n_sel = app.shelf_selected.len();
    let scope = if n_sel > 0 { format!("{n_sel} selected") } else { "all".into() };
    ui.horizontal_wrapped(|ui| {
        ui.add_enabled_ui(any, |ui| {
            if ui.add(egui::Button::new(RichText::new(format!("{} Copy", ph::COPY)).color(Color32::WHITE)).fill(ui::ACCENT)).on_hover_text(format!("Copy {scope} — paste files in any folder, text anywhere")).clicked() && !app.demo {
                do_copy(app);
            }
            if ui::soft_button(ui, format!("{} Copy to…", ph::FOLDER_SIMPLE_PLUS)).on_hover_text(format!("Copy {scope} into a folder")).clicked() && !app.demo {
                do_transfer(app, false);
            }
            if ui::soft_button(ui, format!("{} Move to…", ph::ARROW_SQUARE_RIGHT)).on_hover_text(format!("Move {scope} into a folder")).clicked() && !app.demo {
                do_transfer(app, true);
            }
            let label = if n_sel > 0 { format!("{} Remove", ph::X) } else { format!("{} Clear", ph::TRASH) };
            if ui::soft_button(ui, label).on_hover_text(if n_sel > 0 { "Take the selected items off the Shelf" } else { "Take everything except pinned items off the Shelf. Your files are not deleted." }).clicked() {
                if n_sel > 0 {
                    let ids: Vec<u64> = app.shelf_selected.iter().copied().collect();
                    remove(app, &ids);
                } else {
                    clear(app);
                }
            }
        });
    });
}
