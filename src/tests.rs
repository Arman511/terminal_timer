use crate::{ParseDurationError, format_duration, parse_duration};
use std::time::Duration;

#[test]
fn parses_valid_inputs() {
    assert_eq!(parse_duration("1h 20m 30s"), Ok(Duration::from_secs(4830)));
    assert_eq!(parse_duration("45"), Ok(Duration::from_secs(45)));
    assert_eq!(parse_duration("2h"), Ok(Duration::from_secs(7200)));
    assert_eq!(parse_duration("45m"), Ok(Duration::from_secs(2700)));
    assert_eq!(parse_duration("30s"), Ok(Duration::from_secs(30)));
    assert_eq!(parse_duration("1h20m"), Ok(Duration::from_secs(4800)));
    assert_eq!(parse_duration("  1H 2M "), Ok(Duration::from_secs(3720)));
    assert_eq!(
        parse_duration("1000h 2000m 3000s"),
        Ok(Duration::from_secs(3_600_000 + 120_000 + 3000))
    );
}

#[test]
fn rejects_invalid_inputs() {
    assert_eq!(parse_duration(""), Err(ParseDurationError::Empty));
    assert_eq!(parse_duration("0"), Err(ParseDurationError::Zero));
    assert_eq!(parse_duration("0h 0m"), Err(ParseDurationError::Zero));
    assert!(matches!(
        parse_duration("abc"),
        Err(ParseDurationError::InvalidToken(_))
    ));
    assert!(matches!(
        parse_duration("5x"),
        Err(ParseDurationError::InvalidToken(_))
    ));
    assert!(matches!(
        parse_duration("h"),
        Err(ParseDurationError::InvalidToken(_))
    ));
    assert!(matches!(
        parse_duration("1h 5"),
        Err(ParseDurationError::BareNumberWithUnits)
    ));
    assert!(matches!(
        parse_duration("5 5"),
        Err(ParseDurationError::BareNumberWithUnits)
    ));
}

#[test]
fn rejects_duplicate_units() {
    assert_eq!(
        parse_duration("1h 2h"),
        Err(ParseDurationError::DuplicateUnit('h'))
    );
    assert_eq!(
        parse_duration("1s 2m 3s"),
        Err(ParseDurationError::DuplicateUnit('s'))
    );
}

#[test]
fn rejects_overflow() {
    assert_eq!(
        parse_duration("99999999999999999999s"),
        Err(ParseDurationError::Overflow)
    );
    assert_eq!(
        parse_duration("18446744073709551615h"),
        Err(ParseDurationError::Overflow)
    );
    assert_eq!(
        parse_duration("18446744073709551615"),
        Err(ParseDurationError::Overflow)
    );
}

#[test]
fn formats_durations() {
    assert_eq!(format_duration(3661), "1h 1m 1s");
    assert_eq!(format_duration(125), "2m 5s");
    assert_eq!(format_duration(59), "59s");
    assert_eq!(format_duration(7200), "2h 0m 0s");
    assert_eq!(format_duration(0), "0s");
    assert_eq!(format_duration(90061), "25h 1m 1s");
}
