import { isAbsolute, relative, resolve } from "node:path";
import type { NodeConfig, NodeInstance, NodeSettings } from "../types";
import { getNodePath, paths } from "../utils/paths";
import { getProcessArgv, getProcessCommand, getProcessCwd, isProcessAlive } from "../utils/lifecycle";

export type AttachedProcessState = "active" | "stale";

export function isValidProcessId(pid: unknown): pid is number {
  return Number.isSafeInteger(pid) && Number(pid) > 0;
}

export function parseProcessIds(output: string): number[] {
  const seen = new Set<number>();
  const pids: number[] = [];

  for (const line of output.split("\n")) {
    const pid = Number.parseInt(line.trim(), 10);
    if (isValidProcessId(pid) && !seen.has(pid)) {
      seen.add(pid);
      pids.push(pid);
    }
  }

  return pids;
}

export function getAttachedProcessState(node: NodeInstance, pid = node.process.pid): AttachedProcessState {
  if (!isValidProcessId(pid)) {
    return "stale";
  }

  if (!isProcessAlive(pid)) {
    return "stale";
  }

  return isExpectedNodeProcess(node, pid) ? "active" : "stale";
}

export function scoreAttachCandidate(node: NodeInstance, pid: number): number {
  if (!isValidProcessId(pid) || !isProcessAlive(pid)) {
    return 0;
  }

  const command = getProcessCommand(pid);
  if (!command || !commandMatchesNodeType(command, node.type)) {
    return 0;
  }

  let score = 2;
  const cwd = getProcessCwd(pid);
  if (cwd && isPathWithinOrEqual(cwd, node.paths.base)) {
    score += 4;
  }

  const argv = getProcessArgv(pid) ?? [];
  const pathCandidates = argv.flatMap((arg) => extractPathCandidates(arg));
  if (pathCandidates.some((candidate) => isAbsolute(candidate) && resolve(candidate) === resolve(node.paths.config))) {
    score += 8;
  }
  if (pathCandidates.some((candidate) => pathCandidateMatchesNode(candidate, node))) {
    score += 3;
  }

  const argvText = argv.join(" ");
  const expectedPorts = [node.ports.rpc, node.ports.p2p, node.ports.websocket, node.ports.metrics]
    .filter((port): port is number => Number.isInteger(port));
  for (const port of expectedPorts) {
    if (new RegExp(`(^|[^0-9])${port}([^0-9]|$)`).test(argvText)) {
      score += 1;
    }
  }

  return score;
}

export function isPathWithinOrEqual(pathToCheck: string, allowedPrefix: string): boolean {
  const resolvedPath = resolve(pathToCheck);
  const resolvedPrefix = resolve(allowedPrefix);
  const pathRelativeToPrefix = relative(resolvedPrefix, resolvedPath);
  return pathRelativeToPrefix === "" || (!pathRelativeToPrefix.startsWith("..") && !isAbsolute(pathRelativeToPrefix));
}

export function isManagedNodeDirectory(
  node: Pick<NodeInstance, "id" | "paths" | "settings">,
  managedNodesRoot = paths.nodes,
): boolean {
  if (node.settings?.import || !isSafeGeneratedNodeId(node.id)) {
    return false;
  }

  const managedRoot = resolve(managedNodesRoot);
  const expectedNodePath = resolve(managedNodesRoot === paths.nodes ? getNodePath(node.id) : `${managedNodesRoot}/${node.id}`);
  const expectedRelativeToRoot = relative(managedRoot, expectedNodePath);
  if (
    expectedRelativeToRoot === "" ||
    expectedRelativeToRoot.startsWith("..") ||
    isAbsolute(expectedRelativeToRoot) ||
    expectedRelativeToRoot.includes("/") ||
    expectedRelativeToRoot.includes("\\")
  ) {
    return false;
  }

  return resolve(node.paths.base) === expectedNodePath;
}

export function normalizeImportedOwnershipMode(mode: unknown): NonNullable<NodeSettings["import"]>["ownershipMode"] {
  if (mode === "managed-config" || mode === "managed-process") {
    return mode;
  }
  return "observe-only";
}

function isExpectedNodeProcess(node: NodeInstance, pid: number): boolean {
  const command = getProcessCommand(pid);
  if (!command || !commandMatchesNodeType(command, node.type)) {
    return false;
  }

  const cwd = getProcessCwd(pid);
  if (cwd && isPathWithinOrEqual(cwd, node.paths.base)) {
    return true;
  }

  const argv = getProcessArgv(pid);
  if (!argv) {
    return false;
  }

  return argv
    .flatMap((arg) => extractPathCandidates(arg))
    .some((candidate) => pathCandidateMatchesNode(candidate, node));
}

function pathCandidateMatchesNode(candidate: string, node: NodeInstance): boolean {
  if (!isAbsolute(candidate)) {
    return false;
  }

  return isPathWithinOrEqual(candidate, node.paths.base);
}

function extractPathCandidates(arg: string): string[] {
  const candidates = [arg];
  const equalsIndex = arg.indexOf("=");
  if (equalsIndex >= 0) {
    candidates.push(arg.slice(equalsIndex + 1));
  }
  return candidates.map((candidate) => candidate.trim()).filter(Boolean);
}

function commandMatchesNodeType(command: string, type: NodeConfig["type"]): boolean {
  const normalizedCommand = command.toLowerCase();
  if (type === "neo-cli") {
    return normalizedCommand.includes("neo-cli") || normalizedCommand.includes("neo.cli");
  }
  return normalizedCommand.includes("neo-go");
}

function isSafeGeneratedNodeId(nodeId: string): boolean {
  return /^node-[A-Za-z0-9-]+$/.test(nodeId);
}
