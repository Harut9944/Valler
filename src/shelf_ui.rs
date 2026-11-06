//! Shelf screens: the full Shelf page in Utilities, the floating always-on-top
//! Shelf window, the tile grid they share, and file drops on the main window.

use crate::app::{Sub, ValleApp};
use crate::shelf::{self, Item, Kind};
use crate::ui::{self, caption, card, fmt_bytes, title};
use egui::{Color32, CornerRadius, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use egui_phosphor::regular as ph;
use std::sync::{Arc, Mutex};

pub type ThumbQueue = Arc<Mutex<Vec<(u64, egui::ColorImage)>>>;

// ------------------------------------------------------------- shared actions

fn selected_items(app: &ValleApp) -> Vec<Item> {
    let items = &app.store.shelf_items;
    let sel: Vec<Item> = items.iter().filter(|i| app.shelf_selected.contains(&i.id)).cloned().collect();
    if sel.is_empty() { items.clone() } else { sel }
}

/// After items were used: unpinned ones leave the shelf when that option is on.
fn consume(app: &mut ValleApp, used: &[u64]) {
    if app.store.shelf_remove_after_use {
        app.store.shelf_items.retain(|i| i.pinned || !used.contains(&i.id));
        app.shelf_selected.retain(|id| !used.contains(id));
    }
    app.mark_dirty();
}

fn do_copy(app: &mut ValleApp) {
    let items = selected_items(app);
    let refs: Vec<&Item> = items.iter().collect();
    match shelf::copy(&refs) {
        Ok(m) => app.toast(m),
        Err(e) => app.toast(e),
    }
}

fn do_transfer(app: &mut ValleApp, moving: bool) {
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

fn do_paste(app: &mut ValleApp) {
    match shelf::add_from_clipboard(&mut app.store.shelf_items) {
        Ok(m) | Err(m) => app.toast(m),
    }
    app.mark_dirty();
}

fn do_add_files(app: &mut ValleApp) {
    if let Some(files) = rfd::FileDialog::new().set_title("Add to the Shelf").pick_files() {
        let n = shelf::add_paths(&mut app.store.shelf_items, files);
        if n > 0 {
            app.toast(format!("Added {n} item{}", if n == 1 { "" } else { "s" }));
            app.mark_dirty();
        }
    }
}

fn remove(app: &mut ValleApp, ids: &[u64]) {
    app.store.shelf_items.retain(|i| !ids.contains(&i.id));
    app.shelf_selected.retain(|id| !ids.contains(id));
    app.mark_dirty();
}

fn clear(app: &mut ValleApp) {
    app.store.shelf_items.retain(|i| i.pinned);
    app.shelf_selected.clear();
    app.mark_dirty();
}

/// Ctrl+V (Cmd+V) while the Shelf is showing and no text field is focused.
fn handle_paste_key(app: &mut ValleApp, ctx: &egui::Context) {
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

fn take_drops(app: &mut ValleApp, ctx: &egui::Context) -> usize {
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

fn drop_overlay(ctx: &egui::Context, text: &str) {
    let rect = ctx.content_rect();
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("shelf-drop")));
    p.rect(rect.shrink(6.0), CornerRadius::same(ui::WINDOW_RADIUS), ui::ACCENT.gamma_multiply(0.18), Stroke::new(2.0, ui::ACCENT), StrokeKind::Inside);
    p.text(rect.center() - vec2(0.0, 14.0), egui::Align2::CENTER_CENTER, ph::TRAY_ARROW_DOWN, egui::FontId::proportional(40.0), ui::ACCENT);
    p.text(rect.center() + vec2(0.0, 22.0), egui::Align2::CENTER_CENTER, text, ui::bold(16.0), ui::TEXT);
}

// ------------------------------------------------------------- thumbnails

fn thumbnail(app: &mut ValleApp, ctx: &egui::Context, it: &Item) -> Option<egui::TextureHandle> {
    if let Some(t) = app.shelf_thumbs.get(&it.id) {
        return Some(t.clone());
    }
    // Upload finished decodes.
    let ready: Vec<(u64, egui::ColorImage)> = app.shelf_thumb_queue.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
    for (id, img) in ready {
        let tex = ctx.load_texture(format!("shelf-thumb-{id}"), img, egui::TextureOptions::LINEAR);
        app.shelf_thumbs.insert(id, tex);
    }
    if let Some(t) = app.shelf_thumbs.get(&it.id) {
        return Some(t.clone());
    }
    if it.kind == Kind::Image && app.shelf_thumb_requested.insert(it.id) {
        if let Some(path) = it.path.clone() {
            let q = app.shelf_thumb_queue.clone();
            let (id, ctx) = (it.id, ctx.clone());
            std::thread::spawn(move || {
                if let Ok(img) = image::open(&path) {
                    let t = img.thumbnail(192, 192).to_rgba8();
                    let ci = egui::ColorImage::from_rgba_unmultiplied([t.width() as usize, t.height() as usize], t.as_raw());
                    if let Ok(mut q) = q.lock() {
                        q.push((id, ci));
                    }
                    ctx.request_repaint();
                }
            });
        }
    }
    None
}

// ------------------------------------------------------------- tiles

enum TileAction {
    Open(u64),
    Reveal(u64),
    CopyOne(u64),
    Pin(u64),
    Remove(u64),
}

fn tiles(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context, tile: f32) {
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
                let tex = if exists { thumbnail(app, ctx, it) } else { None };
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
                    if let Some(ext) = it.path.as_ref().and_then(|p| p.extension()).map(|e| e.to_string_lossy().to_uppercase()) {
                        if it.kind == Kind::File && ext.len() <= 5 {
                            p.text(art.center_bottom() - vec2(0.0, 4.0), egui::Align2::CENTER_BOTTOM, ext, ui::bold(10.0), ui::MUTED);
                        }
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

fn action_bar(app: &mut ValleApp, ui: &mut Ui) {
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

// ------------------------------------------------------------- full page

pub fn shelf_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if app.demo && app.store.shelf_items.is_empty() {
        app.store.shelf_items = shelf::demo();
        app.shelf_selected.insert(2);
    }
    handle_paste_key(app, ctx);
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ui::icon(ph::TRAY, 22.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(title("Shelf"));
                ui.label(caption(shelf::summary(&app.store.shelf_items)));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if app.shelf_floating { "Hide floating" } else { "Float" };
                if ui::soft_button(ui, format!("{} {label}", ph::PICTURE_IN_PICTURE)).on_hover_text("A small Shelf window that stays on top of other apps").clicked() && !app.demo {
                    app.shelf_floating = !app.shelf_floating;
                }
            });
        });
        ui.horizontal(|ui| {
            if ui::soft_button(ui, format!("{} Add files…", ph::PLUS)).clicked() && !app.demo {
                do_add_files(app);
            }
            if ui::soft_button(ui, format!("{} Paste", ph::CLIPBOARD)).on_hover_text("Adds files you copied in a file manager, or copied text and links").clicked() && !app.demo {
                do_paste(app);
            }
        });
        ui.horizontal(|ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut app.shelf_note).hint_text("Type a note or link and press Enter").desired_width(f32::INFINITY));
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.shelf_note.trim().is_empty() {
                let t = std::mem::take(&mut app.shelf_note);
                shelf::add_text(&mut app.store.shelf_items, &t);
                app.mark_dirty();
            }
        });
    });
    card(ui, |ui| {
        tiles(app, ui, ctx, 96.0);
        if !app.store.shelf_items.is_empty() {
            ui::separator(ui);
            action_bar(app, ui);
            ui.label(caption("Click to select, Ctrl+click for more, double-click to open, right-click for more actions."));
        }
    });
    card(ui, |ui| {
        let mut v = app.store.shelf_remove_after_use;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() - 60.0);
                ui.label("Take items off after copying or moving them");
                ui.add(egui::Label::new(caption("Pinned items always stay.")).wrap());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui::toggle(ui, &mut v);
            });
        });
        if v != app.store.shelf_remove_after_use {
            app.store.shelf_remove_after_use = v;
            app.mark_dirty();
        }
        if cfg!(target_os = "linux") {
            ui::separator(ui);
            let mut x = app.store.linux_x11;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 60.0);
                    ui.label("Accept dragged files on Wayland");
                    ui.add(egui::Label::new(caption("Runs Valler through XWayland so files can be dropped on it. Takes effect after restarting Valler.")).wrap());
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui::toggle(ui, &mut x);
                });
            });
            if x != app.store.linux_x11 {
                app.store.linux_x11 = x;
                app.mark_dirty();
            }
        }
    });
}

