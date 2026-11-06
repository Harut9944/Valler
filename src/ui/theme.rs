//! The dark palette, text styles and egui style Valler uses everywhere.

use super::fonts::BOLD;
use egui::{Color32, CornerRadius, FontFamily, FontId, RichText, Stroke, vec2};

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

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}
