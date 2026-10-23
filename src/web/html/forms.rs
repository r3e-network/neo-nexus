//! HTML form builders: filter forms, control buttons, text fields, and dropdowns.

use super::primitives::escape;

/// A typed GET-filter control. Page code supplies the human label separately
/// from the query parameter, so operators never have to decipher internal
/// names such as `q` or `high_cpu`.
pub enum FilterControl<'a> {
    Search {
        label: &'a str,
        name: &'a str,
        value: &'a str,
        placeholder: &'a str,
    },
    Text {
        label: &'a str,
        name: &'a str,
        value: &'a str,
    },
    Select {
        label: &'a str,
        name: &'a str,
        selected: &'a str,
        options: &'a [(&'a str, &'a str)],
    },
    Checkbox {
        label: &'a str,
        name: &'a str,
        checked: bool,
    },
    Number {
        label: &'a str,
        name: &'a str,
        value: &'a str,
        min: usize,
        max: usize,
    },
}

/// A progressive, bookmarkable filter form with explicit control types.
pub fn typed_filter_form(
    action: &str,
    hidden: &[(&str, &str)],
    controls: &[FilterControl<'_>],
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
    let fields = controls
        .iter()
        .enumerate()
        .map(|(index, control)| render_filter_control(action, index, control))
        .collect::<String>();
    format!(
        r#"<form class="filters" method="get" action="{action}">{hidden}{fields}<button type="submit">Apply</button></form>"#,
        action = escape(action),
    )
}

fn render_filter_control(action: &str, index: usize, control: &FilterControl<'_>) -> String {
    let (label, name) = match control {
        FilterControl::Search { label, name, .. }
        | FilterControl::Text { label, name, .. }
        | FilterControl::Select { label, name, .. }
        | FilterControl::Checkbox { label, name, .. }
        | FilterControl::Number { label, name, .. } => (*label, *name),
    };
    let id = filter_control_id(action, name, index);
    match control {
        FilterControl::Search {
            value, placeholder, ..
        } => format!(
            r#"<label class="field" for="{id}"><span>{label}</span><input id="{id}" type="search" name="{name}" value="{value}" placeholder="{placeholder}"></label>"#,
            label = escape(label),
            name = escape(name),
            value = escape(value),
            placeholder = escape(placeholder),
        ),
        FilterControl::Text { value, .. } => format!(
            r#"<label class="field" for="{id}"><span>{label}</span><input id="{id}" name="{name}" value="{value}"></label>"#,
            label = escape(label),
            name = escape(name),
            value = escape(value),
        ),
        FilterControl::Select {
            selected, options, ..
        } => {
            let options = options
                .iter()
                .map(|(value, option_label)| {
                    let chosen = if value.eq_ignore_ascii_case(selected.trim()) {
                        " selected"
                    } else {
                        ""
                    };
                    format!(
                        r#"<option value="{}"{chosen}>{}</option>"#,
                        escape(value),
                        escape(option_label)
                    )
                })
                .collect::<String>();
            format!(
                r#"<label class="field" for="{id}"><span>{label}</span><select id="{id}" name="{name}">{options}</select></label>"#,
                label = escape(label),
                name = escape(name),
            )
        }
        FilterControl::Checkbox { checked, .. } => format!(
            r#"<label class="filter-check" for="{id}"><input id="{id}" type="checkbox" name="{name}" value="1"{checked}><span>{label}</span></label>"#,
            label = escape(label),
            name = escape(name),
            checked = if *checked { " checked" } else { "" },
        ),
        FilterControl::Number {
            value, min, max, ..
        } => format!(
            r#"<label class="field" for="{id}"><span>{label}</span><input id="{id}" type="number" inputmode="numeric" name="{name}" value="{value}" min="{min}" max="{max}"></label>"#,
            label = escape(label),
            name = escape(name),
            value = escape(value),
        ),
    }
}

fn filter_control_id(action: &str, name: &str, index: usize) -> String {
    let page = action
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    let name = name.replace('_', "-");
    format!("filter-{page}-{name}-{index}")
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
                escape(&filter_label(name)),
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

fn filter_label(name: &str) -> String {
    match name {
        "q" | "query" => "Search".to_string(),
        "lines" => "Rows".to_string(),
        "high_cpu" => "High CPU".to_string(),
        "high_memory" => "High memory".to_string(),
        _ => name
            .split('_')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut characters = part.chars();
                characters.next().map_or_else(String::new, |first| {
                    format!("{}{}", first.to_ascii_uppercase(), characters.as_str())
                })
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// A POST form carrying hidden fields and one submit button — the shape every
/// workbench control uses so it works without JavaScript.
pub fn control_form(action: &str, fields: &[(&str, &str)], label: &str) -> String {
    control_form_with_class(action, fields, label, "")
}

/// A destructive control with the same no-script POST behavior as
/// [`control_form`], but a visibly distinct button.
pub fn danger_control_form(action: &str, fields: &[(&str, &str)], label: &str) -> String {
    control_form_with_class(action, fields, label, "danger")
}

/// A control for an action that takes a while, with a marker that says so once
/// the form is submitted.
///
/// The marker is invisible until the bundled script sees the submission, so a
/// browser without scripting sees exactly the control it always saw. It sits
/// inside the form, because that is what the styling keys off.
pub fn pending_control_form(
    action: &str,
    fields: &[(&str, &str)],
    label: &str,
    message: &str,
) -> String {
    let hidden = hidden_fields(fields);
    format!(
        r#"<form method="post" action="{action}">{hidden}<button type="submit">{label}</button>{marker}</form>"#,
        action = escape(action),
        hidden = hidden,
        label = escape(label),
        marker = super::primitives::pending_marker(message),
    )
}

fn control_form_with_class(
    action: &str,
    fields: &[(&str, &str)],
    label: &str,
    class: &str,
) -> String {
    let hidden = hidden_fields(fields);
    let class = if class.is_empty() {
        String::new()
    } else {
        format!(r#" class="{}""#, escape(class))
    };
    format!(
        r#"<form method="post" action="{action}">{hidden}<button{class} type="submit">{label}</button></form>"#,
        action = escape(action),
        label = escape(label)
    )
}

fn hidden_fields(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(name, value)| {
            format!(
                r#"<input type="hidden" name="{}" value="{}">"#,
                escape(name),
                escape(value)
            )
        })
        .collect::<String>()
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
    /// Explicit document-unique id. The field name remains the fallback for
    /// pages that render that name only once.
    pub id: Option<&'a str>,
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
    /// Refuse an empty value.
    ///
    /// Only for fields the domain genuinely cannot do without. A field whose
    /// empty value means "assign one for me" — a port, a runtime version —
    /// must not be marked required, or the browser blocks a submission the
    /// server would have accepted.
    pub required: bool,
    /// The lower bound of a numeric value.
    pub min: Option<&'a str>,
    /// The upper bound of a numeric value.
    pub max: Option<&'a str>,
    /// The granularity a numeric value steps by.
    pub step: Option<&'a str>,
    /// The shape a value must match, as a regular expression.
    ///
    /// The pattern is matched against the whole value, so it needs the
    /// anchors the domain expects rather than a loose fragment.
    pub pattern: Option<&'a str>,
    /// The longest value the domain will accept.
    pub maxlength: Option<&'a str>,
    /// The input type. A numeric field gets `type="number"`, which gives the
    /// browser a spinner and rejects stray letters before submission.
    pub input_type: TextInputType,
}

/// The kind of control a [`TextField`] renders.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TextInputType {
    #[default]
    Text,
    Number,
}

impl TextInputType {
    fn attribute(self) -> &'static str {
        match self {
            TextInputType::Text => "text",
            TextInputType::Number => "number",
        }
    }
}

/// A labelled dropdown over server-provided options.
#[derive(Default)]
pub struct ChoiceField<'a> {
    /// Explicit document-unique id. Use this whenever multiple forms reuse a
    /// query/body field name on the same page.
    pub id: Option<&'a str>,
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
    /// Refuse an empty value.
    pub required: bool,
}

impl TextField<'_> {
    pub fn render(&self) -> String {
        let id = escape(
            self.id
                .map_or_else(|| format!("f-{}", self.name), str::to_string)
                .as_str(),
        );
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
            r#"<label class="field{classes}" for="{id}"><span>{label}</span><input id="{id}" type="{input_type}" name="{name}" value="{value}"{control_class}{placeholder}{required}{bounds}{invalid}{described}>{messages}</label>"#,
            classes = field_classes(self.error, self.full_width),
            id = id,
            label = escape(self.label),
            name = escape(self.name),
            value = escape(self.value),
            input_type = self.input_type.attribute(),
            control_class = control_class,
            placeholder = placeholder,
            required = if self.required { " required" } else { "" },
            bounds = self.bounds(),
            invalid = if self.error.is_some() {
                r#" aria-invalid="true""#
            } else {
                ""
            },
            described = describe(&id, self.help, self.error),
            messages = field_messages(&id, self.help, self.error),
        )
    }

    /// The constraint attributes, in a fixed order so a rendered page is
    /// stable between renders.
    fn bounds(&self) -> String {
        let mut bounds = String::new();
        if let Some(min) = self.min {
            bounds.push_str(&format!(r#" min="{}""#, escape(min)));
        }
        if let Some(max) = self.max {
            bounds.push_str(&format!(r#" max="{}""#, escape(max)));
        }
        if let Some(step) = self.step {
            bounds.push_str(&format!(r#" step="{}""#, escape(step)));
        }
        if let Some(pattern) = self.pattern {
            bounds.push_str(&format!(r#" pattern="{}""#, escape(pattern)));
        }
        if let Some(maxlength) = self.maxlength {
            bounds.push_str(&format!(r#" maxlength="{}""#, escape(maxlength)));
        }
        bounds
    }
}

impl ChoiceField<'_> {
    pub fn render(&self) -> String {
        let id = escape(
            self.id
                .map_or_else(|| format!("f-{}", self.name), str::to_string)
                .as_str(),
        );
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
            r#"<label class="field{classes}" for="{id}"><span>{label}</span><select id="{id}" name="{name}"{required}{auto}{invalid}{described}>{options}</select>{messages}</label>"#,
            classes = field_classes(self.error, self.full_width),
            id = id,
            label = escape(self.label),
            name = escape(self.name),
            required = if self.required { " required" } else { "" },
            auto = match self.auto_submit {
                Some(action) => format!(r#" data-autosubmit="{}""#, escape(action)),
                None => String::new(),
            },
            invalid = if self.error.is_some() {
                r#" aria-invalid="true""#
            } else {
                ""
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

/// A bounded number input.
///
/// The bounds belong to the domain, not to the browser: this is the same
/// range the server enforces, so the browser catches a mistake before the
/// round trip rather than after it.
pub fn number_field(
    label: &str,
    name: &str,
    value: &str,
    min: Option<&str>,
    max: Option<&str>,
) -> String {
    TextField {
        label,
        name,
        value,
        min,
        max,
        input_type: TextInputType::Number,
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
