mod alerts;
mod backup;
mod config;
mod health;
mod quality;
mod reports;
mod wallet;

use super::RequiredCommand;

pub(in crate::ci_policy) fn required_commands() -> impl Iterator<Item = &'static RequiredCommand> {
    quality::QUALITY_COMMANDS
        .iter()
        .chain(alerts::ALERT_COMMANDS.iter())
        .chain(health::HEALTH_COMMANDS.iter())
        .chain(reports::REPORT_COMMANDS.iter())
        .chain(config::CONFIG_COMMANDS.iter())
        .chain(backup::BACKUP_COMMANDS.iter())
        .chain(wallet::WALLET_COMMANDS.iter())
}
