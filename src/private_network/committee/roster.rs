use std::collections::BTreeMap;

use anyhow::Result;

use super::*;

mod references;

impl CommitteeRoster {
    pub fn from_public_keys(input: &str) -> Result<Option<Self>> {
        let keys = input
            .split(|character: char| {
                character == ',' || character == ';' || character.is_whitespace()
            })
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_public_key)
            .collect::<Result<Vec<_>>>()?;
        if keys.is_empty() {
            return Ok(None);
        }

        let mut seen = BTreeMap::new();
        let mut signers = Vec::with_capacity(keys.len());
        for (index, public_key) in keys.into_iter().enumerate() {
            if seen.insert(public_key.clone(), index + 1).is_some() {
                anyhow::bail!("duplicate committee public key: {public_key}");
            }
            signers.push(CommitteeSigner {
                label: format!("committee-signer-{}", index + 1),
                public_key,
                wallet_path: None,
                signer_endpoint: None,
                signer_command_template: None,
                signer_command: None,
                signer_command_plan: None,
            });
        }
        Ok(Some(Self { signers }))
    }

    pub fn from_public_keys_and_references(
        public_keys: &str,
        signer_references: &str,
    ) -> Result<Option<Self>> {
        let mut roster = match Self::from_public_keys(public_keys)? {
            Some(roster) => roster,
            None => {
                if has_signer_references(signer_references) {
                    anyhow::bail!("committee public keys are required before signer references");
                }
                return Ok(None);
            }
        };
        roster.apply_signer_references(signer_references)?;
        Ok(Some(roster))
    }

    pub fn public_keys(&self) -> Vec<String> {
        self.signers
            .iter()
            .map(|signer| signer.public_key.clone())
            .collect()
    }
}
