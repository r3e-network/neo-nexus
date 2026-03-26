import Database from "better-sqlite3";
import { mkdirSync } from "node:fs";
import { paths } from "../utils/paths";
import bcrypt from "bcrypt";

export async function initializeDatabase(): Promise<Database.Database> {
  mkdirSync(paths.base, { recursive: true });
  const db = new Database(paths.database);

  // Enable WAL mode for better concurrency
  db.pragma("journal_mode = WAL");

  // Create tables
  db.exec(`
    CREATE TABLE IF NOT EXISTS nodes (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
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
      requires_config INTEGER DEFAULT 0,
      dependencies TEXT, -- JSON array
      default_config TEXT -- JSON
    );

    CREATE TABLE IF NOT EXISTS node_plugins (
      node_id TEXT NOT NULL,
      plugin_id TEXT NOT NULL,
      version TEXT NOT NULL,
      config TEXT, -- JSON
      installed_at INTEGER NOT NULL,
      enabled INTEGER DEFAULT 1,
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
  `);

  ensureColumn(db, "secure_signer_profiles", "workspace_path", "TEXT");
  ensureColumn(db, "secure_signer_profiles", "startup_port", "INTEGER");
  ensureColumn(db, "secure_signer_profiles", "aws_region", "TEXT");
  ensureColumn(db, "secure_signer_profiles", "kms_key_id", "TEXT");
  ensureColumn(db, "secure_signer_profiles", "kms_ciphertext_blob_path", "TEXT");

  // Initialize plugin catalog
  initializePlugins(db);

  // Check if setup is needed and create default user
  await checkInitialSetup(db);

  return db;
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

async function checkInitialSetup(db: Database.Database): Promise<void> {
  // Check if any users exist
  const userCount = db.prepare("SELECT COUNT(*) as count FROM users").get() as { count: number };
  
  if (userCount.count === 0) {
    // Create default admin user
    const { randomUUID } = await import("node:crypto");
    const userId = randomUUID();
    const now = Date.now();
    const passwordHash = await bcrypt.hash("admin", 10);

    const stmt = db.prepare(`
      INSERT INTO users (id, username, password_hash, role, created_at)
      VALUES (?, ?, ?, ?, ?)
    `);
    stmt.run(userId, "admin", passwordHash, "admin", now);

    console.log("🔑 Default admin account created.");
    console.log("   Username: admin");
    console.log("   Password: admin");
    console.log("   ⚠️  IMPORTANT: Please change the default password after first login!");
    console.log("   Go to Settings → Change Password after logging in.\n");
  }
}
