mod events_health;
mod federation;
mod plugins;
mod runtime;
mod snapshots;
mod wallet;

use std::str::FromStr;

pub(super) use events_health::{
    alert_delivery_from_row, event_from_row, rpc_health_record_from_row,
};
pub(super) use federation::{remote_server_from_row, remote_server_probe_record_from_row};
pub(super) use plugins::plugin_installation_from_row;
pub(super) use runtime::{
    runtime_catalog_profile_from_row, runtime_installation_from_row,
    runtime_signer_profile_from_row,
};
pub(super) use snapshots::snapshot_from_row;
pub(super) use wallet::neo_wallet_profile_from_row;

fn parse_field<T>(raw: &str) -> rusqlite::Result<T>
where
    T: FromStr<Err = anyhow::Error>,
{
    T::from_str(raw).map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))
}

fn decode_args_field(raw: &str) -> Vec<String> {
    super::decode_args(raw)
}
