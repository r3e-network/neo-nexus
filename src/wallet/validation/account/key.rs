use crate::wallet::{
    crypto::{looks_like_plain_private_key, valid_nep2_key},
    NeoWalletValidationCheck, NeoWalletValidationStatus,
};

use super::super::add_check;

pub(super) fn encrypted_account_key(key: &str) -> bool {
    !watch_only_account_key(key) && valid_nep2_key(key)
}

pub(super) fn watch_only_account_key(key: &str) -> bool {
    key.trim().is_empty()
}

pub(super) fn check_account_key(
    checks: &mut Vec<NeoWalletValidationCheck>,
    label: &str,
    key: &str,
) {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        add_check(
            checks,
            "account-key",
            label,
            NeoWalletValidationStatus::Warn,
            "account is watch-only; signer wallets need an encrypted NEP-2 key".to_string(),
        );
        return;
    }

    if looks_like_plain_private_key(trimmed) {
        add_check(
            checks,
            "account-key",
            label,
            NeoWalletValidationStatus::Fail,
            "account key looks like plaintext private key material, not encrypted NEP-2"
                .to_string(),
        );
        return;
    }

    let valid_nep2 = valid_nep2_key(trimmed);
    add_check(
        checks,
        "account-key",
        label,
        if valid_nep2 {
            NeoWalletValidationStatus::Pass
        } else {
            NeoWalletValidationStatus::Fail
        },
        if valid_nep2 {
            "encrypted NEP-2 key present".to_string()
        } else {
            "account key is not a valid encrypted NEP-2 key".to_string()
        },
    );
}
