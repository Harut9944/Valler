//! Shared look and feel: dark panel palette, cards, switches, usage bars,
//! sparklines, badges and the Valler mark.

use crate::fonts::BOLD;
use egui::{
    Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Pos2, Rect, Response, RichText,
    Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};

pub const BG: Color32 = Color32::from_rgb(0x1b, 0x1b, 0x1d);
pub const CARD: Color32 = Color32::from_rgb(0x27, 0x27, 0x2a);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x2e, 0x2e, 0x32);
pub const CONTROL: Color32 = Color32::from_rgb(0x32, 0x32, 0x36);
pub const BORDER: Color32 = Color32::from_rgb(0x3a, 0x3a, 0x3e);
pub const TEXT: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf4);
pub const MUTED: Color32 = Color32::from_rgb(0xa0, 0xa0, 0xa8);
pub const FAINT: Color32 = Color32::from_rgb(0x6c, 0x6c, 0x74);
pub const ACCENT: Color32 = Color32::from_rgb(0x2f, 0x8b, 0xff);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0x1d, 0x34, 0x55);
pub const GREEN: Color32 = Color32::from_rgb(0x34, 0xd0, 0x5a);
pub const CYAN: Color32 = Color32::from_rgb(0x3c, 0xd6, 0xe6);
pub const ORANGE: Color32 = Color32::from_rgb(0xff, 0x9f, 0x2e);
pub const YELLOW: Color32 = Color32::from_rgb(0xff, 0xd6, 0x3a);
pub const RED: Color32 = Color32::from_rgb(0xff, 0x45, 0x3a);
pub const PURPLE: Color32 = Color32::from_rgb(0xa7, 0x7b, 0xff);

pub fn install_style(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        *v = egui::Visuals::dark();
        v.panel_fill = BG;
        v.window_fill = BG;
        v.extreme_bg_color = Color32::from_rgb(0x14, 0x14, 0x16);
        v.faint_bg_color = CARD;
        v.override_text_color = Some(TEXT);
        v.selection.bg_fill = ACCENT;
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        v.hyperlink_color = ACCENT;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        v.widgets.inactive.bg_fill = CONTROL;
        v.widgets.inactive.weak_bg_fill = CONTROL;
        v.widgets.inactive.corner_radius = CornerRadius::same(8);
        v.widgets.hovered.bg_fill = Color32::from_rgb(0x3c, 0x3c, 0x42);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x3c, 0x3c, 0x42);
        v.widgets.hovered.corner_radius = CornerRadius::same(8);
        v.widgets.active.corner_radius = CornerRadius::same(8);
        v.widgets.active.bg_fill = ACCENT_SOFT;
        v.widgets.active.weak_bg_fill = ACCENT_SOFT;
        v.widgets.open.corner_radius = CornerRadius::same(8);
        v.window_corner_radius = CornerRadius::same(12);
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(12.0, 6.0);
        style.spacing.interact_size.y = 26.0;
        style.spacing.slider_rail_height = 6.0;
        use egui::TextStyle::*;
        style.text_styles.insert(Body, FontId::proportional(14.0));
        style.text_styles.insert(Button, FontId::proportional(14.0));
        style.text_styles.insert(Small, FontId::proportional(11.5));
        style.text_styles.insert(Heading, FontId::new(20.0, FontFamily::Name(BOLD.into())));
        style.text_styles.insert(Monospace, FontId::monospace(13.0));
    });
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD.into()))
}

pub fn title(text: impl Into<String>) -> RichText {
    RichText::new(text).font(bold(15.0)).color(TEXT)
}

pub fn caption(text: impl Into<String>) -> RichText {
    RichText::new(text).size(12.5).color(MUTED)
}

pub fn icon(glyph: &str, size: f32, color: Color32) -> RichText {
    RichText::new(glyph).size(size).color(color)
}

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

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
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

/// The Valler mark: a "V" valley between two hills under a rising sun.
pub fn paint_logo(p: &egui::Painter, center: Pos2, size: f32) {
    let s = size / 2.0;
    // sun
    p.circle_filled(center + vec2(0.0, -s * 0.18), s * 0.42, Color32::from_rgb(0xff, 0xb3, 0x47));
    // hills forming a V
    let left = vec![
        center + vec2(-s, s * 0.55),
        center + vec2(-s * 0.55, -s * 0.45),
        center + vec2(0.0, s * 0.55),
    ];
    let right = vec![
        center + vec2(0.0, s * 0.55),
        center + vec2(s * 0.55, -s * 0.45),
        center + vec2(s, s * 0.55),
    ];
    p.add(Shape::convex_polygon(left, ACCENT, Stroke::NONE));
    p.add(Shape::convex_polygon(right, Color32::from_rgb(0x1f, 0x6b, 0xd6), Stroke::NONE));
    p.line_segment(
        [center + vec2(-s * 1.05, s * 0.58), center + vec2(s * 1.05, s * 0.58)],
        Stroke::new(2.0, TEXT),
    );
}

