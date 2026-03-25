import { PM2Manager } from "./PM2Manager";
import type Database from "better-sqlite3";

export async function performStartupHealthCheck(db: Database.Database, pm2: PM2Manager): Promise<void> {
  console.log("🔍 Performing startup health check...");

  // Connect to PM2
  await pm2.connect();

  // Get all nodes from database
  const nodes = db.prepare("SELECT id, name FROM nodes").all() as Array<{ id: string; name: string }>;

  // Get all PM2 processes
  const pm2Processes = await pm2.list();
  const pm2ProcessNames = new Set(pm2Processes.map((p) => p.name));

  // Sync process status
  for (const node of nodes) {
    const processName = `neo-node-${node.id}`;
    const pm2Info = pm2Processes.find((p) => p.name === processName);

    if (pm2Info) {
      // Update process status in database
      db.prepare(
        `
        UPDATE node_processes
        SET pid = ?, status = ?, uptime = ?
        WHERE node_id = ?
      `,
      ).run(pm2Info.pid || null, pm2Info.status, pm2Info.uptime || null, node.id);
    } else {
      // Process not found in PM2, mark as stopped
      db.prepare(
        `
        UPDATE node_processes
        SET pid = NULL, status = 'stopped'
        WHERE node_id = ?
      `,
      ).run(node.id);
    }
  }

  console.log("✅ Health check complete");
}
