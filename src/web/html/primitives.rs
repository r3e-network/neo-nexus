//! Fundamental HTML building blocks: escaping, URL encoding, badges, tables, and notices.

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// A live node status badge. `data-node-status` lets the polling script swap
/// the class as states change between reloads.
pub fn status_badge(status: &str) -> String {
    let class = match status {
        "Running" => "badge running",
        "Starting" => "badge starting",
        "Error" => "badge error",
        _ => "badge stopped",
    };
    format!(r#"<span class="{class}" data-node-status="{status}">{status}</span>"#)
}

/// A bare status indicator dot, no label. Compact node rows lead with this and
/// close with the full [`status_badge`] pill; the two share the status palette.
pub fn status_dot(status: &str) -> String {
    let class = match status {
        "Running" => "status-dot running",
        "Starting" => "status-dot starting",
        "Error" => "status-dot error",
        _ => "status-dot stopped",
    };
    format!(r#"<span class="{class}" data-node-status="{status}" aria-hidden="true"></span>"#)
}

pub fn flash_query(message: &str) -> String {
    format!("?flash={}", urlencoding_lite(message))
}

/// How a flash banner reads.
///
/// A banner that looks the same whether the operation succeeded or failed is
/// worse than no banner at all: the operator has to read the sentence to find
/// out which one it was, and a sentence that begins "failed:" is easy to miss
/// in a page of tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlashKind {
    /// The operation did what it was asked to do.
    #[default]
    Ok,
    /// The operation did not complete, and the banner says why.
    Danger,
}

/// The outcome of one control, carried in the query string of the redirect
/// that follows it.
///
/// The message and its severity travel together: the redirect writes
/// `?flash=<message>&flash_kind=danger`, and the page that reads it back
/// reconstructs both. A page cannot accidentally render a failure in the
/// success colour, because the severity is part of the value rather than
/// something each caller has to remember to pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flash {
    pub message: String,
    pub kind: FlashKind,
}

impl Flash {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FlashKind::Ok,
        }
    }

    pub fn danger(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FlashKind::Danger,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.message.trim().is_empty()
    }

    /// The query-string fragment that carries this banner.
    pub fn query(&self) -> String {
        let mut query = format!("flash={}", urlencoding_lite(&self.message));
        if self.kind == FlashKind::Danger {
            query.push_str("&flash_kind=danger");
        }
        query
    }

    /// A redirect back to `path` carrying this banner.
    ///
    /// The separator is `?` or `&` depending on whether the path already
    /// carries a query, so a caller does not have to know which one it is
    /// building.
    pub fn redirect(&self, path: &str) -> String {
        let separator = if path.contains('?') { '&' } else { '?' };
        format!("{path}{separator}{}", self.query())
    }
}

/// A plain string is a successful outcome.
///
/// Every page that renders a layout has to pass *something* for the banner
/// slot, and most of them pass the empty string. Making that the success
/// case means the pages that have no outcome to report keep working without
/// knowing this type exists.
impl From<&str> for Flash {
    fn from(message: &str) -> Self {
        Flash::ok(message)
    }
}

impl From<String> for Flash {
    fn from(message: String) -> Self {
        Flash::ok(message)
    }
}

impl From<&String> for Flash {
    fn from(message: &String) -> Self {
        Flash::ok(message.clone())
    }
}

impl From<&Flash> for Flash {
    fn from(flash: &Flash) -> Self {
        flash.clone()
    }
}

pub fn flash_banner(flash: &Flash) -> String {
    if flash.is_empty() {
        String::new()
    } else {
        // A failure is announced assertively: the operator pressed a button and
        // is waiting for an answer, so it should interrupt rather than queue
        // politely behind whatever the screen reader is still reading.
        match flash.kind {
            FlashKind::Ok => format!(
                r#"<div class="flash" role="status" aria-live="polite">{}</div>"#,
                escape(&flash.message)
            ),
            FlashKind::Danger => format!(
                r#"<div class="flash danger" role="alert" aria-live="assertive">{}</div>"#,
                escape(&flash.message)
            ),
        }
    }
}

