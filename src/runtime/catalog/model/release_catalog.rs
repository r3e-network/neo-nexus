use std::{cmp::Ordering, collections::BTreeSet};

use anyhow::{Context, Result};

use crate::types::NodeType;

use super::super::super::{compare_versions, RuntimePlatform};
use super::super::{dto::RuntimeReleaseCatalogDto, validation::validate_runtime_release};
use super::release::RuntimeRelease;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeReleaseCatalog {
    pub schema_version: u32,
    pub generated_at_unix: Option<u64>,
    pub releases: Vec<RuntimeRelease>,
}

impl RuntimeReleaseCatalog {
    pub fn from_json(text: &str) -> Result<Self> {
        let dto: RuntimeReleaseCatalogDto =
            serde_json::from_str(text).context("runtime release catalog JSON is invalid")?;
        if dto.schema_version != 1 {
            anyhow::bail!(
                "unsupported runtime release catalog schema version {}",
                dto.schema_version
            );
        }

        let mut ids = BTreeSet::new();
        let mut releases = Vec::with_capacity(dto.releases.len());
        for dto_release in dto.releases {
            let release = RuntimeRelease::try_from(dto_release)?;
            validate_runtime_release(&release)?;
            if !ids.insert(release.id.clone()) {
                anyhow::bail!("duplicate runtime release id: {}", release.id);
            }
            releases.push(release);
        }

        Ok(Self {
            schema_version: dto.schema_version,
            generated_at_unix: dto.generated_at_unix,
            releases,
        })
    }

    pub fn get(&self, id: &str) -> Option<&RuntimeRelease> {
        self.releases.iter().find(|release| release.id == id)
    }

    pub fn compatible_releases(&self, platform: &RuntimePlatform) -> Vec<&RuntimeRelease> {
        let mut releases = self
            .releases
            .iter()
            .filter(|release| release.platform_matches(platform))
            .collect::<Vec<_>>();
        releases.sort_by(|left, right| compare_releases(right, left));
        releases
    }

    pub fn latest_for(
        &self,
        node_type: NodeType,
        platform: &RuntimePlatform,
    ) -> Option<&RuntimeRelease> {
        self.releases
            .iter()
            .filter(|release| release.node_type == node_type && release.platform_matches(platform))
            .max_by(|left, right| compare_releases(left, right))
    }
}

fn compare_releases(left: &RuntimeRelease, right: &RuntimeRelease) -> Ordering {
    compare_versions(&left.version, &right.version).then_with(|| left.id.cmp(&right.id))
}
