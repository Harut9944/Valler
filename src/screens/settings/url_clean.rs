//! Clean URL page.

use crate::app::ValleApp;
use crate::ui::{self, caption, card, title};
use egui::{RichText, Ui};
use super::*;

pub(crate) fn url_clean(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Clean URL", "Removes tracking parameters such as utm_source, fbclid and gclid.");
    card(ui, |ui| {
        let mut on = app.store.is_on("url_clean");
        setting_row(ui, "Clean copied links automatically", "Each link you copy is replaced by its clean version.", |ui| {
            ui::toggle(ui, &mut on);
        });
        if on != app.store.is_on("url_clean") {
            app.store.set_on("url_clean", on);
            app.store.dirty_flag = true;
        }
        ui::separator(ui);
        ui.label(RichText::new("Extra parameters to remove").size(14.5));
        ui.label(caption("Comma separated, applied on every site."));
        if ui.add(egui::TextEdit::singleline(&mut app.store.url_extra_params).hint_text("ref, source, campaign").desired_width(f32::INFINITY)).changed() {
            app.store.dirty_flag = true;
        }
    });
    card(ui, |ui| {
        ui.label(title("Built-in rules"));
        ui.label(caption("Everywhere: every utm_ parameter, fbclid, gclid, dclid, msclkid, yclid, igshid, mc_cid, mc_eid, twclid, ttclid and more."));
        ui.label(caption("Site-specific: YouTube (si, pp, feature), X and Twitter (s, t, src), Instagram (igsh), Spotify (si), Reddit, TikTok and Amazon share parameters."));
    });
    app.mark_dirty_if_changed();
}
