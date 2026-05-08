import { createWriteStream, existsSync, lstatSync, mkdirSync, readdirSync, unlinkSync } from "node:fs";
import { copyFile, chmod } from "node:fs/promises";
import type { IncomingMessage } from "node:http";
import { request as httpsRequest } from "node:https";
import { isIP } from "node:net";
import { join } from "node:path";
import { pipeline } from "node:stream/promises";
import { paths } from "../utils/paths";
import { assertReleaseVersion } from "../utils/nodeValidation";
import {
  assertLiteralPublicTarget,
  assertResolvedPublicTarget,
  defaultHostnameResolver,
} from "../utils/outboundTargets";
import { publicFetch } from "../utils/publicFetch";
import type { NodeType, ReleaseInfo } from "../types/index";
import { ApiError } from "../api/errors";

const GITHUB_API_BASE = "https://api.github.com/repos";

// Repository mappings
const REPOS: Record<NodeType, { owner: string; repo: string; binaryName: string }> = {
  "neo-cli": {
    owner: "neo-project",
    repo: "neo-node",
    binaryName: "neo-cli",
  },
  "neo-go": {
    owner: "nspcc-dev",
    repo: "neo-go",
    binaryName: "neo-go",
  },
  "neox-go": {
    owner: "bane-labs",
    repo: "go-ethereum",
    binaryName: "geth",
  },
  // neo3fura ships as source via docker-compose / `go build`; there
  // are no GitHub release artifacts to auto-download. The entry is
  // present for the Record<NodeType,…> completeness check; download
  // paths must gate on isSidecarNodeType() before reaching here.
  "neofura": {
    owner: "neo-ngd",
    repo: "neo3fura",
    binaryName: "neo3fura_http",
  },
};

// Plugin repository
const PLUGIN_REPO = {
  owner: "neo-project",
  repo: "neo-modules",
};

export function hasUsableDownloadFile(path: string): boolean {
  try {
    const stats = lstatSync(path);
    return stats.isFile() && stats.size > 0;
  } catch {
    return false;
  }
}

async function extractArchive(source: string, destination: string): Promise<void> {
  const { default: extractZip } = await import("extract-zip");
  await extractZip(source, { dir: destination });
}

export function getNeoXAssetInfo(
  version: string,
  platform: NodeJS.Platform,
  arch: NodeJS.Architecture,
): {
  downloadFileName: string;
  assetName: string;
  binaryName: string;
  downloadUrl: string;
} {
  // bane-labs/go-ethereum publishes Linux binaries only. Reject other platforms
  // up-front so the failure is obvious instead of hitting a 404 mid-download.
  if (platform !== "linux") {
    throw new Error(`Neo X (geth) is published for Linux only; current platform is ${platform}.`);
  }
  const safeVersion = assertReleaseVersion(version);
  const normalizedVersion = safeVersion.startsWith("v") ? safeVersion : `v${safeVersion}`;
  const archSuffix = arch === "arm64" ? "arm64" : "amd64";
  const assetName = `geth-linux-${archSuffix}`;
  return {
    downloadFileName: `${assetName}-${normalizedVersion}`,
    assetName,
    binaryName: "geth",
    downloadUrl: `https://github.com/bane-labs/go-ethereum/releases/download/${normalizedVersion}/${assetName}`,
  };
}

export function getNeoGoAssetInfo(
  version: string,
  platform: NodeJS.Platform,
  arch: NodeJS.Architecture,
): {
  downloadFileName: string;
  assetName: string;
  binaryName: string;
  downloadUrl: string;
} {
  const safeVersion = assertReleaseVersion(version);
  const normalizedVersion = safeVersion.startsWith("v") ? safeVersion : `v${safeVersion}`;

  let assetName: string;
  let binaryName: string;

  if (platform === "win32") {
    assetName = "neo-go-windows-amd64.exe";
    binaryName = "neo-go.exe";
  } else if (platform === "darwin") {
    assetName = arch === "arm64" ? "neo-go-darwin-arm64" : "neo-go-darwin-amd64";
    binaryName = "neo-go";
  } else {
    assetName = arch === "arm64" ? "neo-go-linux-arm64" : "neo-go-linux-amd64";
    binaryName = "neo-go";
  }

  return {
    downloadFileName: `${assetName}-${normalizedVersion}`,
    assetName,
    binaryName,
    downloadUrl: `https://github.com/nspcc-dev/neo-go/releases/download/${normalizedVersion}/${assetName}`,
  };
}

