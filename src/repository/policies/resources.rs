use super::*;
use crate::resource_health::{ResourcePolicy, ResourceReport};

impl Repository {
    pub fn load_resource_policy(&self) -> Result<ResourcePolicy> {
        let policy: ResourcePolicy = load_setting(&*self.connection()?, "resource_health.policy")?
            .map(|text| serde_json::from_str(&text))
            .transpose()
            .context("invalid resource monitor policy")?
            .unwrap_or_default();
        policy.validate()?;
        Ok(policy)
    }

    pub fn save_resource_policy(&self, policy: &ResourcePolicy) -> Result<()> {
        policy.validate()?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            "resource_health.policy",
            &serde_json::to_string(policy)?,
        )?;
        transaction.execute(
            "DELETE FROM workspace_settings WHERE key='resource_health.snapshot'",
            [],
        )?;
        transaction.execute("INSERT INTO runtime_events(occurred_at_unix,node_id,node_name,kind,severity,message) VALUES(?1,NULL,NULL,?2,'info',?3)",
            params![current_unix_time()?,crate::events::EventKind::ResourcePolicyUpdated.to_string(),"Host resource thresholds and monitored storage paths updated"])?;
        transaction.commit()?;
        Ok(())
    }

    pub fn latest_resource_report(&self) -> Result<Option<ResourceReport>> {
        load_setting(&*self.connection()?, "resource_health.snapshot")?
            .map(|text| serde_json::from_str(&text))
            .transpose()
            .context("invalid resource observation")
    }

    pub(crate) fn record_resource_report(&self, mut report: ResourceReport) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let policy: ResourcePolicy = load_setting(&transaction, "resource_health.policy")?
            .map(|text| serde_json::from_str(&text))
            .transpose()?
            .unwrap_or_default();
        if !policy.enabled || policy != report.policy {
            return Ok(());
        }
        let previous: Option<ResourceReport> =
            load_setting(&transaction, "resource_health.snapshot")?
                .map(|text| serde_json::from_str(&text))
                .transpose()?;
        if let Some(old) = &previous {
            let now = current_unix_time()?;
            // An actual wall-clock correction must not freeze observations until
            // the previous time catches up. Ordinary delayed/duplicate samples
            // still cannot replace a newer observation.
            let clock_corrected = old.checked_at_unix > now.saturating_add(5)
                && report.checked_at_unix.abs_diff(now) <= 5;
            if old.checked_at_unix >= report.checked_at_unix && !clock_corrected {
                return Ok(());
            }
        }
        for event in crate::resource_health::settle(&mut report, previous.as_ref()) {
            transaction.execute("INSERT INTO runtime_events(occurred_at_unix,node_id,node_name,kind,severity,message) VALUES(?1,NULL,NULL,?2,?3,?4)",
                params![report.checked_at_unix,event.kind.to_string(),event.severity.to_string(),event.message])?;
        }
        save_setting(
            &transaction,
            "resource_health.snapshot",
            &serde_json::to_string(&report)?,
        )?;
        transaction.commit()?;
        Ok(())
    }
}
