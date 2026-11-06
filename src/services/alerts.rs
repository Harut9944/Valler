//! Monitor alerts: optional desktop notifications for sustained processor
//! load, high temperatures, critical memory, low disk space and low battery.
//! Like the original, everything is off by default, a reading must hold for
//! 12 seconds before it counts, and each kind waits a cooldown before repeating.

use crate::services::monitor::Snapshot;
use crate::core::store::Store;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const SUSTAINED: Duration = Duration::from_secs(12);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Cpu,
    CpuTemp,
    Memory,
    Disk,
    Battery,
    BatteryTemp,
}

/// Holds a reading above its threshold for `SUSTAINED` before firing, so one
/// short burst never becomes an alert.
#[derive(Default)]
struct Gate {
    since: Option<Instant>,
}

impl Gate {
    fn check(&mut self, reading: Option<f32>, threshold: f32) -> bool {
        match reading {
            Some(v) if v >= threshold => {
                let s = *self.since.get_or_insert_with(Instant::now);
                s.elapsed() >= SUSTAINED
            }
            _ => {
                self.since = None;
                false
            }
        }
    }
}

#[derive(Default)]
pub struct Alerts {
    cpu: Gate,
    cpu_temp: Gate,
    bat_temp: Gate,
    last_sent: HashMap<Kind, Instant>,
    pub last_message: Option<String>,
    /// How many notifications were sent (for the Settings page and tests).
    pub sent: u32,
}

pub fn any_enabled(s: &Store) -> bool {
    let a = &s.alerts;
    a.cpu || a.cpu_temp || a.memory || a.disk || a.battery || a.battery_temp
}

impl Alerts {
    pub fn evaluate(&mut self, snap: &Snapshot, store: &Store) {
        if !snap.ready || !store.is_installed("monitor_alerts") || !any_enabled(store) {
            return;
        }
        let a = &store.alerts;
        let temp = |c: f32| {
            if store.temperature_fahrenheit { format!("{:.0} °F", c * 9.0 / 5.0 + 32.0) } else { format!("{c:.0} °C") }
        };

        if a.cpu && self.cpu.check(Some(snap.cpu_total), a.cpu_threshold as f32) {
            self.send(store, Kind::Cpu, "High processor load", &format!("The processor has been above {}% for a while.", a.cpu_threshold));
        } else if !a.cpu {
            self.cpu.since = None;
        }
        if a.cpu_temp && self.cpu_temp.check(snap.cpu_temp, a.cpu_temp_threshold as f32) {
            let t = temp(snap.cpu_temp.unwrap_or(0.0));
            self.send(store, Kind::CpuTemp, "Processor is hot", &format!("The processor is at {t}, above your limit of {}.", temp(a.cpu_temp_threshold as f32)));
        }
        let bt = snap.battery.as_ref().and_then(|b| b.temp_c);
        if a.battery_temp && self.bat_temp.check(bt, a.battery_temp_threshold as f32) {
            self.send(store, Kind::BatteryTemp, "Battery is hot", &format!("The battery is at {}.", temp(bt.unwrap_or(0.0))));
        }
        if a.memory && snap.pressure().0 == "Critical" {
            self.send(store, Kind::Memory, "Memory is almost full", "Close some apps or browser tabs to keep the computer responsive.");
        }
        if a.disk
            && let Some(d) = snap.disks.iter().find(|d| d.total > 0 && (d.avail as f64 / d.total as f64) * 100.0 < a.disk_free_percent as f64) {
                let name = if d.mount == "/" { "The system drive".to_string() } else { d.mount.clone() };
                self.send(store, Kind::Disk, "Low disk space", &format!("{name} has less than {}% free.", a.disk_free_percent));
            }
        if a.battery
            && let Some(b) = &snap.battery
                && !b.charging && b.percent <= a.battery_percent as f32 {
                    self.send(store, Kind::Battery, "Low battery", &format!("Battery is at {:.0}%. Plug in soon.", b.percent));
                }
    }