function getTargetFrameworkVersion(name: string): { major: number; minor: number } | null {
  const match = /^net(\d+)(?:\.(\d+))?(?:-.+)?$/.exec(name);
  if (!match) return null;
  return {
    major: Number.parseInt(match[1], 10),
    minor: Number.parseInt(match[2] ?? "0", 10),
  };
}

function findLocalPluginBuildOutput(localBuildDir: string, pluginId: string): string | null {
  const releaseDir = join(localBuildDir, pluginId, "bin", "Release");
  if (!existsSync(releaseDir)) return null;

  const releaseEntries = readdirSync(releaseDir);
  if (releaseEntries.some((entry) => entry.endsWith(".dll") && isRegularFile(join(releaseDir, entry)))) {
    return releaseDir;
  }

  const targetFrameworkDirs = releaseEntries
    .map((entry) => {
      const fullPath = join(releaseDir, entry);
      const version = getTargetFrameworkVersion(entry);
      if (!version || !lstatSync(fullPath).isDirectory()) return null;
      return { entry, ...version };
    })
    .filter((entry): entry is { entry: string; major: number; minor: number } => entry !== null)
    .sort((a, b) => b.major - a.major || b.minor - a.minor || b.entry.localeCompare(a.entry));

  return targetFrameworkDirs[0] ? join(releaseDir, targetFrameworkDirs[0].entry) : null;
}

export class DownloadManager {
  /**
   * Get latest release info for a node type
   */
  static async getLatestRelease(nodeType: NodeType): Promise<ReleaseInfo | null> {
    const repo = REPOS[nodeType];
    const url = `${GITHUB_API_BASE}/${repo.owner}/${repo.repo}/releases/latest`;

    try {
      const response = await fetchPublicDownloadTarget(url, {
        headers: {
          Accept: "application/vnd.github.v3+json",
          "User-Agent": "NeoNexus-NodeManager/2.0",
        },
      });

      if (!response.ok) {
        throw new Error(`GitHub API error: ${response.status}`);
      }

      const data = (await response.json()) as { tag_name: string; html_url: string; published_at: string };
      return {
        version: data.tag_name,
        url: data.html_url,
        publishedAt: data.published_at,
      };
    } catch (error) {
      console.error(`Failed to get latest release for ${nodeType}:`, error);
      return null;
    }
  }

  /**
   * Download and extract neo-cli
   */
  static async downloadNeoCli(version: string, onProgress?: (percent: number) => void): Promise<string> {
    const safeVersion = assertReleaseVersion(version);
    const platform = process.platform === "win32" ? "win-x64" : "linux-x64";
    const fileName = `neo-cli-${platform}.zip`;
    const downloadUrl = `https://github.com/neo-project/neo-node/releases/download/${safeVersion}/${fileName}`;

    const downloadPath = join(paths.downloads, `neo-cli-${safeVersion}.zip`);
    const extractPath = join(paths.downloads, `neo-cli-${safeVersion}`);

    // Ensure downloads directory exists
    mkdirSync(paths.downloads, { recursive: true });

    // Download if not already exists
    if (!hasUsableDownloadFile(downloadPath)) {
      await this.downloadFile(downloadUrl, downloadPath, onProgress);
    }

    // Extract
    if (!existsSync(extractPath)) {
      mkdirSync(extractPath, { recursive: true });
      await extractArchive(downloadPath, extractPath);
    }

    return join(extractPath, "neo-cli");
  }

