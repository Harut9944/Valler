//! Background sampler: reads processor, memory, temperatures, GPU, network,
//! disks, processes and battery once a second on its own thread, so the UI
//! never waits on the operating system.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{Components, Disks, Gpus, Networks, ProcessesToUpdate, System};

pub const HISTORY: usize = 90;

#[derive(Clone, Default)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
}

#[derive(Clone, Default)]
pub struct Iface {
    pub name: String,
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub total_rx: u64,
    pub total_tx: u64,
    pub ips: Vec<String>,
}

#[derive(Clone, Default)]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub fs: String,
    pub kind: String,
    pub total: u64,
    pub avail: u64,
    pub removable: bool,
}

#[derive(Clone, Default)]
pub struct GpuInfo {
    pub name: String,
    pub usage: Option<f32>,
    pub mem_used: Option<u64>,
    pub mem_total: Option<u64>,
}

#[derive(Clone, Default)]
pub struct BatteryInfo {
    pub percent: f32,
    pub state: String,
    pub charging: bool,
    pub time_to_empty: Option<u64>,
    pub time_to_full: Option<u64>,
    pub health: f32,
    pub cycles: Option<u32>,
    pub temp_c: Option<f32>,
    pub rate_w: f32,
    pub voltage: f32,
    pub energy_wh: f32,
    pub full_wh: f32,
    pub design_wh: f32,
    pub vendor: String,
    pub model: String,
    pub technology: String,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub ready: bool,
    pub cpu_brand: String,
    pub cpu_total: f32,
    pub per_core: Vec<f32>,
    pub physical_cores: usize,
    pub cpu_hist: VecDeque<f32>,
    pub load: (f64, f64, f64),
    pub mem_used: u64,
    pub mem_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub mem_hist: VecDeque<f32>,
    pub uptime: u64,
    pub host: String,
    pub os: String,
    pub arch: String,
    pub temps: Vec<(String, f32)>,
    pub cpu_temp: Option<f32>,
    pub gpu_temp: Option<f32>,
    pub gpus: Vec<GpuInfo>,
    pub gpu_hist: VecDeque<f32>,
    pub ifaces: Vec<Iface>,
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub rx_hist: VecDeque<f32>,
    pub tx_hist: VecDeque<f32>,
    pub session_rx: u64,
    pub session_tx: u64,
    pub local_ip: Option<String>,
    pub disks: Vec<DiskInfo>,
    pub disk_read_rate: f64,
    pub disk_write_rate: f64,
    pub disk_read_hist: VecDeque<f32>,
    pub disk_write_hist: VecDeque<f32>,
    pub procs: Vec<ProcInfo>,
    pub battery: Option<BatteryInfo>,
    pub battery_hist: VecDeque<f32>,
    pub power_hist: VecDeque<f32>,
}

impl Snapshot {
    pub fn mem_frac(&self) -> f32 {
        if self.mem_total == 0 {
            0.0
        } else {
            self.mem_used as f32 / self.mem_total as f32
        }
    }
    /// A simple, cross-platform stand-in for macOS memory pressure.
    pub fn pressure(&self) -> (&'static str, egui::Color32) {
        let f = self.mem_frac();
        let swap = if self.swap_total > 0 { self.swap_used as f32 / self.swap_total as f32 } else { 0.0 };
        if f > 0.92 || (f > 0.85 && swap > 0.5) {
            ("Critical", crate::ui::RED)
        } else if f > 0.8 {
            ("Elevated", crate::ui::YELLOW)
        } else {
            ("Normal", crate::ui::GREEN)
        }
    }
}

pub type Shared = Arc<Mutex<Snapshot>>;

fn push(h: &mut VecDeque<f32>, v: f32) {
    h.push_back(v);
    while h.len() > HISTORY {
        h.pop_front();
    }
}

pub fn spawn(ctx: egui::Context) -> Shared {
    let shared: Shared = Arc::new(Mutex::new(Snapshot::default()));
    let out = shared.clone();
    std::thread::Builder::new()
        .name("valler-monitor".into())
        .spawn(move || run(out, ctx))
        .ok();
    shared
}

