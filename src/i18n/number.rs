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

/// `value` with thousands grouping: German `7.594`, English `7,594`. Values
/// below 1000 stay plain. One convention for every whole number in report
/// text, so `7.594` and `7594` never appear for the same figure (#952).
pub fn integer(value: i64, en: bool) -> String {
    let separator = if en { ',' } else { '.' };
    let digits = value.unsigned_abs().to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if value < 0 {
        grouped.push('-');
    }
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(separator);
        }
        grouped.push(digit);
    }
    grouped
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
    fn integers_are_grouped_per_language() {
        assert_eq!(integer(7594, false), "7.594");
        assert_eq!(integer(7594, true), "7,594");
        assert_eq!(integer(999, false), "999");
        assert_eq!(integer(-1234567, false), "-1.234.567");
    }

    #[test]
    fn negative_and_small_values() {
        assert_eq!(decimal(-0.5, 1, false), "-0,5");
        assert_eq!(decimal(0.0, 2, false), "0,00");
    }
}
