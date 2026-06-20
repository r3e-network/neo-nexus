pub(super) fn wallet_provisioning_sensitive_key(key: &str) -> bool {
    let key = normalize_wallet_provisioning_key(key);

    is_exact_secret_key(&key) || is_secret_key_fragment(&key)
}

fn normalize_wallet_provisioning_key(key: &str) -> String {
    key.trim()
        .trim_start_matches('-')
        .to_ascii_lowercase()
        .replace('_', "-")
}

fn is_exact_secret_key(key: &str) -> bool {
    matches!(
        key,
        "password"
            | "wallet-password"
            | "passphrase"
            | "mnemonic"
            | "seed"
            | "seed-phrase"
            | "private-key"
            | "privatekey"
            | "wif"
            | "wallet-key"
            | "walletkey"
            | "token"
            | "auth-token"
            | "api-key"
            | "apikey"
            | "access-key"
            | "accesskey"
            | "authorization"
            | "bearer"
            | "secret"
            | "secret-key"
            | "client-secret"
            | "webhook-secret"
    )
}

fn is_secret_key_fragment(key: &str) -> bool {
    key.contains("password")
        || key.contains("passphrase")
        || key.contains("private-key")
        || key.contains("privatekey")
        || key.contains("mnemonic")
        || key.ends_with("-seed")
        || key.contains("api-key")
        || key.contains("apikey")
        || key.ends_with("-token")
        || key == "token"
}
