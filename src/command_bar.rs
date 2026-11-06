//! Command bar engine: a calculator, a unit converter and a fuzzy search over
//! Valler's tools, clipboard history, snippets and notes.

// ------------------------------------------------------------- calculator

/// Evaluates arithmetic like `2*(3+4)^2 / 7`, `sqrt(2)`, `15% of 80`, `pi*2`.
pub fn calc(input: &str) -> Option<f64> {
    let s = input.trim().trim_end_matches('=').trim().to_lowercase();
    if s.is_empty() || !s.chars().any(|c| c.is_ascii_digit() || s.contains("pi")) {
        return None;
    }
    // "15% of 80"
    if let Some((a, b)) = s.split_once("% of ") {
        return Some(eval(a)? / 100.0 * eval(b)?);
    }
    // A bare number is not a calculation worth showing.
    if s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return None;
    }
    eval(&s)
}

fn eval(s: &str) -> Option<f64> {
    let s = s.replace('×', "*").replace('÷', "/").replace(',', "");
    let mut p = Parser { chars: s.chars().filter(|c| !c.is_whitespace()).collect(), pos: 0 };
    let v = p.expr()?;
    (p.pos == p.chars.len() && v.is_finite()).then_some(v)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Option<f64> {
        let mut v = self.term()?;
        loop {
            if self.eat('+') {
                v += self.term()?;
            } else if self.eat('-') {
                v -= self.term()?;
            } else {
                return Some(v);
            }
        }
    }
    fn term(&mut self) -> Option<f64> {
        let mut v = self.power()?;
        loop {
            if self.eat('*') {
                v *= self.power()?;
            } else if self.eat('/') {
                v /= self.power()?;
            } else if self.eat('%') {
                // modulo when followed by an operand, else percent
                if matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '(') {
                    v %= self.power()?;
                } else {
                    v /= 100.0;
                }
            } else {
                return Some(v);
            }
        }
    }
    fn power(&mut self) -> Option<f64> {
        let base = self.unary()?;
        if self.eat('^') {
            let e = self.power()?; // right associative
            return Some(base.powf(e));
        }
        Some(base)
    }
    fn unary(&mut self) -> Option<f64> {
        if self.eat('-') {
            return Some(-self.unary()?);
        }
        if self.eat('+') {
            return self.unary();
        }
        self.atom()
    }
    fn atom(&mut self) -> Option<f64> {
        if self.eat('(') {
            let v = self.expr()?;
            return self.eat(')').then_some(v);
        }
        let start = self.pos;
        if matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '.') {
            while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '.') {
                self.pos += 1;
            }
            let n: String = self.chars[start..self.pos].iter().collect();
            return n.parse().ok();
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let name: String = self.chars[start..self.pos].iter().collect();
        match name.as_str() {
            "pi" => Some(std::f64::consts::PI),
            "e" => Some(std::f64::consts::E),
            "" => None,
            f => {
                if !self.eat('(') {
                    return None;
                }
                let a = self.expr()?;
                if !self.eat(')') {
                    return None;
                }
                Some(match f {
                    "sqrt" => a.sqrt(),
                    "sin" => a.sin(),
                    "cos" => a.cos(),
                    "tan" => a.tan(),
                    "ln" => a.ln(),
                    "log" => a.log10(),
                    "abs" => a.abs(),
                    "round" => a.round(),
                    "floor" => a.floor(),
                    "ceil" => a.ceil(),
                    _ => return None,
                })
            }
        }
    }
}

pub fn fmt_num(v: f64) -> String {
    if v.fract().abs() < 1e-9 && v.abs() < 1e15 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

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

// ------------------------------------------------------------- matching

/// Simple subsequence score: higher is better, None if not all letters appear in order.
pub fn score(query: &str, text: &str) -> Option<i32> {
    let q = query.to_lowercase();
    let t = text.to_lowercase();
    if q.is_empty() {
        return Some(0);
    }
    if let Some(i) = t.find(&q) {
        return Some(1000 - i as i32 * 2 - (t.len() as i32 - q.len() as i32).min(200));
    }
    let mut ti = t.chars();
    let mut gaps = 0;
    for qc in q.chars() {
        let mut found = false;
        for tc in ti.by_ref() {
            if tc == qc {
                found = true;
                break;
            }
            gaps += 1;
        }
        if !found {
            return None;
        }
    }
    Some(300 - gaps.min(299))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn math() {
        assert_eq!(calc("2*(3+4)^2/7"), Some(14.0));
        assert_eq!(calc("15% of 80"), Some(12.0));
        assert_eq!(calc("sqrt(16)+1"), Some(5.0));
        assert_eq!(calc("hello"), None);
        assert_eq!(calc("42"), None);
    }
    #[test]
    fn units() {
        assert_eq!(convert("10 km to m").as_deref(), Some("10 km = 10000 m"));
        assert_eq!(convert("212f to c").as_deref(), Some("212 f = 100 c"));
        assert!(convert("5 kg to km").is_none());
    }
    #[test]
    fn fuzzy() {
        assert!(score("clp", "Clipboard").is_some());
        assert!(score("xyz", "Clipboard").is_none());
    }
}
