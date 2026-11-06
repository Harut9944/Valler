//! Clipboard watcher: polls the system clipboard on a background thread and
//! reports new copies — text, files copied in a file manager, and images.
//! Also cleans links automatically and clears the clipboard after a delay
//! when those switches are on. History rules live in `history`, image
//! storage in `images`.

pub mod history;
pub mod images;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
pub struct WatchConfig {
    pub watch: bool,
    /// Also keep copied images (saved as PNG files).
    pub images: bool,
    pub auto_clean_urls: bool,
    pub extra_params: String,
    pub auto_clear_secs: u32,
}

/// A new copy seen on the system clipboard.
#[derive(Clone, Debug, PartialEq)]
pub enum Clip {
    Text(String),
    Files(Vec<PathBuf>),
    Image { path: PathBuf, width: usize, height: usize },
}

impl Clip {
    pub fn into_entry(self, at: i64) -> history::ClipEntry {
        match self {
            Clip::Text(t) => history::ClipEntry::text(t, at),
            Clip::Files(f) => history::ClipEntry::files(f, at),
            Clip::Image { path, width, height } => history::ClipEntry::image(path, width, height, at),
        }
    }
}

pub struct Watcher {
    pub rx: Receiver<Clip>,
    pub config: Arc<Mutex<WatchConfig>>,
    cmd: Sender<Cmd>,
}

enum Cmd {
    Set(String),
    SetFiles(Vec<PathBuf>),
    SetImage(PathBuf),
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

    /// Put files on the clipboard, ready to paste in a file manager.
    pub fn copy_files(&self, files: Vec<PathBuf>) {
        let _ = self.cmd.send(Cmd::SetFiles(files));
    }

    /// Put a saved image back on the clipboard.
    pub fn copy_image(&self, path: PathBuf) {
        let _ = self.cmd.send(Cmd::SetImage(path));
    }

    pub fn clear(&self) {
        let _ = self.cmd.send(Cmd::Clear);
    }
}

/// What was last seen, so the same copy isn't reported twice.
#[derive(Default)]
struct Last {
    text: Option<String>,
    files: Option<Vec<PathBuf>>,
    image: Option<u64>,
}

const IMAGE_EVERY: Duration = Duration::from_secs(2);

fn run(tx: Sender<Clip>, cmd_rx: Receiver<Cmd>, cfg: Arc<Mutex<WatchConfig>>, ctx: egui::Context) {
    let mut cb = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(_) => return,
    };
    // Whatever is on the clipboard at start-up is not a new copy.
    let mut last = Last {
        text: cb.get_text().ok(),
        files: cb.get().file_list().ok(),
        image: cb.get_image().ok().map(|i| images::signature(i.width, i.height, &i.bytes)),
    };
    let mut copied_at: Option<Instant> = None;
    let mut image_checked = Instant::now();
    loop {
        while let Ok(c) = cmd_rx.try_recv() {
            match c {
                Cmd::Set(t) => {
                    let _ = cb.set_text(t.clone());
                    last.text = Some(t);
                }
                Cmd::SetFiles(f) => {
                    let _ = cb.set().file_list(&f);
                    last.files = Some(f);
                    last.text = cb.get_text().ok();
                }
                Cmd::SetImage(p) => {
                    if let Ok((w, h, px)) = images::load_rgba(&p) {
                        last.image = Some(images::signature(w, h, &px));
                        let _ = cb.set_image(arboard::ImageData { width: w, height: h, bytes: px.into() });
                    }
                }
                Cmd::Clear => {
                    let _ = cb.clear();
                    last = Last::default();
                    copied_at = None;
                    continue;
                }
            }
            copied_at = Some(Instant::now());
        }
        let conf = cfg.lock().map(|c| c.clone()).unwrap_or_default();
        if conf.watch || conf.auto_clean_urls || conf.auto_clear_secs > 0 {
            if let Some(clip) = poll(&mut cb, &mut last, &conf, &mut image_checked) {
                copied_at = Some(Instant::now());
                if conf.watch {
                    let _ = tx.send(clip);
                    ctx.request_repaint();
                }
            }
            if conf.auto_clear_secs > 0
                && let Some(at) = copied_at
                && at.elapsed() >= Duration::from_secs(conf.auto_clear_secs as u64)
            {
                let _ = cb.clear();
                last = Last::default();
                copied_at = None;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// One look at the clipboard. Files win over text (file managers also offer
/// the paths as text); images are read only when there is no text, and at
/// most every two seconds, because reading them is expensive.
fn poll(cb: &mut arboard::Clipboard, last: &mut Last, conf: &WatchConfig, image_checked: &mut Instant) -> Option<Clip> {
    if let Ok(files) = cb.get().file_list()
        && !files.is_empty()
    {
        if last.files.as_ref() == Some(&files) {
            return None;
        }
        last.files = Some(files.clone());
        last.text = cb.get_text().ok();
        return Some(Clip::Files(files));
    }
    last.files = None;
    if let Ok(text) = cb.get_text() {
        if Some(&text) == last.text.as_ref() || text.trim().is_empty() {
            return None;
        }
        let mut text = text;
        if conf.auto_clean_urls
            && let Some(r) = crate::services::urlclean::clean(&text, &conf.extra_params)
            && !r.removed.is_empty()
        {
            let _ = cb.set_text(r.url.clone());
            text = r.url;
        }
        last.text = Some(text.clone());
        return Some(Clip::Text(text));
    }
    if !conf.images || !conf.watch || image_checked.elapsed() < IMAGE_EVERY {
        return None;
    }
    *image_checked = Instant::now();
    let img = cb.get_image().ok()?;
    let sig = images::signature(img.width, img.height, &img.bytes);
    if last.image == Some(sig) {
        return None;
    }
    last.image = Some(sig);
    let path = images::save_png(&images::dir(), img.width, img.height, &img.bytes).ok()?;
    Some(Clip::Image { path, width: img.width, height: img.height })
}
