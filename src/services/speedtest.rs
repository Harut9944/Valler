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

#[cfg(test)]
mod feature_tests {
    use super::*;
    use std::io::Write;

    /// Serves a 2 MB body once on a local port, like a speed test server.
    fn local_server() -> String {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((mut s, _)) = l.accept() {
                let mut buf = [0u8; 1024];
                let _ = std::io::Read::read(&mut s, &mut buf);
                let body = vec![7u8; 2_000_000];
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                let _ = s.write_all(&body);
            }
        });
        format!("http://{addr}/")
    }

    #[test]
    fn measures_a_download() {
        let state: Shared = Default::default();
        start(state.clone(), local_server(), egui::Context::default());
        let begun = std::time::Instant::now();
        loop {
            match state.lock().unwrap().clone() {
                SpeedState::Done { mbps, latency_ms } => {
                    assert!(mbps > 0.0 && latency_ms >= 0.0);
                    break;
                }
                SpeedState::Failed(e) => panic!("speed test failed: {e}"),
                _ => {}
            }
            assert!(begun.elapsed().as_secs() < 20);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn reports_failures() {
        let state: Shared = Default::default();
        // Nothing listens on port 9 of localhost.
        start(state.clone(), "http://127.0.0.1:9/".into(), egui::Context::default());
        let begun = std::time::Instant::now();
        loop {
            if let SpeedState::Failed(e) = state.lock().unwrap().clone() {
                assert!(!e.is_empty());
                break;
            }
            assert!(begun.elapsed().as_secs() < 20);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}
