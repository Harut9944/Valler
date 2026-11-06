//! Download speed test: only runs when the person presses the button.

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Default)]
pub enum SpeedState {
    #[default]
    Idle,
    Running { bytes: u64, secs: f64 },
    Done { mbps: f64, latency_ms: f64 },
    Failed(String),
}

pub type Shared = Arc<Mutex<SpeedState>>;

pub fn start(state: Shared, url: String, ctx: egui::Context) {
    if let Ok(mut s) = state.lock() {
        *s = SpeedState::Running { bytes: 0, secs: 0.0 };
    }
    std::thread::spawn(move || {
        let t0 = Instant::now();
        let result = (|| -> Result<(f64, f64), String> {
            let resp = ureq::get(&url).call().map_err(|e| e.to_string())?;
            let latency = t0.elapsed().as_secs_f64() * 1000.0;
            let mut body = resp.into_body();
            let mut reader = body.as_reader();
            let mut buf = vec![0u8; 64 * 1024];
            let mut total = 0u64;
            let start = Instant::now();
            loop {
                let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                total += n as u64;
                let secs = start.elapsed().as_secs_f64();
                if let Ok(mut s) = state.lock() {
                    *s = SpeedState::Running { bytes: total, secs };
                }
                ctx.request_repaint();
                if secs > 15.0 {
                    break;
                }
            }
            let secs = start.elapsed().as_secs_f64().max(0.001);
            Ok((total as f64 * 8.0 / secs / 1_000_000.0, latency))
        })();
        if let Ok(mut s) = state.lock() {
            *s = match result {
                Ok((mbps, latency_ms)) => SpeedState::Done { mbps, latency_ms },
                Err(e) => SpeedState::Failed(e),
            };
        }
        ctx.request_repaint();
    });
}
