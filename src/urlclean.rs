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

#[cfg(test)]
mod feature_tests {
    use super::clean;

    #[test]
    fn removes_every_utm_parameter_and_global_trackers() {
        let r = clean("https://shop.example/p?id=7&utm_campaign=x&utm_whatever=y&fbclid=z&gclid=1", "").unwrap();
        assert_eq!(r.url, "https://shop.example/p?id=7");
        assert_eq!(r.removed.len(), 4);
    }

    #[test]
    fn site_rules_only_apply_on_their_site() {
        // `t` is a tracker on X but the playback time on YouTube.
        let x = clean("https://x.com/user/status/1?t=abc&s=20", "").unwrap();
        assert_eq!(x.url, "https://x.com/user/status/1");
        let yt = clean("https://youtube.com/watch?v=1&t=42", "").unwrap();
        assert!(yt.removed.is_empty());
        // Subdomains count as the site.
        let m = clean("https://m.youtube.com/watch?v=1&si=abc", "").unwrap();
        assert_eq!(m.removed, vec!["si".to_string()]);
    }

    #[test]
    fn extra_parameters_from_settings_are_removed_everywhere() {
        let r = clean("https://a.example/?ref=me&keep=1", "ref, source").unwrap();
        assert_eq!(r.url, "https://a.example/?keep=1");
    }

    #[test]
    fn keeps_fragment_and_drops_empty_query() {
        let r = clean("https://a.example/page?utm_source=x#section", "").unwrap();
        assert_eq!(r.url, "https://a.example/page#section");
    }

    #[test]
    fn ignores_things_that_are_not_single_web_links() {
        assert!(clean("", "").is_none());
        assert!(clean("two words", "").is_none());
        assert!(clean("ftp://files.example/a", "").is_none());
        assert!(clean("mailto:a@b.c", "").is_none());
        let unchanged = clean("  https://a.example/x?q=1  ", "").unwrap();
        assert!(unchanged.removed.is_empty());
        assert_eq!(unchanged.url, "https://a.example/x?q=1");
    }
}
