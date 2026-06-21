pub(in crate::workspace_integrity) fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[cfg(test)]
#[path = "../../../tests/unit/workspace_integrity/checker/sqlite/tests.rs"]
mod tests;
