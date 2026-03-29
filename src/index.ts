#!/usr/bin/env node

import { initializeDatabase } from './database/schema';
import { createAppServer } from './server';
import { paths } from './utils/paths';

const PORT = parseInt(process.env.PORT || '8080', 10);
const HOST = process.env.HOST || '0.0.0.0';

async function main() {
  console.log('🚀 Starting NeoNexus Node Manager...');
  console.log(`📁 Data directory: ${paths.base}`);

  try {
    // Initialize database
    console.log('📦 Initializing database...');
    const db = await initializeDatabase();

    // Create and start server
    const server = createAppServer({
      port: PORT,
      host: HOST,
      db,
    });

    // Handle graceful shutdown
    process.on('SIGTERM', async () => {
      await server.stop();
      process.exit(0);
    });

    process.on('SIGINT', async () => {
      await server.stop();
      process.exit(0);
    });

    // Handle uncaught errors - exit after logging
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

    console.log('✅ NeoNexus Node Manager is ready!');
    console.log(`🌐 Open http://${HOST === '0.0.0.0' ? 'localhost' : HOST}:${PORT} in your browser`);
  } catch (error) {
    console.error('❌ Failed to start:', error);
    process.exit(1);
  }
}

main();
