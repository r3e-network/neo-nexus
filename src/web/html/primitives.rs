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

pub fn flash_banner(message: &str) -> String {
    if message.trim().is_empty() {
        String::new()
    } else {
        // `role="status"` so the outcome of a control is announced, not just
        // painted where the operator happened to be looking.
        format!(
            r#"<div class="flash" role="status" aria-live="polite">{}</div>"#,
            escape(message)
        )
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

/// The flash message a control handler left in the query string, or an empty
/// string. This is how a start/stop/restart result reaches the operator.
pub fn flash(query: Option<&str>) -> String {
    query_value(query, "flash").unwrap_or_default()
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

/// What a page shows when there is nothing to show, with the action that would
/// fix it. An empty list with no way forward is a dead end, not a state.
pub fn empty_state(title: &str, body: &str, actions: &str) -> String {
    format!(
        r#"<div class="empty"><h2>{title}</h2><p>{body}</p><div class="actions">{actions}</div></div>"#,
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
