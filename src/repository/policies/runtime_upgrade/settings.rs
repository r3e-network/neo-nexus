use std::str::FromStr;

use anyhow::Result;
use rusqlite::{Connection, Transaction};

use crate::repository::helpers::{
    load_setting, optional_setting, parse_bool_setting, save_setting,
};

pub(super) fn load_bool_or(connection: &Connection, key: &str, default: bool) -> Result<bool> {
    Ok(load_setting(connection, key)?
        .as_deref()
        .map_or(default, parse_bool_setting))
}

pub(super) fn load_optional_text(connection: &Connection, key: &str) -> Result<Option<String>> {
    Ok(load_setting(connection, key)?.and_then(|value| optional_setting(&value)))
}

pub(super) fn load_number_or<T>(connection: &Connection, key: &str, default: T) -> Result<T>
where
    T: FromStr,
{
    Ok(load_setting(connection, key)?
        .as_deref()
        .and_then(|value| value.parse::<T>().ok())
        .unwrap_or(default))
}

pub(super) fn load_optional_number<T>(connection: &Connection, key: &str) -> Result<Option<T>>
where
    T: FromStr,
{
    Ok(load_setting(connection, key)?
        .as_deref()
        .and_then(|value| value.parse::<T>().ok()))
}

pub(super) fn save_bool(transaction: &Transaction<'_>, key: &str, value: bool) -> Result<()> {
    save_setting(transaction, key, if value { "true" } else { "false" })
}

pub(super) fn save_display(
    transaction: &Transaction<'_>,
    key: &str,
    value: impl ToString,
) -> Result<()> {
    save_setting(transaction, key, &value.to_string())
}

pub(super) fn save_optional_display<T>(
    transaction: &Transaction<'_>,
    key: &str,
    value: Option<T>,
) -> Result<()>
where
    T: ToString,
{
    save_setting(
        transaction,
        key,
        &value.map_or_else(String::new, |value| value.to_string()),
    )
}
