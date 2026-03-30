#!/usr/bin/env node

import { initializeDatabase } from './database/schema';
import { createAppServer } from './server';
import { paths } from './utils/paths';
import { detectEnvironment } from './utils/environment';
import { printStartupBanner } from './utils/startup';

const PORT = parseInt(process.env.PORT || '8080', 10);
const HOST = process.env.HOST || '0.0.0.0';

async function main() {
  try {
    const db = await initializeDatabase();

    const server = createAppServer({
      port: PORT,
      host: HOST,
      db,
    });

    process.on('SIGTERM', async () => {
      await server.stop();
      process.exit(0);
    });

    process.on('SIGINT', async () => {
      await server.stop();
      process.exit(0);
    });

    process.on('uncaughtException', async (error) => {
      console.error('Uncaught Exception:', error);
      server.integrationManager.captureError(error, { source: 'uncaughtException' });
      await server.stop();
      process.exit(1);
    });

    process.on('unhandledRejection', (reason) => {
      console.error('Unhandled Rejection:', reason);
    });

    await server.start();

    // Gather startup info and print structured banner
    const env = detectEnvironment();
    const protocol = process.env.HTTPS_ENABLED === 'true' ? 'https' : 'http';
    const displayHost = HOST === '0.0.0.0' ? 'localhost' : HOST;
    const nodes = server.nodeManager.getAllNodes();
    const runningCount = nodes.filter(n => n.process.status === 'running').length;

    let isFirstRun = false;
    let hasDefaultPassword = false;
    try {
      const users = server.userManager.getAllUsers();
      isFirstRun = users.length === 0;
      if (!isFirstRun) {
        const admins = users.filter((u: { role: string }) => u.role === 'admin');
        hasDefaultPassword = admins.length > 0
          ? await Promise.resolve(server.userManager.isUsingDefaultPassword(admins[0].id)).catch(() => false)
          : false;
      }
    } catch {
      // Ignore — use defaults
    }

    printStartupBanner({
      version: server.app.get('appVersion') || '0.0.0',
      url: `${protocol}://${displayHost}:${PORT}`,
      dataDir: paths.base,
      nodeCount: nodes.length,
      runningCount,
      isFirstRun,
      hasDefaultPassword,
      isBoundToAllInterfaces: HOST === '0.0.0.0',
      env,
    });
  } catch (error) {
    console.error('Failed to start:', error);
    process.exit(1);
  }
}

main();
