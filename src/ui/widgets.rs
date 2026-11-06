//! Reusable widgets: cards, switches, usage bars, sparklines, badges and buttons.

use egui::{Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2};
use super::*;

/// A rounded dark card.
pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// A card that reacts to hover and click as a whole.
pub fn clickable_card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) -> Response {
    let id = ui.next_auto_id();
    let hovered = ui.ctx().read_response(id).is_some_and(|r| r.hovered());
    let inner = Frame::new()
        .fill(if hovered { CARD_HOVER } else { CARD })
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        });
    ui.interact(inner.response.rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Upper-case grey section title, like "SYSTEM".
pub fn section_title(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(text.to_uppercase())
            .font(bold(13.0))
            .color(MUTED)
            .extra_letter_spacing(1.2),
    );
    ui.add_space(2.0);
}

pub fn separator(ui: &mut Ui) {
    ui.add_space(4.0);
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, BORDER);
    ui.add_space(4.0);
}

/// iOS-style switch.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    toggle_enabled(ui, on, true)
}

pub fn toggle_enabled(ui: &mut Ui, on: &mut bool, enabled: bool) -> Response {
    let size = vec2(46.0, 26.0);
    let (rect, mut resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    if enabled && resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool_responsive(resp.id, *on);
    let off_col = Color32::from_rgb(0x48, 0x48, 0x4e);
    let mut bg = lerp_color(off_col, ACCENT, t);
    let mut knob = Color32::from_rgb(0xe8, 0xe8, 0xee);
    if !enabled {
        bg = bg.gamma_multiply(0.45);
        knob = knob.gamma_multiply(0.5);
    }
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(13), bg);
    let kx = egui::lerp((rect.left() + 13.0)..=(rect.right() - 13.0), t);
    let krect = Rect::from_center_size(pos2(kx, rect.center().y), vec2(if *on { 26.0 } else { 20.0 }, 20.0));
    p.rect_filled(krect, CornerRadius::same(10), knob);
    if enabled {
        resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Horizontal usage bar, `frac` in 0..=1.
pub fn bar(ui: &mut Ui, width: f32, frac: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width.max(20.0), 8.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(0x3a, 0x3a, 0x40));
    let f = frac.clamp(0.0, 1.0);
    if f > 0.0 {
        let mut r = rect;
        r.set_width((rect.width() * f).max(8.0));
        p.rect_filled(r, CornerRadius::same(4), color);
    }
}

/// Line graph with a faint fill underneath. Values are 0..=max.
pub fn sparkline(ui: &mut Ui, values: &[f32], max: f32, color: Color32, height: f32) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, height), Sense::hover());
    let p = ui.painter_at(rect);
    p.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, BORDER),
    );
    if values.len() < 2 {
        return;
    }
    let max = if max <= 0.0 { 1.0 } else { max };
    let n = values.len();
    let pts: Vec<Pos2> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = rect.left() + rect.width() * i as f32 / (n - 1) as f32;
            let y = rect.bottom() - 2.0 - (rect.height() - 4.0) * (v / max).clamp(0.0, 1.0);
            pos2(x, y)
        })
        .collect();
    // Faint fill: vertical strips from the line to the bottom.
    let fill = color.gamma_multiply(0.14);
    let mut mesh = egui::Mesh::default();
    for (i, pt) in pts.iter().enumerate() {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(*pt, fill);
        mesh.colored_vertex(pos2(pt.x, rect.bottom()), Color32::TRANSPARENT);
        if i > 0 {
            mesh.add_triangle(base - 2, base - 1, base);
            mesh.add_triangle(base - 1, base, base + 1);
        }
    }
    p.add(Shape::mesh(mesh));
    p.add(Shape::line(pts, Stroke::new(1.6, color)));
}

/// Small rounded label like "Coming later" or "Works here".
pub fn badge(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) {
    Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(7))
        .inner_margin(Margin::symmetric(7, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(fg));
        });
}

/// Pill with a colored dot, like the memory "Normal" pressure chip.
pub fn status_pill(ui: &mut Ui, text: &str, color: Color32) {
    Frame::new()
        .fill(color.gamma_multiply(0.18))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let (r, _) = ui.allocate_exact_size(vec2(9.0, 9.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 4.5, color);
                ui.label(RichText::new(text).font(bold(13.5)).color(color));
            });
        });
}

/// Keyboard-shortcut style chip.
pub fn key_chip(ui: &mut Ui, text: &str) {
    // Painted at a fixed size so it never stretches with the row around it.
    let galley = ui.painter().layout_no_wrap(text.to_string(), bold(11.5), MUTED);
    let size = galley.size() + vec2(14.0, 6.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(6), CONTROL);
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, MUTED);
}

/// Wide footer button with an icon (Settings / Quit).
pub fn footer_button(ui: &mut Ui, glyph: &str, text: &str, width: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 38.0), Sense::click());
    let fill = if resp.hovered() { CARD_HOVER } else { CARD };
    let p = ui.painter();
    p.rect(rect, CornerRadius::same(10), fill, Stroke::new(1.0, BORDER), StrokeKind::Inside);
    let galley_icon = p.layout_no_wrap(glyph.to_string(), FontId::proportional(17.0), MUTED);
    let galley_text = p.layout_no_wrap(text.to_string(), bold(14.5), TEXT);
    let total = galley_icon.size().x + 10.0 + galley_text.size().x;
    let x0 = rect.center().x - total / 2.0;
    p.galley(
        pos2(x0, rect.center().y - galley_icon.size().y / 2.0),
        galley_icon.clone(),
        MUTED,
    );
    p.galley(
        pos2(x0 + galley_icon.size().x + 10.0, rect.center().y - galley_text.size().y / 2.0),
        galley_text,
        TEXT,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A secondary, outlined button.
pub fn soft_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(RichText::new(text.into()).size(13.5))
            .fill(CONTROL)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(8)),
    )
}

/// A blue, filled button.
pub fn primary_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(RichText::new(text.into()).size(13.5).color(Color32::WHITE))
            .fill(ACCENT)
            .corner_radius(CornerRadius::same(8)),
    )
}

/// Header with the mark and the app name.
pub fn header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(56.0, 40.0), Sense::hover());
        paint_logo(ui.painter(), rect.center(), 40.0);
        ui.label(RichText::new("Valler").font(bold(15.0)).color(TEXT));
    });
}

/// A row: big icon on the left, title + description, and something on the right.
pub fn feature_row(
    ui: &mut Ui,
    glyph: &str,
    title_text: &str,
    desc: &str,
    right: impl FnOnce(&mut Ui),
) {
    let row = ui.horizontal(|ui| {
        // Reserve the icon slot; the icon is painted centred once the row height is known.
        let (slot, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
        ui.add_space(4.0);
        let right_w = 70.0;
        let text_w = (ui.available_width() - right_w).max(120.0);
        ui.allocate_ui_with_layout(
            vec2(text_w, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(text_w);
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(title(title_text));
                ui.add(egui::Label::new(caption(desc)).wrap());
            },
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
        slot
    });
    let slot = row.inner;
    let center = pos2(slot.center().x, row.response.rect.center().y);
    ui.painter().text(center, egui::Align2::CENTER_CENTER, glyph, FontId::proportional(22.0), ACCENT);
}

pub fn empty_note(ui: &mut Ui, glyph: &str, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.label(icon(glyph, 28.0, FAINT));
        ui.add(egui::Label::new(caption(text)).wrap());
        ui.add_space(10.0);
    });
}
