#[cfg(test)]
mod tests {
    use crate::format_duration;
    use crate::parse_duration;

    #[test]
    fn test_parse_duration_with_suffixes() {
        let input = "1h 20m 30s";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (1, 20, 30));
    }

    #[test]
    fn test_parse_duration_without_suffix() {
        let input = "45";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (0, 0, 45));
    }

    #[test]
    fn test_parse_duration_only_hours() {
        let input = "2h";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (2, 0, 0));
    }

    #[test]
    fn test_parse_duration_only_minutes() {
        let input = "45m";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (0, 45, 0));
    }

    #[test]
    fn test_parse_duration_only_seconds() {
        let input = "30s";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (0, 0, 30));
    }

    #[test]
    fn test_parse_duration_combined() {
        let input = "1h 15m 20s";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (1, 15, 20));
    }

    #[test]
    fn test_parse_duration_plain_number() {
        let input = "120";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (0, 0, 120));
    }

    #[test]
    fn test_parse_duration_large_values() {
        let input = "1000h 2000m 3000s";
        let (h, m, s) = parse_duration(input);
        assert_eq!((h, m, s), (1000, 2000, 3000));
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        let result = format_duration(3661); // 1 hour, 1 minute, 1 second
        assert_eq!(result, "1h 1m 1s");
    }

    #[test]
    fn test_format_duration_only_minutes_seconds() {
        let result = format_duration(125); // 2 minutes, 5 seconds
        assert_eq!(result, "2m 5s");
    }

    #[test]
    fn test_format_duration_only_seconds() {
        let result = format_duration(59); // 59 seconds
        assert_eq!(result, "59s");
    }

    #[test]
    fn test_format_duration_only_hours() {
        let result = format_duration(7200); // 2 hours
        assert_eq!(result, "2h 0m 0s");
    }

    #[test]
    fn test_format_duration_zero() {
        let result = format_duration(0);
        assert_eq!(result, "0s");
    }

    #[test]
    fn test_format_duration_large_duration() {
        let result = format_duration(90061); // 25 hours, 1 minute, 1 second
        assert_eq!(result, "25h 1m 1s");
    }
}
