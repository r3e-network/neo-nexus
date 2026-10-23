//! The shared shape of a form that comes back with what the operator typed.
//!
//! A form that fails validation has two ways to report the failure. The
//! common one is a redirect carrying a flash message: the browser lands on a
//! freshly rendered page, the operator's input is gone, and the message says
//! what went wrong somewhere above the form they now have to retype. The
//! other is to render the form again, with the input still in it and the
//! error beside the field that caused it.
//!
//! The second is what an operator wants, and the node editor has done it for
//! a long time. This module carries the two pieces it needs so the other
//! forms can have the same behaviour without each one inventing its own:
//!
//! - [`FieldErrors`] — which field failed and why, keyed by the field's name.
//! - [`FormDraft`] — the posted values, kept as text so the form can be
//!   rendered back exactly as it was submitted.
//!
//! What this deliberately does *not* do is provide a generic form engine.
//! The forms in this console share no field names, no field types and no
//! truth source: a node draft is the submission itself, an upgrade policy is
//! read back from the workspace, and a signer policy is read from a remote
//! custody service. A shared abstraction over all three would be a set of
//! runtime tables where three plain structs stand today. What they can share
//! is the vocabulary — this module.

use std::collections::BTreeMap;

/// Which field failed and why, keyed by the field's `name`.
///
/// The key is the same string the input carries, so an error lands next to
/// the control that caused it rather than in a banner above the form. The
/// key `general` is reserved for a failure that belongs to no single field.
pub type FieldErrors = BTreeMap<&'static str, String>;

/// The key used for a failure that belongs to the form as a whole.
pub const GENERAL: &str = "general";

/// The message for a field, if that field failed.
pub fn error_for<'a>(errors: &'a FieldErrors, key: &str) -> Option<&'a str> {
    errors.get(key).map(String::as_str)
}

/// How many fields need attention, and how to say so.
///
/// One field reads as one thing; two fields read as a list. The sentence is
/// the first thing a screen reader announces from the summary, so it has to
/// be right. The `general` key is never counted — it is not a field.
pub fn summary(errors: &FieldErrors) -> String {
    let fields = errors.keys().filter(|key| **key != GENERAL).count();
    match fields {
        0 => String::new(),
        1 => "One field needs attention.".to_string(),
        count => format!("{count} fields need attention."),
    }
}

/// The banner a redrawn form carries above itself.
///
/// Two things go in it and they are not interchangeable: a count of the fields
/// that failed, and — when the failure belongs to no single field — the message
/// itself, quoted rather than counted. "One field needs attention" over a
/// sentence about the policy as a whole would name a field that is not wrong.
///
/// This lives here because three pages had grown their own version and they
/// disagreed: one counted `general` as a field, one dropped the count, and one
/// never rendered `general` at all — so a whole-policy refusal came back as a
/// page with the input intact and no reason on it.
pub fn banner(errors: &FieldErrors) -> String {
    if errors.is_empty() {
        return String::new();
    }
    let count = summary(errors);
    let header = if count.is_empty() {
        String::new()
    } else {
        super::html::notice("danger", &count)
    };
    let general = errors
        .get(GENERAL)
        .map(|message| super::html::notice("danger", message))
        .unwrap_or_default();
    format!("{header}{general}")
}

/// A form's posted values, kept so the form can be rendered back.
///
/// The values are the raw text the browser posted, not parsed numbers: a
/// rejected submission has to come back showing what the operator actually
/// typed, including the value they got wrong.
pub trait FormDraft: Sized {
    /// The field's current value, as the form should show it.
    fn value(&self, field: &str) -> &str;

    /// The checkbox fields that were ticked.
    fn checked(&self, field: &str) -> bool;
}

/// A draft built from named values.
///
/// A form with a handful of fields does not need a struct with a field per
/// input; this carries the same values without the ceremony, and it lets a
/// page build the draft from whatever it already has.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Draft {
    values: BTreeMap<String, String>,
    checked: Vec<String>,
}

impl Draft {
    /// A draft with nothing in it.
    pub fn empty() -> Self {
        Self::default()
    }

    /// A draft carrying one value.
    pub fn with(mut self, field: &str, value: &str) -> Self {
        self.values.insert(field.to_string(), value.to_string());
        self
    }

    /// A draft carrying a ticked checkbox.
    pub fn with_checked(mut self, field: &str) -> Self {
        if !self.checked.iter().any(|name| name == field) {
            self.checked.push(field.to_string());
        }
        self
    }

    /// Every value in the draft.
    pub fn values(&self) -> &BTreeMap<String, String> {
        &self.values
    }
}

impl FormDraft for Draft {
    fn value(&self, field: &str) -> &str {
        self.values.get(field).map_or("", String::as_str)
    }

    fn checked(&self, field: &str) -> bool {
        self.checked.iter().any(|name| name == field)
    }
}

/// Build the draft a re-rendered form needs from a set of named values.
///
/// The checkbox list is separate because a browser posts nothing at all for
/// an unticked box — the field's absence is the answer.
pub fn draft_from(values: &[(&str, &str)], checked: &[&str]) -> Draft {
    let mut draft = Draft::empty();
    for (field, value) in values {
        draft = draft.with(field, value);
    }
    for field in checked {
        draft = draft.with_checked(field);
    }
    draft
}
