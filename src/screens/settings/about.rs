//! About page.

use crate::app::ValleApp;
use crate::ui::{self, caption};
use egui::{RichText, Sense, Ui, vec2};

pub(crate) fn about(app: &mut ValleApp, ui: &mut Ui) {
    let _ = app;
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(100.0, 80.0), Sense::hover());
        ui::paint_logo(ui.painter(), r.center(), 80.0);
        ui.label(RichText::new("Valler").font(ui::bold(28.0)));
        ui.label(caption(format!("Version {}", env!("CARGO_PKG_VERSION"))));
        ui.add_space(8.0);
        ui.label("Everyday desktop utilities in one window, for Windows, Linux and macOS.");
        ui.label(caption("No account, no analytics. The network is only used for the speed test you start yourself."));
        ui.add_space(8.0);
        ui.label(caption("Free software under the GNU GPL 3.0 or later. A cross-platform rewrite in Rust of an open-source macOS utility collection."));
    });
}
