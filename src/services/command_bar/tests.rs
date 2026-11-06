//! Tests for the calculator, unit conversion and matching.

#[cfg(test)]
mod tests {
    use crate::services::command_bar::*;
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

#[cfg(test)]
mod feature_tests {
    use crate::services::command_bar::*;

    #[test]
    fn operator_precedence_and_powers() {
        assert_eq!(calc("2+3*4"), Some(14.0));
        assert_eq!(calc("(2+3)*4"), Some(20.0));
        assert_eq!(calc("2^3^2"), Some(512.0)); // right associative
        assert_eq!(calc("-2^2"), Some(-4.0));
        assert_eq!(calc("2^-1"), Some(0.5));
        assert_eq!(calc("(-2)^2"), Some(4.0));
        assert_eq!(calc("10/4"), Some(2.5));
        assert_eq!(calc("7%3"), Some(1.0));
        assert_eq!(calc("50%"), Some(0.5));
        assert_eq!(calc("1,000*2"), Some(2000.0));
        assert_eq!(calc("3×4÷2"), Some(6.0));
    }

    #[test]
    fn functions_and_constants() {
        assert_eq!(calc("abs(-3)"), Some(3.0));
        assert_eq!(calc("round(2.6)"), Some(3.0));
        assert!((calc("pi*2").unwrap() - std::f64::consts::TAU).abs() < 1e-9);
        assert_eq!(calc("log(1000)"), Some(3.0));
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(calc("1/0"), None); // infinite
        assert_eq!(calc("2+"), None);
        assert_eq!(calc("(1"), None);
        assert_eq!(calc("foo(2)"), None);
        assert_eq!(calc(""), None);
    }

    #[test]
    fn number_formatting() {
        assert_eq!(fmt_num(84.0), "84");
        assert_eq!(fmt_num(0.1 + 0.2), "0.3");
        assert_eq!(fmt_num(-2.5), "-2.5");
    }

    #[test]
    fn converts_each_kind_of_unit() {
        assert_eq!(convert("1 mi to km").as_deref(), Some("1 mi = 1.609344 km"));
        assert_eq!(convert("2 lb in kg").as_deref(), Some("2 lb = 0.907185 kg"));
        assert_eq!(convert("1 gb to mb").as_deref(), Some("1 gb = 1000 mb"));
        assert_eq!(convert("1 gib to mib").as_deref(), Some("1 gib = 1024 mib"));
        assert_eq!(convert("90 min to h").as_deref(), Some("90 min = 1.5 h"));
        assert_eq!(convert("1 gal to l").as_deref(), Some("1 gal = 3.785412 l"));
        assert_eq!(convert("100 kmh to mph").as_deref(), Some("100 kmh = 62.137119 mph"));
        assert_eq!(convert("0 c to k").as_deref(), Some("0 c = 273.15 k"));
        assert_eq!(convert("1,000 m = km").as_deref(), Some("1000 m = 1 km"));
        assert!(convert("10 parsecs to m").is_none());
        assert!(convert("hello").is_none());
    }

    #[test]
    fn fuzzy_scores_prefer_exact_and_early_matches() {
        let exact = score("clip", "Clipboard").unwrap();
        let later = score("board", "Clipboard").unwrap();
        let spread = score("cbd", "Clipboard").unwrap();
        assert!(exact > later && later > spread);
        assert_eq!(score("", "anything"), Some(0));
    }
}
