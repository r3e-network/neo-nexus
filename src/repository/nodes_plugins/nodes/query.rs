use super::*;

impl Repository {
    pub fn list_nodes(&self) -> Result<Vec<NodeConfig>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, name, node_type, network, binary_path, args,
                    runtime_version, storage_engine, rpc_port, p2p_port, ws_port, status, pid
             FROM nodes
             ORDER BY name COLLATE NOCASE ASC",
        )?;

        let rows = statement.query_map([], |row| {
            let node_type_raw: String = row.get(2)?;
            let network_raw: String = row.get(3)?;
            let storage_engine_raw: String = row.get(7)?;
            let status_raw: String = row.get(11)?;
            let args_raw: String = row.get(5)?;

            Ok(NodeConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                node_type: NodeType::from_str(&node_type_raw)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?,
                network: Network::from_str(&network_raw)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?,
                binary_path: PathBuf::from(row.get::<_, String>(4)?),
                args: decode_args(&args_raw),
                runtime_version: normalize_runtime_version(&row.get::<_, String>(6)?),
                storage_engine: StorageEngine::from_str(&storage_engine_raw)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?,
                rpc_port: row.get::<_, u16>(8)?,
                p2p_port: row.get::<_, u16>(9)?,
                ws_port: row.get::<_, Option<u16>>(10)?,
                status: NodeStatus::from_str(&status_raw)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?,
                pid: row.get::<_, Option<u32>>(12)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load nodes")
    }
}
