mod datadog;
mod opsgenie;
mod pagerduty;
mod telegram;

use anyhow::Result;

use crate::alerts::AlertProvider;

pub(in crate::alerts) use self::{
    datadog::datadog_target, opsgenie::opsgenie_target, pagerduty::pagerduty_target,
    telegram::telegram_target,
};

pub(in crate::alerts) fn validate_provider_target(
    provider: AlertProvider,
    raw: &str,
) -> Result<()> {
    match provider {
        AlertProvider::Telegram => {
            telegram_target(raw)?;
            Ok(())
        }
        AlertProvider::PagerDuty => {
            pagerduty_target(raw)?;
            Ok(())
        }
        AlertProvider::Opsgenie => {
            opsgenie_target(raw)?;
            Ok(())
        }
        AlertProvider::Datadog => {
            datadog_target(raw)?;
            Ok(())
        }
        AlertProvider::Generic | AlertProvider::Slack | AlertProvider::Discord => Ok(()),
    }
}