/// Header with the mark and the app name.
pub fn header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(56.0, 40.0), Sense::hover());
        paint_logo(ui.painter(), rect.center(), 40.0);
        ui.label(RichText::new("Valler").font(bold(15.0)).color(TEXT));
    });
}

/// Draws the window icon (same mark) into an RGBA buffer.
pub fn app_icon() -> egui::IconData {
    let n = 64u32;
    egui::IconData { rgba: icon_rgba(n, None), width: n, height: n }
}

/// Renders the Valler mark at `n`×`n` pixels with 2× supersampling, so it
/// looks clean at any size (window, taskbar, tray, launcher).
/// `gauge` adds a live usage bar on the right edge (tray readouts).
pub fn icon_rgba(n: u32, gauge: Option<(f32, Color32)>) -> Vec<u8> {
    let n = n as usize;
    let mut rgba = vec![0u8; n * n * 4];
    let ss = 3usize;
    for y in 0..n {
        for x in 0..n {
            let mut acc = [0f32; 4];
            for sy in 0..ss {
                for sx in 0..ss {
                    // Map the pixel into the 64-unit design space centred on 0.
                    let fx = ((x as f32 + (sx as f32 + 0.5) / ss as f32) / n as f32) * 64.0 - 32.0;
                    let fy = ((y as f32 + (sy as f32 + 0.5) / ss as f32) / n as f32) * 64.0 - 32.0;
                    let c = icon_sample(fx, fy, gauge);
                    let a = c[3] as f32 / 255.0;
                    acc[0] += c[0] as f32 * a;
                    acc[1] += c[1] as f32 * a;
                    acc[2] += c[2] as f32 * a;
                    acc[3] += a;
                }
            }
            let i = (y * n + x) * 4;
            let cov = acc[3] / (ss * ss) as f32;
            if acc[3] > 0.0 {
                rgba[i] = (acc[0] / acc[3]) as u8;
                rgba[i + 1] = (acc[1] / acc[3]) as u8;
                rgba[i + 2] = (acc[2] / acc[3]) as u8;
            }
            rgba[i + 3] = (cov * 255.0).round() as u8;
        }
    }
    rgba
}

fn icon_sample(fx: f32, fy: f32, gauge: Option<(f32, Color32)>) -> [u8; 4] {
    // Rounded square background.
    let r = 12.0;
    let half = 30.0;
    let dx = (fx.abs() - (half - r)).max(0.0);
    let dy = (fy.abs() - (half - r)).max(0.0);
    if dx.hypot(dy) > r {
        return [0, 0, 0, 0];
    }
    if let Some((frac, col)) = gauge {
        // Bar on the right: track and fill from the bottom.
        if fx > 17.0 && fx < 26.0 && fy > -24.0 && fy < 24.0 {
            let top = 24.0 - 48.0 * frac.clamp(0.0, 1.0);
            return if fy >= top { [col.r(), col.g(), col.b(), 255] } else { [0x3a, 0x3a, 0x40, 255] };
        }
    }
    // Shift the mark left a little when the gauge is shown.
    let fx = if gauge.is_some() { fx + 6.0 } else { fx };
    let mut c = [0x1b, 0x1b, 0x1d, 255];
    if fx.hypot(fy + 6.0) < 13.0 {
        c = [0xff, 0xb3, 0x47, 255];
    }
    let base = 16.0;
    let hill = |px: f32| -> f32 { -12.0 + (fx - px).abs() * (28.0 / 16.0) };
    if fy < base && fy > hill(-14.0) && fx < 0.0 {
        c = [0x2f, 0x8b, 0xff, 255];
    }
    if fy < base && fy > hill(14.0) && fx >= 0.0 {
        c = [0x1f, 0x6b, 0xd6, 255];
    }
    c
}

pub fn fmt_bytes(b: u64) -> String {
    let b = b as f64;
    const K: f64 = 1024.0;
    if b < K {
        format!("{b:.0} B")
    } else if b < K * K {
        format!("{:.1} KB", b / K)
    } else if b < K * K * K {
        format!("{:.1} MB", b / K / K)
    } else if b < K * K * K * K {
        format!("{:.2} GB", b / K / K / K)
    } else {
        format!("{:.2} TB", b / K / K / K / K)
    }
}

pub fn fmt_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", fmt_bytes(bytes_per_sec.max(0.0) as u64))
}

pub fn fmt_duration(secs: u64) -> String {
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        format!("{d}d {h}h {m}min")
    } else if h > 0 {
        format!("{h}h {m}min")
    } else {
        format!("{m}min")
    }
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

#[cfg(test)]
mod icon_tests {
    /// Writes preview images of the icon into shots/ (run with --ignored).
    #[test]
    #[ignore]
    fn preview_icons() {
        let _ = std::fs::create_dir_all("shots");
        for (name, n, g) in [("icon-256", 256u32, None), ("icon-tray", 32, Some((0.42f32, super::GREEN)))] {
            let px = super::icon_rgba(n, g);
            image::save_buffer(format!("shots/{name}.png"), &px, n, n, image::ExtendedColorType::Rgba8).unwrap();
        }
    }
}
