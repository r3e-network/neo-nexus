use super::*;
use std::collections::BTreeMap;

type AlertProgress = (i64, BTreeMap<i64, usize>);
const KEY: &str = "alert_routing.progress";

impl Repository {
    /// Operational delivery state is excluded from portable workspace backups.
    pub fn load_alert_progress(&self) -> Result<Option<AlertProgress>> {
        load_setting(&self.connection()?, KEY)?
            .map(|text| serde_json::from_str(&text).context("invalid alert delivery progress"))
            .transpose()
    }

    pub fn save_alert_progress(&self, cursor: i64, attempts: &BTreeMap<i64, usize>) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            KEY,
            &serde_json::to_string(&(cursor, attempts))?,
        )?;
        transaction.commit()?;
        Ok(())
    }
}
