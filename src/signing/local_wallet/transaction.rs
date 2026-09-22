//! Strict structural validation for Neo N3 unsigned transactions.
//!
//! Local-wallet transaction authority must not be usable as a generic Neo
//! signing oracle. This parser consumes the complete canonical unsigned wire
//! envelope and returns its signer accounts. Unknown witness-rule and attribute
//! shapes fail closed; semantic policy remains the service backend's job.

use anyhow::{bail, Context, Result};

const MAX_TRANSACTION_SIZE: usize = 102_400;
const MAX_TRANSACTION_ATTRIBUTES: usize = 16;
const MAX_SIGNER_SUBITEMS: usize = 16;
const MAX_SCRIPT_BYTES: usize = u16::MAX as usize;
const HASH160_LEN: usize = 20;

pub(super) fn validate_for_signer(bytes: &[u8], display_script_hash: &str) -> Result<()> {
    let signers = parse_unsigned(bytes)?;
    let expected = decode_display_script_hash(display_script_hash)?;
    if !signers.contains(&expected) {
        bail!("the local wallet account is not a signer on the unsigned transaction");
    }
    Ok(())
}

fn parse_unsigned(bytes: &[u8]) -> Result<Vec<[u8; HASH160_LEN]>> {
    if bytes.len() > MAX_TRANSACTION_SIZE {
        bail!("unsigned transaction exceeds the Neo N3 maximum size");
    }
    let mut reader = Reader::new(bytes);
    let version = reader.byte()?;
    if version != 0 {
        bail!("unsupported Neo N3 transaction version {version}");
    }
    reader.take(4).context("transaction nonce is truncated")?;
    let system_fee = reader
        .i64()
        .context("transaction system fee is truncated")?;
    let network_fee = reader
        .i64()
        .context("transaction network fee is truncated")?;
    if system_fee < 0 || network_fee < 0 || system_fee.checked_add(network_fee).is_none() {
        bail!("transaction fees are outside the Neo N3 range");
    }
    reader
        .take(4)
        .context("transaction valid-until-block is truncated")?;
    let signer_count = reader.count("signer", MAX_TRANSACTION_ATTRIBUTES)?;
    if signer_count == 0 {
        bail!("unsigned transaction has no signers");
    }
    let mut signers = Vec::with_capacity(signer_count);
    for _ in 0..signer_count {
        let account = reader.hash160()?;
        if signers.contains(&account) {
            bail!("unsigned transaction contains a duplicate signer");
        }
        let scope = reader.byte()?;
        const CALLED_BY_ENTRY: u8 = 0x01;
        const CUSTOM_CONTRACTS: u8 = 0x10;
        const CUSTOM_GROUPS: u8 = 0x20;
        const WITNESS_RULES: u8 = 0x40;
        const GLOBAL: u8 = 0x80;
        const KNOWN: u8 =
            CALLED_BY_ENTRY | CUSTOM_CONTRACTS | CUSTOM_GROUPS | WITNESS_RULES | GLOBAL;
        if scope & !KNOWN != 0 || (scope & GLOBAL != 0 && scope != GLOBAL) {
            bail!("unsigned transaction contains an invalid witness scope");
        }
        if scope & CUSTOM_CONTRACTS != 0 {
            let count = reader.count("allowed contract", MAX_SIGNER_SUBITEMS)?;
            reader.take(
                count
                    .checked_mul(HASH160_LEN)
                    .context("contract count overflow")?,
            )?;
        }
        if scope & CUSTOM_GROUPS != 0 {
            let count = reader.count("allowed group", MAX_SIGNER_SUBITEMS)?;
            reader.take(count.checked_mul(33).context("group count overflow")?)?;
        }
        if scope & WITNESS_RULES != 0 {
            bail!("local-wallet signing does not accept state-dependent witness rules");
        }
        signers.push(account);
    }

    let attribute_count = reader.count(
        "attribute",
        MAX_TRANSACTION_ATTRIBUTES.saturating_sub(signer_count),
    )?;
    let mut unique_attributes = Vec::with_capacity(attribute_count);
    for _ in 0..attribute_count {
        let kind = reader.byte()?;
        if kind != 0x21 && unique_attributes.contains(&kind) {
            bail!("unsigned transaction contains a duplicate attribute");
        }
        if kind != 0x21 {
            unique_attributes.push(kind);
        }
        match kind {
            0x01 => {}
            0x20 => {
                reader
                    .take(4)
                    .context("NotValidBefore attribute is truncated")?;
            }
            0x21 => {
                reader
                    .take(32)
                    .context("Conflicts attribute is truncated")?;
            }
            _ => bail!("local-wallet signing does not support transaction attribute 0x{kind:02x}"),
        }
    }
    let script_length = reader.count("script", MAX_SCRIPT_BYTES)?;
    if script_length == 0 {
        bail!("unsigned transaction script is empty");
    }
    reader
        .take(script_length)
        .context("unsigned transaction script is truncated")?;
    if !reader.finished() {
        bail!("unsigned transaction contains trailing bytes");
    }
    Ok(signers)
}