    fn send(&mut self, store: &Store, kind: Kind, title: &str, body: &str) {
        let cooldown = Duration::from_secs(store.alerts.cooldown_minutes.max(1) as u64 * 60);
        if self.last_sent.get(&kind).is_some_and(|t| t.elapsed() < cooldown) {
            return;
        }
        self.last_sent.insert(kind, Instant::now());
        self.last_message = Some(format!("{title}: {body}"));
        self.sent += 1;
        // Tests count alerts without putting notifications on the desktop.
        #[cfg(not(test))]
        notify(title, body);
    }
}

/// Shows a desktop notification without blocking the UI.
pub fn notify(title: &str, body: &str) {
    let (t, b) = (title.to_string(), body.to_string());
    std::thread::spawn(move || {
        let _ = notify_rust::Notification::new()
            .appname("Valler")
            .summary(&t)
            .body(&b)
            .show();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gate_needs_sustained_reading() {
        let mut g = Gate::default();
        assert!(!g.check(Some(95.0), 90.0));
        assert!(!g.check(Some(50.0), 90.0));
        assert!(g.since.is_none());
        g.since = Some(Instant::now() - SUSTAINED);
        assert!(g.check(Some(95.0), 90.0));
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;
    use crate::services::monitor::{BatteryInfo, DiskInfo, Snapshot};

    fn ready() -> Snapshot {
        Snapshot { ready: true, mem_total: 100, mem_used: 10, ..Default::default() }
    }

    #[test]
    fn everything_is_off_by_default() {
        let store = Store::default();
        assert!(!any_enabled(&store));
        let mut a = Alerts::default();
        let mut s = ready();
        s.cpu_total = 100.0;
        s.mem_used = 99;
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 0);
    }

    #[test]
    fn processor_alert_needs_a_sustained_reading_and_respects_cooldown() {
        let mut store = Store::default();
        store.alerts.cpu = true;
        let mut a = Alerts::default();
        let mut s = ready();
        s.cpu_total = 95.0;
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 0, "a single reading is not enough");
        a.cpu.since = Some(Instant::now() - SUSTAINED);
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 1);
        assert!(a.last_message.as_deref().unwrap().contains("processor"));
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 1, "cooldown stops repeats");
        s.cpu_total = 20.0;
        a.evaluate(&s, &store);
        assert!(a.cpu.since.is_none(), "dropping below the limit resets the timer");
    }

    #[test]
    fn memory_disk_and_battery_alerts() {
        let mut store = Store::default();
        store.alerts.memory = true;
        store.alerts.disk = true;
        store.alerts.battery = true;
        let mut s = ready();
        s.mem_used = 95;
        s.disks = vec![DiskInfo { mount: "/".into(), total: 100, avail: 5, ..Default::default() }];
        s.battery = Some(BatteryInfo { percent: 10.0, charging: false, ..Default::default() });
        let mut a = Alerts::default();
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 3);
    }

    #[test]
    fn charging_battery_and_healthy_disk_stay_quiet() {
        let mut store = Store::default();
        store.alerts.disk = true;
        store.alerts.battery = true;
        let mut s = ready();
        s.disks = vec![DiskInfo { mount: "/".into(), total: 100, avail: 50, ..Default::default() }];
        s.battery = Some(BatteryInfo { percent: 10.0, charging: true, ..Default::default() });
        let mut a = Alerts::default();
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 0);
    }

    #[test]
    fn uninstalled_feature_or_unready_monitor_never_alerts() {
        let mut store = Store::default();
        store.alerts.memory = true;
        let mut s = ready();
        s.mem_used = 99;
        store.installed.remove("monitor_alerts");
        let mut a = Alerts::default();
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 0);
        store.installed.insert("monitor_alerts".into());
        s.ready = false;
        a.evaluate(&s, &store);
        assert_eq!(a.sent, 0);
    }
}
