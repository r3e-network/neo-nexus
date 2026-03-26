import { execFile } from "node:child_process";
import { randomUUID } from "node:crypto";
import net from "node:net";
import type Database from "better-sqlite3";
import type {
  CreateSecureSignerRequest,
  SecureSignerAttestationResult,
  SecureSignerCommandResult,
  SecureSignerConnectionInfo,
  SecureSignerMode,
  SecureSignerOrchestrationPlan,
  SecureSignerProfile,
  SecureSignerReadinessResult,
  SecureSignerTestResult,
  SecureSignerUnlockMode,
  UpdateSecureSignerRequest,
} from "../types";

interface ProbeResult {
  ok: boolean;
  message: string;
  latencyMs?: number;
}

interface ToolCommandResult {
  stdout: string;
  stderr: string;
}

interface SecureSignerManagerOptions {
  probeEndpoint?: (endpoint: string) => Promise<ProbeResult>;
  runToolCommand?: (toolPath: string, args: string[]) => Promise<ToolCommandResult>;
}

type ProfileRow = {
  id: string;
  name: string;
  mode: SecureSignerMode;
  endpoint: string;
  public_key: string | null;
  account_address: string | null;
  wallet_path: string | null;
  unlock_mode: SecureSignerUnlockMode;
  notes: string | null;
  enabled: number;
  workspace_path: string | null;
  startup_port: number | null;
  aws_region: string | null;
  kms_key_id: string | null;
  kms_ciphertext_blob_path: string | null;
  last_test_status: SecureSignerProfile["lastTestStatus"] | null;
  last_test_message: string | null;
  last_tested_at: number | null;
  created_at: number;
  updated_at: number;
};

function isLocalHttpHost(host: string): boolean {
  return host === "127.0.0.1" || host === "localhost";
}

export class SecureSignerManager {
  private readonly probeEndpoint: (endpoint: string) => Promise<ProbeResult>;
  private readonly runToolCommand: (toolPath: string, args: string[]) => Promise<ToolCommandResult>;

  constructor(
    private readonly db: Database.Database,
    options: SecureSignerManagerOptions = {},
  ) {
    this.probeEndpoint = options.probeEndpoint ?? this.defaultProbeEndpoint;
    this.runToolCommand = options.runToolCommand ?? this.defaultRunToolCommand;
  }

  listProfiles(): SecureSignerProfile[] {
    const rows = this.db
      .prepare("SELECT * FROM secure_signer_profiles ORDER BY created_at")
      .all() as ProfileRow[];

    return rows.map((row) => this.mapRow(row));
  }

  getProfile(id: string): SecureSignerProfile | null {
    const row = this.db
      .prepare("SELECT * FROM secure_signer_profiles WHERE id = ?")
      .get(id) as ProfileRow | undefined;

    return row ? this.mapRow(row) : null;
  }

  createProfile(request: CreateSecureSignerRequest): SecureSignerProfile {
    const normalized = this.normalizeProfile({
      ...request,
      unlockMode: request.unlockMode,
      enabled: request.enabled ?? true,
    });
    const now = Date.now();
    const profile: SecureSignerProfile = {
      id: randomUUID(),
      ...normalized,
      lastTestStatus: undefined,
      lastTestMessage: undefined,
      lastTestedAt: undefined,
      createdAt: now,
      updatedAt: now,
    };

    this.db
      .prepare(`
        INSERT INTO secure_signer_profiles (
          id, name, mode, endpoint, public_key, account_address, wallet_path,
          unlock_mode, notes, enabled, workspace_path, startup_port, aws_region,
          kms_key_id, kms_ciphertext_blob_path, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `)
      .run(
        profile.id,
        profile.name,
        profile.mode,
        profile.endpoint,
        profile.publicKey ?? null,
        profile.accountAddress ?? null,
        profile.walletPath ?? null,
        profile.unlockMode,
        profile.notes ?? null,
        profile.enabled ? 1 : 0,
        profile.workspacePath ?? null,
        profile.startupPort ?? null,
        profile.awsRegion ?? null,
        profile.kmsKeyId ?? null,
        profile.kmsCiphertextBlobPath ?? null,
        profile.createdAt,
        profile.updatedAt,
      );

    return profile;
  }

