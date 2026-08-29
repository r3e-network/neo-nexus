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
    filter_form_with_hidden(action, &[], fields)
}

/// The same, with hidden parameters that identify the page's subject — a node
/// id, say — so applying a filter does not silently drop back to the first one.
pub fn filter_form_with_hidden(
    action: &str,
    hidden: &[(&str, &str)],
    fields: &[(&str, &str)],
) -> String {
    let hidden = hidden
        .iter()
        .map(|(name, value)| {
            format!(
                r#"<input type="hidden" name="{}" value="{}">"#,
                escape(name),
                escape(value)
            )
        })
        .collect::<String>();
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
        r#"<form class="filters" method="get" action="{action}">{hidden}{inputs}<button type="submit">Apply</button></form>"#,
        action = escape(action)
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

/// A labelled text input. A struct rather than five positional arguments:
/// label, name, value, help and error are easy to pass in the wrong order, and
/// a swapped pair still compiles.
#[derive(Default)]
pub struct TextField<'a> {
    pub label: &'a str,
    pub name: &'a str,
    pub value: &'a str,
    pub help: Option<&'a str>,
    pub error: Option<&'a str>,
    /// Paths, versions and digests read better in the console face.
    pub monospace: bool,
    /// Take the whole grid row instead of one column.
    pub full_width: bool,
    pub placeholder: Option<&'a str>,
}

/// A labelled dropdown over server-provided options.
#[derive(Default)]
pub struct ChoiceField<'a> {
    pub label: &'a str,
    pub name: &'a str,
    pub options: &'a [String],
    pub selected: &'a str,
    pub help: Option<&'a str>,
    pub error: Option<&'a str>,
    /// Submit the form when the value changes, reporting this action. Enhancement
    /// only: an explicit submit button with the same value works without scripting.
    pub auto_submit: Option<&'a str>,
    pub full_width: bool,
}

impl TextField<'_> {
    pub fn render(&self) -> String {
        let id = format!("f-{}", self.name);
        let control_class = if self.monospace {
            r#" class="mono""#
        } else {
            ""
        };
        let placeholder = self
            .placeholder
            .map(|text| format!(r#" placeholder="{}""#, escape(text)))
            .unwrap_or_default();
        format!(
            r#"<label class="field{classes}" for="{id}"><span>{label}</span><input id="{id}" name="{name}" value="{value}"{control_class}{placeholder}{described}>{messages}</label>"#,
            classes = field_classes(self.error, self.full_width),
            id = id,
            label = escape(self.label),
            name = escape(self.name),
            value = escape(self.value),
            control_class = control_class,
            placeholder = placeholder,
            described = describe(&id, self.help, self.error),
            messages = field_messages(&id, self.help, self.error),
        )
    }
}

impl ChoiceField<'_> {
    pub fn render(&self) -> String {
        let id = format!("f-{}", self.name);
        let options = self
            .options
            .iter()
            .map(|option| {
                let chosen = if option == self.selected {
                    " selected"
                } else {
                    ""
                };
                format!(
                    r#"<option value="{}"{chosen}>{}</option>"#,
                    escape(option),
                    escape(option)
                )
            })
            .collect::<String>();
        format!(
            r#"<label class="field{classes}" for="{id}"><span>{label}</span><select id="{id}" name="{name}"{auto}{described}>{options}</select>{messages}</label>"#,
            classes = field_classes(self.error, self.full_width),
            id = id,
            label = escape(self.label),
            name = escape(self.name),
            auto = match self.auto_submit {
                Some(action) => format!(r#" data-autosubmit="{}""#, escape(action)),
                None => String::new(),
            },
            described = describe(&id, self.help, self.error),
            options = options,
            messages = field_messages(&id, self.help, self.error),
        )
    }
}

fn field_classes(error: Option<&str>, full_width: bool) -> String {
    let mut classes = String::new();
    if error.is_some() {
        classes.push_str(" invalid");
    }
    if full_width {
        classes.push_str(" span-all");
    }
    classes
}

fn describe(id: &str, help: Option<&str>, error: Option<&str>) -> String {
    let mut parts = Vec::new();
    if error.is_some() {
        parts.push(format!("{id}-error"));
    }
    if help.is_some() {
        parts.push(format!("{id}-help"));
    }
    if parts.is_empty() {
        return String::new();
    }
    format!(r#" aria-describedby="{}""#, parts.join(" "))
}

fn field_messages(id: &str, help: Option<&str>, error: Option<&str>) -> String {
    let error = error
        .map(|message| {
            format!(
                r#"<span class="error" id="{id}-error" role="alert">{}</span>"#,
                escape(message)
            )
        })
        .unwrap_or_default();
    let help = help
        .map(|message| {
            format!(
                r#"<span class="help" id="{id}-help">{}</span>"#,
                escape(message)
            )
        })
        .unwrap_or_default();
    format!("{error}{help}")
}

/// Title, one line of context, and the actions that apply to the page.
pub fn page_head(title: &str, subtitle: &str, actions: &str) -> String {
    let subtitle = if subtitle.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div class="sub">{}</div>"#, escape(subtitle))
    };
    let actions = if actions.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div class="toolbar">{actions}</div>"#)
    };
    format!(
        r#"<div class="page-head"><div><h1>{title}</h1>{subtitle}</div>{actions}</div>"#,
        title = escape(title),
        subtitle = subtitle,
        actions = actions,
    )
}

/// A trail back to the list, so a detail or edit page is never a dead end. An
/// empty href renders the current location as plain text.
pub fn breadcrumb(items: &[(&str, &str)]) -> String {
    let parts = items
        .iter()
        .map(|(label, href)| {
            if href.is_empty() {
                format!("<span>{}</span>", escape(label))
            } else {
                format!(r#"<a href="{}">{}</a>"#, escape(href), escape(label))
            }
        })
        .collect::<Vec<_>>();
    format!(
        r#"<nav class="breadcrumb">{}</nav>"#,
        parts.join(r#"<span class="sep">/</span>"#)
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

/// A labelled text/number input carrying its current value.
pub fn text_field(label: &str, name: &str, value: &str) -> String {
    TextField {
        label,
        name,
        value,
        ..TextField::default()
    }
    .render()
}

/// A labelled dropdown whose options come from the server, so an operator can
/// only submit a value the domain accepts.
pub fn choice_field(label: &str, name: &str, options: &[String], selected: &str) -> String {
    ChoiceField {
        label,
        name,
        options,
        selected,
        ..ChoiceField::default()
    }
    .render()
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
