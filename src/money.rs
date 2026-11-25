use crate::error::Error;

/// 1 GB op de CloudFront-prijspagina. 1 TB = 1024 GB, 1 PB = 1024 TB.
pub const BYTES_PER_GB: u128 = 1_073_741_824;

/// USD-bedrag met maximaal 6 decimalen naar micro-USD (1 USD = 1_000_000).
pub fn parse_usd_micros(token: &str) -> Result<u64, Error> {
    if token.is_empty() || token.starts_with('+') || token.starts_with('-') {
        return Err(Error::new(format!("ongeldig bedrag {token}")));
    }
    let (whole, frac) = match token.split_once('.') {
        Some((whole, frac)) => (whole, frac),
        None => (token, ""),
    };
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !frac.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(Error::new(format!("ongeldig bedrag {token}")));
    }
    if frac.len() > 6 {
        return Err(Error::new(format!("te veel decimalen in {token}")));
    }
    let whole: u64 = whole
        .parse()
        .map_err(|_| Error::new(format!("ongeldig bedrag {token}")))?;
    let mut frac_digits = frac.to_string();
    while frac_digits.len() < 6 {
        frac_digits.push('0');
    }
    let frac_v: u64 = frac_digits
        .parse()
        .map_err(|_| Error::new(format!("ongeldig bedrag {token}")))?;
    whole
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(frac_v))
        .ok_or_else(|| Error::new(format!("bedrag {token} past niet")))
}

/// `(n * mul + div/2) / div` voor niet-negatieve getallen. Halve eenheden naar boven.
pub fn mul_div_round(n: u128, mul: u128, div: u128) -> Result<u128, Error> {
    if div == 0 {
        return Err(Error::new("delen door nul"));
    }
    let product = n
        .checked_mul(mul)
        .ok_or_else(|| Error::new("overflow in een bedrag"))?;
    let sum = product
        .checked_add(div / 2)
        .ok_or_else(|| Error::new("overflow in een bedrag"))?;
    Ok(sum / div)
}

pub fn format_usd(micros: u128) -> String {
    let whole = micros / 1_000_000;
    let frac = micros % 1_000_000;
    format!("{whole}.{frac:06}")
}

fn ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn is_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let year_text = &value[0..4];
    let month_text = &value[5..7];
    let day_text = &value[8..10];
    if !ascii_digits(year_text) || !ascii_digits(month_text) || !ascii_digits(day_text) {
        return false;
    }
    let Ok(year) = year_text.parse::<i32>() else {
        return false;
    };
    let Ok(month) = month_text.parse::<u8>() else {
        return false;
    };
    let Ok(day) = day_text.parse::<u8>() else {
        return false;
    };
    if !(1970..=2100).contains(&year) {
        return false;
    }
    let max = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max).contains(&day)
}

