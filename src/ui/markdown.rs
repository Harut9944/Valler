//! Draws Markdown blocks from `services::markdown` (the Scratchpad preview).

use super::{ACCENT, BORDER, CONTROL, MUTED, TEXT, bold};
use crate::services::markdown::{Block, Span};
use egui::{CornerRadius, Frame, Margin, RichText, Stroke, Ui, vec2};
use egui_phosphor::regular as ph;

fn spans(ui: &mut Ui, spans: &[Span], size: f32, strong: bool, color: egui::Color32) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for s in spans {
            let base = |t: &str| {
                let r = RichText::new(t).size(size).color(color);
                if strong { r.font(bold(size)) } else { r }
            };
            match s {
                Span::Text(t) => {
                    ui.label(base(t));
                }
                Span::Bold(t) => {
                    ui.label(RichText::new(t).font(bold(size)).color(color));
                }
                Span::Italic(t) => {
                    ui.label(base(t).italics());
                }
                Span::Code(t) => {
                    ui.label(RichText::new(t).monospace().size(size - 1.0).background_color(CONTROL).color(TEXT));
                }
                Span::Link { text, url } => {
                    ui.hyperlink_to(RichText::new(text).size(size).color(ACCENT), url);
                }
            }
        }
    });
}

pub fn markdown(ui: &mut Ui, blocks: &[Block]) {
    ui.spacing_mut().item_spacing.y = 6.0;
    for b in blocks {
        match b {
            Block::Heading(level, s) => {
                let size = match level {
                    1 => 22.0,
                    2 => 18.0,
                    3 => 16.0,
                    _ => 14.5,
                };
                ui.add_space(4.0);
                spans(ui, s, size, true, TEXT);
            }
            Block::Paragraph(s) => spans(ui, s, 14.0, false, TEXT),
            Block::Bullet { indent, spans: s } => {
                ui.horizontal(|ui| {
                    ui.add_space(6.0 + *indent as f32 * 16.0);
                    ui.label(RichText::new("•").color(MUTED));
                    spans(ui, s, 14.0, false, TEXT);
                });
            }
            Block::Numbered { number, spans: s } => {
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.label(RichText::new(format!("{number}.")).color(MUTED));
                    spans(ui, s, 14.0, false, TEXT);
                });
            }
            Block::Task { done, spans: s } => {
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let (glyph, color) = if *done { (ph::CHECK_SQUARE, ACCENT) } else { (ph::SQUARE, MUTED) };
                    ui.label(RichText::new(glyph).size(16.0).color(color));
                    spans(ui, s, 14.0, false, if *done { MUTED } else { TEXT });
                });
            }
            Block::Quote(s) => {
                Frame::new().inner_margin(Margin { left: 12, right: 4, top: 2, bottom: 2 }).show(ui, |ui| {
                    let top = ui.cursor().min;
                    spans(ui, s, 14.0, false, MUTED);
                    let bottom = ui.min_rect().bottom();
                    ui.painter().line_segment([top - vec2(9.0, 0.0), egui::pos2(top.x - 9.0, bottom)], Stroke::new(3.0, BORDER));
                });
            }
            Block::Code(code) => {
                Frame::new().fill(CONTROL).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(10)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(code).monospace().color(TEXT));
                });
            }
            Block::Rule => {
                ui.separator();
            }
        }
    }
}
