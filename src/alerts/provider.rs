use std::{fmt, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertProvider {
    Generic,
    Slack,
    Discord,
    Telegram,
    PagerDuty,
    Opsgenie,
    Datadog,
}

impl AlertProvider {
    pub const ALL: [Self; 7] = [
        Self::Generic,
        Self::Slack,
        Self::Discord,
        Self::Telegram,
        Self::PagerDuty,
        Self::Opsgenie,
        Self::Datadog,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::Slack => "slack",
            Self::Discord => "discord",
            Self::Telegram => "telegram",
            Self::PagerDuty => "pagerduty",
            Self::Opsgenie => "opsgenie",
            Self::Datadog => "datadog",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Generic => "Generic JSON",
            Self::Slack => "Slack",
            Self::Discord => "Discord",
            Self::Telegram => "Telegram",
            Self::PagerDuty => "PagerDuty",
            Self::Opsgenie => "Opsgenie",
            Self::Datadog => "Datadog Events",
        }
    }

    pub fn target_label(self) -> &'static str {
        match self {
            Self::Telegram => "Bot API URL",
            Self::PagerDuty => "Events API URL",
            Self::Opsgenie => "Alerts API URL",
            Self::Datadog => "Events Intake URL",
            _ => "Webhook URL",
        }
    }

    pub fn target_hint(self) -> &'static str {
        match self {
            Self::Telegram => "https://api.telegram.org/bot<TOKEN>/sendMessage?chat_id=<CHAT_ID>",
            Self::Slack => "https://hooks.slack.com/services/...",
            Self::Discord => "https://discord.com/api/webhooks/...",
            Self::PagerDuty => "https://events.pagerduty.com/v2/enqueue?routing_key=<KEY>",
            Self::Opsgenie => "https://api.opsgenie.com/v2/alerts?api_key=<GENIE_KEY>",
            Self::Datadog => {
                "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>"
            }
            Self::Generic => "https://hooks.example.com/neonexus",
        }
    }
}

impl fmt::Display for AlertProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for AlertProvider {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "generic" | "generic-json" => Ok(Self::Generic),
            "slack" => Ok(Self::Slack),
            "discord" => Ok(Self::Discord),
            "telegram" => Ok(Self::Telegram),
            "pagerduty" | "pager-duty" => Ok(Self::PagerDuty),
            "opsgenie" | "ops-genie" => Ok(Self::Opsgenie),
            "datadog" | "datadog-events" | "datadog-event" => Ok(Self::Datadog),
            other => anyhow::bail!("unsupported alert provider: {other}"),
        }
    }
}