  updateProfile(id: string, request: UpdateSecureSignerRequest): SecureSignerProfile {
    const existing = this.requireProfile(id);

    const normalized = this.normalizeProfile({
      name: request.name ?? existing.name,
      mode: request.mode ?? existing.mode,
      endpoint: request.endpoint ?? existing.endpoint,
      publicKey: request.publicKey ?? existing.publicKey,
      accountAddress: request.accountAddress ?? existing.accountAddress,
      walletPath: request.walletPath ?? existing.walletPath,
      unlockMode: request.unlockMode ?? existing.unlockMode,
      notes: request.notes ?? existing.notes,
      enabled: request.enabled ?? existing.enabled,
      workspacePath: request.workspacePath ?? existing.workspacePath,
      startupPort: request.startupPort ?? existing.startupPort,
      awsRegion: request.awsRegion ?? existing.awsRegion,
      kmsKeyId: request.kmsKeyId ?? existing.kmsKeyId,
      kmsCiphertextBlobPath: request.kmsCiphertextBlobPath ?? existing.kmsCiphertextBlobPath,
    });
    const updatedAt = Date.now();

    this.db
      .prepare(`
        UPDATE secure_signer_profiles
        SET name = ?, mode = ?, endpoint = ?, public_key = ?, account_address = ?, wallet_path = ?,
            unlock_mode = ?, notes = ?, enabled = ?, workspace_path = ?, startup_port = ?, aws_region = ?,
            kms_key_id = ?, kms_ciphertext_blob_path = ?, updated_at = ?
        WHERE id = ?
      `)
      .run(
        normalized.name,
        normalized.mode,
        normalized.endpoint,
        normalized.publicKey ?? null,
        normalized.accountAddress ?? null,
        normalized.walletPath ?? null,
        normalized.unlockMode,
        normalized.notes ?? null,
        normalized.enabled ? 1 : 0,
        normalized.workspacePath ?? null,
        normalized.startupPort ?? null,
        normalized.awsRegion ?? null,
        normalized.kmsKeyId ?? null,
        normalized.kmsCiphertextBlobPath ?? null,
        updatedAt,
        id,
      );

    return {
      ...existing,
      ...normalized,
      updatedAt,
    };
  }

  deleteProfile(id: string): void {
    this.db.prepare("DELETE FROM secure_signer_profiles WHERE id = ?").run(id);
  }

  async testProfile(id: string): Promise<SecureSignerTestResult> {
    const profile = this.requireProfile(id);
    const checkedAt = Date.now();
    const connection = this.parseConnection(profile);

    if (connection.scheme === "vsock") {
      const result: SecureSignerTestResult = {
        ok: true,
        status: "warning",
        message: "Vsock endpoint format validated; live probing requires local secure-sign-tools access.",
        checkedAt,
      };

      this.persistTestResult(id, result);
      return result;
    }

    const probe = await this.probeEndpoint(profile.endpoint);
    const result: SecureSignerTestResult = {
      ok: probe.ok,
      status: probe.ok ? "reachable" : "unreachable",
      message: probe.message,
      latencyMs: probe.latencyMs,
      checkedAt,
    };

    this.persistTestResult(id, result);
    return result;
  }

  getOrchestration(id: string): SecureSignerOrchestrationPlan {
    const profile = this.requireProfile(id);
    const connection = this.parseConnection(profile);
    const commands = {
      deploy: this.buildDeployCommands(profile, connection),
      unlock: this.buildUnlockCommands(profile, connection),
      status: this.buildStatusCommands(profile, connection),
      attestation: this.buildAttestationCommands(profile, connection),
      startRecipient: this.buildStartRecipientCommands(profile, connection),
    };
    const warnings: string[] = [];

    if (!profile.workspacePath) {
      warnings.push("Local orchestration is unavailable until a secure-sign-service-rs workspace path is configured.");
    }

    if (connection.scheme === "https" || (connection.scheme === "http" && !connection.localToolingCompatible)) {
      warnings.push("Live account-status checks are limited for remote HTTP/HTTPS endpoints because secure-sign-tools only targets localhost or vsock.");
    }

    if (profile.mode === "nitro" && !profile.kmsCiphertextBlobPath) {
      warnings.push("Nitro recipient auto-unlock is configured best with a KMS ciphertext blob path.");
    }

    return {
      connection,
      commands,
      warnings,
    };
  }

