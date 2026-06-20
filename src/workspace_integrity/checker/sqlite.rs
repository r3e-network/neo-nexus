pub(in crate::workspace_integrity) fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::identifier;

    #[test]
    fn identifier_escapes_embedded_quotes() {
        assert_eq!(identifier("event\"journal"), "\"event\"\"journal\"");
    }
}
