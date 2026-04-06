import { createWriteStream, existsSync, mkdirSync, readdirSync, statSync, unlinkSync } from "node:fs";
import { copyFile, chmod } from "node:fs/promises";
import { get, request as httpsRequest } from "node:https";
import { join } from "node:path";
import extractZip from "extract-zip";
import { paths } from "../utils/paths";
import type { NodeType, ReleaseInfo } from "../types/index";

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
};

// Plugin repository
const PLUGIN_REPO = {
  owner: "neo-project",
  repo: "neo-modules",
};

export function hasUsableDownloadFile(path: string): boolean {
  return existsSync(path) && statSync(path).size > 0;
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
  const normalizedVersion = version.startsWith("v") ? version : `v${version}`;

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

export class DownloadManager {
  /**
   * Get latest release info for a node type
   */
  static async getLatestRelease(nodeType: NodeType): Promise<ReleaseInfo | null> {
    const repo = REPOS[nodeType];
    const url = `${GITHUB_API_BASE}/${repo.owner}/${repo.repo}/releases/latest`;

    try {
      const response = await fetch(url, {
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
    const platform = process.platform === "win32" ? "win-x64" : "linux-x64";
    const fileName = `neo-cli-${platform}.zip`;
    const downloadUrl = `https://github.com/neo-project/neo-node/releases/download/${version}/${fileName}`;

    const downloadPath = join(paths.downloads, `neo-cli-${version}.zip`);
    const extractPath = join(paths.downloads, `neo-cli-${version}`);

    // Ensure downloads directory exists
    mkdirSync(paths.downloads, { recursive: true });

    // Download if not already exists
    if (!hasUsableDownloadFile(downloadPath)) {
      await this.downloadFile(downloadUrl, downloadPath, onProgress);
    }

    // Extract
    if (!existsSync(extractPath)) {
      mkdirSync(extractPath, { recursive: true });
      await extractZip(downloadPath, { dir: extractPath });
    }

    return join(extractPath, "neo-cli");
  }

  /**
   * Download neo-go binary
   */
  static async downloadNeoGo(version: string, onProgress?: (percent: number) => void): Promise<string> {
    const asset = getNeoGoAssetInfo(version, process.platform, process.arch);
    const downloadPath = join(paths.downloads, asset.downloadFileName);
    const extractPath = join(paths.downloads, `neo-go-${version}`);
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
    const pluginDir = join(paths.plugins, pluginId, version);

    // Check for local build source first
    const localBuildDir = process.env.NEO_PLUGIN_BUILD_DIR;
    if (localBuildDir) {
      const localPluginOutput = join(localBuildDir, pluginId, "bin", "Release", "net10.0");
      if (existsSync(localPluginOutput)) {
        mkdirSync(pluginDir, { recursive: true });
        return localPluginOutput;
      }
    }

    // Check if already extracted
    if (existsSync(pluginDir) && readdirSync(pluginDir).some(f => f.endsWith(".dll"))) {
      return pluginDir;
    }

    // neo-modules releases use format: v3.9.2/ApplicationLogs.zip
    const fileName = `${pluginId}.zip`;
    const downloadUrl = `https://github.com/${PLUGIN_REPO.owner}/${PLUGIN_REPO.repo}/releases/download/${version}/${fileName}`;

    const downloadPath = join(paths.downloads, `plugin-${pluginId}-${version}.zip`);

    // Ensure directories exist
    mkdirSync(paths.downloads, { recursive: true });
    mkdirSync(pluginDir, { recursive: true });

    // Download if not already exists
    if (!hasUsableDownloadFile(downloadPath)) {
      await this.downloadFile(downloadUrl, downloadPath);
    }

    // Extract
    await extractZip(downloadPath, { dir: pluginDir });

    return pluginDir;
  }

  /**
   * Get latest neo-modules plugin release
   */
  static async getLatestPluginRelease(): Promise<ReleaseInfo | null> {
    const url = `${GITHUB_API_BASE}/${PLUGIN_REPO.owner}/${PLUGIN_REPO.repo}/releases/latest`;

    try {
      const response = await fetch(url, {
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
    if (nodeType === "neo-cli") {
      const path = join(paths.downloads, `neo-cli-${version}`, "neo-cli");
      return existsSync(path) ? path : null;
    } else {
      const binaryName = process.platform === "win32" ? "neo-go.exe" : "neo-go";
      const path = join(paths.downloads, `neo-go-${version}`, binaryName);
      return existsSync(path) ? path : null;
    }
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
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      const file = createWriteStream(destination);
      const cleanup = () => {
        try {
          file.close();
        } catch {}
        try {
          if (existsSync(destination)) {
            unlinkSync(destination);
          }
        } catch {}
      };

      get(url, { headers: { "User-Agent": "NeoNexus-NodeManager/2.0" } }, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Follow redirect
          const redirectUrl = response.headers.location;
          if (redirectUrl) {
            cleanup();
            this.downloadFile(redirectUrl, destination, onProgress).then(resolve).catch(reject);
            return;
          }
        }

        if (response.statusCode !== 200) {
          cleanup();
          reject(new Error(`Download failed with status ${response.statusCode}`));
          return;
        }

        const totalSize = parseInt(response.headers["content-length"] || "0", 10);
        let downloadedSize = 0;

        response.on("data", (chunk: Buffer) => {
          downloadedSize += chunk.length;
          if (totalSize > 0 && onProgress) {
            const percent = Math.round((downloadedSize / totalSize) * 100);
            onProgress(percent);
          }
        });

        response.pipe(file);

        file.on("finish", () => {
          file.close();
          resolve();
        });

        file.on("error", (err) => {
          cleanup();
          reject(err);
        });
      }).on("error", (error) => {
        cleanup();
        reject(error);
      }).on("close", () => {
        // Ensure connection is fully released
      });
    });
  }

  /**
   * Get download size for a URL
   */
  static async getDownloadSize(url: string): Promise<number | null> {
    return new Promise((resolve) => {
      const req = httpsRequest(url, { method: "HEAD" }, (response) => {
        const size = response.headers["content-length"];
        resolve(size ? parseInt(size, 10) : null);
      });
      req.on("error", () => resolve(null));
      req.end();
    });
  }
}