  async getReadiness(id: string): Promise<SecureSignerReadinessResult> {
    const profile = this.requireProfile(id);
    const connection = this.parseConnection(profile);
    const checkedAt = Date.now();

    if (this.canRunToolStatus(profile, connection)) {
      try {
        const output = await this.runToolCommand(this.resolveToolPath(profile), this.buildStatusArgs(profile, connection));
        const accountStatus = this.parseAccountStatus(output.stdout || output.stderr);
        const result: SecureSignerReadinessResult = {
          ok: accountStatus === "Single" || accountStatus === "Multiple",
          status:
            accountStatus === "Single" || accountStatus === "Multiple"
              ? "reachable"
              : accountStatus === "Locked" || accountStatus === "NoPrivateKey"
                ? "warning"
                : "unreachable",
          message: output.stdout.trim() || output.stderr.trim() || "Signer status checked.",
          checkedAt,
          source: "secure-sign-tools",
          accountStatus,
        };
        this.persistTestResult(id, result);
        return result;
      } catch (error) {
        const result: SecureSignerReadinessResult = {
          ok: false,
          status: "unreachable",
          message: error instanceof Error ? error.message : "Failed to query secure-sign-tools status",
          checkedAt,
          source: "secure-sign-tools",
        };
        this.persistTestResult(id, result);
        return result;
      }
    }

    if (connection.scheme === "vsock") {
      const result: SecureSignerReadinessResult = {
        ok: true,
        status: "warning",
        message: "Vsock endpoint format validated; configure a local workspace and public key for live account-status checks.",
        checkedAt,
        source: "vsock-format",
      };
      this.persistTestResult(id, result);
      return result;
    }

    const probe = await this.probeEndpoint(profile.endpoint);
    const result: SecureSignerReadinessResult = {
      ok: probe.ok,
      status: probe.ok ? "reachable" : "unreachable",
      message: probe.message,
      latencyMs: probe.latencyMs,
      checkedAt,
      source: "probe",
    };
    this.persistTestResult(id, result);
    return result;
  }

  async fetchRecipientAttestation(id: string): Promise<SecureSignerAttestationResult> {
    const profile = this.requireProfile(id);
    const connection = this.parseConnection(profile);
    if (profile.mode !== "nitro") {
      throw new Error("Recipient attestation is only available for Nitro secure signers");
    }

    const result = await this.runToolCommand(this.resolveToolPath(profile), this.buildAttestationArgs(profile, connection));
    const attestationBase64 = result.stdout.trim();
    if (!attestationBase64) {
      throw new Error("No recipient attestation document was returned");
    }

    return {
      attestationBase64,
      checkedAt: Date.now(),
    };
  }

  async startRecipientSigner(id: string, ciphertextBase64: string): Promise<SecureSignerCommandResult> {
    const profile = this.requireProfile(id);
    const connection = this.parseConnection(profile);
    if (profile.mode !== "nitro") {
      throw new Error("Recipient ciphertext startup is only available for Nitro secure signers");
    }

    const value = ciphertextBase64.trim();
    if (!value) {
      throw new Error("CiphertextForRecipient is required");
    }

    const result = await this.runToolCommand(
      this.resolveToolPath(profile),
      this.buildStartRecipientArgs(profile, connection, value),
    );

    return {
      ok: true,
      message: result.stdout.trim() || "Signer starting via recipient ciphertext...",
      checkedAt: Date.now(),
    };
  }

  buildSignClientConfig(profile: SecureSignerProfile): { Name: string; Endpoint: string } {
    return {
      Name: profile.name,
      Endpoint: profile.endpoint,
    };
  }

