import type { NodeNetwork, NodeType } from "../types";
import { Errors } from "../api/errors";

const NODE_TYPES = new Set(["neo-cli", "neo-go"]);
const NODE_NETWORKS = new Set(["mainnet", "testnet", "private"]);
const RELEASE_VERSION_PATTERN = /^v?\d+(?:\.\d+){1,3}(?:[-+][0-9A-Za-z.-]+)?$/;

export function isNodeType(value: unknown): value is NodeType {
  return typeof value === "string" && NODE_TYPES.has(value);
}

export function isNodeNetwork(value: unknown): value is NodeNetwork {
  return typeof value === "string" && NODE_NETWORKS.has(value);
}

export function assertNodeType(value: unknown): NodeType {
  if (!isNodeType(value)) {
    throw Errors.invalidNodeType(String(value ?? ""));
  }
  return value;
}

export function assertNodeNetwork(value: unknown): NodeNetwork {
  if (!isNodeNetwork(value)) {
    throw Errors.invalidNodeNetwork(String(value ?? ""));
  }
  return value;
}

export function assertReleaseVersion(value: string): string {
  const version = value.trim();
  if (!RELEASE_VERSION_PATTERN.test(version)) {
    throw Errors.invalidReleaseVersion(value);
  }
  return version;
}
