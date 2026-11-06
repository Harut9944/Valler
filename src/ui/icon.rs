//! The Valler mark: painted in the UI and rendered to pixels for the window,
//! taskbar, launcher and tray icons.

use egui::{Color32, Pos2, Shape, Stroke, vec2};
use super::*;

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
