//! Windows mixer: per-app volume and mute through WASAPI audio sessions
//! (the same sessions the Windows Volume Mixer shows), plus volume and mute
//! for output devices and the microphone through the endpoint volume API.
//!
//! Windows has no public API to move one app to another output or to change
//! the default device, so those controls send people to Sound settings.

use super::*;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    AudioSessionStateExpired, DEVICE_STATE_ACTIVE, EDataFlow, IAudioSessionControl2, IAudioSessionManager2,
    IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator, eCapture, eConsole, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree, STGM_READ};
use windows::core::Interface;

fn com() {
    // Safe to call repeatedly; each mixer thread joins the multithreaded apartment.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

fn enumerator() -> windows::core::Result<IMMDeviceEnumerator> {
    com();
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
}

fn device_id(dev: &IMMDevice) -> String {
    unsafe {
        match dev.GetId() {
            Ok(p) => {
                let s = p.to_string().unwrap_or_default();
                CoTaskMemFree(Some(p.0 as *const _));
                s
            }
            Err(_) => String::new(),
        }
    }
}

fn friendly_name(dev: &IMMDevice) -> String {
    unsafe {
        dev.OpenPropertyStore(STGM_READ)
            .and_then(|store| store.GetValue(&PKEY_Device_FriendlyName))
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "Audio device".into())
    }
}

fn endpoint(dev: &IMMDevice) -> Option<IAudioEndpointVolume> {
    unsafe { dev.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).ok() }
}

fn devices(en: &IMMDeviceEnumerator, flow: EDataFlow) -> Vec<(IMMDevice, Device)> {
    let mut out = Vec::new();
    unsafe {
        let Ok(coll) = en.EnumAudioEndpoints(flow, DEVICE_STATE_ACTIVE) else { return out };
        let n = coll.GetCount().unwrap_or(0);
        for i in 0..n {
            let Ok(dev) = coll.Item(i) else { continue };
            let (volume, muted) = endpoint(&dev)
                .map(|ep| (ep.GetMasterVolumeLevelScalar().unwrap_or(1.0), ep.GetMute().map(|b| b.as_bool()).unwrap_or(false)))
                .unwrap_or((1.0, false));
            let d = Device { id: i, name: device_id(&dev), description: friendly_name(&dev), volume, muted };
            out.push((dev, d));
        }
    }
    out
}

fn default_id(en: &IMMDeviceEnumerator, flow: EDataFlow) -> String {
    unsafe { en.GetDefaultAudioEndpoint(flow, eConsole).map(|d| device_id(&d)).unwrap_or_default() }
}

fn process_name(pid: u32) -> String {
    if pid == 0 {
        return "System sounds".into();
    }
    let mut sys = match sysinfo::System::new() {
        Ok(s) => s,
        Err(_) => return format!("App {pid}"),
    };
    let p = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[p]), true);
    sys.process(p)
        .map(|pr| {
            let n = pr.name().to_string_lossy().to_string();
            let n = n.strip_suffix(".exe").or_else(|| n.strip_suffix(".EXE")).unwrap_or(&n).to_string();
            // "msedge" -> "Msedge": a readable name without a lookup table.
            let mut c = n.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or(n)
        })
        .unwrap_or_else(|| format!("App {pid}"))
}

/// Visits every live audio session on every active output device.
fn for_each_session(mut f: impl FnMut(u32, u32, &ISimpleAudioVolume, bool)) {
    let Ok(en) = enumerator() else { return };
    for (dev, d) in devices(&en, eRender) {
        unsafe {
            let Ok(mgr) = dev.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else { continue };
            let Ok(list) = mgr.GetSessionEnumerator() else { continue };
            let n = list.GetCount().unwrap_or(0);
            for i in 0..n {
                let Ok(ctl) = list.GetSession(i) else { continue };
                if ctl.GetState().map(|s| s == AudioSessionStateExpired).unwrap_or(true) {
                    continue;
                }
                let Ok(ctl2) = ctl.cast::<IAudioSessionControl2>() else { continue };
                let pid = ctl2.GetProcessId().unwrap_or(0);
                let system = ctl2.IsSystemSoundsSession().0 == 0; // S_OK means "is system sounds"
                let Ok(vol) = ctl.cast::<ISimpleAudioVolume>() else { continue };
                f(if system { 0 } else { pid }, d.id, &vol, system);
            }
        }
    }
}

