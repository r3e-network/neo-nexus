use super::*;

pub(in crate::cli::actions) fn release_transaction_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 5, "--release-transaction")?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;
    let node = repository
        .list_nodes()?
        .into_iter()
        .find(|node| node.name == args[3])
        .with_context(|| format!("node {:?} was not found", args[3]))?;
    let installation = repository
        .list_runtime_installations()?
        .into_iter()
        .find(|installation| {
            installation.node_type == node.node_type && installation.version == args[4]
        })
        .with_context(|| {
            format!(
                "no installed {} runtime version {:?} was found",
                node.node_type, args[4]
            )
        })?;
    let target = crate::release_transaction::TargetRelease {
        version: installation.version,
        binary_path: installation.binary_path,
    };
    let node_type = node.node_type;
    let node_args = node.args.clone();
    let acceptance = Box::new(move |path: &std::path::Path| {
        let report = crate::core::runtime::smoke_runtime_command(
            node_type,
            path,
            &node_args,
            std::time::Duration::from_secs(3),
        );
        anyhow::ensure!(
            report.status.is_success(),
            "runtime smoke acceptance failed: {}",
            report.message
        );
        Ok(())
    });
    let message = crate::release_transaction::apply_release_transaction(
        &repository,
        repository
            .db_path()
            .parent()
            .unwrap_or(std::path::Path::new(".")),
        &node,
        &target,
        Some(acceptance),
    )?;
    Ok(CliAction::Print(format!("{message}\n")))
}