fn run(out: Shared, ctx: egui::Context) {
    let mut sys = match System::new() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut networks = Networks::new_with_refreshed_list().ok();
    let mut disks = Disks::new_with_refreshed_list().ok();
    let mut components = Components::new_with_refreshed_list().ok();
    let mut gpus = Gpus::new_with_refreshed_list().ok();
    let battery_mgr = starship_battery::Manager::new().ok();

    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let cpu_brand = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default();
    let physical = System::physical_core_count().unwrap_or(0);
    let host = System::host_name().unwrap_or_default();
    let os = System::long_os_version().unwrap_or_else(|_| std::env::consts::OS.to_string());
    let arch = System::cpu_arch();

    let mut last = Instant::now();
    let mut tick: u64 = 0;
    let mut session_rx = 0u64;
    let mut session_tx = 0u64;
    std::thread::sleep(Duration::from_millis(400));

    loop {
        let dt = last.elapsed().as_secs_f64().max(0.2);
        last = Instant::now();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        if tick % 2 == 0 {
            sys.refresh_processes(ProcessesToUpdate::All, true);
        }
        if let Some(n) = networks.as_mut() {
            n.refresh(true);
        }
        if tick % 5 == 0 {
            if let Some(d) = disks.as_mut() {
                d.refresh(true);
            }
        }
        if let Some(c) = components.as_mut() {
            c.refresh(false);
        }
        if let Some(g) = gpus.as_mut() {
            g.refresh(false);
        }

        let mut s = out.lock().map(|g| g.clone()).unwrap_or_default();
        s.ready = tick > 0;
        s.cpu_brand = cpu_brand.clone();
        s.physical_cores = physical;
        s.host = host.clone();
        s.os = os.clone();
        s.arch = arch.clone();
        s.cpu_total = sys.global_cpu_usage();
        s.per_core = sys.cpus().iter().map(|c| c.usage()).collect();
        push(&mut s.cpu_hist, s.cpu_total);
        s.load = System::load_average().map(|l| (l.one, l.five, l.fifteen)).unwrap_or_default();
        s.mem_total = sys.total_memory();
        s.mem_used = sys.used_memory();
        s.swap_total = sys.total_swap();
        s.swap_used = sys.used_swap();
        let mf = s.mem_frac();
        push(&mut s.mem_hist, mf);
        s.uptime = System::uptime().unwrap_or(0);

        // Temperatures
        if let Some(c) = components.as_ref() {
            let mut temps: Vec<(String, f32)> = c
                .iter()
                .filter_map(|c| c.temperature().filter(|t| t.is_finite() && *t > 0.0).map(|t| (c.label().to_string(), t)))
                .collect();
            temps.sort_by(|a, b| a.0.cmp(&b.0));
            let pick = |keys: &[&str]| -> Option<f32> {
                temps
                    .iter()
                    .filter(|(l, _)| {
                        let l = l.to_lowercase();
                        keys.iter().any(|k| l.contains(k))
                    })
                    .map(|(_, t)| *t)
                    .fold(None, |acc: Option<f32>, t| Some(acc.map_or(t, |a| a.max(t))))
            };
            s.cpu_temp = pick(&["package", "tctl", "tdie", "cpu", "k10temp", "coretemp", "core "]);
            s.gpu_temp = pick(&["gpu", "amdgpu", "nouveau", "radeon", "edge", "junction"]);
            s.temps = temps;
        }

        // GPU
        if let Some(g) = gpus.as_ref() {
            s.gpus = g
                .iter()
                .map(|g| GpuInfo {
                    name: g.model().or(g.vendor()).unwrap_or("Graphics").to_string(),
                    usage: g.usage(),
                    mem_used: g.used_memory(),
                    mem_total: g.total_memory(),
                })
                .collect();
            let max_usage = s.gpus.iter().filter_map(|g| g.usage).fold(-1.0f32, f32::max);
            if max_usage >= 0.0 {
                push(&mut s.gpu_hist, max_usage);
            }
        }

        // Network
        if let Some(n) = networks.as_ref() {
            let mut ifaces: Vec<Iface> = n
                .iter()
                .map(|(name, d)| Iface {
                    name: name.to_string_lossy().to_string(),
                    rx_rate: d.received() as f64 / dt,
                    tx_rate: d.transmitted() as f64 / dt,
                    total_rx: d.total_received(),
                    total_tx: d.total_transmitted(),
                    ips: d.ip_networks().iter().map(|ip| ip.addr.to_string()).collect(),
                })
                .collect();
            ifaces.sort_by(|a, b| (b.total_rx + b.total_tx).cmp(&(a.total_rx + a.total_tx)));
            let real = |i: &Iface| !(i.name == "lo" || i.name.starts_with("lo") && i.name.len() <= 3 || i.name.to_lowercase().contains("loopback"));
            let rx: f64 = ifaces.iter().filter(|i| real(i)).map(|i| i.rx_rate).sum();
            let tx: f64 = ifaces.iter().filter(|i| real(i)).map(|i| i.tx_rate).sum();
            if tick > 0 {
                session_rx += (rx * dt) as u64;
                session_tx += (tx * dt) as u64;
            }
            s.rx_rate = rx;
            s.tx_rate = tx;
            s.session_rx = session_rx;
            s.session_tx = session_tx;
            push(&mut s.rx_hist, rx as f32);
            push(&mut s.tx_hist, tx as f32);
            s.ifaces = ifaces.into_iter().filter(|i| real(i)).collect();
        }
        if tick % 10 == 0 {
            s.local_ip = local_ip_address::local_ip().ok().map(|ip| ip.to_string());
        }

        // Disks
        if let Some(d) = disks.as_ref() {
            let mut seen = std::collections::HashSet::new();
            s.disks = d
                .iter()
                .filter(|d| d.total_space() > 0)
                .filter(|d| {
                    let fs = d.file_system().to_string_lossy().to_lowercase();
                    !matches!(fs.as_str(), "squashfs" | "tmpfs" | "devtmpfs" | "overlay" | "efivarfs")
                })
                .filter(|d| {
                    let m = d.mount_point().to_string_lossy().to_string();
                    let fs = d.file_system().to_string_lossy().to_lowercase();
                    let hidden_prefix = ["/tmp/", "/snap/", "/run/", "/var/lib/", "/proc", "/sys", "/dev", "/System/Volumes/", "/private/var/"];
                    !(hidden_prefix.iter().any(|p| m.starts_with(p)) || fs.contains("appimage") || fs == "fuse.portal" || fs.contains("gvfsd"))
                })
                .filter(|d| seen.insert(d.name().to_string_lossy().to_string() + &d.total_space().to_string()))
                .map(|d| DiskInfo {
                    name: d.name().to_string_lossy().to_string(),
                    mount: d.mount_point().to_string_lossy().to_string(),
                    fs: d.file_system().to_string_lossy().to_string(),
                    kind: format!("{:?}", d.kind()),
                    total: d.total_space(),
                    avail: d.available_space(),
                    removable: d.is_removable(),
                })
                .collect();
        }

        // Processes (and disk activity from their I/O)
        if tick % 2 == 0 {
            let mut read = 0u64;
            let mut write = 0u64;
            let mut by_pid: HashMap<u32, ProcInfo> = HashMap::new();
            for (pid, p) in sys.processes() {
                let du = p.disk_usage();
                read += du.read_bytes;
                write += du.written_bytes;
                // Skip kernel threads / tasks without a name.
                if p.thread_kind().is_some() {
                    continue;
                }
                let name = p.name().to_string_lossy().to_string();
                if name.is_empty() {
                    continue;
                }
                by_pid.insert(
                    pid.as_u32(),
                    ProcInfo {
                        pid: pid.as_u32(),
                        name,
                        cpu: p.cpu_usage(),
                        mem: p.memory(),
                    },
                );
            }
            let span = dt * 2.0;
            s.disk_read_rate = read as f64 / span;
            s.disk_write_rate = write as f64 / span;
            let mut procs: Vec<ProcInfo> = by_pid.into_values().collect();
            procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
            s.procs = procs;
        }
        push(&mut s.disk_read_hist, s.disk_read_rate as f32);
        push(&mut s.disk_write_hist, s.disk_write_rate as f32);

        // Battery
        if let Some(m) = battery_mgr.as_ref() {
            s.battery = read_battery(m);
            if let Some(b) = &s.battery {
                let (pct, w) = (b.percent, b.rate_w);
                push(&mut s.battery_hist, pct);
                push(&mut s.power_hist, w);
            }
        }

        if let Ok(mut g) = out.lock() {
            *g = s;
        }
        ctx.request_repaint();
        tick += 1;
        let spent = last.elapsed();
        std::thread::sleep(Duration::from_millis(1000).saturating_sub(spent));
    }
}

