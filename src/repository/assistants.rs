use super::*;
use crate::{
    agents::AgentKind,
    assistants::{AssistantDraft, AssistantProfile},
};
use sha2::{Digest, Sha256};

fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssistantProfile> {
    let nodes: String = row.get(3)?;
    let node_ids = serde_json::from_str(&nodes).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(AssistantProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        agent_id: row.get(2)?,
        node_ids,
        all_nodes: row.get(4)?,
        can_operate: row.get(5)?,
        enabled: row.get(6)?,
    })
}

impl Repository {
    pub fn list_assistants(&self) -> Result<Vec<AssistantProfile>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT id,name,agent_id,node_ids,all_nodes,can_operate,enabled FROM assistant_grants ORDER BY name,id")?;
        let rows = statement.query_map([], from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("cannot load assistant connections")
    }

    pub(crate) fn connect_assistant(
        &self,
        mut draft: AssistantDraft,
    ) -> Result<(AssistantProfile, String)> {
        if draft.id.is_empty() {
            draft.id = Uuid::new_v4().to_string();
        }
        if draft.id.len() > 80
            || !draft
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            anyhow::bail!("invalid assistant id");
        }
        if draft.name.trim().is_empty() || draft.name.len() > 120 {
            anyhow::bail!("assistant name is required and must be at most 120 bytes");
        }
        let agents = self.list_agents()?;
        let agent = agents
            .iter()
            .find(|agent| agent.profile.id == draft.agent_id)
            .context("Hermes instance not found")?;
        if agent.profile.kind != AgentKind::Hermes {
            anyhow::bail!("assistant connections require a Hermes instance");
        }
        if agent.pid.is_some() || agent.desired_running {
            anyhow::bail!("stop Hermes before connecting or changing permissions");
        }
        draft.node_ids.sort();
        draft.node_ids.dedup();
        if !draft.all_nodes && draft.node_ids.is_empty() {
            anyhow::bail!("choose at least one node or explicitly allow the fleet");
        }
        let nodes = self.list_nodes()?;
        if draft
            .node_ids
            .iter()
            .any(|id| !nodes.iter().any(|node| node.id == *id))
        {
            anyhow::bail!("one of the selected nodes no longer exists");
        }
        let token = format!("nnx_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let digest = format!("{:x}", Sha256::digest(token.as_bytes()));
        let profile = AssistantProfile {
            id: draft.id,
            name: draft.name.trim().to_string(),
            agent_id: draft.agent_id,
            node_ids: draft.node_ids,
            all_nodes: draft.all_nodes,
            can_operate: draft.can_operate,
            enabled: true,
        };
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute("INSERT INTO assistant_grants(id,name,agent_id,node_ids,all_nodes,can_operate,enabled,token_sha256)
            VALUES(?1,?2,?3,?4,?5,?6,1,?7) ON CONFLICT(id) DO UPDATE SET name=excluded.name,agent_id=excluded.agent_id,node_ids=excluded.node_ids,all_nodes=excluded.all_nodes,can_operate=excluded.can_operate,enabled=1,token_sha256=excluded.token_sha256",
            params![profile.id, profile.name, profile.agent_id, serde_json::to_string(&profile.node_ids)?, profile.all_nodes, profile.can_operate, digest])?;
        record_grant_change(
            &transaction,
            &format!(
                "Assistant {} connected to Hermes {}; scope={}, operate={}",
                profile.name,
                profile.agent_id,
                if profile.all_nodes {
                    "fleet".to_string()
                } else {
                    profile.node_ids.join(",")
                },
                profile.can_operate
            ),
        )?;
        transaction.commit()?;
        Ok((profile, token))
    }

    pub fn revoke_assistant(&self, id: &str) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE assistant_grants SET enabled=0 WHERE id=?1",
            params![id],
        )?;
        record_grant_change(&transaction, &format!("Assistant {id} access revoked"))?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn authenticate_assistant(&self, token: &str) -> Result<Option<AssistantProfile>> {
        if !token.starts_with("nnx_") || token.len() != 68 {
            return Ok(None);
        }
        let digest = format!("{:x}", Sha256::digest(token.as_bytes()));
        self.connection()?.query_row("SELECT id,name,agent_id,node_ids,all_nodes,can_operate,enabled FROM assistant_grants
            WHERE token_sha256=?1 AND enabled=1 AND EXISTS(SELECT 1 FROM managed_agents WHERE managed_agents.id=assistant_grants.agent_id)", params![digest], from_row).optional().context("assistant authentication unavailable")
    }
}

fn record_grant_change(connection: &Connection, message: &str) -> Result<()> {
    connection.execute(
        "INSERT INTO runtime_events(occurred_at_unix,node_id,node_name,kind,severity,message)
        VALUES(?1,NULL,NULL,?2,?3,?4)",
        params![
            current_unix_time()?,
            crate::events::EventKind::AssistantConfigured.to_string(),
            EventSeverity::Info.to_string(),
            message
        ],
    )?;
    Ok(())
}
