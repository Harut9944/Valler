//! Linux: built-in screens through /sys/class/backlight (changed via
//! systemd-logind, which lets the logged-in user do it without root, or
//! brightnessctl), and external monitors through ddcutil (DDC/CI).

use super::parse::{ddc_detect, ddc_vcp, raw_level};
use super::{Kind, Screen};
use std::path::Path;
use std::process::Command;

const BACKLIGHT: &str = "/sys/class/backlight";

fn read_u32(p: &Path) -> Option<u32> {
    std::fs::read_to_string(p).ok()?.trim().parse().ok()
}

fn backlights() -> Vec<Screen> {
    let Ok(rd) = std::fs::read_dir(BACKLIGHT) else { return vec![] };
    let mut v: Vec<Screen> = rd
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            let max = read_u32(&p.join("max_brightness")).filter(|m| *m > 0)?;
            let cur = read_u32(&p.join("brightness"))?;
            let id = e.file_name().to_string_lossy().to_string();
            Some(Screen { id, name: "Built-in display".into(), kind: Kind::BuiltIn, level: cur as f32 / max as f32 })
        })
        .collect();
    // Several drivers can expose the same panel; one control is enough.
    v.truncate(1);
    v
}

fn ddc_screens() -> Vec<Screen> {
    let Ok(out) = Command::new("ddcutil").args(["detect", "--brief"]).output() else { return vec![] };
    let text = String::from_utf8_lossy(&out.stdout);
    ddc_detect(&text)
        .into_iter()
        .filter_map(|(n, name)| {
            let o = Command::new("ddcutil").args(["getvcp", "10", "--brief", "--display", &n]).output().ok()?;
            let (cur, max) = ddc_vcp(&String::from_utf8_lossy(&o.stdout))?;
            Some(Screen { id: n, name, kind: Kind::External, level: cur as f32 / max as f32 })
        })
        .collect()
}

pub fn list() -> Vec<Screen> {
    let mut v = backlights();
    v.extend(ddc_screens());
    v
}

fn set_backlight(id: &str, level: f32) -> Result<(), String> {
    let dir = Path::new(BACKLIGHT).join(id);
    let max = read_u32(&dir.join("max_brightness")).ok_or("The backlight can't be read")?;
    let raw = raw_level(level, max, 1).to_string();
    let via_logind = Command::new("busctl")
        .args(["call", "--system", "org.freedesktop.login1", "/org/freedesktop/login1/session/auto", "org.freedesktop.login1.Session", "SetBrightness", "ssu", "backlight", id, &raw])
        .status()
        .is_ok_and(|s| s.success());
    if via_logind {
        return Ok(());
    }
    let via_tool = Command::new("brightnessctl").args(["-d", id, "set", &raw]).status().is_ok_and(|s| s.success());
    if via_tool { Ok(()) } else { Err("The system didn't allow changing the brightness".into()) }
}

fn set_external(display: &str, level: f32) -> Result<(), String> {
    let max = Command::new("ddcutil")
        .args(["getvcp", "10", "--brief", "--display", display])
        .output()
        .ok()
        .and_then(|o| ddc_vcp(&String::from_utf8_lossy(&o.stdout)))
        .map(|(_, m)| m)
        .unwrap_or(100);
    let raw = raw_level(level, max, 0).to_string();
    let ok = Command::new("ddcutil").args(["setvcp", "10", &raw, "--display", display]).status().is_ok_and(|s| s.success());
    if ok { Ok(()) } else { Err("The monitor didn't accept the change (is DDC/CI on in its menu?)".into()) }
}

pub fn set(screen: &Screen, level: f32) -> Result<(), String> {
    match screen.kind {
        Kind::BuiltIn => set_backlight(&screen.id, level),
        Kind::External => set_external(&screen.id, level),
    }
}
