//! macOS mixer through CoreAudio: output and input devices with volume, mute
//! and default-device switching, and (macOS 14 and later) the list of apps
//! that are playing sound right now.
//!
//! Changing one app's volume on macOS needs a CoreAudio process tap that
//! captures that app and plays it back scaled; that part is still to come,
//! so app rows show what is playing without a per-app slider.

#![allow(non_upper_case_globals)]

use super::*;
use std::ffi::c_void;

type AudioObjectID = u32;
type OSStatus = i32;
type CFStringRef = *const c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct Address {
    selector: u32,
    scope: u32,
    element: u32,
}

#[link(name = "CoreAudio", kind = "framework")]
unsafe extern "C" {
    fn AudioObjectHasProperty(id: AudioObjectID, addr: *const Address) -> u8;
    fn AudioObjectGetPropertyDataSize(id: AudioObjectID, addr: *const Address, qual_size: u32, qual: *const c_void, size: *mut u32) -> OSStatus;
    fn AudioObjectGetPropertyData(id: AudioObjectID, addr: *const Address, qual_size: u32, qual: *const c_void, size: *mut u32, data: *mut c_void) -> OSStatus;
    fn AudioObjectSetPropertyData(id: AudioObjectID, addr: *const Address, qual_size: u32, qual: *const c_void, size: u32, data: *const c_void) -> OSStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringGetCString(s: CFStringRef, buf: *mut i8, size: isize, encoding: u32) -> u8;
    fn CFRelease(cf: *const c_void);
}

const fn fourcc(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
}

const SYSTEM: AudioObjectID = 1;
const ELEMENT_MAIN: u32 = 0;
const SCOPE_GLOBAL: u32 = fourcc(b"glob");
const SCOPE_OUTPUT: u32 = fourcc(b"outp");
const SCOPE_INPUT: u32 = fourcc(b"inpt");
const DEVICES: u32 = fourcc(b"dev#");
const DEFAULT_OUTPUT: u32 = fourcc(b"dOut");
const DEFAULT_INPUT: u32 = fourcc(b"dIn ");
const NAME: u32 = fourcc(b"lnam");
const UID: u32 = fourcc(b"uid ");
const STREAMS: u32 = fourcc(b"stm#");
const VIRTUAL_MAIN_VOLUME: u32 = fourcc(b"vmvc");
const VOLUME_SCALAR: u32 = fourcc(b"volm");
const MUTE: u32 = fourcc(b"mute");
const PROCESS_LIST: u32 = fourcc(b"prs#");
const PROCESS_PID: u32 = fourcc(b"ppid");
const PROCESS_BUNDLE: u32 = fourcc(b"pbid");
const PROCESS_RUNNING_OUTPUT: u32 = fourcc(b"piro");
const UTF8: u32 = 0x0800_0100;

fn addr(selector: u32, scope: u32) -> Address {
    Address { selector, scope, element: ELEMENT_MAIN }
}

fn has(id: AudioObjectID, a: Address) -> bool {
    unsafe { AudioObjectHasProperty(id, &a) != 0 }
}

fn get<T: Copy + Default>(id: AudioObjectID, a: Address) -> Option<T> {
    let mut v = T::default();
    let mut size = std::mem::size_of::<T>() as u32;
    let st = unsafe { AudioObjectGetPropertyData(id, &a, 0, std::ptr::null(), &mut size, &mut v as *mut T as *mut c_void) };
    (st == 0).then_some(v)
}

fn set<T: Copy>(id: AudioObjectID, a: Address, v: T) -> bool {
    let st = unsafe { AudioObjectSetPropertyData(id, &a, 0, std::ptr::null(), std::mem::size_of::<T>() as u32, &v as *const T as *const c_void) };
    st == 0
}

