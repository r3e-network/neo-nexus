use std::path::{Path, PathBuf};

pub(in crate::snapshots::import) fn temporary_import_path(target: &Path) -> PathBuf {
    let mut temp_path = target.to_path_buf();
    temp_path.set_file_name(temporary_import_file_name(target));
    temp_path
}

fn temporary_import_file_name(target: &Path) -> String {
    target
        .file_name()
        .and_then(|value| value.to_str())
        .map_or_else(
            || "snapshot.neonexus-import".to_string(),
            |name| format!("{name}.neonexus-import"),
        )
}
