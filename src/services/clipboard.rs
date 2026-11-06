//! Clipboard watcher: polls the system clipboard twice a second on a
//! background thread and reports new text. Also cleans links automatically
//! and clears the clipboard after a delay when those switches are on.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
pub struct WatchConfig {
    pub watch: bool,
    pub auto_clean_urls: bool,
    pub extra_params: String,
    pub auto_clear_secs: u32,
}

pub struct Watcher {
    pub rx: Receiver<String>,
    pub config: Arc<Mutex<WatchConfig>>,
    cmd: Sender<Cmd>,
}

enum Cmd {
    Set(String),
    Clear,
}

impl Watcher {
    pub fn spawn(ctx: egui::Context, cfg: WatchConfig) -> Self {
        let (tx, rx) = channel();
        let (cmd_tx, cmd_rx) = channel::<Cmd>();
        let config = Arc::new(Mutex::new(cfg));
        let c2 = config.clone();
        std::thread::Builder::new()
            .name("valler-clipboard".into())
            .spawn(move || run(tx, cmd_rx, c2, ctx))
            .ok();
        Self { rx, config, cmd: cmd_tx }
    }

    /// Put text on the system clipboard (it will not be re-added as "new").
    pub fn copy(&self, text: String) {
        let _ = self.cmd.send(Cmd::Set(text));
    }

    pub fn clear(&self) {
        let _ = self.cmd.send(Cmd::Clear);
    }
}

fn run(tx: Sender<String>, cmd_rx: Receiver<Cmd>, cfg: Arc<Mutex<WatchConfig>>, ctx: egui::Context) {
    let mut cb = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut last: Option<String> = cb.get_text().ok();
    let mut copied_at: Option<Instant> = None;
    loop {
        while let Ok(c) = cmd_rx.try_recv() {
            match c {
                Cmd::Set(t) => {
                    let _ = cb.set_text(t.clone());
                    last = Some(t);
                    copied_at = Some(Instant::now());
                }
                Cmd::Clear => {
                    let _ = cb.clear();
                    last = None;
                    copied_at = None;
                }
            }
        }
        let conf = cfg.lock().map(|c| c.clone()).unwrap_or_default();
        if conf.watch || conf.auto_clean_urls || conf.auto_clear_secs > 0 {
            if let Ok(text) = cb.get_text()
                && Some(&text) != last.as_ref() && !text.trim().is_empty() {
                    let mut text = text;
                    if conf.auto_clean_urls
                        && let Some(r) = crate::services::urlclean::clean(&text, &conf.extra_params)
                            && !r.removed.is_empty() {
                                let _ = cb.set_text(r.url.clone());
                                text = r.url;
                            }
                    last = Some(text.clone());
                    copied_at = Some(Instant::now());
                    if conf.watch {
                        let _ = tx.send(text);
                        ctx.request_repaint();
                    }
                }
            if conf.auto_clear_secs > 0
                && let Some(at) = copied_at
                    && at.elapsed() >= Duration::from_secs(conf.auto_clear_secs as u64) {
                        let _ = cb.clear();
                        last = None;
                        copied_at = None;
                    }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
