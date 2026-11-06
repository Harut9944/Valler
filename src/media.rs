//! Media tools: batch-convert, resize and compress images locally.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Png,
    Jpeg,
    Webp,
}

impl Format {
    pub const ALL: [Format; 3] = [Format::Jpeg, Format::Png, Format::Webp];
    pub fn label(self) -> &'static str {
        match self {
            Format::Png => "PNG",
            Format::Jpeg => "JPEG",
            Format::Webp => "WebP (lossless)",
        }
    }
    fn ext(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Jpeg => "jpg",
            Format::Webp => "webp",
        }
    }
}

#[derive(Clone)]
pub struct Job {
    pub files: Vec<PathBuf>,
    pub format: Format,
    pub max_side: u32, // 0 = keep size
    pub quality: u8,
}

#[derive(Clone, Default)]
pub struct Progress {
    pub running: bool,
    pub done: usize,
    pub total: usize,
    pub saved_bytes: i64,
    pub errors: Vec<String>,
    pub outputs: Vec<PathBuf>,
}

pub type Shared = Arc<Mutex<Progress>>;

fn target_path(src: &Path, f: Format) -> PathBuf {
    let stem = src.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "image".into());
    let dir = src.parent().unwrap_or(Path::new("."));
    let mut p = dir.join(format!("{stem}.{}", f.ext()));
    let mut n = 1;
    while p.exists() {
        p = dir.join(format!("{stem} ({n}).{}", f.ext()));
        n += 1;
    }
    p
}

fn convert_one(src: &Path, job: &Job) -> Result<(PathBuf, i64), String> {
    let mut img = image::open(src).map_err(|e| e.to_string())?;
    if job.max_side > 0 && (img.width() > job.max_side || img.height() > job.max_side) {
        img = img.resize(job.max_side, job.max_side, image::imageops::FilterType::Lanczos3);
    }
    let out = target_path(src, job.format);
    let file = std::fs::File::create(&out).map_err(|e| e.to_string())?;
    let mut w = std::io::BufWriter::new(file);
    match job.format {
        Format::Jpeg => {
            let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut w, job.quality.clamp(10, 100));
            img.to_rgb8().write_with_encoder(enc).map_err(|e| e.to_string())?;
        }
        Format::Png => img.write_to(&mut w, image::ImageFormat::Png).map_err(|e| e.to_string())?,
        Format::Webp => img.to_rgba8().write_to(&mut w, image::ImageFormat::WebP).map_err(|e| e.to_string())?,
    }
    drop(w);
    let before = std::fs::metadata(src).map(|m| m.len() as i64).unwrap_or(0);
    let after = std::fs::metadata(&out).map(|m| m.len() as i64).unwrap_or(0);
    Ok((out, before - after))
}

pub fn start(state: Shared, ctx: egui::Context, job: Job) {
    if let Ok(mut s) = state.lock() {
        *s = Progress { running: true, total: job.files.len(), ..Default::default() };
    }
    std::thread::spawn(move || {
        for f in &job.files {
            let r = convert_one(f, &job);
            if let Ok(mut s) = state.lock() {
                s.done += 1;
                match r {
                    Ok((out, saved)) => {
                        s.saved_bytes += saved;
                        s.outputs.push(out);
                    }
                    Err(e) => s.errors.push(format!("{}: {e}", f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())),
                }
            }
            ctx.request_repaint();
        }
        if let Ok(mut s) = state.lock() {
            s.running = false;
        }
        ctx.request_repaint();
    });
}

pub const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff"];

pub fn pick_files() -> Vec<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Choose images to convert")
        .add_filter("Images", EXTENSIONS)
        .pick_files()
        .unwrap_or_default()
}

#[cfg(test)]
mod feature_tests {
    use super::*;
    use crate::test_util::TempDir;

    fn sample(t: &TempDir) -> PathBuf {
        let p = t.join("photo.png");
        let img = image::RgbImage::from_fn(400, 300, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
        img.save(&p).unwrap();
        p
    }

    #[test]
    fn converts_to_jpeg_and_shrinks_large_images() {
        let t = TempDir::new("media-jpg");
        let src = sample(&t);
        let job = Job { files: vec![src.clone()], format: Format::Jpeg, max_side: 100, quality: 70 };
        let (out, _saved) = convert_one(&src, &job).unwrap();
        assert_eq!(out.extension().unwrap(), "jpg");
        let img = image::open(&out).unwrap();
        assert_eq!((img.width(), img.height()), (100, 75));
        assert!(src.exists(), "the original is kept");
    }

    #[test]
    fn converts_to_png_and_webp_keeping_size() {
        let t = TempDir::new("media-png");
        let src = sample(&t);
        for f in [Format::Webp, Format::Png] {
            let job = Job { files: vec![src.clone()], format: f, max_side: 0, quality: 90 };
            let (out, _) = convert_one(&src, &job).unwrap();
            let img = image::open(&out).unwrap();
            assert_eq!((img.width(), img.height()), (400, 300));
        }
        // Converting PNG to PNG next to the original gets a new name.
        assert!(t.join("photo (1).png").exists());
    }

    #[test]
    fn reports_files_that_are_not_images() {
        let t = TempDir::new("media-bad");
        let bad = t.join("notes.png");
        std::fs::write(&bad, "not an image").unwrap();
        let job = Job { files: vec![bad.clone()], format: Format::Jpeg, max_side: 0, quality: 80 };
        assert!(convert_one(&bad, &job).is_err());
    }

    #[test]
    fn batch_job_reports_progress() {
        let t = TempDir::new("media-batch");
        let src = sample(&t);
        let state: Shared = Default::default();
        start(state.clone(), egui::Context::default(), Job { files: vec![src, t.join("missing.png")], format: Format::Jpeg, max_side: 50, quality: 80 });
        let begun = std::time::Instant::now();
        while state.lock().unwrap().running {
            assert!(begun.elapsed().as_secs() < 20);
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let p = state.lock().unwrap().clone();
        assert_eq!((p.done, p.total, p.outputs.len(), p.errors.len()), (2, 2, 1, 1));
    }
}
