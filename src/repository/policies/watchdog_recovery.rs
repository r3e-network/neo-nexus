use super::*;
use crate::watchdog::{RecoveryClaim, RecoveryState, RestartOutcome};
use std::collections::BTreeMap;

const PREFIX: &str = "watchdog.recovery.";

fn key(id: &str) -> String {
    format!("{PREFIX}{id}")
}

fn load(connection: &Connection, id: &str) -> Result<Option<RecoveryState>> {
    load_setting(connection, &key(id))?
        .map(|text| {
            let state: RecoveryState = serde_json::from_str(&text)
                .context("invalid node recovery record; stop the node to clear it")?;
            state.validate()?;
            Ok(state)
        })
        .transpose()
}

fn save(connection: &Connection, id: &str, state: &RecoveryState) -> Result<()> {
    connection.execute(
        "INSERT INTO workspace_settings(key,value) VALUES(?1,?2)
        ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key(id), serde_json::to_string(state)?],
    )?;
    Ok(())
}

fn erase(connection: &Connection, id: &str) -> Result<()> {
    connection.execute(
        "DELETE FROM workspace_settings WHERE key=?1",
        params![key(id)],
    )?;
    Ok(())
}

fn records(connection: &Connection) -> Result<BTreeMap<String, RecoveryState>> {
    let mut statement = connection.prepare(
        "SELECT key,value FROM workspace_settings
        WHERE key GLOB 'watchdog.recovery.*' AND substr(key,19) IN (SELECT id FROM nodes)",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.map(|row| {
        let (key, text) = row?;
        let state: RecoveryState = serde_json::from_str(&text).context(
            "invalid node recovery record; stop the affected node before automatic recovery",
        )?;
        state.validate()?;
        Ok((key[PREFIX.len()..].to_string(), state))
    })
    .collect()
}

