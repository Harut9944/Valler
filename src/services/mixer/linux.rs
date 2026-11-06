//! Linux mixer: PulseAudio or PipeWire (pipewire-pulse) through `pactl`,
//! which both GNOME and KDE use.

use super::*;
use serde_json::Value;
use std::process::Command;

fn pactl_json(args: &[&str]) -> Option<Value> {
    let out = Command::new("pactl").args(["-f", "json"]).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

fn pactl_text(args: &[&str]) -> Option<String> {
    let out = Command::new("pactl").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn avg_volume(v: &Value) -> f32 {
    // "volume": {"front-left": {"value": 65536, "value_percent": "100%", ...}, ...}
    let Some(obj) = v.get("volume").and_then(|v| v.as_object()) else { return 1.0 };
    let vals: Vec<f32> = obj
        .values()
        .filter_map(|c| c.get("value").and_then(|x| x.as_f64()))
        .map(|x| (x / 65536.0) as f32)
        .collect();
    if vals.is_empty() { 1.0 } else { vals.iter().sum::<f32>() / vals.len() as f32 }
}

fn prop(v: &Value, key: &str) -> String {
    v.get("properties").and_then(|p| p.get(key)).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

fn devices(kind: &str) -> Vec<Device> {
    let Some(Value::Array(arr)) = pactl_json(&["list", kind]) else { return vec![] };
    devices_from(&arr, kind)
}

/// Turns `pactl -f json list sinks|sources` output into devices (monitors skipped).
pub(super) fn devices_from(arr: &[Value], kind: &str) -> Vec<Device> {
    arr.iter()
        .filter(|d| kind != "sources" || prop(d, "device.class") != "monitor" && !d.get("name").and_then(|n| n.as_str()).unwrap_or("").ends_with(".monitor"))
        .map(|d| Device {
            id: d.get("index").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
            name: d.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            description: d.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            volume: avg_volume(d),
            muted: d.get("mute").and_then(|x| x.as_bool()).unwrap_or(false),
        })
        .collect()
}

pub fn read() -> MixerState {
    let Some(Value::Array(inputs)) = pactl_json(&["list", "sink-inputs"]) else {
        return MixerState {
            available: false,
            message: Some("Couldn't reach the sound server. Valler needs PulseAudio or PipeWire with the 'pactl' tool (package pulseaudio-utils or libpulse).".into()),
            ..Default::default()
        };
    };
    let streams = streams_from(&inputs);
    MixerState {
        available: true,
        message: None,
        caps: Caps::default(),
        streams,
        sinks: devices("sinks"),
        sources: devices("sources"),
        default_sink: pactl_text(&["get-default-sink"]).unwrap_or_default(),
        default_source: pactl_text(&["get-default-source"]).unwrap_or_default(),
    }
}

/// Turns `pactl -f json list sink-inputs` output into mixer rows.
pub(super) fn streams_from(inputs: &[Value]) -> Vec<Stream> {
    inputs
        .iter()
        .map(|s| {
            let mut app = prop(s, "application.name");
            if app.is_empty() {
                app = prop(s, "application.process.binary");
            }
            if app.is_empty() {
                app = "Unknown app".into();
            }
            Stream {
                id: s.get("index").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                app,
                media: prop(s, "media.name"),
                volume: avg_volume(s),
                muted: s.get("mute").and_then(|x| x.as_bool()).unwrap_or(false),
                sink: s.get("sink").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
            }
        })
        .collect()
}

fn run(args: &[&str]) {
    let _ = Command::new("pactl").args(args).status();
}

pub fn set_stream_volume(id: u32, vol: f32) {
    run(&["set-sink-input-volume", &id.to_string(), &format!("{}%", (vol * 100.0).round() as i32)]);
}
pub fn set_stream_mute(id: u32, mute: bool) {
    run(&["set-sink-input-mute", &id.to_string(), if mute { "1" } else { "0" }]);
}
pub fn move_stream(id: u32, sink: u32) {
    run(&["move-sink-input", &id.to_string(), &sink.to_string()]);
}
pub fn set_sink_volume(name: &str, vol: f32) {
    run(&["set-sink-volume", name, &format!("{}%", (vol * 100.0).round() as i32)]);
}
pub fn set_sink_mute(name: &str, mute: bool) {
    run(&["set-sink-mute", name, if mute { "1" } else { "0" }]);
}
pub fn set_default_sink(name: &str) {
    run(&["set-default-sink", name]);
}
pub fn set_source_mute(name: &str, mute: bool) {
    run(&["set-source-mute", name, if mute { "1" } else { "0" }]);
}
pub fn set_source_volume(name: &str, vol: f32) {
    run(&["set-source-volume", name, &format!("{}%", (vol * 100.0).round() as i32)]);
}
pub fn set_default_source(name: &str) {
    run(&["set-default-source", name]);
}
