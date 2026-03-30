import { RuntimeEnvironment } from "./environment";

interface StartupInfo {
  version: string;
  url: string;
  dataDir: string;
  nodeCount: number;
  runningCount: number;
  isFirstRun: boolean;
  hasDefaultPassword: boolean;
  isBoundToAllInterfaces: boolean;
  env: RuntimeEnvironment;
}

export function printStartupBanner(info: StartupInfo): void {
  console.log(`\nNeoNexus v${info.version}\n`);

  if (info.isFirstRun) {
    console.log("  Getting started:");
    console.log(`    1. Open ${info.url}`);
    console.log("    2. Create your admin account");
    console.log("    3. Deploy your first Neo node");
    console.log(`\n  Data: ${info.dataDir}\n`);
    return;
  }

  console.log(`  URL:   ${info.url}`);
  console.log(`  Nodes: ${info.nodeCount} managed (${info.runningCount} running)`);
  console.log(`  Data:  ${info.dataDir}`);

  const warnings: string[] = [];

  if (info.hasDefaultPassword) {
    warnings.push("Default admin password is still in use. Change it in Settings.");
  }

  if (info.isBoundToAllInterfaces && !info.env.isDocker) {
    warnings.push("Server is bound to all interfaces (0.0.0.0). Ensure firewall is configured.");
  }

  if (warnings.length > 0) {
    console.log("\n  Warnings:");
    for (const warning of warnings) {
      console.log(`    - ${warning}`);
    }
  }

  console.log("");
}

export function printShutdownMessage(stoppedCount: number): void {
  console.log(`Shutting down...${stoppedCount > 0 ? ` stopped ${stoppedCount} node${stoppedCount === 1 ? "" : "s"}.` : ""} Goodbye.`);
}
