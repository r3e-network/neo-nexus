use crate::{
    signing::{LocalWalletSigner, SignerBackendKind},
    web::{html, Custody},
};

use super::super::SignerTab;

pub fn custody_notice(custody: &Custody) -> String {
    let registry = registry_notice(custody);
    if let Some(signer) = custody.local_wallet_signer() {
        let key = signer.key_info();
        return format!(
            "{}{}",
            registry,
            custody_band(
                "",
                "Local encrypted wallet ready",
                &format!(
                    "Account {} · Neo N3 transaction signing only · encrypted wallet fingerprint {}. No public relay or remote-admin surface is enabled.",
                    key.address,
                    fingerprint(signer.wallet_sha256())
                ),
            )
        );
    }
    let client = match custody.client() {
        Ok(client) => client,
        Err(problem) => {
            return format!(
                "{}{}",
                registry,
                custody_band("danger", "Custody is not configured", &problem.to_string())
            )
        }
    };
    let config = client.config();
    let kind = custody.kind().unwrap_or(SignerBackendKind::NeoOsService);
    let title = match kind {
        SignerBackendKind::LocalSigner => "Local signer connected",
        SignerBackendKind::NeoOsService => "Remote custody connected · NeoOS signer service",
        SignerBackendKind::LocalWallet => "Local encrypted wallet ready",
    };
    let location = match kind {
        SignerBackendKind::LocalSigner => "This machine · loopback",
        SignerBackendKind::NeoOsService => "Remote service",
        SignerBackendKind::LocalWallet => "This machine",
    };
    let mut notice = match config.admin() {
        Some(_) => custody_band(
            "",
            title,
            &format!(
                "{location} · service {} · NeoNexus holds only its admin credential.",
                config.base_url()
            ),
        ),
        None => custody_band(
            "danger",
            "Custody credential missing",
            &format!(
                "Service {} is configured, but this workbench has no credential to read it.",
                config.base_url()
            ),
        ),
    };
    if config.uses_cleartext() {
        notice.push_str(&html::notice(
            "warn",
            "This loopback service uses plain HTTP. Put an authenticated TLS proxy in front of custody before moving it to another host.",
        ));
    }
    format!("{registry}{notice}")
}

fn registry_notice(custody: &Custody) -> String {
    let profiles = custody.profiles().collect::<Vec<_>>();
    if profiles.is_empty() {
        return String::new();
    }
    let registry = custody.registry();
    let rows = profiles
        .iter()
        .map(|profile| {
            let mut roles = Vec::new();
            if registry.console_backend_id() == Some(profile.id.as_str()) {
                roles.push("console");
            }
            if registry.relay_backend_id() == Some(profile.id.as_str()) {
                roles.push("public relay");
            }
            let roles = if roles.is_empty() {
                "available for an explicit node binding".to_string()
            } else {
                roles.join(" + ")
            };
            format!(
                "<li><strong>{}</strong><span>{} · {}</span></li>",
                html::escape(&profile.label),
                html::escape(profile.kind.slug()),
                html::escape(&roles)
            )
        })
        .collect::<String>();
    format!(
        r#"<section class="surface"><div class="section-head"><h2>Signer profiles</h2><span class="muted">{} loaded; every route is explicit and has no fallback.</span></div><ul class="summary-list">{rows}</ul></section>"#,
        profiles.len()
    )
}

pub fn render_local_wallet(
    banner: &str,
    signer: &LocalWalletSigner,
    requested_tab: SignerTab,
) -> String {
    let key = signer.key_info();
    let capabilities = signer.capabilities();
    let unsupported = (requested_tab != SignerTab::Overview).then(|| {
        html::notice(
            "warn",
            "This backend has no remote Keys, Callers, Policy, or Audit control plane. The local wallet overview is shown instead.",
        )
    });
    let rows = [
        ("Backend type", "Local encrypted wallet".to_string()),
        ("Account", key.address.clone()),
        ("Public key", key.public_key.clone()),
        ("Network", key.network.clone()),
        (
            "Network magic",
            key.network_magic
                .map_or_else(|| "—".to_string(), |magic| magic.to_string()),
        ),
        ("Wallet SHA-256", signer.wallet_sha256().to_string()),
        (
            "Transaction signing",
            capability_word(capabilities.neo_n3_transaction).to_string(),
        ),
        (
            "Consensus signing",
            capability_word(capabilities.neo_n3_consensus).to_string(),
        ),
        (
            "Raw signing",
            capability_word(capabilities.neo_n3_raw).to_string(),
        ),
        ("NeoX", "unsupported by NEP-6 / P-256".to_string()),
        ("Public relay", "never exposed".to_string()),
    ]
    .into_iter()
    .map(|(label, value)| html::row(&[html::cell(label), html::cell(&value)]))
    .collect::<Vec<_>>();
    format!(
        "{}{}{}{}{}",
        page_head_local(),
        banner,
        unsupported.unwrap_or_default(),
        html::notice(
            "",
            "The password is read once from a protected local file and is not retained. One zeroizing private-key allocation remains in this process until the profile is dropped; it is never stored in neonexus.db, rendered in HTML, or accepted from a browser. The wallet is hash-pinned and rechecked before each operation."
        ),
        html::table(&["Property", "Value"], &rows),
    )
}

fn page_head_local() -> String {
    let breadcrumb = html::breadcrumb(&[
        ("KMS", "/signer"),
        ("Customer managed keys", "/signer"),
    ]);
    format!(
        "{breadcrumb}{}",
        html::page_head(
            "KMS & Custody Signer",
            "Explicit local-wallet backend with a narrow Neo N3 capability set.",
            "",
        )
    )
}

fn capability_word(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

fn fingerprint(value: &str) -> String {
    let prefix = value.chars().take(12).collect::<String>();
    format!("{prefix}…")
}

fn custody_band(kind: &str, title: &str, detail: &str) -> String {
    format!(
        r#"<div class="custody-band {kind}"><span class="custody-mark" aria-hidden="true">S</span><div><strong>{title}</strong><p>{detail}</p></div></div>"#,
        kind = html::escape(kind),
        title = html::escape(title),
        detail = html::escape(detail),
    )
}
