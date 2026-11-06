//! Picks the platform's own UI font (Segoe UI, San Francisco, Noto/Cantarell…)
//! so the app feels native everywhere, with egui's bundled font as fallback.
//! Adds a "bold" family used for titles, and Phosphor icons.

use egui::{FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;
use std::sync::Arc;

pub const BOLD: &str = "bold";

fn candidates(bold: bool) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    #[cfg(target_os = "linux")]
    {
        let pattern = if bold { "sans-serif:weight=semibold" } else { "sans-serif" };
        if let Ok(out) = std::process::Command::new("fc-match")
            .args(["-f", "%{file}", pattern])
            .output()
        {
            let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !p.is_empty() {
                v.push(PathBuf::from(p));
            }
        }
        let names: &[&str] = if bold {
            &[
                "/usr/share/fonts/noto/NotoSans-SemiBold.ttf",
                "/usr/share/fonts/noto/NotoSans-Bold.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
            ]
        } else {
            &[
                "/usr/share/fonts/noto/NotoSans-Regular.ttf",
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
            ]
        };
        v.extend(names.iter().map(PathBuf::from));
    }
    #[cfg(target_os = "windows")]
    {
        let dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        let names: &[&str] = if bold {
            &["seguisb.ttf", "segoeuib.ttf", "arialbd.ttf"]
        } else {
            &["segoeui.ttf", "arial.ttf"]
        };
        v.extend(names.iter().map(|n| PathBuf::from(format!("{dir}\\Fonts\\{n}"))));
    }
    #[cfg(target_os = "macos")]
    {
        let names: &[&str] = if bold {
            &[
                "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                "/Library/Fonts/Arial Bold.ttf",
            ]
        } else {
            &[
                "/System/Library/Fonts/Supplemental/Arial.ttf",
                "/Library/Fonts/Arial.ttf",
            ]
        };
        v.extend(names.iter().map(PathBuf::from));
    }
    let _ = bold;
    v
}

fn load(bold: bool) -> Option<Vec<u8>> {
    candidates(bold)
        .into_iter()
        .filter(|p| {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            ext == "ttf" || ext == "otf"
        })
        .find_map(|p| std::fs::read(p).ok())
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    if let Some(bytes) = load(false) {
        fonts
            .font_data
            .insert("system".into(), Arc::new(FontData::from_owned(bytes)));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "system".into());
    }

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    // Bold family: system bold first, then whatever proportional uses.
    let mut bold_chain: Vec<String> = Vec::new();
    if let Some(bytes) = load(true) {
        fonts
            .font_data
            .insert("system-bold".into(), Arc::new(FontData::from_owned(bytes)));
        bold_chain.push("system-bold".into());
    }
    if let Some(p) = fonts.families.get(&FontFamily::Proportional) {
        bold_chain.extend(p.iter().cloned());
    }
    fonts
        .families
        .insert(FontFamily::Name(BOLD.into()), bold_chain);

    ctx.set_fonts(fonts);
}
