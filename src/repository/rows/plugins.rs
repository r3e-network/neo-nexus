use std::path::PathBuf;

use crate::{catalog::PluginId, plugins::PluginInstallation};

use super::parse_field;

pub(in crate::repository) fn plugin_installation_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<PluginInstallation> {
    let plugin_id_raw: String = row.get(1)?;
    let installed_files = row.get::<_, u64>(7)?;

    Ok(PluginInstallation {
        node_id: row.get(0)?,
        plugin_id: parse_field::<PluginId>(&plugin_id_raw)?,
        installed_path: PathBuf::from(row.get::<_, String>(2)?),
        manifest_path: PathBuf::from(row.get::<_, String>(3)?),
        source_path: PathBuf::from(row.get::<_, String>(4)?),
        sha256: row.get(5)?,
        package_bytes: row.get(6)?,
        installed_files: installed_files as usize,
        expanded_bytes: row.get(8)?,
        installed_at_unix: row.get(9)?,
    })
}
