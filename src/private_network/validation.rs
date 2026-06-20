use super::*;

mod artifacts;
mod committee;
mod nodes;
mod schema;
mod scripts;

pub(in crate::private_network) use self::{
    artifacts::check_artifacts, committee::check_committee, nodes::check_nodes,
    schema::check_schema, scripts::check_scripts,
};

pub(super) fn launch_pack_manifest_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("manifest.json")
    } else {
        path.to_path_buf()
    }
}
