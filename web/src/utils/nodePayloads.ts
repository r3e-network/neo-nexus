export interface NodeFormValues {
  name: string;
  type: "neo-cli" | "neo-go";
  network: "mainnet" | "testnet" | "private";
  syncMode: "full" | "light";
  maxConnections: string;
  minPeers: string;
  maxPeers: string;
  relay: boolean;
  debugMode: boolean;
  customConfig: string;
  keyProtectionMode: "standard" | "secure-signer";
  secureSignerProfileId: string;
}

interface NodeLike {
  name: string;
  type?: "neo-cli" | "neo-go";
  network?: "mainnet" | "testnet" | "private";
  syncMode?: "full" | "light";
  settings?: {
    maxConnections?: number;
    minPeers?: number;
    maxPeers?: number;
    relay?: boolean;
    debugMode?: boolean;
    customConfig?: Record<string, unknown>;
    keyProtection?: {
      mode?: "standard" | "secure-signer";
      signerProfileId?: string;
    };
  };
}

function parseOptionalInteger(value: string): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return undefined;
  }

  const parsed = Number.parseInt(trimmed, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error("Numeric settings must be valid integers");
  }

  return parsed;
}

export function normalizeNodeUpsertPayload(values: Partial<NodeFormValues>) {
  const name = values.name?.trim() || "";
  const settings: Record<string, unknown> = {};

  const maxConnections = parseOptionalInteger(values.maxConnections || "");
  const minPeers = parseOptionalInteger(values.minPeers || "");
  const maxPeers = parseOptionalInteger(values.maxPeers || "");

  if (maxConnections !== undefined) settings.maxConnections = maxConnections;
  if (minPeers !== undefined) settings.minPeers = minPeers;
  if (maxPeers !== undefined) settings.maxPeers = maxPeers;
  if (typeof values.relay === "boolean") settings.relay = values.relay;
  if (typeof values.debugMode === "boolean") settings.debugMode = values.debugMode;

  const customConfigRaw = values.customConfig?.trim();
  if (customConfigRaw) {
    try {
      settings.customConfig = JSON.parse(customConfigRaw) as Record<string, unknown>;
    } catch {
      throw new Error("Custom config must be valid JSON");
    }
  }

  if (values.keyProtectionMode === "secure-signer") {
    const signerProfileId = values.secureSignerProfileId?.trim();
    if (!signerProfileId) {
      throw new Error("Select a secure signer profile");
    }

    settings.keyProtection = {
      mode: "secure-signer",
      signerProfileId,
    };
  }

  return {
    ...(name ? { name } : {}),
    ...(values.type ? { type: values.type } : {}),
    ...(values.network ? { network: values.network } : {}),
    ...(values.syncMode ? { syncMode: values.syncMode } : {}),
    ...(Object.keys(settings).length > 0 ? { settings } : {}),
  };
}

export function toNodeFormValues(node: NodeLike): NodeFormValues {
  return {
    name: node.name,
    type: node.type || "neo-go",
    network: node.network || "mainnet",
    syncMode: node.syncMode || "full",
    maxConnections: node.settings?.maxConnections?.toString() || "",
    minPeers: node.settings?.minPeers?.toString() || "",
    maxPeers: node.settings?.maxPeers?.toString() || "",
    relay: node.settings?.relay ?? true,
    debugMode: node.settings?.debugMode ?? false,
    customConfig: node.settings?.customConfig ? JSON.stringify(node.settings.customConfig, null, 2) : "",
    keyProtectionMode: node.settings?.keyProtection?.mode || "standard",
    secureSignerProfileId: node.settings?.keyProtection?.signerProfileId || "",
  };
}