fn get_list(id: AudioObjectID, a: Address) -> Vec<AudioObjectID> {
    unsafe {
        let mut size = 0u32;
        if AudioObjectGetPropertyDataSize(id, &a, 0, std::ptr::null(), &mut size) != 0 || size == 0 {
            return vec![];
        }
        let mut v = vec![0u32; size as usize / 4];
        if AudioObjectGetPropertyData(id, &a, 0, std::ptr::null(), &mut size, v.as_mut_ptr() as *mut c_void) != 0 {
            return vec![];
        }
        v.truncate(size as usize / 4);
        v
    }
}

fn get_string(id: AudioObjectID, a: Address) -> Option<String> {
    let s: usize = get::<usize>(id, a)?;
    let cf = s as CFStringRef;
    if cf.is_null() {
        return None;
    }
    let mut buf = vec![0i8; 512];
    let ok = unsafe { CFStringGetCString(cf, buf.as_mut_ptr(), buf.len() as isize, UTF8) } != 0;
    unsafe { CFRelease(cf) };
    if !ok {
        return None;
    }
    let bytes: Vec<u8> = buf.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect();
    Some(String::from_utf8_lossy(&bytes).to_string())
}

fn has_streams(dev: AudioObjectID, scope: u32) -> bool {
    !get_list(dev, addr(STREAMS, scope)).is_empty()
}

fn volume(dev: AudioObjectID, scope: u32) -> f32 {
    if has(dev, addr(VIRTUAL_MAIN_VOLUME, scope)) {
        return get::<f32>(dev, addr(VIRTUAL_MAIN_VOLUME, scope)).unwrap_or(1.0);
    }
    if has(dev, addr(VOLUME_SCALAR, scope)) {
        return get::<f32>(dev, addr(VOLUME_SCALAR, scope)).unwrap_or(1.0);
    }
    // Some devices only have per-channel volume: report channel 1.
    let ch = Address { selector: VOLUME_SCALAR, scope, element: 1 };
    get::<f32>(dev, ch).unwrap_or(1.0)
}

fn set_volume(dev: AudioObjectID, scope: u32, v: f32) {
    let v = v.clamp(0.0, 1.0);
    if has(dev, addr(VIRTUAL_MAIN_VOLUME, scope)) && set(dev, addr(VIRTUAL_MAIN_VOLUME, scope), v) {
        return;
    }
    if set(dev, addr(VOLUME_SCALAR, scope), v) {
        return;
    }
    for ch in 1..=2 {
        let _ = set(dev, Address { selector: VOLUME_SCALAR, scope, element: ch }, v);
    }
}

fn devices(scope: u32) -> Vec<Device> {
    get_list(SYSTEM, addr(DEVICES, SCOPE_GLOBAL))
        .into_iter()
        .filter(|d| has_streams(*d, scope))
        .map(|d| Device {
            id: d,
            name: get_string(d, addr(UID, SCOPE_GLOBAL)).unwrap_or_else(|| d.to_string()),
            description: get_string(d, addr(NAME, SCOPE_GLOBAL)).unwrap_or_else(|| "Audio device".into()),
            volume: volume(d, scope),
            muted: get::<u32>(d, addr(MUTE, scope)).unwrap_or(0) != 0,
        })
        .collect()
}

fn device_by_uid(uid: &str, scope: u32) -> Option<AudioObjectID> {
    devices(scope).into_iter().find(|d| d.name == uid).map(|d| d.id)
}

fn app_name(pid: i32, bundle: &str) -> String {
    if let Ok(mut sys) = sysinfo::System::new() {
        let p = sysinfo::Pid::from_u32(pid as u32);
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[p]), true);
        if let Some(pr) = sys.process(p) {
            // Prefer the .app bundle name when the binary lives inside one.
            if let Some(exe) = pr.exe()
                && let Some(app) = exe.ancestors().find(|a| a.extension().is_some_and(|e| e == "app"))
                && let Some(stem) = app.file_stem()
            {
                return stem.to_string_lossy().to_string();
            }
            return pr.name().to_string_lossy().to_string();
        }
    }
    bundle.rsplit('.').next().unwrap_or("App").to_string()
}