// ------------------------------------------------------------- floating shelf

pub fn floating(app: &mut ValleApp, ctx: &egui::Context) {
    let vid = egui::ViewportId::from_hash_of("valler-shelf");
    let builder = egui::ViewportBuilder::default()
        .with_title("Valler Shelf")
        .with_app_id("valler")
        .with_inner_size([372.0, 330.0])
        .with_min_inner_size([240.0, 200.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top();
    ctx.show_viewport_immediate(vid, builder, |ui, _| {
        let c = ui.ctx().clone();
        if c.input(|i| i.viewport().close_requested()) {
            app.shelf_floating = false;
        }
        floating_contents(app, ui, &c);
    });
}

/// Body of the floating Shelf (also drawn inline in screenshot mode).
pub fn floating_contents(app: &mut ValleApp, ui: &mut Ui, c: &egui::Context) {
    let c = c.clone();
    {
        if app.demo && app.store.shelf_items.is_empty() {
            app.store.shelf_items = shelf::demo();
        }
        ui::window_surface(ui);
        handle_paste_key(app, &c);
        if c.input(|i| !i.raw.hovered_files.is_empty()) {
            drop_overlay(&c, "Drop to keep it here");
        }
        take_drops(app, &c);
        let mut close = false;
        egui::Panel::top("shelf-head")
            .show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(egui::Margin { left: 14, right: 14, top: 10, bottom: 4 }))
            .show(ui, |ui| {
                let head = ui.max_rect().expand2(vec2(14.0, 10.0));
                ui::drag_area(ui, egui::Rect::from_min_max(head.min, pos2(head.max.x, head.min.y + 40.0)), "shelf-drag", false);
                ui.horizontal(|ui| {
                    ui.label(ui::icon(ph::TRAY, 17.0, ui::ACCENT));
                    ui.label(RichText::new("Shelf").font(ui::bold(14.0)));
                    ui.label(caption(shelf::summary(&app.store.shelf_items)));
                });
                let x = pos2(head.right() - 20.0, head.top() + 18.0);
                if ui::chrome_button(ui, x, ph::X, "shelf-close", true).on_hover_text("Hide the floating Shelf").clicked() {
                    close = true;
                }
                if ui::chrome_button(ui, x - vec2(30.0, 0.0), ph::PLUS, "shelf-add", false).on_hover_text("Add files…").clicked() {
                    do_add_files(app);
                }
                if ui::chrome_button(ui, x - vec2(60.0, 0.0), ph::CLIPBOARD, "shelf-paste", false).on_hover_text("Paste copied files or text").clicked() {
                    do_paste(app);
                }
            });
        egui::Panel::bottom("shelf-foot")
            .show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(egui::Margin { left: 14, right: 14, top: 4, bottom: 12 }))
            .show(ui, |ui| action_bar(app, ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(14, 4)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| tiles(app, ui, &c, 80.0));
            });
        ui::resize_grip(ui, "shelf-grip");
        if close {
            app.shelf_floating = false;
        }
    }
}
