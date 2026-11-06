//! Emoji search: `emoji heart` or `:fire` lists matching emoji to copy.

/// (emoji, name, extra keywords)
const TABLE: &[(&str, &str, &str)] = &[
    ("😀", "grinning", "smile happy face"),
    ("😂", "joy", "laugh tears funny"),
    ("🙂", "slightly smiling", "smile face"),
    ("😉", "wink", "face"),
    ("😍", "heart eyes", "love face"),
    ("😎", "sunglasses", "cool face"),
    ("🤔", "thinking", "hmm face"),
    ("😴", "sleeping", "tired sleep zzz"),
    ("😢", "crying", "sad tear"),
    ("😡", "angry", "mad face"),
    ("😱", "scream", "shock fear"),
    ("🥳", "party", "celebrate birthday"),
    ("🤯", "mind blown", "explode shock"),
    ("🙃", "upside down", "silly face"),
    ("😅", "sweat smile", "relief nervous"),
    ("🙏", "pray", "please thanks hands"),
    ("👍", "thumbs up", "yes ok like good"),
    ("👎", "thumbs down", "no dislike bad"),
    ("👏", "clap", "applause bravo"),
    ("👋", "wave", "hello hi bye hand"),
    ("💪", "muscle", "strong flex"),
    ("🤝", "handshake", "deal agree"),
    ("✌️", "victory", "peace hand"),
    ("👀", "eyes", "look see watch"),
    ("❤️", "red heart", "love like"),
    ("💔", "broken heart", "sad love"),
    ("💯", "hundred", "perfect score 100"),
    ("🔥", "fire", "hot lit flame"),
    ("✨", "sparkles", "shiny magic new"),
    ("⭐", "star", "favorite"),
    ("🎉", "tada", "party celebrate congratulations"),
    ("🎂", "birthday cake", "party"),
    ("🎁", "gift", "present"),
    ("✅", "check mark", "done yes ok tick"),
    ("❌", "cross mark", "no wrong x"),
    ("⚠️", "warning", "caution alert"),
    ("❓", "question", "what ask"),
    ("❗", "exclamation", "important alert"),
    ("💡", "light bulb", "idea tip"),
    ("📌", "pushpin", "pin location"),
    ("📎", "paperclip", "attach"),
    ("📅", "calendar", "date schedule"),
    ("⏰", "alarm clock", "time wake"),
    ("⏳", "hourglass", "wait time"),
    ("🔒", "lock", "secure private"),
    ("🔑", "key", "password"),
    ("🔔", "bell", "notification"),
    ("📧", "email", "mail message"),
    ("📞", "telephone", "phone call"),
    ("💻", "laptop", "computer"),
    ("🖥️", "desktop computer", "monitor screen"),
    ("⌨️", "keyboard", "type"),
    ("🖱️", "mouse", "computer"),
    ("📱", "mobile phone", "smartphone"),
    ("🔋", "battery", "power charge"),
    ("🔌", "plug", "power electric"),
    ("💾", "floppy disk", "save"),
    ("📁", "folder", "files directory"),
    ("📄", "page", "document file"),
    ("📊", "bar chart", "stats graph"),
    ("📈", "chart up", "growth increase"),
    ("📉", "chart down", "decrease"),
    ("🐛", "bug", "insect error"),
    ("🚀", "rocket", "launch ship fast"),
    ("⚙️", "gear", "settings"),
    ("🔧", "wrench", "tool fix"),
    ("🧪", "test tube", "experiment science"),
    ("🎵", "musical note", "music song"),
    ("🎧", "headphones", "music audio"),
    ("📷", "camera", "photo picture"),
    ("🎮", "video game", "controller play"),
    ("☕", "coffee", "hot drink morning"),
    ("🍕", "pizza", "food"),
    ("🍔", "hamburger", "food burger"),
    ("🍎", "apple", "fruit food"),
    ("🍺", "beer", "drink cheers"),
    ("🍷", "wine", "drink"),
    ("☀️", "sun", "sunny weather"),
    ("🌙", "moon", "night"),
    ("🌧️", "rain", "weather cloud"),
    ("❄️", "snowflake", "cold winter"),
    ("🌈", "rainbow", "weather"),
    ("🌍", "globe", "earth world"),
    ("🏠", "house", "home"),
    ("🚗", "car", "drive"),
    ("✈️", "airplane", "flight travel"),
    ("🐶", "dog", "puppy animal"),
    ("🐱", "cat", "kitten animal"),
    ("🦀", "crab", "rust"),
    ("🌱", "seedling", "plant grow"),
    ("🌸", "cherry blossom", "flower spring"),
    ("💰", "money bag", "cash rich"),
    ("💸", "money flying", "spend"),
    ("🏆", "trophy", "win award"),
    ("🎯", "direct hit", "target goal"),
    ("🧠", "brain", "smart think"),
    ("👻", "ghost", "halloween boo"),
    ("💩", "poop", "pile"),
    ("🤖", "robot", "bot ai"),
    ("👑", "crown", "king queen"),
    ("➡️", "right arrow", "next"),
    ("⬅️", "left arrow", "back"),
    ("⬆️", "up arrow", ""),
    ("⬇️", "down arrow", ""),
];

/// Matching emoji for `emoji <words>` or `:<word>` queries; empty otherwise.
pub fn emoji(q: &str) -> Vec<(&'static str, &'static str)> {
    let s = q.trim().to_lowercase();
    let Some(term) = s.strip_prefix("emoji").map(str::trim).or_else(|| s.strip_prefix(':').map(str::trim)) else {
        return vec![];
    };
    let words: Vec<&str> = term.split_whitespace().collect();
    let mut hits: Vec<(usize, &(&str, &str, &str))> = TABLE
        .iter()
        .filter_map(|e| {
            let hay = format!("{} {}", e.1, e.2);
            if !words.iter().all(|w| hay.contains(w)) {
                return None;
            }
            // Name matches first, then keyword matches.
            let rank = if words.is_empty() { 1 } else if e.1.starts_with(words[0]) { 0 } else if e.1.contains(words[0]) { 1 } else { 2 };
            Some((rank, e))
        })
        .collect();
    hits.sort_by_key(|(r, _)| *r);
    hits.into_iter().take(12).map(|(_, e)| (e.0, e.1)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_emoji_by_name_and_keyword() {
        assert_eq!(emoji("emoji fire")[0], ("🔥", "fire"));
        assert_eq!(emoji(":rocket")[0].0, "🚀");
        assert!(emoji("emoji love").iter().any(|(e, _)| *e == "❤️"), "keywords count");
        assert_eq!(emoji("emoji heart")[0].1, "heart eyes", "names beat keywords");
        assert!(emoji("emoji thumbs up").iter().all(|(_, n)| n.contains("thumbs")));
    }

    #[test]
    fn only_answers_emoji_queries() {
        assert!(emoji("fire").is_empty());
        assert!(emoji("emoji zzzqqq").is_empty());
        assert_eq!(emoji("emoji").len(), 12, "browsing shows the first page");
    }

    #[test]
    fn table_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for (e, n, _) in TABLE {
            assert!(seen.insert(*e), "duplicate {e} {n}");
        }
    }
}
