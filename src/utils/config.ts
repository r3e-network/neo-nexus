import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import { dirname } from "node:path";

export async function readJsonFile<T>(path: string): Promise<T> {
  if (!existsSync(path)) {
    throw new Error(`File not found: ${path}`);
  }
  const content = readFileSync(path, "utf8");
  return JSON.parse(content);
}

export async function writeJsonFile(path: string, data: any): Promise<void> {
  await mkdir(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify(data, null, 2), "utf8");
}

export function readJsonFileSync<T>(path: string): T {
  const content = readFileSync(path, "utf8");
  return JSON.parse(content);
}

export function writeJsonFileSync(path: string, data: any): void {
  writeFileSync(path, JSON.stringify(data, null, 2), "utf8");
}
