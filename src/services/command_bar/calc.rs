//! Calculator: arithmetic with precedence, powers, percent and functions.

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
        let mut v = self.unary()?;
        loop {
            if self.eat('*') {
                v *= self.unary()?;
            } else if self.eat('/') {
                v /= self.unary()?;
            } else if self.eat('%') {
                // modulo when followed by an operand, else percent
                if matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '(') {
                    v %= self.unary()?;
                } else {
                    v /= 100.0;
                }
            } else {
                return Some(v);
            }
        }
    }
    /// `^` binds tighter than a leading minus: -2^2 is -(2^2) = -4.
    fn power(&mut self) -> Option<f64> {
        let base = self.atom()?;
        if self.eat('^') {
            let e = self.unary()?; // right associative, allows 2^-1
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
        self.power()
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
