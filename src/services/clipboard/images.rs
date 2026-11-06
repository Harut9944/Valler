//! Copied images are kept as PNG files in Valler's own settings folder (never
//! inside the settings file), named after their content so the same picture
//! is stored once.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub fn dir() -> PathBuf {
    crate::core::store::config_dir().join("clipboard")
}

/// A cheap fingerprint: size plus a sample of the pixels, enough to tell
/// whether the clipboard image changed without hashing megabytes each poll.
pub fn signature(width: usize, height: usize, rgba: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (width, height, rgba.len()).hash(&mut h);
    let step = (rgba.len() / 4096).max(1);
    for i in (0..rgba.len()).step_by(step) {
        rgba[i].hash(&mut h);
    }
    h.finish()
}

pub fn save_png(dir: &Path, width: usize, height: usize, rgba: &[u8]) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{:016x}.png", signature(width, height, rgba)));
    if !path.exists() {
        image::save_buffer(&path, rgba, width as u32, height as u32, image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

pub fn load_rgba(path: &Path) -> Result<(usize, usize, Vec<u8>), String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
    Ok((img.width() as usize, img.height() as usize, img.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    fn sample(w: usize, h: usize, v: u8) -> Vec<u8> {
        (0..w * h * 4).map(|i| (i as u8).wrapping_mul(v)).collect()
    }

    #[test]
    fn saves_and_loads_images() {
        let t = TempDir::new("clip-img");
        let px = sample(20, 10, 3);
        let p = save_png(&t.0, 20, 10, &px).unwrap();
        assert!(p.exists());
        assert_eq!(load_rgba(&p).unwrap(), (20, 10, px.clone()));
        // Same picture, same file.
        assert_eq!(save_png(&t.0, 20, 10, &px).unwrap(), p);
        assert_ne!(save_png(&t.0, 20, 10, &sample(20, 10, 5)).unwrap(), p);
    }

    #[test]
    fn signature_tracks_size_and_content() {
        let a = sample(8, 8, 1);
        assert_eq!(signature(8, 8, &a), signature(8, 8, &a));
        assert_ne!(signature(8, 8, &a), signature(4, 16, &a));
        assert_ne!(signature(8, 8, &a), signature(8, 8, &sample(8, 8, 2)));
    }
}
