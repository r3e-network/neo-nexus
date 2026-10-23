//! The content security policy allows no inline script, so an inline event
//! handler never runs.
//!
//! The console serves `script-src 'sha256-<hash of the bundled script>'` with
//! no `'unsafe-inline'`. That is the right policy — it is why a string that
//! happens to contain markup cannot become code — but it has one consequence
//! that is invisible until an operator clicks the wrong thing:
//!
//! ```text
//! <button onclick="return confirm('delete?')">Delete</button>
//! ```
//!
//! The browser refuses to compile that handler and logs a CSP violation. The
//! button still submits its form. The confirmation never appears. The
//! operator deletes every log file in the workspace.
//!
//! Two of these shipped. One guarded a permanent, irreversible deletion; the
//! other two made a checkbox and a set of preset cards silently do nothing.
//! Nothing in a test failed, because the page renders exactly the markup it
//! always rendered — the handler is simply never executed by a browser that
//! obeys the policy this product already sets.
//!
//! The rule reads the syntax tree for the same reason the other two do: a
//! comment that *mentions* `onclick` is not an inline handler, and a string
//! that merely contains the word is not one either. Only a string literal
//! that is actual markup, carrying an actual event attribute, is an offence.

use proc_macro2::TokenTree;
use syn::visit::Visit;

/// The event attributes a browser would compile if the policy allowed it.
const EVENT_ATTRIBUTES: &[&str] = &[
    "onclick",
    "onchange",
    "onsubmit",
    "oninput",
    "onfocus",
    "onblur",
    "onload",
    "onerror",
    "onkeydown",
    "onkeyup",
    "onkeypress",
    "onmouseover",
    "onmouseout",
    "onmousedown",
    "onmouseup",
    "ondblclick",
];

/// An inline event handler found in a rendered markup string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineHandler {
    /// The attribute name, lowercased.
    pub attribute: String,
    /// The 1-based line of the file the string literal starts on.
    pub line: usize,
    /// The 1-based column the attribute starts at.
    pub column: usize,
}

/// Find every inline event handler in the string literals of a parsed file.
///
/// Only literals that look like markup are considered. A literal such as
/// `"the onchange attribute is blocked"` is prose about the attribute, not
/// an attribute, and treating it as one would make the rule cry wolf.
pub fn find_inline_handlers(parsed: &syn::File) -> Vec<InlineHandler> {
    let mut visitor = MarkupVisitor::default();
    visitor.visit_file(parsed);
    visitor.handlers
}

/// Collects the markup strings in a file and the event attributes in them.
#[derive(Default)]
struct MarkupVisitor {
    handlers: Vec<InlineHandler>,
}

impl<'ast> Visit<'ast> for MarkupVisitor {
    fn visit_lit_str(&mut self, literal: &'ast syn::LitStr) {
        self.inspect(literal);
    }

    /// A macro's arguments are a token stream, not expressions, so the default
    /// visitor never reaches the string literals inside them.
    ///
    /// That matters here more than it usually would: almost every page in
    /// this product renders markup through `format!`, so a visitor that only
    /// walks the expression tree sees almost none of the console's markup and
    /// passes the rule for the wrong reason.
    ///
    /// The token stream is scanned for its string literals, which is exactly
    /// what the rule is about: a literal that renders an event attribute.
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        for literal in string_literals_in(&node.tokens) {
            self.inspect(&literal);
        }
        syn::visit::visit_macro(self, node);
    }
}

/// Every string literal in a token stream, including the ones nested in
/// groups.
///
/// A group is a macro argument — `format!("<button onclick=...>", value)` —
/// so the literal is inside the group, not beside it.
fn string_literals_in(tokens: &proc_macro2::TokenStream) -> Vec<syn::LitStr> {
    let mut found = Vec::new();
    collect_string_literals(tokens, &mut found);
    found
}

fn collect_string_literals(tokens: &proc_macro2::TokenStream, found: &mut Vec<syn::LitStr>) {
    for token in tokens.clone() {
        match token {
            TokenTree::Group(group) => collect_string_literals(&group.stream(), found),
            TokenTree::Literal(literal) => {
                if let Ok(syn::Lit::Str(string)) = syn::parse2::<syn::Lit>(
                    proc_macro2::TokenStream::from(TokenTree::Literal(literal)),
                ) {
                    found.push(string);
                }
            }
            _ => {}
        }
    }
}

impl MarkupVisitor {
    fn inspect(&mut self, literal: &syn::LitStr) {
        let value = literal.value();
        if !looks_like_markup(&value) {
            return;
        }
        for (attribute, offset) in event_attributes(&value) {
            let (line, column) = position_of(literal, offset);
            self.handlers.push(InlineHandler {
                attribute,
                line,
                column,
            });
        }
    }
}

/// Does this string contain a tag?
fn looks_like_markup(value: &str) -> bool {
    let bytes = value.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] != b'<' {
            continue;
        }
        let next = bytes.get(index + 1).copied();
        if matches!(next, Some(byte) if byte.is_ascii_alphabetic() || byte == b'/') {
            return true;
        }
    }
    false
}

