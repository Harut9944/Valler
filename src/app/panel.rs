//! The main panel: header with the mark, tab bar, section title, content and footer.

use crate::core::features::Tab;
use crate::ui;
use egui::{CornerRadius, Margin, RichText, Sense, Stroke, Ui, vec2};
use egui_phosphor::regular as ph;
use std::time::Duration;
use super::*;

impl ValleApp {
    pub(super) fn tab_bar(&mut self, ui: &mut Ui) {
        let tabs = self.visible_tabs();
        if tabs.is_empty() {
            return;
        }
        let w = ui.available_width();
        let h = 48.0;
        let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        ui.painter().rect(
            rect,
            CornerRadius::same(14),
            ui::CARD,
            Stroke::new(1.0, ui::BORDER),
            egui::StrokeKind::Inside,
        );
        let inner = rect.shrink(5.0);
        let slot = inner.width() / tabs.len() as f32;
        for (i, t) in tabs.iter().enumerate() {
            let r = egui::Rect::from_min_size(inner.min + vec2(slot * i as f32, 0.0), vec2(slot, inner.height()));
            let id = ui.id().with(("tab", t.id()));
            let resp = ui.interact(r, id, Sense::click()).on_hover_text(t.title());
            let selected = self.tab == *t && self.sub.is_none() || self.tab == *t;
            let p = ui.painter();
            if selected {
                p.rect_filled(r.shrink2(vec2(3.0, 0.0)), CornerRadius::same(10), ui::ACCENT_SOFT);
            } else if resp.hovered() {
                p.rect_filled(r.shrink2(vec2(3.0, 0.0)), CornerRadius::same(10), ui::CARD_HOVER);
            }
            let color = if selected { ui::ACCENT } else { ui::MUTED };
            p.text(r.center(), egui::Align2::CENTER_CENTER, t.icon(), egui::FontId::proportional(21.0), color);
            if resp.clicked() {
                self.tab = *t;
                self.sub = None;
            }
        }
    }

    pub(super) fn panel_title(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if let Some(sub) = self.sub {
                if ui
                    .add(egui::Button::new(RichText::new(format!("{}  Back", ph::CARET_LEFT)).color(ui::ACCENT)).frame(false))
                    .clicked()
                {
                    self.sub = None;
                }
                ui.label(RichText::new(sub_title(sub).to_uppercase()).font(ui::bold(13.0)).color(ui::MUTED).extra_letter_spacing(1.2));
            } else {
                ui::section_title(ui, self.tab.title());
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let r = ui
                    .add(egui::Button::new(RichText::new(ph::SLIDERS).size(17.0).color(ui::MUTED)).frame(false))
                    .on_hover_text("Settings for this section");
                if r.clicked() {
                    self.settings_open = true;
                    self.settings_page = match (self.tab, self.sub) {
                        (_, Some(Sub::Clipboard)) => SettingsPage::Clipboard,
                        (_, Some(Sub::Snippets)) => SettingsPage::Snippets,
                        (_, Some(Sub::UrlClean)) => SettingsPage::UrlClean,
                        (Tab::KeepAwake, _) => SettingsPage::KeepAwake,
                        (Tab::System | Tab::Network | Tab::Disk | Tab::Power, _) => SettingsPage::Monitor,
                        _ => SettingsPage::Features,
                    };
                }
            });
        });
    }

    pub(super) fn footer(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 10.0) / 2.0;
            if ui::footer_button(ui, ph::GEAR, "Settings", w).clicked() {
                self.settings_open = true;
            }
            ui.add_space(2.0);
            if ui::footer_button(ui, ph::POWER, "Quit", w).clicked() {
                self.quit(ctx);
            }
        });
        ui.add_space(4.0);
    }

    pub(crate) fn panel(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let margin = Margin { left: 16, right: 16, top: 10, bottom: 8 };
        let mut close = false;
        egui::Panel::top("header").show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(margin))
            .show(ui, |ui| {
                // The whole header moves the window (there is no system title bar).
                let head = ui.max_rect().expand2(vec2(16.0, 10.0));
                let drag_rect = egui::Rect::from_min_max(head.min, egui::pos2(head.max.x, head.min.y + 104.0));
                ui::drag_area(ui, drag_rect, "main-drag", false);
                close = ui::window_buttons(ui, head, "main");
                ui::header(ui);
                ui.add_space(8.0);
                self.tab_bar(ui);
                ui.add_space(8.0);
                self.panel_title(ui);
            });
        if close {
            self.close_window(ctx);
        }
        egui::Panel::bottom("footer").show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(Margin::symmetric(16, 6)))
            .show(ui, |ui| {
                if let Some((msg, at)) = &self.toast
                    && at.elapsed() < Duration::from_millis(2500) {
                        ui.vertical_centered(|ui| {
                            egui::Frame::new()
                                .fill(ui::ACCENT_SOFT)
                                .corner_radius(CornerRadius::same(10))
                                .inner_margin(Margin::symmetric(12, 4))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(format!("{}  {msg}", ph::CHECK_CIRCLE)).color(ui::TEXT));
                                });
                        });
                        ctx.request_repaint_after(Duration::from_millis(300));
                    }
                self.footer(ui, ctx);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(Margin { left: 16, right: 16, top: 2, bottom: 4 }))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    if self.visible_tabs().is_empty() {
                        ui::card(ui, |ui| {
                            ui::empty_note(ui, ph::PACKAGE, "No features are installed. Open Settings → Features to install some.");
                        });
                        return;
                    }
                    match self.sub {
                        Some(sub) => crate::screens::tools::sub_view(self, ui, ctx, sub),
                        None => match self.tab {
                            Tab::KeepAwake => crate::screens::tabs::keep_awake(self, ui),
                            Tab::Displays => crate::screens::tabs::displays(self, ui, ctx),
                            Tab::Mixer => crate::screens::tabs::mixer(self, ui),
                            Tab::System => crate::screens::tabs::system(self, ui),
                            Tab::Network => crate::screens::tabs::network(self, ui, ctx),
                            Tab::Disk => crate::screens::tabs::disk(self, ui),
                            Tab::Power => crate::screens::tabs::power(self, ui),
                            Tab::Utilities => crate::screens::tools::utilities(self, ui),
                            Tab::Controls => crate::screens::tabs::controls(self, ui),
                        },
                    }
                    ui.add_space(6.0);
                });
            });
    }
}
