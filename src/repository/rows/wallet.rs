use crate::wallet::NeoWalletProfile;

use super::decode_args_field;

pub(in crate::repository) fn neo_wallet_profile_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<NeoWalletProfile> {
    let contract_public_keys_raw: String = row.get(5)?;
    Ok(NeoWalletProfile {
        id: row.get(0)?,
        label: row.get(1)?,
        source_path: row.get(2)?,
        wallet_version: row.get(3)?,
        primary_address: row.get(4)?,
        contract_public_keys: decode_args_field(&contract_public_keys_raw),
        wallet_sha256: row.get(6)?,
        account_count: row.get(7)?,
        encrypted_account_count: row.get(8)?,
        default_account_count: row.get(9)?,
        watch_only_account_count: row.get(10)?,
        validated_at_unix: row.get(11)?,
        last_used_at_unix: row.get(12)?,
    })
}
