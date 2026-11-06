//! Volume mixer. On Linux it talks to PulseAudio or PipeWire (pipewire-pulse),
//! which both GNOME and KDE use, through `pactl`. On Windows it uses WASAPI
//! audio sessions (see mixer_win.rs) and on macOS CoreAudio (mixer_mac.rs).

/// What the current system lets the mixer do; the UI hides the rest.
#[derive(Clone, Copy, Debug)]
pub struct Caps {
    pub per_app_volume: bool,
    pub per_app_output: bool,
    /// Volume above 100 %.
    pub boost: bool,
    /// Change the default output/input device from Valler.
    pub switch_default: bool,
}

impl Default for Caps {
    fn default() -> Self {
        Self { per_app_volume: true, per_app_output: true, boost: true, switch_default: true }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stream {
    pub id: u32,
    pub app: String,
    pub media: String,
    pub volume: f32, // 1.0 = 100 %
    pub muted: bool,
    pub sink: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Device {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub volume: f32,
    pub muted: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MixerState {
    pub available: bool,
    pub message: Option<String>,
    pub caps: Caps,
    pub streams: Vec<Stream>,
    pub sinks: Vec<Device>,
    pub sources: Vec<Device>,
    pub default_sink: String,
    pub default_source: String,
}

pub fn demo() -> MixerState {
    MixerState {
        available: true,
        message: None,
        caps: Caps::default(),
        streams: vec![
            Stream { id: 1, app: "Chat app".into(), media: "Voice call".into(), volume: 0.29, muted: false, sink: 1 },
            Stream { id: 2, app: "Music player".into(), media: "Playback".into(), volume: 1.87, muted: false, sink: 1 },
            Stream { id: 3, app: "Web browser".into(), media: "Video".into(), volume: 1.0, muted: false, sink: 1 },
        ],
        sinks: vec![Device { id: 1, name: "speakers".into(), description: "Built-in speakers".into(), volume: 0.6, muted: false }],
        sources: vec![Device { id: 2, name: "mic".into(), description: "Built-in microphone".into(), volume: 0.8, muted: false }],
        default_sink: "speakers".into(),
        default_source: "mic".into(),
    }
}

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
mod imp {
    use super::*;
    pub fn read() -> MixerState {
        MixerState { available: false, message: Some("The mixer isn't available on this system.".into()), ..Default::default() }
    }
    pub fn set_stream_volume(_: u32, _: f32) {}
    pub fn set_stream_mute(_: u32, _: bool) {}
    pub fn move_stream(_: u32, _: u32) {}
    pub fn set_sink_volume(_: &str, _: f32) {}
    pub fn set_sink_mute(_: &str, _: bool) {}
    pub fn set_default_sink(_: &str) {}
    pub fn set_source_mute(_: &str, _: bool) {}
    pub fn set_source_volume(_: &str, _: f32) {}
    pub fn set_default_source(_: &str) {}
}

pub use imp::*;

/// Commands run off the UI thread so a slow sound server never freezes the panel.
pub fn spawn_cmd(f: impl FnOnce() + Send + 'static) {
    std::thread::spawn(f);
}

#[cfg(test)]
mod feature_tests {
    use super::*;

    #[test]
    fn demo_and_capabilities() {
        let d = demo();
        assert!(d.available);
        assert_eq!(d.streams.len(), 3);
        assert!(d.streams.iter().any(|s| s.volume > 1.0), "demo shows a boosted app");
        let c = Caps::default();
        assert!(c.per_app_volume && c.per_app_output && c.boost && c.switch_default);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn reads_pactl_json() {
        let inputs: serde_json::Value = serde_json::from_str(r#"[
            {"index": 42, "sink": 3, "mute": false,
             "volume": {"front-left": {"value": 65536}, "front-right": {"value": 32768}},
             "properties": {"application.name": "Music", "media.name": "Song"}},
            {"index": 7, "sink": 3, "mute": true, "volume": {},
             "properties": {"application.process.binary": "game"}}
        ]"#).unwrap();
        let s = imp::streams_from(inputs.as_array().unwrap());
        assert_eq!(s.len(), 2);
        assert_eq!((s[0].id, s[0].app.as_str(), s[0].media.as_str(), s[0].sink), (42, "Music", "Song", 3));
        assert!((s[0].volume - 0.75).abs() < 1e-6, "average of both channels");
        assert_eq!(s[1].app, "game");
        assert!(s[1].muted);
        assert_eq!(s[1].volume, 1.0, "missing volume counts as 100 %");

        let sources: serde_json::Value = serde_json::from_str(r#"[
            {"index": 1, "name": "mic", "description": "Microphone", "mute": false, "volume": {"mono": {"value": 65536}}, "properties": {}},
            {"index": 2, "name": "speakers.monitor", "description": "Monitor of Speakers", "mute": false, "volume": {}, "properties": {"device.class": "monitor"}}
        ]"#).unwrap();
        let d = imp::devices_from(sources.as_array().unwrap(), "sources");
        assert_eq!(d.len(), 1, "monitor sources are not microphones");
        assert_eq!(d[0].description, "Microphone");
    }

    #[test]
    fn reading_the_sound_system_never_panics() {
        let s = read();
        if !s.available {
            assert!(s.message.is_some(), "an unavailable mixer explains why");
        }
    }
}
