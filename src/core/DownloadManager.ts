import { createWriteStream, existsSync, mkdirSync } from "node:fs";
import { writeFile, chmod } from "node:fs/promises";
import { get } from "node:https";
import { join, basename } from "node:path";
import extractZip from "extract-zip";
import * as tar from "tar";
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
    if (!existsSync(downloadPath)) {
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
    const platform = process.platform === "win32" ? "windows-amd64" : "linux-amd64";
    // neo-go releases don't use 'v' prefix in download URLs
    const versionWithoutV = version.startsWith("v") ? version.slice(1) : version;
    const fileName = `neo-go-${versionWithoutV}-${platform}.gz`;
    const downloadUrl = `https://github.com/nspcc-dev/neo-go/releases/download/v${versionWithoutV}/${fileName}`;

    const downloadPath = join(paths.downloads, fileName);
    const extractPath = join(paths.downloads, `neo-go-${version}`);
    const binaryPath = join(extractPath, "neo-go");

    // Ensure downloads directory exists
    mkdirSync(paths.downloads, { recursive: true });
    mkdirSync(extractPath, { recursive: true });

    // Download if not already exists
    if (!existsSync(downloadPath)) {
      await this.downloadFile(downloadUrl, downloadPath, onProgress);
    }

    // Extract
    if (!existsSync(binaryPath)) {
      await tar.extract({
        file: downloadPath,
        cwd: extractPath,
      });
      await chmod(binaryPath, 0o755);
    }

    return binaryPath;
  }

  /**
   * Download a plugin for neo-cli
   */
  static async downloadPlugin(pluginId: string, version: string): Promise<string> {
    // neo-modules releases use format: v3.9.2/ApplicationLogs.zip
    const fileName = `${pluginId}.zip`;
    const downloadUrl = `https://github.com/${PLUGIN_REPO.owner}/${PLUGIN_REPO.repo}/releases/download/${version}/${fileName}`;

    const pluginDir = join(paths.plugins, pluginId, version);
    const downloadPath = join(paths.downloads, `plugin-${pluginId}-${version}.zip`);

    // Ensure directories exist
    mkdirSync(paths.downloads, { recursive: true });
    mkdirSync(pluginDir, { recursive: true });

    // Download if not already exists
    if (!existsSync(downloadPath)) {
      await this.downloadFile(downloadUrl, downloadPath);
    }

    // Extract
    await extractZip(downloadPath, { dir: pluginDir });

    return pluginDir;
  }

  /**
   * Get binary path for a node
   */
  static getNodeBinaryPath(nodeType: NodeType, version: string): string | null {
    if (nodeType === "neo-cli") {
      const path = join(paths.downloads, `neo-cli-${version}`, "neo-cli");
      return existsSync(path) ? path : null;
    } else {
      const path = join(paths.downloads, `neo-go-${version}`, "neo-go");
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

      get(url, { headers: { "User-Agent": "NeoNexus-NodeManager/2.0" } }, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Follow redirect
          const redirectUrl = response.headers.location;
          if (redirectUrl) {
            file.close();
            this.downloadFile(redirectUrl, destination, onProgress).then(resolve).catch(reject);
            return;
          }
        }

        if (response.statusCode !== 200) {
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
          file.close();
          reject(err);
        });
      }).on("error", reject);
    });
  }

  /**
   * Get download size for a URL
   */
  static async getDownloadSize(url: string): Promise<number | null> {
    return new Promise((resolve) => {
      get(url, { method: "HEAD" }, (response) => {
        const size = response.headers["content-length"];
        resolve(size ? parseInt(size, 10) : null);
      }).on("error", () => resolve(null));
    });
  }
}
