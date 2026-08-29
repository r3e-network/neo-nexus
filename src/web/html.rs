//! HTML building blocks: escaping, the shared page shell, and the small
//! component helpers the pages compose. All interpolated values flow through
//! [`escape`] so operator data can never inject markup.

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

pub fn flash_query(message: &str) -> String {
    format!("?flash={}", urlencoding_lite(message))
}

pub fn flash_banner(message: &str) -> String {
    if message.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div class="flash">{}</div>"#, escape(message))
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
pub fn table(headers: &[&str], rows: &[String]) -> String {
    let head = headers
        .iter()
        .map(|header| format!("<th>{}</th>", escape(header)))
        .collect::<String>();
    format!("<table><tr>{head}</tr>{}</table>", rows.concat())
}

/// A GET form for page filters. Plain query parameters keep filtering usable
/// without JavaScript and make the result linkable.
pub fn filter_form(action: &str, fields: &[(&str, &str)]) -> String {
    let inputs = fields
        .iter()
        .map(|(name, value)| {
            format!(
                r#"<label class="field"><span>{}</span><input name="{}" value="{}"></label>"#,
                escape(name),
                escape(name),
                escape(value)
            )
        })
        .collect::<String>();
    format!(
        r#"<form class="filters" method="get" action="{}">{inputs}<button type="submit">Apply</button></form>"#,
        escape(action)
    )
}

/// A POST form carrying hidden fields and one submit button — the shape every
/// workbench control uses so it works without JavaScript.
pub fn control_form(action: &str, fields: &[(&str, &str)], label: &str) -> String {
    let hidden = fields
        .iter()
        .map(|(name, value)| {
            format!(
                r#"<input type="hidden" name="{}" value="{}">"#,
                escape(name),
                escape(value)
            )
        })
        .collect::<String>();
    format!(
        r#"<form method="post" action="{action}">{hidden}<button type="submit">{label}</button></form>"#,
        action = escape(action),
        label = escape(label)
    )
}

/// A dropdown whose options are values the server produced, so an operator can
/// only pick something the workspace actually has.
pub fn select(name: &str, options: &[String], selected: &str) -> String {
    let items = options
        .iter()
        .map(|option| {
            let chosen = if option == selected { " selected" } else { "" };
            format!(
                r#"<option value="{}"{chosen}>{}</option>"#,
                escape(option),
                escape(option)
            )
        })
        .collect::<String>();
    format!(r#"<select name="{}">{items}</select>"#, escape(name))
}

/// A labelled text/number input carrying its current value.
pub fn text_field(label: &str, name: &str, value: &str) -> String {
    format!(
        r#"<label class="field"><span>{}</span><input name="{}" value="{}"></label>"#,
        escape(label),
        escape(name),
        escape(value)
    )
}

/// A labelled dropdown whose options come from the server, so an operator can
/// only submit a value the domain accepts.
pub fn choice_field(label: &str, name: &str, options: &[String], selected: &str) -> String {
    format!(
        r#"<label class="field"><span>{}</span>{}</label>"#,
        escape(label),
        select(name, options, selected)
    )
}

/// A block of log or report text. The content is escaped; the browser preserves
/// the line breaks the node wrote.
pub fn text_block(content: &str) -> String {
    format!("<pre>{}</pre>", escape(content))
}

/// The page shell: grouped sidebar navigation, a header, and the page body.
/// The sidebar comes from [`super::nav`], so a new page appears everywhere once
/// its route exists.
pub fn layout(title: &str, active: &str, flash: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · NeoNexus</title>
<style>{css}</style>
</head>
<body>
<div class="shell">
<nav class="sidebar">
<div class="brand">NeoNexus</div>
{nav}
<form method="post" action="/logout"><button class="nav-item logout" type="submit">Sign out</button></form>
</nav>
<main class="content">
{flash_banner}
{body}
</main>
</div>
<script>{script}</script>
</body>
</html>"#,
        title = escape(title),
        css = super::assets::CSS,
        script = super::assets::SCRIPT,
        nav = super::nav::render(active),
        flash_banner = flash_banner(flash),
        body = body,
    )
}

#[cfg(test)]
#[path = "../../tests/unit/web/html/tests.rs"]
mod tests;
