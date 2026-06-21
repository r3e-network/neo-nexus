use super::identifier;

#[test]
fn identifier_escapes_embedded_quotes() {
    assert_eq!(identifier("event\"journal"), "\"event\"\"journal\"");
}
