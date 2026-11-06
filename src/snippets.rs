//! Expands snippet variables: {{date}}, {{time}}, {{datetime}}, {{clipboard}},
//! {{date:%Y-%m-%d}} (any strftime format) and {{name}} (the user's name).

use chrono::Local;

pub fn expand(text: &str, clipboard: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("}}") {
            let token = after[..end].trim();
            out.push_str(&resolve(token, clipboard).unwrap_or_else(|| format!("{{{{{token}}}}}")));
            rest = &after[end + 2..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

fn resolve(token: &str, clipboard: &str) -> Option<String> {
    let now = Local::now();
    let (name, fmt) = match token.split_once(':') {
        Some((n, f)) => (n.trim(), Some(f.trim())),
        None => (token, None),
    };
    let format = |default: &str| now.format(fmt.unwrap_or(default)).to_string();
    match name {
        "date" => Some(format("%Y-%m-%d")),
        "time" => Some(format("%H:%M")),
        "datetime" => Some(format("%Y-%m-%d %H:%M")),
        "clipboard" => Some(clipboard.to_string()),
        "name" => Some(user_name()),
        _ => None,
    }
}

fn user_name() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "me".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn expands() {
        let s = super::expand("a {{clipboard}} b {{nope}}", "X");
        assert_eq!(s, "a X b {{nope}}");
        assert!(super::expand("{{date:%Y}}", "").len() == 4);
    }
}
