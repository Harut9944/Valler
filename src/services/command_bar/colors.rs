//! Colour answers: type `#ff8800`, `#f80`, `rgb(255, 136, 0)` or
//! `hsl(32, 100%, 50%)` and get the same colour in every notation.

#[derive(Clone, Debug, PartialEq)]
pub struct ColorAnswer {
    pub rgb: [u8; 3],
    pub hex: String,
    pub rgb_text: String,
    pub hsl_text: String,
}

pub fn color(input: &str) -> Option<ColorAnswer> {
    let s = input.trim().to_lowercase();
    let rgb = parse_hex(&s).or_else(|| parse_rgb(&s)).or_else(|| parse_hsl(&s))?;
    let (h, sat, l) = rgb_to_hsl(rgb);
    Some(ColorAnswer {
        rgb,
        hex: format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
        rgb_text: format!("rgb({}, {}, {})", rgb[0], rgb[1], rgb[2]),
        hsl_text: format!("hsl({:.0}, {:.0}%, {:.0}%)", h, sat * 100.0, l * 100.0),
    })
}

fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let h = s.strip_prefix('#')?;
    if !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let full = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => h.to_string(),
        _ => return None,
    };
    let v = u32::from_str_radix(&full, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

/// The numbers inside `name(...)` or after `name `.
fn args<'a>(s: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let rest = s.strip_prefix(name)?.trim();
    let inner = rest.strip_prefix('(').and_then(|r| r.strip_suffix(')')).unwrap_or(rest);
    let parts: Vec<&str> = inner.split([',', ' ']).map(str::trim).filter(|p| !p.is_empty()).collect();
    (parts.len() == 3).then_some(parts)
}

fn parse_rgb(s: &str) -> Option<[u8; 3]> {
    let p = args(s, "rgb")?;
    let mut out = [0u8; 3];
    for (i, v) in p.iter().enumerate() {
        out[i] = v.parse::<u8>().ok()?;
    }
    Some(out)
}

fn parse_hsl(s: &str) -> Option<[u8; 3]> {
    let p = args(s, "hsl")?;
    let h: f32 = p[0].trim_end_matches("deg").parse().ok()?;
    let sat: f32 = p[1].trim_end_matches('%').parse().ok()?;
    let l: f32 = p[2].trim_end_matches('%').parse().ok()?;
    if !(0.0..=100.0).contains(&sat) || !(0.0..=100.0).contains(&l) {
        return None;
    }
    Some(hsl_to_rgb(h.rem_euclid(360.0), sat / 100.0, l / 100.0))
}

fn rgb_to_hsl([r, g, b]: [u8; 3]) -> (f32, f32, f32) {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    [to(r), to(g), to(b)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_every_notation() {
        let a = color("#FF8800").unwrap();
        assert_eq!(a.rgb, [255, 136, 0]);
        assert_eq!(a.rgb_text, "rgb(255, 136, 0)");
        assert_eq!(a.hsl_text, "hsl(32, 100%, 50%)");
        assert_eq!(color("#f80").unwrap().hex, "#FF8800");
        assert_eq!(color("rgb(255, 136, 0)").unwrap().hex, "#FF8800");
        assert_eq!(color("rgb 0 0 255").unwrap().hex, "#0000FF");
        assert_eq!(color("hsl(120, 100%, 25%)").unwrap().hex, "#008000");
    }

    #[test]
    fn greys_and_round_trips() {
        assert_eq!(color("#808080").unwrap().hsl_text, "hsl(0, 0%, 50%)");
        for hex in ["#123456", "#ABCDEF", "#00FF7F", "#FFFFFF", "#000000"] {
            let a = color(hex).unwrap();
            let back = color(&a.rgb_text).unwrap();
            assert_eq!(back.hex, hex);
        }
    }

    #[test]
    fn rejects_non_colours() {
        for bad in ["#12", "#ggg", "rgb(300, 0, 0)", "rgb(1,2)", "hsl(10, 200%, 50%)", "orange", ""] {
            assert!(color(bad).is_none(), "{bad}");
        }
    }
}
