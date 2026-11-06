//! Human-readable sizes, speeds and durations.

pub fn fmt_bytes(b: u64) -> String {
    let b = b as f64;
    const K: f64 = 1024.0;
    if b < K {
        format!("{b:.0} B")
    } else if b < K * K {
        format!("{:.1} KB", b / K)
    } else if b < K * K * K {
        format!("{:.1} MB", b / K / K)
    } else if b < K * K * K * K {
        format!("{:.2} GB", b / K / K / K)
    } else {
        format!("{:.2} TB", b / K / K / K / K)
    }
}

pub fn fmt_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", fmt_bytes(bytes_per_sec.max(0.0) as u64))
}

pub fn fmt_duration(secs: u64) -> String {
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        format!("{d}d {h}h {m}min")
    } else if h > 0 {
        format!("{h}h {m}min")
    } else {
        format!("{m}min")
    }
}
