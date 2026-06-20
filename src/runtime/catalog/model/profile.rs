use super::load::{RuntimeCatalogLoad, RuntimeCatalogLoadRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCatalogProfile {
    pub id: String,
    pub label: String,
    pub source: String,
    pub signature_source: Option<String>,
    pub ed25519_public_key: Option<String>,
    pub max_bytes: u64,
    pub enabled: bool,
    pub last_loaded_at_unix: Option<u64>,
    pub last_signature_verified: Option<bool>,
    pub last_bytes: Option<u64>,
}

impl RuntimeCatalogProfile {
    pub fn load_request(&self) -> RuntimeCatalogLoadRequest {
        RuntimeCatalogLoadRequest {
            source: self.source.clone(),
            signature_source: self.signature_source.clone(),
            ed25519_public_key: self.ed25519_public_key.clone(),
            max_bytes: self.max_bytes,
        }
    }

    pub fn with_load_result(&self, load: &RuntimeCatalogLoad) -> Self {
        let mut updated = self.clone();
        updated.last_loaded_at_unix = Some(load.loaded_at_unix);
        updated.last_signature_verified = load.signature_verified;
        updated.last_bytes = Some(load.bytes);
        updated
    }
}