/// Apps currently sending audio to an output (macOS 14+; empty before that).
fn playing_apps(default_out: u32) -> Vec<Stream> {
    if !has(SYSTEM, addr(PROCESS_LIST, SCOPE_GLOBAL)) {
        return vec![];
    }
    let mut out: Vec<Stream> = Vec::new();
    for p in get_list(SYSTEM, addr(PROCESS_LIST, SCOPE_GLOBAL)) {
        if get::<u32>(p, addr(PROCESS_RUNNING_OUTPUT, SCOPE_GLOBAL)).unwrap_or(0) == 0 {
            continue;
        }
        let pid = get::<i32>(p, addr(PROCESS_PID, SCOPE_GLOBAL)).unwrap_or(0);
        if pid as u32 == std::process::id() {
            continue;
        }
        let bundle = get_string(p, addr(PROCESS_BUNDLE, SCOPE_GLOBAL)).unwrap_or_default();
        let name = app_name(pid, &bundle);
        if out.iter().any(|s| s.app == name) {
            continue;
        }
        out.push(Stream { id: pid as u32, app: name, media: bundle, volume: 1.0, muted: false, sink: default_out });
    }
    out.sort_by_key(|s| s.app.to_lowercase());
    out
}

pub fn read() -> MixerState {
    let out_dev = get::<u32>(SYSTEM, addr(DEFAULT_OUTPUT, SCOPE_GLOBAL)).unwrap_or(0);
    let in_dev = get::<u32>(SYSTEM, addr(DEFAULT_INPUT, SCOPE_GLOBAL)).unwrap_or(0);
    let sinks = devices(SCOPE_OUTPUT);
    let sources = devices(SCOPE_INPUT);
    if sinks.is_empty() && sources.is_empty() {
        return MixerState { available: false, message: Some("No audio devices were found.".into()), ..Default::default() };
    }
    let uid_of = |list: &Vec<Device>, id: u32| list.iter().find(|d| d.id == id).map(|d| d.name.clone()).unwrap_or_default();
    MixerState {
        available: true,
        message: Some("Per-app volume on macOS is coming next. You can already see which apps are playing and control your speakers, headphones and microphone.".into()),
        caps: Caps { per_app_volume: false, per_app_output: false, boost: false, switch_default: true },
        streams: playing_apps(out_dev),
        default_sink: uid_of(&sinks, out_dev),
        default_source: uid_of(&sources, in_dev),
        sinks,
        sources,
    }
}

pub fn set_stream_volume(_: u32, _: f32) {}
pub fn set_stream_mute(_: u32, _: bool) {}
pub fn move_stream(_: u32, _: u32) {}

pub fn set_sink_volume(uid: &str, v: f32) {
    if let Some(d) = device_by_uid(uid, SCOPE_OUTPUT) {
        set_volume(d, SCOPE_OUTPUT, v);
    }
}

pub fn set_sink_mute(uid: &str, mute: bool) {
    if let Some(d) = device_by_uid(uid, SCOPE_OUTPUT) {
        let _ = set::<u32>(d, addr(MUTE, SCOPE_OUTPUT), mute as u32);
    }
}

pub fn set_default_sink(uid: &str) {
    if let Some(d) = device_by_uid(uid, SCOPE_OUTPUT) {
        let _ = set::<u32>(SYSTEM, addr(DEFAULT_OUTPUT, SCOPE_GLOBAL), d);
    }
}

pub fn set_source_volume(uid: &str, v: f32) {
    if let Some(d) = device_by_uid(uid, SCOPE_INPUT) {
        set_volume(d, SCOPE_INPUT, v);
    }
}

pub fn set_source_mute(uid: &str, mute: bool) {
    if let Some(d) = device_by_uid(uid, SCOPE_INPUT) {
        let _ = set::<u32>(d, addr(MUTE, SCOPE_INPUT), mute as u32);
    }
}

pub fn set_default_source(uid: &str) {
    if let Some(d) = device_by_uid(uid, SCOPE_INPUT) {
        let _ = set::<u32>(SYSTEM, addr(DEFAULT_INPUT, SCOPE_GLOBAL), d);
    }
}
