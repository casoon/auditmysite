//! The one number formatter of the reports (#902).
//!
//! German text writes the decimal comma (`1,25:1`, `2,4 MB`), English the
//! point. Report text builders format measured values through here instead of
//! `format!("{:.1}")`, so a German sentence never mixes both notations.

/// `value` with `decimals` places in the notation of the language
/// (`en = true`: decimal point, otherwise decimal comma).
pub fn decimal(value: f64, decimals: usize, en: bool) -> String {
    let plain = format!("{value:.decimals$}");
    if en {
        plain
    } else {
        plain.replace('.', ",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_uses_a_comma_english_a_point() {
        assert_eq!(decimal(1.25, 2, false), "1,25");
        assert_eq!(decimal(1.25, 2, true), "1.25");
        assert_eq!(decimal(17.2, 1, false), "17,2");
        assert_eq!(decimal(4.0, 1, false), "4,0");
        assert_eq!(decimal(100.0, 0, false), "100");
    }

    #[test]
    fn negative_and_small_values() {
        assert_eq!(decimal(-0.5, 1, false), "-0,5");
        assert_eq!(decimal(0.0, 2, false), "0,00");
    }
}