  /**
   * Download neox-go (Neo X geth) binary. The bane-labs releases ship the raw
   * binary (no archive) alongside genesis JSON, so we download geth directly
   * into a versioned directory.
   */
  static async downloadNeoX(version: string, onProgress?: (percent: number) => void): Promise<string> {
    const safeVersion = assertReleaseVersion(version);
    const asset = getNeoXAssetInfo(safeVersion, process.platform, process.arch);
    const downloadPath = join(paths.downloads, asset.downloadFileName);
    const extractPath = join(paths.downloads, `neox-go-${safeVersion}`);
    const binaryPath = join(extractPath, asset.binaryName);

    mkdirSync(paths.downloads, { recursive: true });
    mkdirSync(extractPath, { recursive: true });

    if (!hasUsableDownloadFile(downloadPath)) {
      await this.downloadFile(asset.downloadUrl, downloadPath, onProgress);
    }

    if (!existsSync(binaryPath)) {
      await copyFile(downloadPath, binaryPath);
      await chmod(binaryPath, 0o755);
    }

    return binaryPath;
  }

  /**
   * Download neo-go binary
   */
  static async downloadNeoGo(version: string, onProgress?: (percent: number) => void): Promise<string> {
    const safeVersion = assertReleaseVersion(version);
    const asset = getNeoGoAssetInfo(safeVersion, process.platform, process.arch);
    const downloadPath = join(paths.downloads, asset.downloadFileName);
    const extractPath = join(paths.downloads, `neo-go-${safeVersion}`);
    const binaryPath = join(extractPath, asset.binaryName);

    // Ensure downloads directory exists
    mkdirSync(paths.downloads, { recursive: true });
    mkdirSync(extractPath, { recursive: true });

    // Download if not already exists
    if (!hasUsableDownloadFile(downloadPath)) {
      await this.downloadFile(asset.downloadUrl, downloadPath, onProgress);
    }

    // Copy downloaded binary into the versioned directory
    if (!existsSync(binaryPath)) {
      await copyFile(downloadPath, binaryPath);
      if (process.platform !== "win32") {
        await chmod(binaryPath, 0o755);
      }
    }

    return binaryPath;
  }

  /**
   * Download a plugin for neo-cli.
   * Checks for a local build directory first (e.g. from a locally-built neo-node
   * checkout) before attempting to download from GitHub.
   */
  static async downloadPlugin(pluginId: string, version: string): Promise<string> {
    const safeVersion = assertReleaseVersion(version);
    const pluginDir = join(paths.plugins, pluginId, safeVersion);

    // Check for local build source first
    const localBuildDir = process.env.NEO_PLUGIN_BUILD_DIR;
    if (localBuildDir) {
      const localPluginOutput = findLocalPluginBuildOutput(localBuildDir, pluginId);
      if (localPluginOutput && existsSync(localPluginOutput)) {
        mkdirSync(pluginDir, { recursive: true });
        return localPluginOutput;
      }
    }

    // Check if already extracted
    if (existsSync(pluginDir) && readdirSync(pluginDir).some((f) => f.endsWith(".dll") && isRegularFile(join(pluginDir, f)))) {
      return pluginDir;
    }

    // neo-modules releases use format: v3.9.2/ApplicationLogs.zip
    const fileName = `${pluginId}.zip`;
    const downloadUrl = `https://github.com/${PLUGIN_REPO.owner}/${PLUGIN_REPO.repo}/releases/download/${safeVersion}/${fileName}`;

    const downloadPath = join(paths.downloads, `plugin-${pluginId}-${safeVersion}.zip`);

    // Ensure directories exist
    mkdirSync(paths.downloads, { recursive: true });
    mkdirSync(pluginDir, { recursive: true });

    // Download if not already exists
    if (!hasUsableDownloadFile(downloadPath)) {
      await this.downloadFile(downloadUrl, downloadPath);
    }

    // Extract
    await extractArchive(downloadPath, pluginDir);

    return pluginDir;
  }

