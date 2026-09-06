use super::*;

#[test]
fn event_filters_are_bounded_and_strict() {
    assert_eq!(parse_limit(""), 100);
    assert_eq!(parse_limit("999999"), 500);
    assert_eq!(parse_limit("0"), 1);
    assert_eq!(parse_severity("").unwrap(), None);
    assert_eq!(
        parse_severity("warning").unwrap().unwrap().label(),
        "warning"
    );
    assert!(parse_severity("urgent").is_err());
}
