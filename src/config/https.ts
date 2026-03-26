import { readFileSync, existsSync } from "node:fs";

export interface HttpsConfig {
  enabled: boolean;
  keyPath?: string;
  certPath?: string;
}

export function loadHttpsConfig(): HttpsConfig {
  const enabled = process.env.HTTPS_ENABLED === "true";
  const keyPath = process.env.HTTPS_KEY_PATH;
  const certPath = process.env.HTTPS_CERT_PATH;

  if (enabled && (!keyPath || !certPath)) {
    throw new Error("HTTPS enabled but HTTPS_KEY_PATH or HTTPS_CERT_PATH not set");
  }

  return { enabled, keyPath, certPath };
}

export function loadHttpsCredentials(config: HttpsConfig) {
  if (!config.enabled || !config.keyPath || !config.certPath) {
    return null;
  }

  if (!existsSync(config.keyPath)) {
    throw new Error(`HTTPS key file not found: ${config.keyPath}`);
  }

  if (!existsSync(config.certPath)) {
    throw new Error(`HTTPS cert file not found: ${config.certPath}`);
  }

  return {
    key: readFileSync(config.keyPath, "utf8"),
    cert: readFileSync(config.certPath, "utf8"),
  };
}
