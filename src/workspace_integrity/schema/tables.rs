mod core;
mod events_health;
mod federation;
mod runtime_assets;

use super::RequiredTable;

pub(in crate::workspace_integrity) fn required_tables(
) -> impl Iterator<Item = &'static RequiredTable> {
    core::CORE_TABLES
        .iter()
        .chain(federation::FEDERATION_TABLES)
        .chain(events_health::EVENTS_HEALTH_TABLES)
        .chain(runtime_assets::RUNTIME_ASSET_TABLES)
}
