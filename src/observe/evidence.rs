//! What was read, from where, and when — or why nothing was.
//!
//! Every number this workspace shows about a chain has to be traceable to a
//! response some node actually gave. That is not a style preference: the
//! console previously reported `1.2% (Active)`, `64.5 MB` and `3.2 ms` for
//! every running node, four alarms permanently reading `● OK`, and a fixed SVG
//! path captioned as a sixty-minute chart. Each was individually plausible.
//! Together they meant an operator could read a screen of green during an
//! incident and stop looking.
//!
//! The defence is a type rather than a rule, because a rule is one refactor
//! away from being forgotten. A value that was measured arrives as
//! [`Observation::Known`] carrying its [`Evidence`]; a value that was not
//! arrives as [`Observation::Unknown`] carrying the reason it is missing. There
//! is no third shape, and in particular there is no shape that renders as a
//! plausible default.

use std::fmt;

/// Where a single observed value came from.
///
/// Minted only inside [`crate::observe`] — the fields are private and the
/// constructor is crate-internal — so a page cannot manufacture provenance for
/// a number it invented.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    method: &'static str,
    field: &'static str,
    value: String,
    endpoint: String,
    sampled_at_unix: u64,
}

impl Evidence {
    /// Record what a node answered.
    ///
    /// `value` is the response as read, not as interpreted: when a derived
    /// figure later looks wrong, the operator needs the raw answer to tell a
    /// parsing bug from a node problem.
    pub(crate) fn recorded(
        method: &'static str,
        field: &'static str,
        value: impl Into<String>,
        endpoint: impl Into<String>,
        sampled_at_unix: u64,
    ) -> Self {
        Self {
            method,
            field,
            value: value.into(),
            endpoint: endpoint.into(),
            sampled_at_unix,
        }
    }

    pub fn method(&self) -> &str {
        self.method
    }

    pub fn field(&self) -> &str {
        self.field
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn sampled_at_unix(&self) -> u64 {
        self.sampled_at_unix
    }

    /// One line an operator can read: the call, the field, and what came back.
    pub fn summary(&self) -> String {
        format!("{}.{} = {}", self.method, self.field, self.value)
    }
}

/// Why a value is absent.
///
/// Absence is not one condition. "This client does not implement that method"
/// and "we asked and the node did not answer" and "we have never asked" lead an
/// operator to three different actions, so they are three variants rather than
/// one `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotSampled {
    /// No sample has ever been taken for this node.
    NeverSampled,
    /// The node has no RPC port, so there is nothing to ask.
    SamplingDisabled,
    /// A sample exists but is older than the workspace trusts.
    Stale { age_seconds: u64 },
    /// The node answered `-32601`: this client does not implement the method.
    ///
    /// Distinct from a failure. A Neo X node has no `getversion`, and treating
    /// that as an outage is what once made a perfectly healthy node read
    /// `Unreachable`.
    MethodUnsupported { method: &'static str },
    /// The call was attempted and did not produce a usable answer.
    CallFailed {
        method: &'static str,
        detail: String,
    },
}

impl fmt::Display for NotSampled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeverSampled => formatter.write_str("not checked yet"),
            Self::SamplingDisabled => {
                formatter.write_str("RPC is disabled on this node, so chain state cannot be read")
            }
            Self::Stale { age_seconds } => {
                write!(
                    formatter,
                    "last checked {age_seconds}s ago, which is too long to trust"
                )
            }
            Self::MethodUnsupported { method } => {
                write!(formatter, "this client does not implement {method}")
            }
            Self::CallFailed { method, detail } => write!(formatter, "{method} failed: {detail}"),
        }
    }
}

/// A value that was read, a value that was not, or a question this client
/// cannot be asked.
///
/// `Unanswerable` is deliberately separate from `Unknown`. "neo-go exposes no
/// plugin list" is a permanent property of the client and should read as such
/// forever; "we have not checked yet" should read as a gap that will close.
/// Rendering both as an em dash tells the operator to keep waiting for one of
/// them indefinitely.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation<T> {
    Known(T, Evidence),
    Unknown(NotSampled),
    Unanswerable(&'static str),
}

impl<T> Observation<T> {
    /// The value, if one was read. Use this only where absence is genuinely
    /// uninteresting — prefer matching, so the absent case has to be rendered.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Known(value, _) => Some(value),
            Self::Unknown(_) | Self::Unanswerable(_) => None,
        }
    }

    pub fn evidence(&self) -> Option<&Evidence> {
        match self {
            Self::Known(_, evidence) => Some(evidence),
            Self::Unknown(_) | Self::Unanswerable(_) => None,
        }
    }

    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(..))
    }

    /// Apply `f` to a known value, preserving its evidence and any reason for
    /// absence. Deriving from an observation must not quietly turn "unknown"
    /// into a computed number.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Observation<U> {
        match self {
            Self::Known(value, evidence) => Observation::Known(f(value), evidence),
            Self::Unknown(reason) => Observation::Unknown(reason),
            Self::Unanswerable(reason) => Observation::Unanswerable(reason),
        }
    }

    /// What to show where the value would go.
    ///
    /// Never an empty string and never a zero: the point of this type is that
    /// the absent case says which absence it is.
    pub fn render(&self, present: impl FnOnce(&T) -> String) -> String {
        match self {
            Self::Known(value, _) => present(value),
            Self::Unknown(reason) => reason.to_string(),
            Self::Unanswerable(reason) => (*reason).to_string(),
        }
    }
}

impl<T> Observation<T> {
    /// The conventional short form for a table cell, where the row already
    /// carries the explanation.
    pub fn cell(&self, present: impl FnOnce(&T) -> String) -> String {
        match self {
            Self::Known(value, _) => present(value),
            Self::Unknown(NotSampled::NeverSampled) => "not checked".to_string(),
            Self::Unknown(NotSampled::SamplingDisabled) => "RPC off".to_string(),
            Self::Unknown(_) => "unknown".to_string(),
            Self::Unanswerable(_) => "n/a".to_string(),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/observe/evidence_tests.rs"]
mod tests;
