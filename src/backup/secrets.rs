use crate::types::NodeType;

/// Credentials are not portable inventory. Retain explicit file references,
/// including geth's --password (which names a password file), without reading
/// those files. Unknown secret-valued options remain blocked.
pub(super) fn secret_arguments(node_type: NodeType, args: &[String]) -> bool {
    let mut scanned = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if secret_url(argument) {
            return true;
        }
        let (flag, inline) = argument
            .split_once('=')
            .map_or((argument.as_str(), None), |(flag, value)| {
                (flag, Some(value))
            });
        let file_reference = flag.starts_with("--")
            && (flag.ends_with("-file")
                || node_type == NodeType::NeoXGeth
                    && matches!(flag, "--password" | "--authrpc.jwtsecret"));
        if file_reference {
            let value = inline.or_else(|| args.get(index + 1).map(String::as_str));
            if value.is_none_or(|value| value.is_empty() || value.starts_with("--")) {
                return true;
            }
            index += if inline.is_some() { 1 } else { 2 };
            continue;
        }
        if flag == "--nodekeyhex" {
            return true;
        }
        let value = inline.unwrap_or(argument);
        if secret_url(value) {
            return true;
        }
        scanned.push(argument.clone());
        index += 1;
    }
    crate::redaction::redact_sensitive_args(&scanned) != scanned
}

fn secret_url(value: &str) -> bool {
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    if !url.username().is_empty() || url.password().is_some() {
        return true;
    }
    url.query_pairs().any(|(key, _)| {
        // Query decoding catches escaped names as well as signed download links.
        let key = key.to_ascii_lowercase();
        let argument = format!("--{key}=reference");
        key == "sig"
            || key == "signature"
            || key.ends_with("-signature")
            || crate::redaction::redact_sensitive_args(std::slice::from_ref(&argument))
                != [argument]
    })
}

pub(super) fn validate_reference_credentials(
    backup: &super::schema::WorkspaceBackup,
) -> anyhow::Result<()> {
    let references = backup
        .runtime_catalog_profiles
        .iter()
        .flat_map(|profile| {
            std::iter::once(profile.source.as_str()).chain(profile.signature_source.as_deref())
        })
        .chain(
            backup
                .fast_sync_snapshots
                .iter()
                .filter_map(|snapshot| snapshot.source_url.as_deref()),
        );
    if references.into_iter().any(secret_url) {
        anyhow::bail!("backup download references must not contain credentials or signed URL secrets; use a public source reference");
    }
    for agent in &backup.agents {
        if crate::redaction::redact_sensitive_args(&agent.args) != agent.args
            || agent.args.iter().any(|argument| {
                secret_url(argument)
                    || argument
                        .split_once('=')
                        .is_some_and(|(_, value)| secret_url(value))
            })
        {
            anyhow::bail!("backup agent arguments must not contain credentials");
        }
    }
    Ok(())
}
