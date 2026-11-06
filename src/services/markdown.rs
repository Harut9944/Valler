//! A small Markdown reader for the Scratchpad preview: headings, bullet,
//! numbered and task lists, quotes, code blocks, rules and inline **bold**,
//! *italic*, `code` and [links](https://…). Turns text into blocks the screen
//! draws; it never runs code or loads anything.

#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Link { text: String, url: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(u8, Vec<Span>),
    Bullet { indent: u8, spans: Vec<Span> },
    Numbered { number: u32, spans: Vec<Span> },
    Task { done: bool, spans: Vec<Span> },
    Quote(Vec<Span>),
    Code(String),
    Rule,
    Paragraph(Vec<Span>),
}

pub fn parse(text: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut code: Option<Vec<&str>> = None;
    let mut para: Vec<&str> = Vec::new();
    let flush = |para: &mut Vec<&str>, out: &mut Vec<Block>| {
        if !para.is_empty() {
            out.push(Block::Paragraph(inline(&para.join(" "))));
            para.clear();
        }
    };
    for line in text.lines() {
        if let Some(lines) = code.as_mut() {
            if line.trim_start().starts_with("```") {
                out.push(Block::Code(lines.join("\n")));
                code = None;
            } else {
                lines.push(line);
            }
            continue;
        }
        let t = line.trim_start();
        let indent = ((line.len() - t.len()) / 2).min(3) as u8;
        if t.starts_with("```") {
            flush(&mut para, &mut out);
            code = Some(Vec::new());
        } else if t.is_empty() {
            flush(&mut para, &mut out);
        } else if let Some(level) = heading_level(t) {
            flush(&mut para, &mut out);
            out.push(Block::Heading(level, inline(t[level as usize..].trim())));
        } else if matches!(t, "---" | "***" | "___") {
            flush(&mut para, &mut out);
            out.push(Block::Rule);
        } else if let Some(rest) = t.strip_prefix("> ").or_else(|| t.strip_prefix('>')) {
            flush(&mut para, &mut out);
            out.push(Block::Quote(inline(rest.trim())));
        } else if let Some(rest) = bullet(t) {
            flush(&mut para, &mut out);
            if let Some(task) = rest.strip_prefix("[ ] ") {
                out.push(Block::Task { done: false, spans: inline(task) });
            } else if let Some(task) = rest.strip_prefix("[x] ").or_else(|| rest.strip_prefix("[X] ")) {
                out.push(Block::Task { done: true, spans: inline(task) });
            } else {
                out.push(Block::Bullet { indent, spans: inline(rest) });
            }
        } else if let Some((n, rest)) = numbered(t) {
            flush(&mut para, &mut out);
            out.push(Block::Numbered { number: n, spans: inline(rest) });
        } else {
            para.push(t);
        }
    }
    if let Some(lines) = code {
        out.push(Block::Code(lines.join("\n"))); // unclosed fence: keep what was typed
    }
    flush(&mut para, &mut out);
    out
}

fn heading_level(t: &str) -> Option<u8> {
    let n = t.chars().take_while(|c| *c == '#').count();
    ((1..=6).contains(&n) && t[n..].starts_with(' ')).then_some(n as u8)
}

fn bullet(t: &str) -> Option<&str> {
    ["- ", "* ", "+ "].iter().find_map(|p| t.strip_prefix(p))
}

fn numbered(t: &str) -> Option<(u32, &str)> {
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || digits > 9 {
        return None;
    }
    let rest = t[digits..].strip_prefix(". ").or_else(|| t[digits..].strip_prefix(") "))?;
    Some((t[..digits].parse().ok()?, rest))
}

