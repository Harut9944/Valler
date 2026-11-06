//! Unit conversion: length, mass, data, time, volume, speed and temperature.

use super::fmt_num;

// ------------------------------------------------------------- units

#[derive(Clone, Copy, PartialEq)]
enum Dim {
    Length,
    Mass,
    Data,
    Time,
    Volume,
    Speed,
    Temp,
}

/// (names, dimension, factor to the base unit)
const UNITS: &[(&[&str], Dim, f64)] = &[
    (&["mm", "millimeter", "millimeters"], Dim::Length, 0.001),
    (&["cm", "centimeter", "centimeters"], Dim::Length, 0.01),
    (&["m", "meter", "meters", "metre", "metres"], Dim::Length, 1.0),
    (&["km", "kilometer", "kilometers"], Dim::Length, 1000.0),
    (&["in", "inch", "inches", "\""], Dim::Length, 0.0254),
    (&["ft", "foot", "feet", "'"], Dim::Length, 0.3048),
    (&["yd", "yard", "yards"], Dim::Length, 0.9144),
    (&["mi", "mile", "miles"], Dim::Length, 1609.344),
    (&["mg", "milligram", "milligrams"], Dim::Mass, 0.001),
    (&["g", "gram", "grams"], Dim::Mass, 1.0),
    (&["kg", "kilogram", "kilograms", "kilo", "kilos"], Dim::Mass, 1000.0),
    (&["t", "tonne", "tonnes", "ton"], Dim::Mass, 1_000_000.0),
    (&["oz", "ounce", "ounces"], Dim::Mass, 28.349523125),
    (&["lb", "lbs", "pound", "pounds"], Dim::Mass, 453.59237),
    (&["b", "byte", "bytes"], Dim::Data, 1.0),
    (&["kb", "kilobyte", "kilobytes"], Dim::Data, 1e3),
    (&["mb", "megabyte", "megabytes"], Dim::Data, 1e6),
    (&["gb", "gigabyte", "gigabytes"], Dim::Data, 1e9),
    (&["tb", "terabyte", "terabytes"], Dim::Data, 1e12),
    (&["kib"], Dim::Data, 1024.0),
    (&["mib"], Dim::Data, 1048576.0),
    (&["gib"], Dim::Data, 1073741824.0),
    (&["ms", "millisecond", "milliseconds"], Dim::Time, 0.001),
    (&["s", "sec", "second", "seconds"], Dim::Time, 1.0),
    (&["min", "minute", "minutes"], Dim::Time, 60.0),
    (&["h", "hr", "hour", "hours"], Dim::Time, 3600.0),
    (&["d", "day", "days"], Dim::Time, 86400.0),
    (&["wk", "week", "weeks"], Dim::Time, 604800.0),
    (&["ml", "milliliter", "milliliters"], Dim::Volume, 0.001),
    (&["l", "liter", "liters", "litre", "litres"], Dim::Volume, 1.0),
    (&["gal", "gallon", "gallons"], Dim::Volume, 3.785411784),
    (&["floz"], Dim::Volume, 0.0295735295625),
    (&["cup", "cups"], Dim::Volume, 0.2365882365),
    (&["kmh", "km/h", "kph"], Dim::Speed, 1.0 / 3.6),
    (&["mph"], Dim::Speed, 0.44704),
    (&["m/s", "mps"], Dim::Speed, 1.0),
    (&["kn", "knot", "knots"], Dim::Speed, 0.514444),
    (&["c", "°c", "celsius"], Dim::Temp, 0.0),
    (&["f", "°f", "fahrenheit"], Dim::Temp, 1.0),
    (&["k", "kelvin"], Dim::Temp, 2.0),
];

fn unit(name: &str) -> Option<(Dim, f64, &'static str)> {
    let n = name.trim().to_lowercase();
    UNITS.iter().find(|(names, _, _)| names.contains(&n.as_str())).map(|(names, d, f)| (*d, *f, names[0]))
}

fn to_celsius(v: f64, code: f64) -> f64 {
    match code as i32 {
        1 => (v - 32.0) * 5.0 / 9.0,
        2 => v - 273.15,
        _ => v,
    }
}

fn from_celsius(c: f64, code: f64) -> f64 {
    match code as i32 {
        1 => c * 9.0 / 5.0 + 32.0,
        2 => c + 273.15,
        _ => c,
    }
}

/// Converts queries like `10 km to mi`, `72f in c`, `1.5 gb = mb`.
pub fn convert(input: &str) -> Option<String> {
    let s = input.trim().to_lowercase();
    let (lhs, rhs) = [" to ", " in ", " = ", " as "].iter().find_map(|sep| s.split_once(sep))?;
    let lhs = lhs.trim();
    let split = lhs.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == ','))?;
    let value: f64 = lhs[..split].replace(',', "").trim().parse().ok()?;
    let (d1, f1, n1) = unit(&lhs[split..])?;
    let (d2, f2, n2) = unit(rhs)?;
    if d1 != d2 {
        return None;
    }
    let out = if d1 == Dim::Temp { from_celsius(to_celsius(value, f1), f2) } else { value * f1 / f2 };
    Some(format!("{} {n1} = {} {n2}", fmt_num(value), fmt_num((out * 1e6).round() / 1e6)))
}
