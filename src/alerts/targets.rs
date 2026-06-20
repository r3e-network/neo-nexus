mod label;
mod providers;
mod url;

pub use self::{label::alert_target_label, url::normalized_webhook_url};

pub(super) use self::providers::{
    datadog_target, opsgenie_target, pagerduty_target, telegram_target, validate_provider_target,
};
