use super::release_catalog::RuntimeReleaseCatalog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCatalogLoadRequest {
    pub source: String,
    pub signature_source: Option<String>,
    pub ed25519_public_key: Option<String>,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCatalogLoad {
    pub catalog: RuntimeReleaseCatalog,
    pub source: String,
    pub bytes: u64,
    pub signature_verified: Option<bool>,
    pub loaded_at_unix: u64,
}
