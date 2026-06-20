use std::collections::BTreeMap;

use anyhow::{Context, Result};

use super::super::{
    expand_signer_command_template, has_signer_references, normalize_public_key,
    parse_signer_command_plan, validate_signer_command_template, validate_signer_endpoint,
    validate_signer_wallet_path, CommitteeRoster, CommitteeSigner,
};

impl CommitteeRoster {
    pub fn apply_signer_references(&mut self, input: &str) -> Result<()> {
        if !has_signer_references(input) {
            return Ok(());
        }

        let mut referenced = BTreeMap::new();
        for (line_index, line) in input.lines().enumerate() {
            self.apply_signer_reference_line(line_index + 1, line, &mut referenced)?;
        }

        Ok(())
    }

    fn apply_signer_reference_line(
        &mut self,
        line_number: usize,
        line: &str,
        referenced: &mut BTreeMap<String, usize>,
    ) -> Result<()> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        let reference = SignerReferenceLine::parse(line_number, trimmed)?;
        if referenced
            .insert(reference.public_key.clone(), line_number)
            .is_some()
        {
            anyhow::bail!(
                "duplicate signer reference for committee public key: {}",
                reference.public_key
            );
        }

        let signer = self
            .signers
            .iter_mut()
            .find(|signer| signer.public_key == reference.public_key)
            .with_context(|| {
                format!(
                    "signer reference line {line_number} uses unknown committee public key {}",
                    reference.public_key
                )
            })?;
        reference.apply_to(signer)
    }
}

struct SignerReferenceLine {
    public_key: String,
    wallet_path: String,
    signer_endpoint: String,
    signer_command_template: String,
}

impl SignerReferenceLine {
    fn parse(line_number: usize, line: &str) -> Result<Self> {
        let parts = line.split('|').map(str::trim).collect::<Vec<_>>();
        if parts.len() > 4 {
            anyhow::bail!(
                "signer reference line {line_number} must use public_key | wallet_path | signer_endpoint | sidecar_command_template"
            );
        }
        let public_key = normalize_public_key(parts.first().copied().unwrap_or_default())
            .with_context(|| format!("invalid signer reference line {line_number}"))?;
        let wallet_path = parts.get(1).copied().unwrap_or_default().to_string();
        let signer_endpoint = parts.get(2).copied().unwrap_or_default().to_string();
        let signer_command_template = parts.get(3).copied().unwrap_or_default().to_string();
        if wallet_path.is_empty()
            && signer_endpoint.is_empty()
            && signer_command_template.is_empty()
        {
            anyhow::bail!(
                "signer reference line {line_number} must include a wallet path, signer endpoint, or sidecar command template"
            );
        }

        Ok(Self {
            public_key,
            wallet_path,
            signer_endpoint,
            signer_command_template,
        })
    }

    fn apply_to(&self, signer: &mut CommitteeSigner) -> Result<()> {
        if !self.wallet_path.is_empty() {
            signer.wallet_path = Some(validate_signer_wallet_path(&self.wallet_path)?);
        }
        if !self.signer_endpoint.is_empty() {
            signer.signer_endpoint = Some(validate_signer_endpoint(&self.signer_endpoint)?);
        }
        if !self.signer_command_template.is_empty() {
            signer.signer_command_template = Some(validate_signer_command_template(
                &self.signer_command_template,
            )?);
            signer.signer_command = Some(expand_signer_command_template(
                signer
                    .signer_command_template
                    .as_deref()
                    .unwrap_or_default(),
                signer,
            )?);
            signer.signer_command_plan = Some(parse_signer_command_plan(
                signer.signer_command.as_deref().unwrap_or_default(),
            )?);
        }
        Ok(())
    }
}
