use crate::app::domain::NodeType;

pub(in crate::app) struct RuntimePackageDraft {
    pub(in crate::app) id: String,
    pub(in crate::app) label: String,
    pub(in crate::app) node_type: NodeType,
    pub(in crate::app) version: String,
    pub(in crate::app) os: String,
    pub(in crate::app) arch: String,
    pub(in crate::app) source_path: String,
    pub(in crate::app) executable_name: String,
    pub(in crate::app) expected_sha256: String,
    pub(in crate::app) signature_path: String,
    pub(in crate::app) ed25519_public_key: String,
    pub(in crate::app) download_url: String,
    pub(in crate::app) download_file_name: String,
    pub(in crate::app) download_max_mib: u64,
}
