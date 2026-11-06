//! Windows: built-in laptop screens through WMI (the same control as the
//! brightness slider in Quick Settings). External monitors aren't listed yet.

use super::{Kind, Screen};
use std::process::Command;

fn powershell(script: &str) -> Option<String> {
    let o = Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", script]).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

pub fn list() -> Vec<Screen> {
    let Some(out) = powershell("(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1).CurrentBrightness") else {
        return vec![];
    };
    match out.lines().next().and_then(|l| l.trim().parse::<f32>().ok()) {
        Some(v) => vec![Screen { id: "wmi".into(), name: "Built-in display".into(), kind: Kind::BuiltIn, level: v / 100.0 }],
        None => vec![],
    }
}

pub fn set(_screen: &Screen, level: f32) -> Result<(), String> {
    let pct = (level.clamp(0.0, 1.0) * 100.0).round() as u32;
    let script = format!(
        "Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=0;Brightness={pct}}} | Out-Null"
    );
    powershell(&script).map(|_| ()).ok_or_else(|| "Windows didn't allow changing the brightness".into())
}