pub fn read() -> MixerState {
    let Ok(en) = enumerator() else {
        return MixerState {
            available: false,
            message: Some("Couldn't reach the Windows audio service.".into()),
            ..Default::default()
        };
    };
    let sinks: Vec<Device> = devices(&en, eRender).into_iter().map(|(_, d)| d).collect();
    let sources: Vec<Device> = devices(&en, eCapture).into_iter().map(|(_, d)| d).collect();
    let mut streams: Vec<Stream> = Vec::new();
    for_each_session(|pid, sink, vol, _system| unsafe {
        // One row per app: several sessions of the same app share a row.
        if streams.iter().any(|s| s.id == pid) {
            return;
        }
        streams.push(Stream {
            id: pid,
            app: process_name(pid),
            media: String::new(),
            volume: vol.GetMasterVolume().unwrap_or(1.0),
            muted: vol.GetMute().map(|b| b.as_bool()).unwrap_or(false),
            sink,
        });
    });
    // System sounds last, apps alphabetically.
    streams.sort_by_key(|s| (s.id == 0, s.app.to_lowercase()));
    MixerState {
        available: true,
        message: None,
        caps: Caps { per_app_volume: true, per_app_output: false, boost: false, switch_default: false },
        streams,
        sinks,
        sources,
        default_sink: default_id(&en, eRender),
        default_source: default_id(&en, eCapture),
    }
}

pub fn set_stream_volume(id: u32, v: f32) {
    for_each_session(|pid, _, vol, _| unsafe {
        if pid == id {
            let _ = vol.SetMasterVolume(v.clamp(0.0, 1.0), std::ptr::null());
        }
    });
}

pub fn set_stream_mute(id: u32, mute: bool) {
    for_each_session(|pid, _, vol, _| unsafe {
        if pid == id {
            let _ = vol.SetMute(mute, std::ptr::null());
        }
    });
}

pub fn move_stream(_: u32, _: u32) {}

fn with_endpoint(name: &str, flow: EDataFlow, f: impl Fn(&IAudioEndpointVolume)) {
    let Ok(en) = enumerator() else { return };
    for (dev, d) in devices(&en, flow) {
        if d.name == name
            && let Some(ep) = endpoint(&dev)
        {
            f(&ep);
        }
    }
}

pub fn set_sink_volume(name: &str, v: f32) {
    with_endpoint(name, eRender, |ep| unsafe {
        let _ = ep.SetMasterVolumeLevelScalar(v.clamp(0.0, 1.0), std::ptr::null());
    });
}

pub fn set_sink_mute(name: &str, mute: bool) {
    with_endpoint(name, eRender, |ep| unsafe {
        let _ = ep.SetMute(mute, std::ptr::null());
    });
}

pub fn set_source_volume(name: &str, v: f32) {
    with_endpoint(name, eCapture, |ep| unsafe {
        let _ = ep.SetMasterVolumeLevelScalar(v.clamp(0.0, 1.0), std::ptr::null());
    });
}

pub fn set_source_mute(name: &str, mute: bool) {
    with_endpoint(name, eCapture, |ep| unsafe {
        let _ = ep.SetMute(mute, std::ptr::null());
    });
}

/// No public API; open Sound settings where the person can pick the device.
pub fn set_default_sink(_: &str) {
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", "ms-settings:sound"]).spawn();
}

pub fn set_default_source(_: &str) {
    set_default_sink("");
}

#[allow(dead_code)]
fn _render() -> EDataFlow {
    eRender
}
