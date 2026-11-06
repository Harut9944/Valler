//! Removes tracking parameters from links. Global names apply everywhere;
//! some names are only trackers on particular sites (e.g. `si` on YouTube,
//! `t` on X), so those are matched by host.

use url::Url;

const GLOBAL: &[&str] = &[
    "fbclid", "gclid", "dclid", "gbraid", "wbraid", "msclkid", "yclid", "mc_cid", "mc_eid",
    "igshid", "twclid", "ttclid", "li_fat_id", "mkt_tok", "_hsenc", "_hsmi",
    "__twitter_impression", "fb_action_ids", "fb_action_types", "fb_source", "mibextid",
];

const SITES: &[(&str, &[&str])] = &[
    ("youtube.com", &["si", "pp", "feature", "kw"]),
    ("youtu.be", &["si", "pp", "feature", "kw"]),
    ("twitter.com", &["s", "t", "cn", "src", "refsrc", "ref_src", "ref_url"]),
    ("x.com", &["s", "t", "cn", "src", "refsrc", "ref_src", "ref_url"]),
    ("instagram.com", &["igsh"]),
    ("spotify.com", &["si"]),
    ("reddit.com", &["correlation_id", "ref_campaign", "ref_source", "rdt", "share_id", "_branch_match_id", "$deep_link", "$3p", "$original_url"]),
    ("tiktok.com", &["u_code", "preview_pb", "_d", "_t", "_r", "timestamp", "user_id", "share_app_name", "share_iid"]),
    ("amazon.com", &["ref", "ref_", "pd_rd_r", "pd_rd_w", "pd_rd_wg", "pf_rd_p", "pf_rd_r", "psc", "crid", "sprefix"]),
];

pub struct Cleaned {
    pub url: String,
    pub removed: Vec<String>,
}

fn host_matches(host: &str, site: &str) -> bool {
    host == site || host.ends_with(&format!(".{site}"))
}

/// Returns None if the text is not a single http(s) link.
pub fn clean(text: &str, extra: &str) -> Option<Cleaned> {
    let t = text.trim();
    if t.is_empty() || t.contains(char::is_whitespace) {
        return None;
    }
    let mut url = Url::parse(t).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let host = url.host_str().unwrap_or("").to_lowercase();
    let extras: Vec<String> = extra
        .split([',', ' ', '\n'])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let is_tracker = |name: &str| -> bool {
        let n = name.to_lowercase();
        n.starts_with("utm_")
            || GLOBAL.contains(&n.as_str())
            || extras.iter().any(|e| e == &n)
            || SITES
                .iter()
                .any(|(site, names)| host_matches(&host, site) && names.contains(&n.as_str()))
    };
    let pairs: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    let mut removed = Vec::new();
    let kept: Vec<(String, String)> = pairs
        .into_iter()
        .filter(|(k, _)| {
            if is_tracker(k) {
                removed.push(k.clone());
                false
            } else {
                true
            }
        })
        .collect();
    if removed.is_empty() {
        return Some(Cleaned { url: t.to_string(), removed });
    }
    if kept.is_empty() {
        url.set_query(None);
    } else {
        url.query_pairs_mut().clear().extend_pairs(kept.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    }
    Some(Cleaned { url: url.to_string(), removed })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strips_utm_and_site_params() {
        let r = clean("https://www.youtube.com/watch?v=abc&si=XYZ&utm_source=x", "").unwrap();
        assert_eq!(r.url, "https://www.youtube.com/watch?v=abc");
        assert_eq!(r.removed.len(), 2);
        let r = clean("https://example.com/?t=5", "").unwrap();
        assert!(r.removed.is_empty());
        assert!(clean("hello world", "").is_none());
    }
}
