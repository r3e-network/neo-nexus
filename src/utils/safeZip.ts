import { createWriteStream, existsSync, lstatSync, mkdirSync } from "node:fs";
import { unlink } from "node:fs/promises";
import { dirname, posix, relative, resolve, sep } from "node:path";
import { pipeline } from "node:stream/promises";
import yauzl, { type Entry, type ZipFile } from "yauzl";

const DEFAULT_MAX_ENTRIES = 10_000;
const DEFAULT_MAX_UNCOMPRESSED_BYTES = 2 * 1024 * 1024 * 1024;
const DEFAULT_MAX_COMPRESSION_RATIO = 1_000;

function positiveIntegerEnvironment(name: string, fallback: number): number {
  const value = Number.parseInt(process.env[name] || "", 10);
  return Number.isSafeInteger(value) && value > 0 ? value : fallback;
}

export function validateZipEntryPath(destination: string, entryName: string): string {
  if (!entryName || entryName.includes("\0") || entryName.includes("\\")) {
    throw new Error("ZIP archive contains an invalid entry path");
  }

  const normalized = posix.normalize(entryName);
  if (
    posix.isAbsolute(normalized) ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    /^[A-Za-z]:/.test(normalized)
  ) {
    throw new Error(`ZIP entry escapes the extraction directory: ${entryName}`);
  }

  const root = resolve(destination);
  const target = resolve(root, ...normalized.split("/"));
  if (target !== root && !target.startsWith(`${root}${sep}`)) {
    throw new Error(`ZIP entry escapes the extraction directory: ${entryName}`);
  }
  return target;
}

export function isZipSymlink(entry: Pick<Entry, "externalFileAttributes">): boolean {
  const unixMode = (entry.externalFileAttributes >>> 16) & 0xffff;
  return (unixMode & 0o170000) === 0o120000;
}

function assertNoSymlinkParents(root: string, target: string): void {
  const rootPath = resolve(root);
  const parentPath = dirname(target);
  const segments = relative(rootPath, parentPath).split(sep).filter(Boolean);
  let current = rootPath;

  if (existsSync(current) && lstatSync(current).isSymbolicLink()) {
    throw new Error("ZIP extraction directory must not be a symbolic link");
  }

  for (const segment of segments) {
    current = resolve(current, segment);
    if (existsSync(current) && lstatSync(current).isSymbolicLink()) {
      throw new Error(`ZIP entry would traverse a symbolic link: ${segment}`);
    }
  }
}

function openZip(source: string): Promise<ZipFile> {
  return new Promise((resolveZip, rejectZip) => {
    yauzl.open(
      source,
      { lazyEntries: true, autoClose: true, decodeStrings: true, validateEntrySizes: true },
      (error, zipFile) => {
        if (error || !zipFile) rejectZip(error || new Error("Unable to open ZIP archive"));
        else resolveZip(zipFile);
      },
    );
  });
}

function openEntryStream(zipFile: ZipFile, entry: Entry): Promise<NodeJS.ReadableStream> {
  return new Promise((resolveStream, rejectStream) => {
    zipFile.openReadStream(entry, (error, stream) => {
      if (error || !stream) rejectStream(error || new Error(`Unable to read ZIP entry ${entry.fileName}`));
      else resolveStream(stream);
    });
  });
}

export async function extractSafeZip(source: string, destination: string): Promise<void> {
  const maxEntries = positiveIntegerEnvironment("NEONEXUS_ZIP_MAX_ENTRIES", DEFAULT_MAX_ENTRIES);
  const maxBytes = positiveIntegerEnvironment("NEONEXUS_ZIP_MAX_UNCOMPRESSED_BYTES", DEFAULT_MAX_UNCOMPRESSED_BYTES);
  const maxRatio = positiveIntegerEnvironment("NEONEXUS_ZIP_MAX_COMPRESSION_RATIO", DEFAULT_MAX_COMPRESSION_RATIO);
  const zipFile = await openZip(source);
  let entryCount = 0;
  let totalUncompressedBytes = 0;

  mkdirSync(destination, { recursive: true, mode: 0o700 });

  await new Promise<void>((resolveExtraction, rejectExtraction) => {
    let settled = false;
    const fail = (error: unknown) => {
      if (settled) return;
      settled = true;
      zipFile.close();
      rejectExtraction(error instanceof Error ? error : new Error(String(error)));
    };

    zipFile.once("error", fail);
    zipFile.once("end", () => {
      if (settled) return;
      settled = true;
      resolveExtraction();
    });
    zipFile.on("entry", (entry) => {
      void (async () => {
        entryCount += 1;
        if (entryCount > maxEntries) throw new Error(`ZIP archive exceeds ${maxEntries} entries`);
        if (entry.generalPurposeBitFlag & 0x1) throw new Error("Encrypted ZIP entries are not supported");
        if (isZipSymlink(entry)) throw new Error(`ZIP archive contains a symbolic link: ${entry.fileName}`);

        totalUncompressedBytes += entry.uncompressedSize;
        if (!Number.isSafeInteger(totalUncompressedBytes) || totalUncompressedBytes > maxBytes) {
          throw new Error(`ZIP archive exceeds the ${maxBytes}-byte extraction limit`);
        }
        if (
          entry.uncompressedSize > 0 &&
          (entry.compressedSize === 0 || entry.uncompressedSize / entry.compressedSize > maxRatio)
        ) {
          throw new Error(`ZIP entry exceeds the allowed compression ratio: ${entry.fileName}`);
        }

        const target = validateZipEntryPath(destination, entry.fileName);
        assertNoSymlinkParents(destination, target);
        if (entry.fileName.endsWith("/")) {
          mkdirSync(target, { recursive: true, mode: 0o700 });
          zipFile.readEntry();
          return;
        }

        mkdirSync(dirname(target), { recursive: true, mode: 0o700 });
        assertNoSymlinkParents(destination, target);
        const stream = await openEntryStream(zipFile, entry);
        const unixMode = (entry.externalFileAttributes >>> 16) & 0xffff;
        const mode = unixMode & 0o111 ? 0o700 : 0o600;
        try {
          await pipeline(stream, createWriteStream(target, { flags: "wx", mode }));
        } catch (error) {
          await unlink(target).catch(() => undefined);
          throw error;
        }
        zipFile.readEntry();
      })().catch(fail);
    });

    zipFile.readEntry();
  });
}