  /**
   * Get latest neo-modules plugin release
   */
  static async getLatestPluginRelease(): Promise<ReleaseInfo | null> {
    const url = `${GITHUB_API_BASE}/${PLUGIN_REPO.owner}/${PLUGIN_REPO.repo}/releases/latest`;

    try {
      const response = await fetchPublicDownloadTarget(url, {
        headers: {
          Accept: "application/vnd.github.v3+json",
          "User-Agent": "NeoNexus-NodeManager/2.0",
        },
      });

      if (!response.ok) {
        throw new Error(`GitHub API error: ${response.status}`);
      }

      const data = (await response.json()) as { tag_name: string; html_url: string; published_at: string };
      return {
        version: data.tag_name,
        url: data.html_url,
        publishedAt: data.published_at,
      };
    } catch (error) {
      console.error("Failed to get latest plugin release:", error);
      return null;
    }
  }

  /**
   * Get binary path for a node
   */
  static getNodeBinaryPath(nodeType: NodeType, version: string): string | null {
    const safeVersion = assertReleaseVersion(version);
    if (nodeType === "neo-cli") {
      const path = join(paths.downloads, `neo-cli-${safeVersion}`, "neo-cli");
      return isRegularDirectory(path) ? path : null;
    }
    if (nodeType === "neox-go") {
      const path = join(paths.downloads, `neox-go-${safeVersion}`, "geth");
      return isRegularFile(path) ? path : null;
    }
    const binaryName = process.platform === "win32" ? "neo-go.exe" : "neo-go";
    const path = join(paths.downloads, `neo-go-${safeVersion}`, binaryName);
    return isRegularFile(path) ? path : null;
  }

  /**
   * Check if node binary exists
   */
  static hasNodeBinary(nodeType: NodeType, version: string): boolean {
    return this.getNodeBinaryPath(nodeType, version) !== null;
  }

  /**
   * Download a file with progress tracking
   */
  private static async downloadFile(
    url: string,
    destination: string,
    onProgress?: (percent: number) => void,
    redirectsRemaining = 5,
  ): Promise<void> {
    const parsed = validateDownloadUrl(url);
    const targetAddress = await resolvePublicDownloadTarget(parsed);
    const response = await requestPinnedDownloadResponse(parsed, targetAddress, "GET");
    const status = response.statusCode ?? 599;
    if (status >= 300 && status < 400) {
      response.resume();
      if (redirectsRemaining <= 0) {
        throw new Error("Too many redirects");
      }
      const redirectUrl = response.headers.location;
      if (!redirectUrl) {
        throw new Error("Download redirect did not include a Location header");
      }
      const next = new URL(redirectUrl, parsed).toString();
      await this.downloadFile(next, destination, onProgress, redirectsRemaining - 1);
      return;
    }
    if (status !== 200) {
      response.resume();
      throw new Error(`Download failed with status ${status}`);
    }

    await writeDownloadResponse(response, destination, onProgress);
  }

  /**
   * Get download size for a URL
   */
  static async getDownloadSize(url: string): Promise<number | null> {
    try {
      const parsed = validateDownloadUrl(url);
      const targetAddress = await resolvePublicDownloadTarget(parsed);
      const response = await requestPinnedDownloadResponse(parsed, targetAddress, "HEAD");
      response.resume();
      const status = response.statusCode ?? 599;
      if (status < 200 || status >= 300) {
        return null;
      }
      const size = firstHeader(response.headers["content-length"]);
      const parsedSize = size ? Number.parseInt(size, 10) : Number.NaN;
      return Number.isFinite(parsedSize) ? parsedSize : null;
    } catch {
      return null;
    }
  }
}