/// Minimal percent-encoding for query values (flash messages and ids).
pub fn urlencoding_lite(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// Percent-decode a query value — the exact inverse of [`urlencoding_lite`].
/// Anything the redirect escapes has to come back as the operator's text, or
/// a blocked launch reads as `%E2%80%94` where the reason should be.
pub fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        // Read the pair from the byte slice: a raw `%` followed by a UTF-8
        // continuation byte must fall through, not panic slicing the str.
        let escape = match bytes[index] {
            b'%' if index + 3 <= bytes.len() => std::str::from_utf8(&bytes[index + 1..index + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok()),
            _ => None,
        };
        match escape {
            Some(byte) => {
                decoded.push(byte);
                index += 3;
            }
            None => {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

pub fn query_value(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == key).then(|| percent_decode(value))
    })
}

/// The flash message a control handler left in the query string.
///
/// The severity comes back with it: a handler that failed writes
/// `flash_kind=danger` beside the message, and the page renders the banner in
/// the failure colour rather than the success one.
pub fn flash(query: Option<&str>) -> Flash {
    let message = query_value(query, "flash").unwrap_or_default();
    let kind = match query_value(query, "flash_kind").as_deref() {
        Some("danger") | Some("error") => FlashKind::Danger,
        _ => FlashKind::Ok,
    };
    Flash { message, kind }
}

/// A row of stat tiles — a value over its label.
pub fn cards(items: &[(&str, String)]) -> String {
    let tiles = items
        .iter()
        .map(|(label, value)| {
            format!(
                r#"<div class="card"><div class="num">{}</div><div class="lbl">{}</div></div>"#,
                escape(value),
                escape(label)
            )
        })
        .collect::<String>();
    format!(r#"<div class="cards">{tiles}</div>"#)
}

/// A muted line for the states where a page has nothing to show yet.
pub fn note(message: &str) -> String {
    format!(r#"<p class="muted">{}</p>"#, escape(message))
}

/// An escaped table cell holding operator data.
pub fn cell(value: &str) -> String {
    format!("<td>{}</td>", escape(value))
}

/// A cell holding markup the page already rendered, such as a badge or a form.
pub fn raw_cell(markup: &str) -> String {
    format!("<td>{markup}</td>")
}

/// A table row assembled from [`cell`] and [`raw_cell`] output.
pub fn row(cells: &[String]) -> String {
    format!("<tr>{}</tr>", cells.concat())
}

/// A header row followed by the given rows, which arrive as complete `<tr>`
/// markup from [`row`].
///
/// Wrapped in a scroll container: the widest tables (runtimes, federation) are
/// far wider than a laptop column, and letting the table itself overflow beats
/// squeezing eight columns into unreadable slivers.
pub fn table(headers: &[&str], rows: &[String]) -> String {
    let head = headers
        .iter()
        .map(|header| format!(r#"<th scope="col">{}</th>"#, escape(header)))
        .collect::<String>();
    let label = format!("Scrollable data table: {}", headers.join(", "));
    format!(
        r#"<div class="scroll-x" role="region" aria-label="{label}" tabindex="0"><table><thead><tr>{head}</tr></thead><tbody>{rows}</tbody></table></div>"#,
        label = escape(&label),
        rows = rows.concat(),
    )
}

/// A status line that is not a transient flash: `ok`, `warn` or `danger`.
pub fn notice(kind: &str, message: &str) -> String {
    let class = match kind {
        "warn" | "danger" => format!("notice {kind}"),
        _ => "notice".to_string(),
    };
    format!(
        r#"<div class="{class}">{}</div>"#,
        escape(message),
        class = class
    )
}

/// A callout that work is underway. The same amber `notice warn` surface, but
/// announced with `role="status"` so a running background job is heard, not
/// only seen; the message is escaped like every other operator-facing string.
pub fn loading_callout(message: &str) -> String {
    format!(
        r#"<div class="notice warn" role="status">{}</div>"#,
        escape(message)
    )
}

/// A pending marker for a form whose submission takes a while.
///
/// The mark starts hidden and is revealed by the bundled script the moment the
/// form is submitted, so the operator sees that the click registered. Without
/// scripting the form submits exactly as before — the marker simply never
/// appears — which is why it is a marker rather than a disabled button.
pub fn pending_marker(message: &str) -> String {
    format!(
        r#"<span class="pending" data-pending role="status" aria-live="polite">{}</span>"#,
        escape(message)
    )
}

/// What a page shows when there is nothing to show, with the action that would
/// fix it. An empty list with no way forward is a dead end, not a state.
pub fn empty_state(title: &str, body: &str, actions: &str) -> String {
    empty_state_at(2, title, body, actions)
}

/// The same empty state, at a heading level that fits where it sits.
///
/// The default is `h2`, which is right for a section of a page. A block
/// inside a panel that is itself titled `h3` needs `h3` too: a heading that
/// jumps *up* a level is not a heading the screen reader can follow.
pub fn empty_state_at(level: u8, title: &str, body: &str, actions: &str) -> String {
    let heading = match level {
        3 => "h3",
        4 => "h4",
        _ => "h2",
    };
    format!(
        r#"<div class="empty"><{heading}>{title}</{heading}><p>{body}</p><div class="actions">{actions}</div></div>"#,
        heading = heading,
        title = escape(title),
        body = escape(body),
        actions = actions,
    )
}

/// A block of log or report text. The content is escaped; the browser preserves
/// the line breaks the node wrote.
pub fn text_block(content: &str) -> String {
    format!("<pre>{}</pre>", escape(content))
}
