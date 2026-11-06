//! Fuzzy matching used to rank command bar results.

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
