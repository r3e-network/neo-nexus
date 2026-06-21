use crate::app::domain::{
    RuntimeCatalogProfile, RuntimePackageManager, RuntimePlatform, RuntimeReleaseCatalog,
    RuntimeUpgradePolicy,
};

use super::super::NeoNexusApp;

pub(super) struct RuntimeUpgradeCatalogContext {
    pub(super) profile: RuntimeCatalogProfile,
    pub(super) catalog: RuntimeReleaseCatalog,
    signature_verified: Option<bool>,
    bytes: u64,
}

impl NeoNexusApp {
    pub(super) fn load_runtime_upgrade_catalog(
        &mut self,
        policy: &RuntimeUpgradePolicy,
    ) -> anyhow::Result<RuntimeUpgradeCatalogContext> {
        let catalog_profile_id = policy
            .catalog_profile_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("runtime upgrade policy has no catalog profile"))?;
        let profile = self
            .runtime_catalog_profiles
            .iter()
            .find(|profile| profile.id == catalog_profile_id)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("runtime catalog profile {catalog_profile_id} was not found")
            })?;
        if !profile.enabled {
            anyhow::bail!("runtime catalog profile is disabled: {}", profile.label);
        }

        let load = RuntimePackageManager::load_release_catalog(&profile.load_request())?;
        if policy.require_signed_catalog && load.signature_verified != Some(true) {
            anyhow::bail!("runtime upgrade policy requires a signed catalog");
        }
        self.repository
            .mark_runtime_catalog_profile_loaded(&profile.id, &load)?;

        Ok(RuntimeUpgradeCatalogContext {
            profile,
            catalog: load.catalog,
            signature_verified: load.signature_verified,
            bytes: load.bytes,
        })
    }

    pub(super) fn publish_runtime_upgrade_catalog(
        &mut self,
        context: RuntimeUpgradeCatalogContext,
    ) {
        self.selected_runtime_catalog_profile = Some(context.profile.id.clone());
        self.apply_runtime_catalog_profile_to_form(&context.profile);
        self.selected_runtime_release = context
            .catalog
            .compatible_releases(&RuntimePlatform::current())
            .first()
            .map(|release| release.id.clone())
            .or_else(|| {
                context
                    .catalog
                    .releases
                    .first()
                    .map(|release| release.id.clone())
            });
        self.runtime_catalog_page = 0;
        self.runtime_catalog_signature_verified = context.signature_verified;
        self.runtime_catalog_bytes = context.bytes;
        self.runtime_catalog = Some(context.catalog);
        self.mark_runtime_signer_used_by_key(context.profile.ed25519_public_key.as_deref());
        self.reload_runtime_catalog_profiles();
        self.selected_runtime_catalog_profile = Some(context.profile.id);
    }
}
