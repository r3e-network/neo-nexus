use std::time::Duration;

use crate::types::NodeType;

use super::{
    identity::{output_contains_runtime_identity, success_message},
    RuntimeSmokeAttempt, RuntimeSmokeStatus,
};

pub(super) enum ProbeOutcome {
    Continue,
    Passed(String),
}

#[derive(Default)]
pub(super) struct ProbeEvaluation {
    timeout_seen: bool,
    best_review: Option<String>,
    best_failure: Option<String>,
}

impl ProbeEvaluation {
    pub(super) fn record_attempt(
        &mut self,
        node_type: NodeType,
        attempt: &RuntimeSmokeAttempt,
    ) -> ProbeOutcome {
        if attempt.timed_out {
            self.timeout_seen = true;
            return ProbeOutcome::Continue;
        }

        let output_text = attempt.output_text();
        let identity = output_contains_runtime_identity(node_type, &output_text);
        let exit_success = attempt.exit_code == Some(0);

        if exit_success && identity {
            return ProbeOutcome::Passed(success_message(node_type, &output_text));
        }

        if exit_success {
            self.best_review.get_or_insert_with(|| {
                "Probe command exited successfully but did not identify the runtime.".to_string()
            });
        } else {
            self.best_failure.get_or_insert_with(|| {
                format!(
                    "Probe command exited with {:?}.",
                    attempt.exit_code.unwrap_or(-1)
                )
            });
        }

        ProbeOutcome::Continue
    }

    pub(super) fn record_error(&mut self, error: String) {
        self.best_failure.get_or_insert(error);
    }

    pub(super) fn finish(self, timeout: Duration) -> (RuntimeSmokeStatus, String) {
        if let Some(message) = self.best_review {
            return (RuntimeSmokeStatus::Review, message);
        }

        if self.timeout_seen {
            return (
                RuntimeSmokeStatus::TimedOut,
                format!("Runtime probe exceeded {}ms.", timeout.as_millis()),
            );
        }

        (
            RuntimeSmokeStatus::Failed,
            self.best_failure
                .unwrap_or_else(|| "No runtime probe succeeded.".to_string()),
        )
    }
}
