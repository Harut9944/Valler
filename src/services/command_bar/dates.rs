//! Date answers: `tomorrow`, `today + 10 days`, `2026-12-25 - 3 weeks`,
//! `in 2 months`, `5 days ago`, `next friday`, `days until 2026-12-25`,
//! `days since 2026-01-01` and `week`.

use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};

fn pretty(d: NaiveDate) -> String {
    d.format("%A, %-d %B %Y").to_string()
}

pub fn date_answer(q: &str) -> Option<String> {
    date_answer_on(q, Local::now().date_naive())
}

/// Same as [`date_answer`] with a fixed "today", so answers are testable.
pub fn date_answer_on(q: &str, today: NaiveDate) -> Option<String> {
    let s = q.trim().to_lowercase();
    let words: Vec<&str> = s.split_whitespace().collect();
    match words.as_slice() {
        ["week"] | ["week", "number"] => return Some(format!("Week {}", today.iso_week().week())),
        ["days", "until" | "till" | "to", d] => {
            let target = base(d, today)?;
            let n = (target - today).num_days();
            return Some(format!("{n} day{} ({})", if n.abs() == 1 { "" } else { "s" }, pretty(target)));
        }
        ["days", "since", d] => {
            let from = base(d, today)?;
            let n = (today - from).num_days();
            return Some(format!("{n} day{}", if n.abs() == 1 { "" } else { "s" }));
        }
        ["next", day] => {
            let wd = weekday(day)?;
            let mut d = today + Days::new(1);
            while d.weekday() != wd {
                d = d + Days::new(1);
            }
            return Some(pretty(d));
        }
        ["in", n, unit] => return shift(today, n.parse().ok()?, unit).map(pretty),
        [n, unit, "ago"] => return shift(today, -n.parse::<i64>().ok()?, unit).map(pretty),
        [b] => return base(b, today).filter(|_| !b.contains('-') || b.len() == 10).map(pretty),
        [b, op @ ("+" | "-"), n, unit] => {
            let start = base(b, today)?;
            let n: i64 = n.parse().ok()?;
            return shift(start, if *op == "-" { -n } else { n }, unit).map(pretty);
        }
        _ => {}
    }
    None
}

fn base(word: &str, today: NaiveDate) -> Option<NaiveDate> {
    match word {
        "today" | "now" => Some(today),
        "tomorrow" => Some(today + Days::new(1)),
        "yesterday" => Some(today - Days::new(1)),
        _ => NaiveDate::parse_from_str(word, "%Y-%m-%d").ok(),
    }
}

fn shift(d: NaiveDate, n: i64, unit: &str) -> Option<NaiveDate> {
    let unit = unit.trim_end_matches('s');
    let abs = n.unsigned_abs();
    let fwd = n >= 0;
    match unit {
        "day" => if fwd { d.checked_add_days(Days::new(abs)) } else { d.checked_sub_days(Days::new(abs)) },
        "week" => if fwd { d.checked_add_days(Days::new(abs * 7)) } else { d.checked_sub_days(Days::new(abs * 7)) },
        "month" => if fwd { d.checked_add_months(Months::new(abs as u32)) } else { d.checked_sub_months(Months::new(abs as u32)) },
        "year" => if fwd { d.checked_add_months(Months::new(abs as u32 * 12)) } else { d.checked_sub_months(Months::new(abs as u32 * 12)) },
        _ => None,
    }
}

fn weekday(w: &str) -> Option<Weekday> {
    let w = w.get(..3)?;
    Some(match w {
        "mon" => Weekday::Mon,
        "tue" => Weekday::Tue,
        "wed" => Weekday::Wed,
        "thu" => Weekday::Thu,
        "fri" => Weekday::Fri,
        "sat" => Weekday::Sat,
        "sun" => Weekday::Sun,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(q: &str) -> Option<String> {
        // Friday, 6 November 2026
        date_answer_on(q, NaiveDate::from_ymd_opt(2026, 11, 6).unwrap())
    }

    #[test]
    fn relative_days() {
        assert_eq!(on("today").as_deref(), Some("Friday, 6 November 2026"));
        assert_eq!(on("tomorrow").as_deref(), Some("Saturday, 7 November 2026"));
        assert_eq!(on("yesterday").as_deref(), Some("Thursday, 5 November 2026"));
        assert_eq!(on("today + 10 days").as_deref(), Some("Monday, 16 November 2026"));
        assert_eq!(on("in 2 weeks").as_deref(), Some("Friday, 20 November 2026"));
        assert_eq!(on("3 days ago").as_deref(), Some("Tuesday, 3 November 2026"));
    }

    #[test]
    fn months_years_and_explicit_dates() {
        assert_eq!(on("in 3 months").as_deref(), Some("Saturday, 6 February 2027"));
        assert_eq!(on("2026-01-31 + 1 month").as_deref(), Some("Saturday, 28 February 2026"), "clamps to month end");
        assert_eq!(on("2024-02-29 + 1 year").as_deref(), Some("Friday, 28 February 2025"));
        assert_eq!(on("2026-12-25").as_deref(), Some("Friday, 25 December 2026"));
    }

    #[test]
    fn counting_days_and_weekdays() {
        assert_eq!(on("days until 2026-12-25").as_deref(), Some("49 days (Friday, 25 December 2026)"));
        assert_eq!(on("days until tomorrow").as_deref(), Some("1 day (Saturday, 7 November 2026)"));
        assert_eq!(on("days since 2026-11-01").as_deref(), Some("5 days"));
        assert_eq!(on("next friday").as_deref(), Some("Friday, 13 November 2026"), "next means after today");
        assert_eq!(on("next mon").as_deref(), Some("Monday, 9 November 2026"));
        assert_eq!(on("week").as_deref(), Some("Week 45"));
    }

    #[test]
    fn ignores_other_text() {
        for q in ["hello", "in 3 parsecs", "next blursday", "2026-13-40", "today * 2 days", "12*7"] {
            assert!(on(q).is_none(), "{q}");
        }
    }
}
