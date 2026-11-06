//! Tests for the visual building blocks.

#[cfg(test)]
mod icon_tests {
    /// Writes preview images of the icon into shots/ (run with --ignored).
    #[test]
    #[ignore]
    fn preview_icons() {
        let _ = std::fs::create_dir_all("shots");
        for (name, n, g) in [("icon-256", 256u32, None), ("icon-tray", 32, Some((0.42f32, crate::ui::GREEN)))] {
            let px = crate::ui::icon_rgba(n, g);
            image::save_buffer(format!("shots/{name}.png"), &px, n, n, image::ExtendedColorType::Rgba8).unwrap();
        }
    }
}

#[cfg(test)]
mod feature_tests {
    use crate::ui::*;
    use egui::Color32;

    #[test]
    fn human_readable_sizes_speeds_and_durations() {
        assert_eq!(fmt_bytes(512), "512 B");
        assert_eq!(fmt_bytes(1536), "1.5 KB");
        assert_eq!(fmt_bytes(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(fmt_bytes(3 * 1024 * 1024 * 1024), "3.00 GB");
        assert_eq!(fmt_rate(2048.0), "2.0 KB/s");
        assert_eq!(fmt_rate(-5.0), "0 B/s");
        assert_eq!(fmt_duration(59), "0min");
        assert_eq!(fmt_duration(3 * 3600 + 5 * 60), "3h 5min");
        assert_eq!(fmt_duration(2 * 86_400 + 3600), "2d 1h 0min");
    }

    #[test]
    fn colour_blending() {
        assert_eq!(lerp_color(Color32::BLACK, Color32::WHITE, 0.0), Color32::BLACK);
        assert_eq!(lerp_color(Color32::BLACK, Color32::WHITE, 1.0), Color32::WHITE);
    }

    #[test]
    fn icon_has_transparent_corners_and_solid_middle() {
        for n in [16u32, 32, 64, 256] {
            let px = icon_rgba(n, None);
            assert_eq!(px.len(), (n * n * 4) as usize);
            assert_eq!(px[3], 0, "top-left corner is transparent at {n}px");
            let mid = ((n / 2 * n + n / 2) * 4) as usize;
            assert_eq!(px[mid + 3], 255, "middle is opaque at {n}px");
        }
        let app = app_icon();
        assert_eq!((app.width, app.height), (64, 64));
    }

    #[test]
    fn tray_gauge_is_drawn_into_the_icon() {
        let n = 64u32;
        let green = icon_rgba(n, Some((1.0, GREEN)));
        // Bottom of the gauge column (x ≈ 21.5/64 of the right half).
        let (x, y) = (((21.5 + 32.0) / 64.0 * n as f32) as u32, ((20.0 + 32.0) / 64.0 * n as f32) as u32);
        let i = ((y * n + x) * 4) as usize;
        assert_eq!(&green[i..i + 3], &[GREEN.r(), GREEN.g(), GREEN.b()]);
        let empty = icon_rgba(n, Some((0.0, GREEN)));
        assert_ne!(&empty[i..i + 3], &[GREEN.r(), GREEN.g(), GREEN.b()]);
    }
}