fn decode_display_script_hash(value: &str) -> Result<[u8; HASH160_LEN]> {
    let encoded = value.trim().strip_prefix("0x").unwrap_or(value.trim());
    if encoded.len() != HASH160_LEN * 2 || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("local wallet script hash is not a 20-byte display hash");
    }
    let mut wire = [0_u8; HASH160_LEN];
    for (index, pair) in encoded.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let text = std::str::from_utf8(pair).context("script hash is not ASCII")?;
        wire[HASH160_LEN - 1 - index] =
            u8::from_str_radix(text, 16).context("script hash is not hexadecimal")?;
    }
    Ok(wire)
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .context("unsigned transaction is truncated")?;
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn i64(&mut self) -> Result<i64> {
        let bytes: [u8; 8] = self
            .take(8)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("unsigned transaction is truncated"))?;
        Ok(i64::from_le_bytes(bytes))
    }

    fn hash160(&mut self) -> Result<[u8; HASH160_LEN]> {
        self.take(HASH160_LEN)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction signer is truncated"))
    }

    fn count(&mut self, field: &str, maximum: usize) -> Result<usize> {
        let first = self.byte()?;
        let value = match first {
            0xfd => {
                let bytes: [u8; 2] = self
                    .take(2)?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("unsigned transaction is truncated"))?;
                let value = u64::from(u16::from_le_bytes(bytes));
                if value < 0xfd {
                    bail!("{field} uses a noncanonical VarInt");
                }
                value
            }
            0xfe => {
                let bytes: [u8; 4] = self
                    .take(4)?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("unsigned transaction is truncated"))?;
                let value = u64::from(u32::from_le_bytes(bytes));
                if value <= u64::from(u16::MAX) {
                    bail!("{field} uses a noncanonical VarInt");
                }
                value
            }
            0xff => {
                let bytes: [u8; 8] = self
                    .take(8)?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("unsigned transaction is truncated"))?;
                let value = u64::from_le_bytes(bytes);
                if value <= u64::from(u32::MAX) {
                    bail!("{field} uses a noncanonical VarInt");
                }
                value
            }
            value => u64::from(value),
        };
        let value =
            usize::try_from(value).with_context(|| format!("{field} count is too large"))?;
        if value > maximum || value > self.bytes.len().saturating_sub(self.offset) {
            bail!("{field} count exceeds its bound");
        }
        Ok(value)
    }

    fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_consensus_shaped_and_noncanonical_inputs() {
        assert!(parse_unsigned(&[0_u8; 7]).is_err());
        let canonical = minimal_transaction([1_u8; 20]);
        let mut transaction = Vec::with_capacity(canonical.len() + 2);
        transaction.extend_from_slice(&canonical[..25]);
        // One signer encoded through the 0xfd form is valid in value but not
        // canonical Neo wire format.
        transaction.extend_from_slice(&[0xfd, 0x01, 0x00]);
        transaction.extend_from_slice(&canonical[26..]);
        assert!(parse_unsigned(&transaction).is_err());
    }

    pub(super) fn minimal_transaction(account: [u8; 20]) -> Vec<u8> {
        let mut bytes = vec![0];
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&0_i64.to_le_bytes());
        bytes.extend_from_slice(&0_i64.to_le_bytes());
        bytes.extend_from_slice(&100_u32.to_le_bytes());
        bytes.push(1);
        bytes.extend_from_slice(&account);
        bytes.push(0x01);
        bytes.push(0);
        bytes.push(1);
        bytes.push(0x10);
        bytes
    }
}