/// Days since 1970-01-01 of a date that `is_date` accepts.
pub fn day_number(value: &str) -> Option<i64> {
    if !is_date(value) {
        return None;
    }
    let year: i64 = value[0..4].parse().ok()?;
    let month: i64 = value[5..7].parse().ok()?;
    let day: i64 = value[8..10].parse().ok()?;
    // Civil-from-days, the other way round: shift the year to start in March.
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// Today (UTC) as `YYYY-MM-DD`, from the system clock.
pub fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let z = i64::try_from(secs / 86_400).unwrap_or(0) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Days after which the price date counts as old.
pub const PRICE_MAX_AGE_DAYS: i64 = 180;

/// The warning for prices older than `PRICE_MAX_AGE_DAYS` before `reference`, if any.
pub fn price_age_warning(checked_on: &str, reference: &str) -> Option<String> {
    let age = day_number(reference)? - day_number(checked_on)?;
    if age > PRICE_MAX_AGE_DAYS {
        Some(format!(
            "waarschuwing: de prijzen zijn van {checked_on}, {age} dagen voor {reference} (meer dan {PRICE_MAX_AGE_DAYS} dagen oud)"
        ))
    } else {
        None
    }
}

pub fn is_month(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 7 || bytes[4] != b'-' {
        return false;
    }
    let year_text = &value[0..4];
    let month_text = &value[5..7];
    if !ascii_digits(year_text) || !ascii_digits(month_text) {
        return false;
    }
    let Ok(year) = year_text.parse::<i32>() else {
        return false;
    };
    let Ok(month) = month_text.parse::<u8>() else {
        return false;
    };
    (1970..=2100).contains(&year) && (1..=12).contains(&month)
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_numbers_follow_the_calendar() {
        assert_eq!(day_number("1970-01-01"), Some(0));
        assert_eq!(day_number("1970-03-01"), Some(59));
        assert_eq!(day_number("2000-03-01"), Some(11_017));
        assert_eq!(day_number("2026-11-20"), Some(20_777));
        assert_eq!(day_number("2026-02-30"), None);
    }

    #[test]
    fn price_age_is_over_180_days_only() {
        assert!(price_age_warning("2026-01-01", "2026-06-30").is_none());
        let text = price_age_warning("2026-01-01", "2026-07-01").unwrap();
        assert!(text.contains("2026-01-01") && text.contains("181"));
        assert!(price_age_warning("2026-11-09", "2026-01-01").is_none());
    }

    #[test]
    fn today_is_a_valid_date() {
        assert!(is_date(&today_utc()));
    }

    #[test]
    fn parses_three_decimals() {
        assert_eq!(parse_usd_micros("0.085").unwrap(), 85_000);
    }

    #[test]
    fn parses_four_decimals() {
        assert_eq!(parse_usd_micros("0.0075").unwrap(), 7_500);
        assert_eq!(parse_usd_micros("0.0125").unwrap(), 12_500);
        assert_eq!(parse_usd_micros("0.0220").unwrap(), 22_000);
    }

    #[test]
    fn parses_whole_dollar() {
        assert_eq!(parse_usd_micros("1").unwrap(), 1_000_000);
    }

    #[test]
    fn rejects_negative_amount() {
        assert!(parse_usd_micros("-0.085").is_err());
    }

    #[test]
    fn rejects_seventh_decimal() {
        assert!(parse_usd_micros("0.0000001").is_err());
    }

    #[test]
    fn rejects_empty_amount() {
        assert!(parse_usd_micros("").is_err());
        assert!(parse_usd_micros(".").is_err());
    }

    #[test]
    fn amount_error_texts_are_fixed() {
        for token in ["", ".", "-1", "+1", ".5", "1.x", "a", "1.2.3"] {
            assert_eq!(
                parse_usd_micros(token).unwrap_err().message,
                format!("ongeldig bedrag {token}")
            );
        }
        let huge = "99999999999999999999";
        assert_eq!(
            parse_usd_micros(huge).unwrap_err().message,
            format!("ongeldig bedrag {huge}")
        );
        assert_eq!(
            parse_usd_micros("0.0000001").unwrap_err().message,
            "te veel decimalen in 0.0000001"
        );
        assert_eq!(
            parse_usd_micros("18446744073710").unwrap_err().message,
            "bedrag 18446744073710 past niet"
        );
    }

    #[test]
    fn division_error_texts_are_fixed() {
        assert_eq!(
            mul_div_round(1, 1, 0).unwrap_err().message,
            "delen door nul"
        );
        assert_eq!(
            mul_div_round(u128::MAX, 2, 1).unwrap_err().message,
            "overflow in een bedrag"
        );
        assert_eq!(
            mul_div_round(u128::MAX, 1, 3).unwrap_err().message,
            "overflow in een bedrag"
        );
    }

    #[test]
    fn one_gb_times_rate_is_exact() {
        let cost = mul_div_round(BYTES_PER_GB, 85_000, BYTES_PER_GB).unwrap();
        assert_eq!(cost, 85_000);
    }

    #[test]
    fn zero_bytes_cost_nothing() {
        assert_eq!(mul_div_round(0, 85_000, BYTES_PER_GB).unwrap(), 0);
    }

    #[test]
    fn rounds_half_up() {
        assert_eq!(mul_div_round(5, 1, 10).unwrap(), 1);
        assert_eq!(mul_div_round(15, 1, 10).unwrap(), 2);
    }

    #[test]
    fn rounds_below_half_down() {
        assert_eq!(mul_div_round(4, 1, 10).unwrap(), 0);
        assert_eq!(format_usd(85_000), "0.085000");
        assert!(is_date("2026-11-09"));
        assert!(!is_date("2026-02-29"));
        assert!(is_date("2024-02-29"));
        assert!(is_month("2026-11"));
        assert!(!is_month("2026-13"));
    }

    #[test]
    fn plus_sign_is_not_a_date_or_month() {
        assert!(!is_date("2026-+3-02"));
        assert!(!is_date("2026-03-+1"));
        assert!(!is_date("2026- 3-02"));
        assert!(!is_month("2026-+3"));
        assert!(!is_month("2026- 3"));
        assert!(is_date("2026-03-02"));
        assert!(is_month("2026-03"));
    }
}
