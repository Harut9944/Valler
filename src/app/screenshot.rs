//! `--screenshot <file.png>`: draw about 120 frames, save the window as PNG, exit.

use super::*;

impl ValleApp {
    pub(super) fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.opts.screenshot.clone() else { return };
        ctx.request_repaint();
        self.frames += 1;
        if self.frames >= 120 && !self.shot_sent {
            self.shot_sent = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let [w, h] = img.size;
            let mut buf = Vec::with_capacity(w * h * 4);
            for p in &img.pixels {
                buf.extend_from_slice(&p.to_srgba_unmultiplied());
            }
            match image::save_buffer(&path, &buf, w as u32, h as u32, image::ExtendedColorType::Rgba8) {
                Ok(_) => println!("Saved screenshot to {}", path.display()),
                Err(e) => eprintln!("Could not save screenshot: {e}"),
            }
            std::process::exit(0);
        }
        if self.frames > 600 {
            eprintln!("Screenshot was not delivered; exiting");
            std::process::exit(1);
        }
    }

    // ----------------------------------------------------------------- panel
}
