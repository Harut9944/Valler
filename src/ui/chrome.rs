//! Frameless window chrome drawn by Valler itself on every platform: the
//! rounded surface, drag area, minimize/close buttons and resize grip.

use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use super::*;

// ------------------------------------------------------------- frameless window chrome

pub const WINDOW_RADIUS: u8 = 14;

/// Paints the rounded app surface that replaces the system window frame.
pub fn window_surface(ui: &Ui) {
    let r = ui.max_rect();
    ui.painter().rect(
        r,
        CornerRadius::same(WINDOW_RADIUS),
        BG,
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );
}

/// Makes `rect` a handle that moves the window; double click toggles maximize when allowed.
/// Call it before adding the widgets that sit on top of the handle.
pub fn drag_area(ui: &mut Ui, rect: Rect, id: &str, maximizable: bool) {
    let resp = ui.interact(rect, egui::Id::new(id), Sense::click_and_drag());
    if resp.drag_started_by(egui::PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if maximizable && resp.double_clicked() {
        let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
    }
}

/// Small round window button (minimize / close) painted at `center`.
pub fn chrome_button(ui: &mut Ui, center: Pos2, glyph: &str, id: &str, danger: bool) -> Response {
    let rect = Rect::from_center_size(center, vec2(24.0, 24.0));
    let resp = ui.interact(rect, egui::Id::new(id), Sense::click());
    let fill = if resp.hovered() {
        if danger { RED } else { Color32::from_rgb(0x44, 0x44, 0x4a) }
    } else {
        Color32::from_rgb(0x2c, 0x2c, 0x30)
    };
    ui.painter().circle_filled(center, 11.0, fill);
    let col = if resp.hovered() { Color32::WHITE } else { MUTED };
    ui.painter().text(center, egui::Align2::CENTER_CENTER, glyph, FontId::proportional(13.0), col);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Draws minimize and close buttons in the top-right corner of `area`.
/// Returns true when close was pressed.
pub fn window_buttons(ui: &mut Ui, area: Rect, id: &str) -> bool {
    let y = area.top() + 18.0;
    let close = pos2(area.right() - 20.0, y);
    let min = pos2(area.right() - 50.0, y);
    if chrome_button(ui, min, egui_phosphor::regular::MINUS, &format!("{id}-min"), false)
        .on_hover_text("Minimize")
        .clicked()
    {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
    chrome_button(ui, close, egui_phosphor::regular::X, &format!("{id}-close"), true)
        .on_hover_text("Close")
        .clicked()
}

/// A grip in the bottom-right corner that resizes the frameless window.
pub fn resize_grip(ui: &mut Ui, id: &str) {
    let r = ui.max_rect();
    let rect = Rect::from_min_max(r.right_bottom() - vec2(18.0, 18.0), r.right_bottom());
    let resp = ui
        .interact(rect, egui::Id::new(id), Sense::drag())
        .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
    if resp.drag_started_by(egui::PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::SouthEast));
    }
    let c = if resp.hovered() { MUTED } else { FAINT };
    for i in 0..2 {
        let o = 5.0 + i as f32 * 4.0;
        ui.painter().line_segment(
            [r.right_bottom() - vec2(o, 4.0), r.right_bottom() - vec2(4.0, o)],
            Stroke::new(1.2, c),
        );
    }
}