fn node_state(connection: &Connection, id: &str) -> Result<Option<(String, Option<u32>)>> {
    Ok(connection
        .query_row(
            "SELECT status,pid FROM nodes WHERE id=?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?)
}

fn eligible(status: &str, pid: Option<u32>) -> bool {
    pid.is_none() && matches!(status, "crashed" | "error")
}

impl Repository {
    pub(crate) fn load_node_recoveries(&self) -> Result<BTreeMap<String, RecoveryState>> {
        let connection = self.connection()?;
        records(&connection)
    }

    /// Persist the failed node state and its next attempt together. A stale exit
    /// cannot resurrect a row that another controller has already stopped.
    pub(crate) fn schedule_node_recovery(
        &self,
        node: &NodeConfig,
        status: NodeStatus,
        now_ms: u64,
    ) -> Result<Option<RestartOutcome>> {
        if node.status == NodeStatus::Stopped {
            return Ok(None);
        }
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        if node_state(&transaction, &node.id)? != Some((node.status.to_string(), node.pid)) {
            return Ok(None);
        }
        let policy = super::watchdog::read_policy(&transaction)?;
        let mut recovery = match load(&transaction, &node.id) {
            Ok(recovery) => recovery.unwrap_or_default(),
            Err(error) => {
                // Keep the unreadable budget for review while persisting the
                // observed failure under the same node-state comparison.
                transaction.execute(
                    "UPDATE nodes SET status=?1,pid=NULL WHERE id=?2",
                    params![status.to_string(), node.id],
                )?;
                transaction.commit()?;
                return Err(error);
            }
        };
        let outcome = recovery.schedule(policy, now_ms);
        transaction.execute(
            "UPDATE nodes SET status=?1,pid=NULL WHERE id=?2",
            params![status.to_string(), node.id],
        )?;
        save(&transaction, &node.id, &recovery)?;
        transaction.commit()?;
        Ok(Some(outcome))
    }

    /// Claim before launching. Concurrent claimants cannot spend the same retry.
    pub(crate) fn claim_node_recovery(
        &self,
        id: &str,
        now_ms: u64,
    ) -> Result<Option<RecoveryClaim>> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let Some((status, pid)) = node_state(&transaction, id)? else {
            return Ok(None);
        };
        let Some(mut recovery) = load(&transaction, id)? else {
            return Ok(None);
        };
        let policy = super::watchdog::read_policy(&transaction)?;
        if !eligible(&status, pid)
            || !policy.enabled
            || recovery.attempts >= policy.max_restart_attempts
            || recovery.claim.is_some()
            || recovery
                .next_attempt_at_unix_ms
                .is_none_or(|at| at > now_ms)
        {
            return Ok(None);
        }
        recovery.attempts += 1;
        recovery.next_attempt_at_unix_ms = None;
        let token = Uuid::new_v4().to_string();
        recovery.claim = Some(token.clone());
        recovery.exhausted = false;
        save(&transaction, id, &recovery)?;
        transaction.commit()?;
        Ok(Some(RecoveryClaim {
            node_id: id.into(),
            attempt: recovery.attempts,
            token,
        }))
    }

    pub(crate) fn validate_recovery_claim(&self, claim: &RecoveryClaim) -> Result<()> {
        let connection = self.connection()?;
        let policy = super::watchdog::read_policy(&connection)?;
        let current = load(&connection, &claim.node_id)?;
        anyhow::ensure!(
            policy.enabled
                && policy.max_restart_attempts >= claim.attempt
                && current
                    .as_ref()
                    .is_some_and(|current| current.claim.as_deref() == Some(&claim.token)),
            "automatic restart was cancelled or its policy changed"
        );
        Ok(())
    }

    pub(crate) fn fail_node_recovery(
        &self,
        claim: &RecoveryClaim,
        now_ms: u64,
    ) -> Result<Option<RestartOutcome>> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let Some((status, pid)) = node_state(&transaction, &claim.node_id)? else {
            return Ok(None);
        };
        let Some(mut recovery) = load(&transaction, &claim.node_id)? else {
            return Ok(None);
        };
        if !eligible(&status, pid) || recovery.claim.as_deref() != Some(&claim.token) {
            return Ok(None);
        }
        let outcome = recovery.schedule(super::watchdog::read_policy(&transaction)?, now_ms);
        transaction.execute(
            "UPDATE nodes SET status='error',pid=NULL WHERE id=?1",
            params![claim.node_id],
        )?;
        save(&transaction, &claim.node_id, &recovery)?;
        transaction.commit()?;
        Ok(Some(outcome))
    }

    /// Repair only persisted in-flight work. Its consumed attempt is retained.
    /// Called once when the engine boots, never during a live launch.
    pub(crate) fn recover_interrupted_node_attempts(&self, now_ms: u64) -> Result<Vec<String>> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let policy = super::watchdog::read_policy(&transaction)?;
        let mut recovered = Vec::new();
        for (id, mut recovery) in records(&transaction)? {
            let Some((status, pid)) = node_state(&transaction, &id)? else {
                continue;
            };
            if status == "stopped" || status == "error" && pid.is_some() {
                erase(&transaction, &id)?;
                continue;
            }
            recovery.apply_policy(policy);
            if recovery.claim.is_some() {
                if eligible(&status, pid) {
                    recovery.schedule(policy, now_ms);
                    recovered.push(id.clone());
                } else {
                    recovery.claim = None;
                }
            }
            save(&transaction, &id, &recovery)?;
        }
        transaction.commit()?;
        Ok(recovered)
    }

    pub(in crate::repository) fn sync_node_recovery_status(
        connection: &Connection,
        id: &str,
        status: NodeStatus,
        pid: Option<u32>,
    ) -> Result<()> {
        if status == NodeStatus::Stopped || status == NodeStatus::Error && pid.is_some() {
            return erase(connection, id);
        }
        if status == NodeStatus::Running {
            // Automatic launch owns a durable claim. A manual successful start
            // has no claim and deliberately starts with a new recovery budget.
            match load(connection, id) {
                Ok(Some(mut recovery)) if recovery.claim.is_some() => {
                    recovery.claim = None;
                    save(connection, id, &recovery)?;
                }
                Ok(_) => erase(connection, id)?,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}

pub(super) fn apply_policy(connection: &Connection, policy: RestartPolicy) -> Result<()> {
    let records = match records(connection) {
        Ok(records) => records,
        // Disabling remains available when a ledger is unreadable. Keep the
        // record intact; enabling again requires explicit recovery/Stop.
        Err(_) if !policy.enabled => return Ok(()),
        Err(error) => return Err(error),
    };
    for (id, mut recovery) in records {
        recovery.apply_policy(policy);
        save(connection, &id, &recovery)?;
    }
    Ok(())
}