  private requireProfile(id: string): SecureSignerProfile {
    const profile = this.getProfile(id);
    if (!profile) {
      throw new Error(`Secure signer profile ${id} not found`);
    }
    return profile;
  }

  private mapRow(row: ProfileRow): SecureSignerProfile {
    return {
      id: row.id,
      name: row.name,
      mode: row.mode,
      endpoint: row.endpoint,
      publicKey: row.public_key ?? undefined,
      accountAddress: row.account_address ?? undefined,
      walletPath: row.wallet_path ?? undefined,
      unlockMode: row.unlock_mode,
      notes: row.notes ?? undefined,
      enabled: row.enabled === 1,
      workspacePath: row.workspace_path ?? undefined,
      startupPort: row.startup_port ?? undefined,
      awsRegion: row.aws_region ?? undefined,
      kmsKeyId: row.kms_key_id ?? undefined,
      kmsCiphertextBlobPath: row.kms_ciphertext_blob_path ?? undefined,
      lastTestStatus: row.last_test_status ?? undefined,
      lastTestMessage: row.last_test_message ?? undefined,
      lastTestedAt: row.last_tested_at ?? undefined,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }

  private normalizeProfile(input: {
    name: string;
    mode: SecureSignerMode;
    endpoint: string;
    publicKey?: string;
    accountAddress?: string;
    walletPath?: string;
    unlockMode?: SecureSignerUnlockMode;
    notes?: string;
    enabled?: boolean;
    workspacePath?: string;
    startupPort?: number;
    awsRegion?: string;
    kmsKeyId?: string;
    kmsCiphertextBlobPath?: string;
  }): Omit<SecureSignerProfile, "id" | "createdAt" | "updatedAt" | "lastTestStatus" | "lastTestMessage" | "lastTestedAt"> {
    const name = input.name.trim();
    if (!name) {
      throw new Error("Secure signer name is required");
    }

    const endpoint = this.normalizeEndpoint(input.endpoint, input.mode);
    const publicKey = this.normalizePublicKey(input.publicKey);
    const unlockMode = input.unlockMode ?? this.defaultUnlockMode(input.mode);

    return {
      name,
      mode: input.mode,
      endpoint,
      publicKey,
      accountAddress: this.normalizeOptional(input.accountAddress),
      walletPath: this.normalizeOptional(input.walletPath),
      unlockMode,
      notes: this.normalizeOptional(input.notes),
      enabled: input.enabled ?? true,
      workspacePath: this.normalizeOptional(input.workspacePath),
      startupPort: this.normalizeOptionalInteger(input.startupPort),
      awsRegion: this.normalizeOptional(input.awsRegion),
      kmsKeyId: this.normalizeOptional(input.kmsKeyId),
      kmsCiphertextBlobPath: this.normalizeOptional(input.kmsCiphertextBlobPath),
    };
  }

  private normalizeEndpoint(endpoint: string, mode: SecureSignerMode): string {
    const value = endpoint.trim();
    if (!value) {
      throw new Error("Secure signer endpoint is required");
    }

    let parsed: URL;
    try {
      parsed = new URL(value);
    } catch {
      throw new Error("Secure signer endpoint must be a valid URL");
    }

    if (mode === "nitro" && parsed.protocol !== "vsock:") {
      throw new Error("Nitro secure signers must use a vsock:// endpoint");
    }

    if ((mode === "software" || mode === "sgx") && !["http:", "https:"].includes(parsed.protocol)) {
      throw new Error(`${mode.toUpperCase()} secure signers must use an http:// or https:// endpoint`);
    }

    if (!["http:", "https:", "vsock:"].includes(parsed.protocol)) {
      throw new Error("Secure signer endpoint must use http://, https://, or vsock://");
    }

    if (!parsed.port) {
      throw new Error("Secure signer endpoint must include an explicit port");
    }

    return value;
  }

  private normalizePublicKey(publicKey?: string): string | undefined {
    const value = this.normalizeOptional(publicKey);
    if (!value) {
      return undefined;
    }

    if (!/^(02|03)[0-9a-fA-F]{64}$/.test(value) && !/^04[0-9a-fA-F]{128}$/.test(value)) {
      throw new Error("Secure signer public keys must be valid compressed or uncompressed secp256r1 hex");
    }

    return value;
  }

  private normalizeOptional(value?: string): string | undefined {
    const normalized = value?.trim();
    return normalized ? normalized : undefined;
  }

  private normalizeOptionalInteger(value?: number): number | undefined {
    if (value === undefined || value === null) {
      return undefined;
    }
    if (!Number.isInteger(value) || value <= 0) {
      throw new Error("Startup port must be a positive integer");
    }
    return value;
  }

  private defaultUnlockMode(mode: SecureSignerMode): SecureSignerUnlockMode {
    if (mode === "nitro") {
      return "recipient-attestation";
    }
    return "manual";
  }

  private parseConnection(profile: SecureSignerProfile): SecureSignerConnectionInfo {
    const parsed = new URL(profile.endpoint);
    const scheme = parsed.protocol.replace(":", "") as SecureSignerConnectionInfo["scheme"];
    const servicePort = Number.parseInt(parsed.port, 10);
    const startupPort = profile.startupPort ?? servicePort + 1;
    const host = parsed.hostname;

    return {
      scheme,
      host,
      servicePort,
      startupPort,
      cid: scheme === "vsock" ? Number.parseInt(host, 10) : undefined,
      localToolingCompatible:
        scheme === "vsock" || (scheme === "http" && isLocalHttpHost(host)),
    };
  }

  private buildDeployCommands(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): string[] {
    if (!profile.workspacePath) {
      return [];
    }

    if (profile.mode === "software") {
      return [
        `cd ${profile.workspacePath} && ./target/secure-sign-tcp mock --wallet ${profile.walletPath || "config/nep6_wallet.json"} --port ${connection.servicePort}`,
      ];
    }

    if (profile.mode === "sgx") {
      return [`cd ${profile.workspacePath} && ./scripts/sgx/run.sh --daemon`];
    }

    if (profile.mode === "nitro") {
      return [
        `cd ${profile.workspacePath} && ./scripts/nitro/run.sh --cid ${connection.cid} --eif-path secure-sign-nitro.eif`,
      ];
    }

    return [];
  }

  private buildUnlockCommands(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): string[] {
    if (!profile.workspacePath) {
      return [];
    }

    if (profile.mode === "nitro" && profile.kmsCiphertextBlobPath) {
      return [
        `cd ${profile.workspacePath} && SIGNER_CID=${connection.cid} SIGNER_SERVICE_PORT=${connection.servicePort} SIGNER_STARTUP_PORT=${connection.startupPort} AWS_REGION=${profile.awsRegion || "ap-southeast-1"}${profile.kmsKeyId ? ` KMS_KEY_ID=${profile.kmsKeyId}` : ""} KMS_CIPHERTEXT_BLOB_PATH=${profile.kmsCiphertextBlobPath}${profile.publicKey ? ` SIGNER_PUBLIC_KEY=${profile.publicKey}` : ""} ./scripts/auto-unlock-kms-recipient.sh`,
      ];
    }

    return [
      `cd ${profile.workspacePath} && ./target/secure-sign-tools decrypt${connection.cid ? ` --cid ${connection.cid}` : ""} --port ${connection.startupPort}`,
    ];
  }

  private buildStatusCommands(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): string[] {
    if (!profile.workspacePath || !profile.publicKey) {
      return [];
    }

    return [
      `cd ${profile.workspacePath} && ./target/secure-sign-tools status${connection.cid ? ` --cid ${connection.cid}` : ""} --port ${connection.servicePort} --public-key ${profile.publicKey}`,
    ];
  }

  private buildAttestationCommands(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): string[] {
    if (!profile.workspacePath || profile.mode !== "nitro") {
      return [];
    }

    return [
      `cd ${profile.workspacePath} && ./target/secure-sign-tools recipient-attestation --cid ${connection.cid} --port ${connection.startupPort} --output -`,
    ];
  }

  private buildStartRecipientCommands(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): string[] {
    if (!profile.workspacePath || profile.mode !== "nitro") {
      return [];
    }

    return [
      `cd ${profile.workspacePath} && ./target/secure-sign-tools start-recipient --cid ${connection.cid} --port ${connection.startupPort} --ciphertext-base64 <CiphertextForRecipient>`,
    ];
  }

  private canRunToolStatus(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): boolean {
    return Boolean(profile.workspacePath && profile.publicKey && connection.localToolingCompatible);
  }

  private resolveToolPath(profile: SecureSignerProfile): string {
    if (!profile.workspacePath) {
      throw new Error("A secure-sign-service-rs workspace path is required for local tooling operations");
    }

    return `${profile.workspacePath}/target/secure-sign-tools`;
  }

  private buildStatusArgs(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): string[] {
    if (!profile.publicKey) {
      throw new Error("A signer public key is required for account-status checks");
    }

    return [
      "status",
      ...(connection.cid ? ["--cid", String(connection.cid)] : []),
      "--port",
      String(connection.servicePort),
      "--public-key",
      profile.publicKey,
    ];
  }

  private buildAttestationArgs(profile: SecureSignerProfile, connection: SecureSignerConnectionInfo): string[] {
    return [
      "recipient-attestation",
      ...(connection.cid ? ["--cid", String(connection.cid)] : []),
      "--port",
      String(connection.startupPort),
      "--output",
      "-",
    ];
  }

  private buildStartRecipientArgs(
    profile: SecureSignerProfile,
    connection: SecureSignerConnectionInfo,
    ciphertextBase64: string,
  ): string[] {
    return [
      "start-recipient",
      ...(connection.cid ? ["--cid", String(connection.cid)] : []),
      "--port",
      String(connection.startupPort),
      "--ciphertext-base64",
      ciphertextBase64,
    ];
  }

  private parseAccountStatus(output: string): string {
    const match = output.match(/status:\s*([A-Za-z]+)/i);
    if (!match) {
      throw new Error(output.trim() || "Could not parse signer account status");
    }
    return match[1];
  }

  private persistTestResult(id: string, result: SecureSignerTestResult): void {
    this.db
      .prepare(`
        UPDATE secure_signer_profiles
        SET last_test_status = ?, last_test_message = ?, last_tested_at = ?
        WHERE id = ?
      `)
      .run(result.status, result.message, result.checkedAt, id);
  }

  private defaultRunToolCommand(toolPath: string, args: string[]): Promise<ToolCommandResult> {
    return new Promise((resolve, reject) => {
      execFile(toolPath, args, { maxBuffer: 10 * 1024 * 1024 }, (error, stdout, stderr) => {
        if (error) {
          reject(new Error(stderr?.trim() || error.message));
          return;
        }

        resolve({
          stdout,
          stderr,
        });
      });
    });
  }

  private defaultProbeEndpoint(endpoint: string): Promise<ProbeResult> {
    return new Promise((resolve) => {
      let parsed: URL;
      try {
        parsed = new URL(endpoint);
      } catch {
        resolve({ ok: false, message: "Endpoint could not be parsed" });
        return;
      }

      const port = Number.parseInt(parsed.port, 10) || (parsed.protocol === "https:" ? 443 : 80);
      const startedAt = Date.now();
      const socket = net.createConnection({ host: parsed.hostname, port });

      socket.setTimeout(1500);

      socket.on("connect", () => {
        const latencyMs = Date.now() - startedAt;
        socket.destroy();
        resolve({
          ok: true,
          message: `Connected to ${parsed.hostname}:${port}`,
          latencyMs,
        });
      });

      socket.on("timeout", () => {
        socket.destroy();
        resolve({
          ok: false,
          message: `Timed out connecting to ${parsed.hostname}:${port}`,
        });
      });

      socket.on("error", (error) => {
        resolve({
          ok: false,
          message: error.message,
        });
      });
    });
  }
}