function validateDownloadUrl(rawUrl: string): URL {
  let parsed: URL;
  try {
    parsed = new URL(rawUrl);
  } catch {
    throw new Error(`Invalid download URL: ${rawUrl}`);
  }
  if (parsed.protocol !== "https:") {
    throw new Error(`Refusing to download from non-HTTPS URL: ${rawUrl}`);
  }
  assertLiteralPublicTarget(
    parsed.hostname,
    downloadPrivateTargetError,
    process.env.NEONEXUS_ALLOW_PRIVATE_DOWNLOAD_TARGETS === "true",
  );
  return parsed;
}

async function fetchPublicDownloadTarget(url: string, init: RequestInit): Promise<Response> {
  const parsed = validateDownloadUrl(url);
  return publicFetch(parsed, init, {
    resolveHostname: defaultHostnameResolver,
    makeError: downloadPrivateTargetError,
    allowPrivateTarget: process.env.NEONEXUS_ALLOW_PRIVATE_DOWNLOAD_TARGETS === "true",
  });
}

async function resolvePublicDownloadTarget(parsed: URL): Promise<string> {
  const allowPrivate = process.env.NEONEXUS_ALLOW_PRIVATE_DOWNLOAD_TARGETS === "true";
  assertLiteralPublicTarget(parsed.hostname, downloadPrivateTargetError, allowPrivate);
  const addresses = await assertResolvedPublicTarget(
    parsed.hostname,
    defaultHostnameResolver,
    downloadPrivateTargetError,
    allowPrivate,
  );
  const targetAddress = addresses[0]?.address;
  if (!targetAddress) {
    throw new Error(`Unable to resolve download target: ${parsed.hostname}`);
  }
  return targetAddress;
}

function requestPinnedDownloadResponse(
  source: URL,
  targetAddress: string,
  method: "GET" | "HEAD",
): Promise<IncomingMessage> {
  return new Promise((resolve, reject) => {
    const req = httpsRequest(
      {
        protocol: "https:",
        hostname: targetAddress,
        port: source.port || "443",
        path: `${source.pathname}${source.search}`,
        method,
        headers: {
          Host: source.host,
          "User-Agent": "NeoNexus-NodeManager/2.0",
        },
        servername: isIP(stripBrackets(source.hostname)) === 0 ? stripBrackets(source.hostname) : undefined,
      },
      resolve,
    );
    req.on("error", reject);
    req.end();
  });
}

async function writeDownloadResponse(
  response: IncomingMessage,
  destination: string,
  onProgress?: (percent: number) => void,
): Promise<void> {
  const totalSize = Number.parseInt(firstHeader(response.headers["content-length"]) || "0", 10);
  let downloadedSize = 0;

  response.on("data", (chunk: Buffer) => {
    downloadedSize += chunk.length;
    if (totalSize > 0 && onProgress) {
      onProgress(Math.round((downloadedSize / totalSize) * 100));
    }
  });

  try {
    await pipeline(response, createWriteStream(destination));
  } catch (error) {
    try {
      if (existsSync(destination)) {
        unlinkSync(destination);
      }
    } catch {}
    throw error;
  }
}

function stripBrackets(hostname: string): string {
  return hostname.replace(/^\[/, "").replace(/\]$/, "");
}

function firstHeader(value: string | string[] | undefined): string | undefined {
  return Array.isArray(value) ? value[0] : value;
}

function isRegularFile(path: string): boolean {
  try {
    return lstatSync(path).isFile();
  } catch {
    return false;
  }
}

function isRegularDirectory(path: string): boolean {
  try {
    return lstatSync(path).isDirectory();
  } catch {
    return false;
  }
}

function downloadPrivateTargetError(hostname: string): ApiError {
  return new ApiError(
    "DOWNLOAD_PRIVATE_TARGET",
    `Download URL targets a private or local address: ${hostname}`,
    "Use public GitHub release URLs, or set NEONEXUS_ALLOW_PRIVATE_DOWNLOAD_TARGETS=true only for trusted private mirrors.",
  );
}