/// The event attributes in a markup string, with the byte offset each starts
/// at.
///
/// An attribute has to sit inside a tag and outside a quoted value: the `on`
/// has to be preceded by whitespace and followed by `=`, and a `>` inside an
/// attribute value does not close the tag. That excludes a sentence that
/// happens to contain the word `onclick` between two tags, and an attribute
/// value that mentions another attribute by name.
fn event_attributes(value: &str) -> Vec<(String, usize)> {
    let bytes = value.as_bytes();
    let mut found = Vec::new();
    let mut in_tag = false;
    let mut quote: Option<u8> = None;
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(open) = quote {
            if byte == open {
                quote = None;
            }
            index += 1;
            continue;
        }
        if byte == b'"' || byte == b'\'' {
            if in_tag {
                quote = Some(byte);
            }
            index += 1;
            continue;
        }
        if byte == b'<' {
            in_tag = true;
            index += 1;
            continue;
        }
        if byte == b'>' {
            in_tag = false;
            index += 1;
            continue;
        }
        if in_tag && (byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r') {
            index += 1;
            continue;
        }
        if in_tag {
            let rest = &value[index..];
            if let Some(attribute) = EVENT_ATTRIBUTES
                .iter()
                .find(|attribute| starts_with_attribute(rest, attribute))
            {
                found.push(((*attribute).to_string(), index));
                index += attribute.len();
                continue;
            }
        }
        index += 1;
    }
    found
}

/// Does the text start with `name` used as an attribute — that is, `name=`
/// with no room for a longer name in between?
fn starts_with_attribute(text: &str, name: &str) -> bool {
    let Some(rest) = text.strip_prefix(name) else {
        return false;
    };
    let mut characters = rest.chars();
    match characters.next() {
        Some('=') => true,
        // `onclickfoo=` is a different attribute, not this one.
        Some(character)
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' =>
        {
            false
        }
        // `onclick =` with spaces around the equals sign is still an attribute.
        Some(character) if character.is_whitespace() => rest.trim_start().starts_with('='),
        _ => false,
    }
}

/// The 1-based line and column of a byte offset inside a string literal.
///
/// `syn` reports the position of the literal's own start, so the offset inside
/// the value is added to it. The column is counted in characters rather than
/// bytes, because that is what an editor shows.
fn position_of(literal: &syn::LitStr, offset: usize) -> (usize, usize) {
    let start = literal.span().start();
    let value = literal.value();
    let prefix = &value[..offset.min(value.len())];
    let extra_lines = prefix.matches('\n').count();
    let last_line = prefix.rsplit('\n').next().unwrap_or("");
    // A raw string starts with `r` and one or more `#` before its quote, and
    // the value begins after that prefix rather than at the literal's own
    // start.
    let literal_text = literal.token().to_string();
    let opening = literal_text.find('"').map_or(1, |index| index + 1);
    let column = start.column + opening + last_line.chars().count();
    (start.line + extra_lines, column + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handlers_in(source: &str) -> Vec<InlineHandler> {
        let parsed = syn::parse_file(source).expect("the fixture parses");
        find_inline_handlers(&parsed)
    }

    #[test]
    fn an_inline_handler_in_markup_is_found() {
        let handlers = handlers_in(
            r##"fn render() -> String {
                r#"<button onclick="return confirm('x')">Delete</button>"#.to_string()
            }"##,
        );
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].attribute, "onclick");
    }

    #[test]
    fn prose_about_an_attribute_is_not_an_offence() {
        let handlers = handlers_in(
            r#"fn render() -> String {
                "The onchange attribute is blocked by the content security policy.".to_string()
            }"#,
        );
        assert!(handlers.is_empty(), "{handlers:?}");
    }

    #[test]
    fn a_comment_mentioning_the_attribute_is_not_an_offence() {
        // The comment is not part of the syntax tree, so it cannot be one.
        let handlers = handlers_in(
            r#"fn render() -> String {
                // this used to carry onclick="return confirm(...)"
                "<p>ok</p>".to_string()
            }"#,
        );
        assert!(handlers.is_empty(), "{handlers:?}");
    }

    #[test]
    fn a_longer_attribute_name_is_not_confused_with_the_short_one() {
        let handlers = handlers_in(
            r##"fn render() -> String { r#"<div onclickable="x"></div>"#.to_string() }"##,
        );
        assert!(handlers.is_empty(), "{handlers:?}");
    }

    #[test]
    fn an_attribute_value_naming_another_attribute_is_not_an_offence() {
        let handlers = handlers_in(
            r##"fn render() -> String { r#"<div title="use onclick= carefully"></div>"#.to_string() }"##,
        );
        assert!(handlers.is_empty(), "{handlers:?}");
    }

    #[test]
    fn the_position_points_at_the_attribute() {
        let handlers = handlers_in(
            r##"fn render() -> String {
    r#"<button class="x" onclick="go()">Go</button>"#.to_string()
}"##,
        );
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].line, 2);
        // The attribute starts after `<button class="x" ` on the second line.
        assert!(handlers[0].column > 1, "{handlers:?}");
    }

    #[test]
    fn markup_built_from_a_delegated_listener_is_allowed() {
        let handlers = handlers_in(
            r##"fn render() -> String {
                r#"<button type="button" data-role-preset="1">Pick</button>"#.to_string()
            }"##,
        );
        assert!(handlers.is_empty(), "{handlers:?}");
    }

    #[test]
    fn a_handler_inside_a_format_argument_is_found() {
        // A macro argument is a token stream, not an expression, so this is
        // the case the default visitor misses entirely.
        let handlers = handlers_in(
            r##"fn render() -> String {
                format!(r#"<button onclick="go()">Go</button>"#)
            }"##,
        );
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].attribute, "onclick");
    }

    #[test]
    fn a_handler_with_escaped_quotes_in_a_format_argument_is_found() {
        // The escaped form is what the pages actually render: the literal is
        // read after the escapes are resolved, so the attribute is spelled
        // with plain quotes in the value.
        let handlers = handlers_in(
            r##"fn render() -> String {
                format!("<button onchange=\"go()\">Go</button>", 1)
            }"##,
        );
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].attribute, "onchange");
    }
}