/// Inline formatting inside one block.
pub fn inline(s: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut rest = s;
    let push_text = |text: &mut String, out: &mut Vec<Span>| {
        if !text.is_empty() {
            out.push(Span::Text(std::mem::take(text)));
        }
    };
    while let Some(c) = rest.chars().next() {
        let try_wrap = |rest: &str, mark: &str| -> Option<(String, usize)> {
            let inner = rest.strip_prefix(mark)?;
            let end = inner.find(mark)?;
            (end > 0).then(|| (inner[..end].to_string(), mark.len() * 2 + end))
        };
        if let Some((inner, len)) = try_wrap(rest, "**").or_else(|| try_wrap(rest, "__")) {
            push_text(&mut text, &mut out);
            out.push(Span::Bold(inner));
            rest = &rest[len..];
        } else if let Some((inner, len)) = try_wrap(rest, "`") {
            push_text(&mut text, &mut out);
            out.push(Span::Code(inner));
            rest = &rest[len..];
        } else if let Some((inner, len)) = try_wrap(rest, "*").or_else(|| try_wrap(rest, "_")).filter(|(i, _)| !i.starts_with(' ')) {
            push_text(&mut text, &mut out);
            out.push(Span::Italic(inner));
            rest = &rest[len..];
        } else if let Some((label, url, len)) = link(rest) {
            push_text(&mut text, &mut out);
            out.push(Span::Link { text: label, url });
            rest = &rest[len..];
        } else {
            text.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    push_text(&mut text, &mut out);
    out
}

/// `[text](http…)` — only web links, so a preview can never open files.
fn link(rest: &str) -> Option<(String, String, usize)> {
    let inner = rest.strip_prefix('[')?;
    let close = inner.find("](")?;
    let after = &inner[close + 2..];
    let end = after.find(')')?;
    let url = &after[..end];
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return None;
    }
    Some((inner[..close].to_string(), url.to_string(), 1 + close + 2 + end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Span::*;

    fn t(s: &str) -> Span {
        Text(s.into())
    }

    #[test]
    fn blocks() {
        let md = "# Title\n\nSome text\nthat wraps.\n\n- one\n  - nested\n1. first\n- [ ] todo\n- [x] done\n> quoted\n---\n```\nlet x = 1;\n```";
        let b = parse(md);
        assert_eq!(b[0], Block::Heading(1, vec![t("Title")]));
        assert_eq!(b[1], Block::Paragraph(vec![t("Some text that wraps.")]));
        assert_eq!(b[2], Block::Bullet { indent: 0, spans: vec![t("one")] });
        assert_eq!(b[3], Block::Bullet { indent: 1, spans: vec![t("nested")] });
        assert_eq!(b[4], Block::Numbered { number: 1, spans: vec![t("first")] });
        assert_eq!(b[5], Block::Task { done: false, spans: vec![t("todo")] });
        assert_eq!(b[6], Block::Task { done: true, spans: vec![t("done")] });
        assert_eq!(b[7], Block::Quote(vec![t("quoted")]));
        assert_eq!(b[8], Block::Rule);
        assert_eq!(b[9], Block::Code("let x = 1;".into()));
        assert_eq!(b.len(), 10);
    }

    #[test]
    fn inline_formatting() {
        assert_eq!(inline("a **b** c"), vec![t("a "), Bold("b".into()), t(" c")]);
        assert_eq!(inline("*it* and `code`"), vec![Italic("it".into()), t(" and "), Code("code".into())]);
        assert_eq!(inline("see [site](https://example.com)!"), vec![t("see "), Link { text: "site".into(), url: "https://example.com".into() }, t("!")]);
        assert_eq!(inline("2 * 3 * 4"), vec![t("2 * 3 * 4")], "lone stars are just text");
        assert_eq!(inline("[file](/etc/passwd)"), vec![t("[file](/etc/passwd)")], "only web links");
        assert_eq!(inline("ünïcode **ok**"), vec![t("ünïcode "), Bold("ok".into())]);
    }

    #[test]
    fn edge_cases() {
        assert!(parse("").is_empty());
        assert_eq!(parse("#nospace"), vec![Block::Paragraph(vec![t("#nospace")])]);
        assert_eq!(parse("```\nunclosed"), vec![Block::Code("unclosed".into())]);
        assert_eq!(parse("3) third"), vec![Block::Numbered { number: 3, spans: vec![t("third")] }]);
    }
}
