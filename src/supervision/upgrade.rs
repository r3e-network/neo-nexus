//! The scheduled runtime upgrade pass: when the policy says a fleet upgrade is
//! due, download and install the catalog release for a bounded batch of nodes.
//!
//! A node that was running before the upgrade is stopped first and started again
//! afterwards; a node that was already stopped is left stopped. Each batch is
//! capped by `max_nodes_per_run` so one tick cannot rebuild the whole fleet.

use std::collections::HashMap;

use log::{info, warn};

use crate::{
    core::{lifecycle::LaunchAction, node::NodeConfig},
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    runtime::{RuntimePackageManager, RuntimePlatform},
    types::{node_workspace_path, NewNode},
};

use super::{
    launch::{launch_node, stop_node},
    state::{EngineState, LoopState},
};

impl LoopState {
    /// Check if runtime upgrade policy is due and execute upgrade batch.
    pub(super) fn probe_runtime_upgrade(&mut self, state: &EngineState) {
        // 1. 加载 policy
        let Ok(policy) = state.repository.load_runtime_upgrade_policy() else {
            warn!("neo-nexus: failed to load runtime upgrade policy");
            return;
        };

        if !policy.enabled {
            return; // disabled by config
        }

        // 2. 计算 now_unix
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if !policy.is_due(now_unix) {
            return; // not yet due
        }

        // 3. 获取 nodes
        let Ok(nodes) = state.repository.list_nodes() else {
            warn!("neo-nexus: failed to list nodes for upgrade check");
            return;
        };

        if nodes.is_empty() {
            // No nodes at all - still update last_checked_at
            let updated_policy = policy.with_checked_at(now_unix);
            let _ = state
                .repository
                .save_runtime_upgrade_policy(&updated_policy);
            return;
        }

        // 4. 验证 catalog_profile_id 配置
        let Some(profile_id) = policy.catalog_profile_id.as_deref() else {
            warn!("neo-nexus: catalog profile ID not configured");
            let updated_policy = policy.with_checked_at(now_unix);
            let _ = state
                .repository
                .save_runtime_upgrade_policy(&updated_policy);
            return;
        };

        // 5. 读取 catalog profile
        let Ok(profiles) = state.repository.list_runtime_catalog_profiles() else {
            warn!("neo-nexus: failed to list catalog profiles");
            let updated_policy = policy.with_checked_at(now_unix);
            let _ = state
                .repository
                .save_runtime_upgrade_policy(&updated_policy);
            return;
        };

        let Some(profile) = profiles.iter().find(|p| p.id == *profile_id) else {
            warn!("neo-nexus: catalog profile {} not found", profile_id);
            let updated_policy = policy.with_checked_at(now_unix);
            let _ = state
                .repository
                .save_runtime_upgrade_policy(&updated_policy);
            return;
        };

        // 6. 从 source 读取 catalog JSON
        let request = profile.load_request();
        let Ok(load_result) = RuntimePackageManager::load_release_catalog(&request) else {
            warn!(
                "neo-nexus: failed to load release catalog from {}",
                request.source
            );
            let updated_policy = policy.with_checked_at(now_unix);
            let _ = state
                .repository
                .save_runtime_upgrade_policy(&updated_policy);
            return;
        };

        let catalog = &load_result.catalog;

        // 7. 使用当前平台
        let platform = RuntimePlatform::current();

        // 8. 生成 upgrade plan
        let plan = RuntimePackageManager::plan_catalog_fleet_upgrades(&nodes, catalog, &platform);
        let candidate_count = plan.ready_count();

        if candidate_count == 0 {
            // No candidates - just update last_checked_at
            let updated_policy = policy.with_checked_at(now_unix);
            let _ = state
                .repository
                .save_runtime_upgrade_policy(&updated_policy);
            return;
        }

        // 9. 记录开始事件 (fleet 级事件，使用 None)
        let event_message = format!(
            "Runtime upgrade policy triggered: {} ready ({} stopped, {} running)",
            candidate_count,
            plan.stopped_ready_count(),
            plan.running_ready_count()
        );
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::RuntimeUpgradePolicyRun,
            severity: EventSeverity::Info,
            message: event_message,
        });

        // 10. 分批执行升级（最多 max_nodes_per_run）
        let max_per_run = policy.max_nodes_per_run;
        let mut success_count = 0usize;

        // Get full node info for each candidate by looking up by node_id
        let all_nodes_map: HashMap<String, NodeConfig> =
            nodes.iter().cloned().map(|n| (n.id.clone(), n)).collect();

        for candidate in plan.into_ready_candidates() {
            if success_count >= max_per_run {
                break; // reach limit
            }

            let node_id = candidate.node_id.clone();
            let node_name = candidate.node_name.clone();
            let from_ver = candidate.from_version.clone();
            let to_ver = candidate.to_version.clone();
            let release = &candidate.release;

            // Look up full node config
            let node = match all_nodes_map.get(&node_id) {
                Some(n) => n,
                None => {
                    warn!("neo-nexus: node {} not found in lookup", node_name);
                    continue;
                }
            };

            // Track if node was running before upgrade
            let was_running_before_upgrade = node.status.is_running();

            // Stop if running
            if was_running_before_upgrade && !node_id.is_empty() {
                match stop_node(state, node) {
                    Ok(_) => warn!(
                        "neo-nexus: stopped {} ({}) before upgrade",
                        node_name, node_id
                    ),
                    Err(e) => {
                        warn!(
                            "neo-nexus: failed to stop {} before upgrade: {e}",
                            node_name
                        );
                        continue;
                    }
                }
            }

            // Download binary using the release URL
            let download_dir = state.workspace_child_dir("runtimes/downloads");
            let download_request = release.download_request();

            match RuntimePackageManager::download_https(&download_request, &download_dir) {
                Ok(download) => {
                    // Create manifest pointing to downloaded file
                    let manifest = release.manifest_for_source(&download.path);

                    // Install runtime
                    let install_root = match node_workspace_path(
                        state.workspace_child_dir("runtimes/installed"),
                        &node_id,
                    ) {
                        Ok(path) => path,
                        Err(e) => {
                            warn!(
                                "neo-nexus: failed to create install path for {}: {e}",
                                node_name
                            );
                            continue;
                        }
                    };

                    match RuntimePackageManager::install(&manifest, &install_root) {
                        Ok(installation) => {
                            // Update node with new binary
                            let new_node = NewNode {
                                name: node.name.clone(),
                                node_type: node.node_type,
                                network: node.network,
                                binary_path: installation.binary_path.clone(),
                                args: node.args.clone(),
                                runtime_version: installation.version.clone(),
                                storage_engine: node.storage_engine,
                                rpc_port: node.rpc_port,
                                p2p_port: node.p2p_port,
                                ws_port: node.ws_port,
                            };

                            match state.repository.update_node(&node_id, new_node) {
                                Ok(_updated) => {
                                    // Restart node if it was running before
                                    if was_running_before_upgrade {
                                        match launch_node(state, node, LaunchAction::Start) {
                                            Ok(msg) => {
                                                info!("neo-nexus: {} upgraded from {from_ver} to {to_ver}: {msg}", node_name);
                                                success_count += 1;
                                            }
                                            Err(e) => {
                                                warn!("neo-nexus: failed to start {} after upgrade: {e}", node_name);
                                            }
                                        }
                                    } else {
                                        info!("neo-nexus: {} upgraded from {from_ver} to {to_ver} (not restarting)", node_name);
                                        success_count += 1;
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        "neo-nexus: failed to update node {} binary: {e}",
                                        node_name
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "neo-nexus: failed to install runtime for {}: {e}",
                                node_name
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "neo-nexus: failed to download runtime for {}: {e}",
                        node_name
                    );
                }
            }
        }

        // 11. 更新 policy 时间戳
        let updated_policy = policy.with_applied_at(now_unix);
        let _ = state
            .repository
            .save_runtime_upgrade_policy(&updated_policy);

        // 12. 记录最终结果
        let final_message = format!(
            "Runtime upgrade batch completed: {}/{} successful",
            success_count, candidate_count
        );
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::RuntimeFleetUpgradeRun,
            severity: EventSeverity::Info,
            message: final_message,
        });
    }
}
