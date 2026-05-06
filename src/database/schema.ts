import Database from "better-sqlite3";
import { mkdirSync } from "node:fs";
import { paths } from "../utils/paths";

export async function initializeDatabase(): Promise<Database.Database> {
  mkdirSync(paths.base, { recursive: true });
  const db = new Database(paths.database);

  // Enable WAL mode for better concurrency
  db.pragma("journal_mode = WAL");
  // SQLite does not guarantee foreign key enforcement for every connection
  // unless it is explicitly enabled. The schema relies on ON DELETE CASCADE
  // for node, user, conversation, and plugin cleanup.
  db.pragma("foreign_keys = ON");

  // Create tables
  db.exec(`
    CREATE TABLE IF NOT EXISTS nodes (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      chain TEXT NOT NULL DEFAULT 'n3',
      type TEXT NOT NULL,
      network TEXT NOT NULL,
      sync_mode TEXT NOT NULL DEFAULT 'full',
      version TEXT NOT NULL,
      rpc_port INTEGER NOT NULL,
      p2p_port INTEGER NOT NULL,
      websocket_port INTEGER,
      metrics_port INTEGER,
      base_path TEXT NOT NULL,
      data_path TEXT NOT NULL,
      logs_path TEXT NOT NULL,
      config_path TEXT NOT NULL,
      wallet_path TEXT,
      settings TEXT, -- JSON
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS node_processes (
      node_id TEXT PRIMARY KEY,
      pid INTEGER,
      status TEXT NOT NULL,
      uptime INTEGER,
      exit_code INTEGER,
      error_message TEXT,
      last_started INTEGER,
      last_stopped INTEGER,
      FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS node_metrics (
      node_id TEXT PRIMARY KEY,
      block_height INTEGER DEFAULT 0,
      header_height INTEGER DEFAULT 0,
      connected_peers INTEGER DEFAULT 0,
      unconnected_peers INTEGER DEFAULT 0,
      sync_progress REAL DEFAULT 0,
      memory_usage INTEGER DEFAULT 0,
      cpu_usage REAL DEFAULT 0,
      last_update INTEGER,
      FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS plugins (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      category TEXT NOT NULL,
      requires_config INTEGER DEFAULT 0 CHECK (requires_config IN (0, 1)),
      dependencies TEXT, -- JSON array
      default_config TEXT -- JSON
    );

    CREATE TABLE IF NOT EXISTS node_plugins (
      node_id TEXT NOT NULL,
      plugin_id TEXT NOT NULL,
      version TEXT NOT NULL,
      config TEXT, -- JSON
      installed_at INTEGER NOT NULL,
      enabled INTEGER DEFAULT 1 CHECK (enabled IN (0, 1)),
      PRIMARY KEY (node_id, plugin_id),
      FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
      FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS logs (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      node_id TEXT NOT NULL,
      timestamp INTEGER NOT NULL,
      level TEXT NOT NULL,
      source TEXT,
      message TEXT NOT NULL,
      FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS metrics_history (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      node_id TEXT NOT NULL,
      block_height INTEGER,
      memory_usage INTEGER,
      cpu_usage REAL,
      timestamp INTEGER NOT NULL,
      FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_logs_node_time ON logs(node_id, timestamp);
    CREATE INDEX IF NOT EXISTS idx_metrics_history_node_time ON metrics_history(node_id, timestamp);
    CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(type);
    CREATE INDEX IF NOT EXISTS idx_nodes_status ON node_processes(status);
    CREATE INDEX IF NOT EXISTS idx_node_metrics_node_id ON node_metrics(node_id);
    CREATE INDEX IF NOT EXISTS idx_node_plugins_plugin_id ON node_plugins(plugin_id);

    CREATE TABLE IF NOT EXISTS node_role_profiles (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      kind TEXT NOT NULL CHECK (kind IN ('builtin', 'custom')),
      node_types TEXT NOT NULL,
      profile TEXT NOT NULL,
      created_by TEXT,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS node_role_applications (
      id TEXT PRIMARY KEY,
      node_id TEXT NOT NULL,
      role_id TEXT NOT NULL,
      role_name TEXT NOT NULL,
      application_plan TEXT NOT NULL,
      previous_state TEXT,
      applied_at INTEGER NOT NULL,
      applied_by TEXT,
      status TEXT NOT NULL CHECK (status IN ('planned', 'applied', 'failed')),
      error_message TEXT,
      FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_node_role_applications_node ON node_role_applications(node_id, applied_at);

    CREATE TABLE IF NOT EXISTS node_data_contexts (
      id TEXT PRIMARY KEY,
      node_id TEXT NOT NULL,
      label TEXT NOT NULL,
      storage_engine TEXT NOT NULL CHECK (storage_engine IN ('leveldb', 'rocksdb')),
      sync_strategy TEXT NOT NULL CHECK (sync_strategy IN ('full', 'light', 'fast-sync')),
      checkpoint_height INTEGER,
      checkpoint_hash TEXT,
      snapshot_id TEXT,
      active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1)),
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL,
      FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_node_data_contexts_node ON node_data_contexts(node_id, active);
    CREATE UNIQUE INDEX IF NOT EXISTS idx_node_data_contexts_one_active ON node_data_contexts(node_id) WHERE active = 1;

    CREATE TABLE IF NOT EXISTS fast_sync_snapshots (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      source_type TEXT NOT NULL CHECK (source_type IN ('local', 'url', 'catalog')),
      source TEXT NOT NULL,
      chain TEXT NOT NULL,
      network TEXT NOT NULL,
      node_type TEXT NOT NULL,
      storage_engine TEXT NOT NULL CHECK (storage_engine IN ('leveldb', 'rocksdb')),
      height INTEGER NOT NULL,
      block_hash TEXT,
      sha256 TEXT NOT NULL,
      size_bytes INTEGER,
      signature TEXT,
      trusted INTEGER NOT NULL DEFAULT 0 CHECK (trusted IN (0, 1)),
      created_at INTEGER NOT NULL,
      last_verified_at INTEGER
    );
    CREATE INDEX IF NOT EXISTS idx_fast_sync_snapshots_network ON fast_sync_snapshots(chain, network, node_type, storage_engine);

    CREATE TABLE IF NOT EXISTS private_network_plans (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      template TEXT NOT NULL CHECK (template IN ('single', 'four', 'seven')),
      network_magic INTEGER NOT NULL,
      plan TEXT NOT NULL,
      status TEXT NOT NULL CHECK (status IN ('draft', 'applied', 'failed')),
      created_at INTEGER NOT NULL,
      applied_at INTEGER
    );

    -- Users table for authentication
    CREATE TABLE IF NOT EXISTS users (
      id TEXT PRIMARY KEY,
      username TEXT UNIQUE NOT NULL,
      password_hash TEXT NOT NULL,
      role TEXT DEFAULT 'admin',
      created_at INTEGER NOT NULL,
      last_login INTEGER
    );

    -- Sessions table for tracking logins
    CREATE TABLE IF NOT EXISTS sessions (
      id TEXT PRIMARY KEY,
      user_id TEXT NOT NULL,
      token TEXT UNIQUE NOT NULL,
      expires_at INTEGER NOT NULL,
      created_at INTEGER NOT NULL,
      FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
    CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);

    CREATE TABLE IF NOT EXISTS remote_servers (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      base_url TEXT NOT NULL,
      description TEXT,
      enabled INTEGER NOT NULL DEFAULT 1,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_remote_servers_enabled ON remote_servers(enabled);

    CREATE TABLE IF NOT EXISTS secure_signer_profiles (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      mode TEXT NOT NULL,
      endpoint TEXT NOT NULL,
      public_key TEXT,
      account_address TEXT,
      wallet_path TEXT,
      unlock_mode TEXT NOT NULL,
      notes TEXT,
      enabled INTEGER NOT NULL DEFAULT 1,
      workspace_path TEXT,
      startup_port INTEGER,
      aws_region TEXT,
      kms_key_id TEXT,
      kms_ciphertext_blob_path TEXT,
      last_test_status TEXT,
      last_test_message TEXT,
      last_tested_at INTEGER,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_secure_signer_profiles_enabled ON secure_signer_profiles(enabled);

    CREATE TABLE IF NOT EXISTS audit_log (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      timestamp INTEGER NOT NULL,
      user_id TEXT,
      username TEXT,
      action TEXT NOT NULL,
      resource_type TEXT NOT NULL,
      resource_id TEXT,
      details TEXT,
      ip_address TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);

    CREATE TABLE IF NOT EXISTS integrations (
      id TEXT PRIMARY KEY,
      category TEXT NOT NULL,
      enabled INTEGER DEFAULT 0,
      config TEXT NOT NULL DEFAULT '{}',
      last_test_at TEXT,
      last_error TEXT,
      created_at TEXT DEFAULT (datetime('now')),
      updated_at TEXT DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS agent_settings (
      user_id TEXT PRIMARY KEY,
      provider TEXT NOT NULL,
      model TEXT NOT NULL,
      api_key TEXT NOT NULL,
      base_url TEXT,
      enabled INTEGER NOT NULL DEFAULT 1,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL,
      FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS agent_conversations (
      id TEXT PRIMARY KEY,
      user_id TEXT NOT NULL,
      title TEXT,
      provider TEXT NOT NULL,
      model TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL,
      FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_agent_conversations_user ON agent_conversations(user_id, updated_at);

    CREATE TABLE IF NOT EXISTS agent_messages (
      id TEXT PRIMARY KEY,
      conversation_id TEXT NOT NULL,
      role TEXT NOT NULL,
      content TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      FOREIGN KEY (conversation_id) REFERENCES agent_conversations(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS idx_agent_messages_conv ON agent_messages(conversation_id, created_at);
  `);

  ensureChainColumn(db);
  ensureColumn(db, "secure_signer_profiles", "workspace_path", "TEXT");
  ensureColumn(db, "secure_signer_profiles", "startup_port", "INTEGER");
  ensureColumn(db, "secure_signer_profiles", "aws_region", "TEXT");
  ensureColumn(db, "secure_signer_profiles", "kms_key_id", "TEXT");
  ensureColumn(db, "secure_signer_profiles", "kms_ciphertext_blob_path", "TEXT");

  // Initialize plugin catalog
  initializePlugins(db);

  return db;
}