fn read_battery(m: &starship_battery::Manager) -> Option<BatteryInfo> {
    use starship_battery::State;
    use starship_battery::units::electric_potential::volt;
    use starship_battery::units::energy::watt_hour;
    use starship_battery::units::power::watt;
    use starship_battery::units::ratio::percent;
    use starship_battery::units::thermodynamic_temperature::degree_celsius;
    use starship_battery::units::time::second;

    let b = m.batteries().ok()?.flatten().next()?;
    let state = b.state();
    Some(BatteryInfo {
        percent: b.state_of_charge().get::<percent>(),
        state: match state {
            State::Charging => "Charging",
            State::Discharging => "On battery",
            State::Full => "Fully charged",
            State::Empty => "Empty",
            _ => "Plugged in",
        }
        .to_string(),
        charging: matches!(state, State::Charging),
        time_to_empty: b.time_to_empty().map(|t| t.get::<second>() as u64),
        time_to_full: b.time_to_full().map(|t| t.get::<second>() as u64),
        health: b.state_of_health().get::<percent>(),
        cycles: b.cycle_count(),
        temp_c: b.temperature().map(|t| t.get::<degree_celsius>()),
        rate_w: b.energy_rate().get::<watt>().abs(),
        voltage: b.voltage().get::<volt>(),
        energy_wh: b.energy().get::<watt_hour>(),
        full_wh: b.energy_full().get::<watt_hour>(),
        design_wh: b.energy_full_design().get::<watt_hour>(),
        vendor: b.vendor().unwrap_or("").trim().to_string(),
        model: b.model().unwrap_or("").trim().to_string(),
        technology: format!("{:?}", b.technology()),
    })
}