/**
 * Add the `chain` column to existing nodes tables. New databases get this from
 * the CREATE TABLE above; pre-existing rows backfill to 'n3'.
 */
function ensureChainColumn(db: Database.Database): void {
  try {
    db.exec("ALTER TABLE nodes ADD COLUMN chain TEXT NOT NULL DEFAULT 'n3'");
  } catch (error) {
    if (!String(error).includes("duplicate column")) {
      throw error;
    }
  }
}

const VALID_IDENTIFIER = /^[a-zA-Z_][a-zA-Z0-9_]*$/;

function ensureColumn(db: Database.Database, table: string, column: string, definition: string): void {
  if (!VALID_IDENTIFIER.test(table) || !VALID_IDENTIFIER.test(column) || !VALID_IDENTIFIER.test(definition)) {
    throw new Error(`Invalid identifier in ensureColumn: ${table}.${column} ${definition}`);
  }
  try {
    db.exec(`ALTER TABLE ${table} ADD COLUMN ${column} ${definition}`);
  } catch (error) {
    if (!String(error).includes("duplicate column")) {
      throw error;
    }
  }
}

function initializePlugins(db: Database.Database): void {
  const plugins = [
    {
      id: "ApplicationLogs",
      name: "Application Logs",
      description: "Synchronizes smart contract execution logs into node storage",
      category: "Core",
      requires_config: 1,
      default_config: JSON.stringify({
        Network: 860833102,
        MaxLogSize: 2147483647,
      }),
    },
    {
      id: "DBFTPlugin",
      name: "dBFT Consensus",
      description: "Provides the dBFT consensus algorithm for consensus nodes",
      category: "Core",
      requires_config: 1,
      default_config: JSON.stringify({
        Network: 860833102,
        AutoStart: true,
        BlockTxNumber: 512,
        MaxBlockSize: 262144,
      }),
    },
    {
      id: "LevelDBStore",
      name: "LevelDB Store",
      description: "Uses LevelDB for the underlying node storage engine",
      category: "Storage",
      requires_config: 0,
    },
    {
      id: "RocksDBStore",
      name: "RocksDB Store",
      description: "Uses RocksDB for the underlying node storage engine (faster than LevelDB)",
      category: "Storage",
      requires_config: 0,
    },
    {
      id: "OracleService",
      name: "Oracle Service",
      description: "Enables the node to participate in the native Oracle network",
      category: "Core",
      requires_config: 1,
      default_config: JSON.stringify({
        Network: 860833102,
        AutoStart: true,
      }),
    },
    {
      id: "RestServer",
      name: "REST Server",
      description: "Provides a RESTful API interface for interacting with the Neo node",
      category: "API",
      requires_config: 1,
      default_config: JSON.stringify({
        Network: 860833102,
        BindAddress: "0.0.0.0",
        Port: 10334,
        KeepAliveTimeout: 60,
      }),
    },
    {
      id: "RpcServer",
      name: "RPC Server",
      description: "Provides the standard JSON-RPC interface for the Neo node",
      category: "API",
      requires_config: 1,
      default_config: JSON.stringify({
        Network: 860833102,
        BindAddress: "0.0.0.0",
        Port: 10332,
        MaxConcurrentConnections: 40,
        KeepAliveTimeout: 60,
        DisabledMethods: [],
      }),
    },
    {
      id: "SignClient",
      name: "Sign Client",
      description: "Allows node to securely communicate with a remote multi-sig wallet",
      category: "Tooling",
      requires_config: 0,
    },
    {
      id: "SQLiteWallet",
      name: "SQLite Wallet",
      description: "Allows Neo-CLI to open and manage SQLite based NEP-6 wallets",
      category: "Tooling",
      requires_config: 0,
    },
    {
      id: "StateService",
      name: "State Service",
      description: "Provides MPT state root tracking and validation",
      category: "Core",
      requires_config: 1,
      default_config: JSON.stringify({
        Network: 860833102,
        AutoStart: true,
        FullState: false,
      }),
    },
    {
      id: "StorageDumper",
      name: "Storage Dumper",
      description: "Provides tools for dumping and migrating Neo node storage states",
      category: "Tooling",
      requires_config: 0,
    },
    {
      id: "TokensTracker",
      name: "Tokens Tracker",
      description: "Tracks NEP-11 and NEP-17 token transfers and balances",
      category: "API",
      requires_config: 1,
      default_config: JSON.stringify({
        Network: 860833102,
      }),
    },
  ];

  const insert = db.prepare(`
    INSERT OR IGNORE INTO plugins (id, name, description, category, requires_config, default_config)
    VALUES (?, ?, ?, ?, ?, ?)
  `);

  for (const plugin of plugins) {
    insert.run(
      plugin.id,
      plugin.name,
      plugin.description,
      plugin.category,
      plugin.requires_config,
      plugin.default_config || null,
    );
  }
}
