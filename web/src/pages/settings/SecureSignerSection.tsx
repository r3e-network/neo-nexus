import { useState, type FormEvent } from "react";
import { Shield } from "lucide-react";
import { FeedbackBanner } from "../../components/FeedbackBanner";
import {
  useCreateSecureSigner,
  useDeleteSecureSigner,
  useFetchSecureSignerAttestation,
  useSecureSignerOrchestration,
  useSecureSigners,
  useStartSecureSignerRecipient,
  useTestSecureSigner,
  useUpdateSecureSigner,
} from "../../hooks/useSecureSigners";
import type { CreateSecureSignerRequest, SecureSignerMode, SecureSignerProfile, SecureSignerUnlockMode } from "../../../../src/types";
import { signerReadinessColor } from "../../utils/signerVisibility";

const SECURE_SIGNER_MODES: Array<{ value: SecureSignerMode; label: string; description: string }> = [
  { value: "software", label: "Software", description: "Local or remote secure-sign-service mock/TCP signer" },
  { value: "sgx", label: "Intel SGX", description: "HTTP or HTTPS signer backed by an SGX enclave" },
  { value: "nitro", label: "AWS Nitro", description: "Vsock signer backed by a Nitro Enclave" },
  { value: "custom", label: "Custom", description: "Manual endpoint for compatible signer implementations" },
];

const SECURE_SIGNER_UNLOCK_MODES: Array<{ value: SecureSignerUnlockMode; label: string }> = [
  { value: "manual", label: "Manual unlock" },
  { value: "interactive-passphrase", label: "Interactive passphrase" },
  { value: "recipient-attestation", label: "Recipient attestation" },
];

function createEmptySignerForm(): CreateSecureSignerRequest {
  return {
    name: "",
    mode: "software",
    endpoint: "http://127.0.0.1:9991",
    publicKey: "",
    accountAddress: "",
    walletPath: "",
    unlockMode: "manual",
    notes: "",
    enabled: true,
    workspacePath: "",
    startupPort: 9992,
    awsRegion: "ap-southeast-1",
    kmsKeyId: "",
    kmsCiphertextBlobPath: "",
  };
}

export function SecureSignerSection() {
  const secureSigners = useSecureSigners();
  const createSecureSigner = useCreateSecureSigner();
  const updateSecureSigner = useUpdateSecureSigner();
  const deleteSecureSigner = useDeleteSecureSigner();
  const testSecureSigner = useTestSecureSigner();
  const [secureSignerForm, setSecureSignerForm] = useState<CreateSecureSignerRequest>(createEmptySignerForm);
  const [editingSecureSignerId, setEditingSecureSignerId] = useState<string | null>(null);
  const [secureSignerMessage, setSecureSignerMessage] = useState("");
  const [secureSignerError, setSecureSignerError] = useState("");

  const resetSecureSignerForm = () => {
    setEditingSecureSignerId(null);
    setSecureSignerForm(createEmptySignerForm());
  };

  const handleSecureSignerSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setSecureSignerError("");
    setSecureSignerMessage("");

    try {
      if (editingSecureSignerId) {
        await updateSecureSigner.mutateAsync({
          id: editingSecureSignerId,
          payload: secureSignerForm,
        });
        setSecureSignerMessage("Secure signer profile updated.");
      } else {
        await createSecureSigner.mutateAsync(secureSignerForm);
        setSecureSignerMessage("Secure signer profile created.");
      }
      resetSecureSignerForm();
    } catch (error) {
      setSecureSignerError(error instanceof Error ? error.message : "Failed to save secure signer profile.");
    }
  };

  const handleEditSecureSigner = (profile: CreateSecureSignerRequest & { id?: string }) => {
    setSecureSignerError("");
    setSecureSignerMessage("");
    setEditingSecureSignerId(profile.id || null);
    setSecureSignerForm({
      name: profile.name,
      mode: profile.mode,
      endpoint: profile.endpoint,
      publicKey: profile.publicKey || "",
      accountAddress: profile.accountAddress || "",
      walletPath: profile.walletPath || "",
      unlockMode: profile.unlockMode,
      notes: profile.notes || "",
      enabled: profile.enabled,
      workspacePath: profile.workspacePath || "",
      startupPort: profile.startupPort || undefined,
      awsRegion: profile.awsRegion || "ap-southeast-1",
      kmsKeyId: profile.kmsKeyId || "",
      kmsCiphertextBlobPath: profile.kmsCiphertextBlobPath || "",
    });
  };

  const handleDeleteSecureSigner = async (id: string) => {
    if (!window.confirm("Delete this secure signer profile?")) {
      return;
    }

    setSecureSignerError("");
    setSecureSignerMessage("");

    try {
      await deleteSecureSigner.mutateAsync(id);
      setSecureSignerMessage("Secure signer profile deleted.");
      if (editingSecureSignerId === id) {
        resetSecureSignerForm();
      }
    } catch (error) {
      setSecureSignerError(error instanceof Error ? error.message : "Failed to delete secure signer profile.");
    }
  };

  const handleTestSecureSigner = async (id: string) => {
    setSecureSignerError("");
    setSecureSignerMessage("");

    try {
      const result = await testSecureSigner.mutateAsync(id);
      setSecureSignerMessage(`${result.status.toUpperCase()}: ${result.message}`);
    } catch (error) {
      setSecureSignerError(error instanceof Error ? error.message : "Failed to test secure signer profile.");
    }
  };

  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-4">
        <div className="w-10 h-10 rounded-lg bg-cyan-500/10 flex items-center justify-center">
          <Shield className="w-5 h-5 text-cyan-400" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-white">Secure Signers / TEE Profiles</h2>
          <p className="text-slate-400 text-sm">
            Register software, SGX, or Nitro signing endpoints. NeoNexus stores references and metadata only, never raw WIF.
          </p>
        </div>
      </div>

      <div className="space-y-4">
        <FeedbackBanner error={secureSignerError} success={secureSignerMessage} />

        <form className="grid grid-cols-1 gap-4 lg:grid-cols-2" onSubmit={handleSecureSignerSubmit}>
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Profile Name</label>
            <input
              type="text"
              value={secureSignerForm.name}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, name: event.target.value })}
              className="input"
              placeholder="Council Nitro signer"
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Mode</label>
            <select
              value={secureSignerForm.mode}
              onChange={(event) => {
                const mode = event.target.value as SecureSignerMode;
                setSecureSignerForm({
                  ...secureSignerForm,
                  mode,
                  endpoint: mode === "nitro" ? "vsock://2345:9991" : secureSignerForm.endpoint.startsWith("vsock://") ? "http://127.0.0.1:9991" : secureSignerForm.endpoint,
                  unlockMode: mode === "nitro" ? "recipient-attestation" : secureSignerForm.unlockMode === "recipient-attestation" ? "manual" : secureSignerForm.unlockMode,
                });
              }}
              className="input"
            >
              {SECURE_SIGNER_MODES.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <p className="mt-2 text-xs text-slate-500">
              {SECURE_SIGNER_MODES.find((option) => option.value === secureSignerForm.mode)?.description}
            </p>
          </div>

          <div className="lg:col-span-2">
            <label className="mb-2 block text-sm font-medium text-slate-300">Endpoint</label>
            <input
              type="text"
              value={secureSignerForm.endpoint}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, endpoint: event.target.value })}
              className="input"
              placeholder={secureSignerForm.mode === "nitro" ? "vsock://2345:9991" : "https://signer.example.com:9443"}
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Account Public Key</label>
            <input
              type="text"
              value={secureSignerForm.publicKey}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, publicKey: event.target.value })}
              className="input font-mono text-xs"
              placeholder="Optional compressed public key"
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Account Address</label>
            <input
              type="text"
              value={secureSignerForm.accountAddress}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, accountAddress: event.target.value })}
              className="input"
              placeholder="Optional account address"
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Encrypted Wallet Path</label>
            <input
              type="text"
              value={secureSignerForm.walletPath}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, walletPath: event.target.value })}
              className="input"
              placeholder="Optional NEP-6 wallet path"
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Unlock Mode</label>
            <select
              value={secureSignerForm.unlockMode}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, unlockMode: event.target.value as SecureSignerUnlockMode })}
              className="input"
            >
              {SECURE_SIGNER_UNLOCK_MODES.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Workspace Path</label>
            <input
              type="text"
              value={secureSignerForm.workspacePath}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, workspacePath: event.target.value })}
              className="input"
              placeholder="/home/neo/git/secure-sign-service-rs"
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Startup Port</label>
            <input
              type="number"
              min="1"
              value={secureSignerForm.startupPort ?? ""}
              onChange={(event) =>
                setSecureSignerForm({
                  ...secureSignerForm,
                  startupPort: event.target.value ? Number.parseInt(event.target.value, 10) : undefined,
                })
              }
              className="input"
              placeholder="Defaults to service port + 1"
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">AWS Region</label>
            <input
              type="text"
              value={secureSignerForm.awsRegion}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, awsRegion: event.target.value })}
              className="input"
              placeholder="ap-southeast-1"
            />
          </div>

          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">KMS Key ID</label>
            <input
              type="text"
              value={secureSignerForm.kmsKeyId}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, kmsKeyId: event.target.value })}
              className="input"
              placeholder="Optional for Nitro auto-unlock"
            />
          </div>

          <div className="lg:col-span-2">
            <label className="mb-2 block text-sm font-medium text-slate-300">KMS Ciphertext Blob Path</label>
            <input
              type="text"
              value={secureSignerForm.kmsCiphertextBlobPath}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, kmsCiphertextBlobPath: event.target.value })}
              className="input"
              placeholder="/home/ec2-user/neo/secure/wallet-passphrase.kms.bin"
            />
          </div>

          <div className="lg:col-span-2">
            <label className="mb-2 block text-sm font-medium text-slate-300">Notes</label>
            <textarea
              value={secureSignerForm.notes}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, notes: event.target.value })}
              className="input min-h-28"
              placeholder="Optional operational notes, attestation expectations, or wallet references"
            />
          </div>

          <div className="lg:col-span-2 flex items-center justify-between rounded-lg border border-slate-700 bg-slate-800/40 px-4 py-3">
            <div>
              <p className="text-sm font-medium text-white">Profile Enabled</p>
              <p className="text-xs text-slate-400">Disabled profiles cannot be attached to nodes.</p>
            </div>
            <input
              type="checkbox"
              checked={secureSignerForm.enabled ?? true}
              onChange={(event) => setSecureSignerForm({ ...secureSignerForm, enabled: event.target.checked })}
              className="h-4 w-4"
            />
          </div>

          <div className="lg:col-span-2 flex gap-3 justify-end">
            {editingSecureSignerId && (
              <button className="btn btn-secondary" type="button" onClick={resetSecureSignerForm}>
                Cancel
              </button>
            )}
            <button className="btn btn-primary" disabled={createSecureSigner.isPending || updateSecureSigner.isPending} type="submit">
              {editingSecureSignerId
                ? updateSecureSigner.isPending
                  ? "Updating..."
                  : "Update Profile"
                : createSecureSigner.isPending
                  ? "Creating..."
                  : "Create Profile"}
            </button>
          </div>
        </form>

        <div className="space-y-3">
          {(secureSigners.data ?? []).length === 0 ? (
            <div className="rounded-lg border border-slate-700 bg-slate-800/40 px-4 py-6 text-sm text-slate-400">
              No secure signer profiles yet. Add a software, SGX, or Nitro signer endpoint here before attaching one to a node.
            </div>
          ) : (
            (secureSigners.data ?? []).map((profile) => (
              <SecureSignerProfileCard
                key={profile.id}
                profile={profile}
                onEdit={() => handleEditSecureSigner(profile)}
                onDelete={() => handleDeleteSecureSigner(profile.id)}
                onTest={() => handleTestSecureSigner(profile.id)}
                testing={testSecureSigner.isPending}
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
}

function SecureSignerProfileCard({
  profile,
  onEdit,
  onDelete,
  onTest,
  testing,
}: {
  profile: SecureSignerProfile;
  onEdit: () => void;
  onDelete: () => void;
  onTest: () => void;
  testing: boolean;
}) {
  const orchestration = useSecureSignerOrchestration(profile.id);
  const fetchAttestation = useFetchSecureSignerAttestation();
  const startRecipient = useStartSecureSignerRecipient();
  const [attestationBase64, setAttestationBase64] = useState("");
  const [ciphertextBase64, setCiphertextBase64] = useState("");
  const [localMessage, setLocalMessage] = useState("");
  const [localError, setLocalError] = useState("");

  const readiness = orchestration.data?.readiness;

  const handleFetchAttestation = async () => {
    setLocalError("");
    setLocalMessage("");
    try {
      const result = await fetchAttestation.mutateAsync(profile.id);
      setAttestationBase64(result.attestationBase64);
      setLocalMessage("Recipient attestation fetched.");
    } catch (error) {
      setLocalError(error instanceof Error ? error.message : "Failed to fetch attestation.");
    }
  };

  const handleStartRecipient = async () => {
    setLocalError("");
    setLocalMessage("");
    try {
      const result = await startRecipient.mutateAsync({ id: profile.id, ciphertextBase64 });
      setLocalMessage(result.message);
    } catch (error) {
      setLocalError(error instanceof Error ? error.message : "Failed to start signer with recipient ciphertext.");
    }
  };

  return (
    <div className="rounded-lg border border-slate-700 bg-slate-800/40 p-4 space-y-4">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <p className="font-medium text-white">{profile.name}</p>
            <span className="rounded-full bg-slate-700 px-2.5 py-1 text-xs uppercase tracking-wide text-slate-300">
              {profile.mode}
            </span>
            <span
              className={`rounded-full px-2.5 py-1 text-xs ${
                profile.enabled ? "bg-emerald-500/10 text-emerald-300" : "bg-slate-700 text-slate-400"
              }`}
            >
              {profile.enabled ? "Enabled" : "Disabled"}
            </span>
            {readiness && (
              <span
                className={`rounded-full px-2.5 py-1 text-xs ${signerReadinessColor(readiness.status)}`}
              >
                {readiness.status}
              </span>
            )}
          </div>
          <p className="text-sm text-slate-400 break-all">{profile.endpoint}</p>
          <div className="grid grid-cols-1 gap-2 text-xs text-slate-500 sm:grid-cols-2">
            <span>Unlock: {profile.unlockMode}</span>
            <span>Public key: {profile.publicKey || "Not set"}</span>
            <span>Address: {profile.accountAddress || "Not set"}</span>
            <span>Wallet: {profile.walletPath || "Reference only"}</span>
            <span>Workspace: {profile.workspacePath || "Not set"}</span>
            <span>Startup port: {profile.startupPort || "Auto (+1)"}</span>
          </div>
          {readiness?.message && <p className="text-xs text-slate-500">{readiness.message}</p>}
          {profile.lastTestMessage && !readiness?.message && <p className="text-xs text-slate-500">{profile.lastTestMessage}</p>}
        </div>

        <div className="flex flex-wrap gap-2">
          <button className="btn btn-secondary" type="button" onClick={onTest}>
            {testing ? "Testing..." : "Test"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={onEdit}>
            Edit
          </button>
          <button className="btn btn-error" type="button" onClick={onDelete}>
            Delete
          </button>
        </div>
      </div>

      {(localError || localMessage) && (
        <div
          className={`rounded-lg px-4 py-3 text-sm ${
            localError
              ? "border border-red-500/20 bg-red-500/10 text-red-300"
              : "border border-cyan-500/20 bg-cyan-500/10 text-cyan-300"
          }`}
        >
          {localError || localMessage}
        </div>
      )}

      {orchestration.data && (
        <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
          <div className="rounded-lg border border-slate-700 bg-slate-900/50 p-4 space-y-3">
            <div>
              <p className="text-sm font-medium text-white">Connection</p>
              <p className="text-xs text-slate-500">
                {orchestration.data.connection.scheme} · service {orchestration.data.connection.servicePort} · startup {orchestration.data.connection.startupPort}
                {orchestration.data.connection.cid ? ` · cid ${orchestration.data.connection.cid}` : ""}
              </p>
            </div>

            {orchestration.data.warnings.length > 0 && (
              <div className="space-y-2">
                {orchestration.data.warnings.map((warning, index) => (
                  <p key={index} className="text-xs text-amber-300">
                    {warning}
                  </p>
                ))}
              </div>
            )}

            {([
              ["Deploy", orchestration.data.commands.deploy],
              ["Unlock", orchestration.data.commands.unlock],
              ["Status", orchestration.data.commands.status],
            ] as Array<[string, string[]]>).map(([label, commands]) =>
              commands.length > 0 ? (
                <div key={label}>
                  <p className="mb-2 text-xs font-medium uppercase tracking-wide text-slate-400">{label}</p>
                  <pre className="overflow-x-auto rounded-lg bg-slate-950 p-3 text-xs text-slate-300">{commands.join("\n")}</pre>
                </div>
              ) : null,
            )}
          </div>

          <div className="rounded-lg border border-slate-700 bg-slate-900/50 p-4 space-y-3">
            <div>
              <p className="text-sm font-medium text-white">Attestation / Ciphertext Startup</p>
              <p className="text-xs text-slate-500">
                Nitro profiles can fetch recipient attestation and start the signer from `CiphertextForRecipient` without exposing plaintext passphrases to NeoNexus.
              </p>
            </div>

            {orchestration.data.commands.attestation.length > 0 && (
              <pre className="overflow-x-auto rounded-lg bg-slate-950 p-3 text-xs text-slate-300">{orchestration.data.commands.attestation.join("\n")}</pre>
            )}

            <div className="flex flex-wrap gap-2">
              <button
                className="btn btn-secondary"
                type="button"
                disabled={fetchAttestation.isPending || profile.mode !== "nitro"}
                onClick={handleFetchAttestation}
              >
                {fetchAttestation.isPending ? "Fetching..." : "Fetch Attestation"}
              </button>
            </div>

            {attestationBase64 && (
              <textarea
                readOnly
                value={attestationBase64}
                className="input min-h-28 font-mono text-xs"
              />
            )}

            <textarea
              value={ciphertextBase64}
              onChange={(event) => setCiphertextBase64(event.target.value)}
              className="input min-h-28 font-mono text-xs"
              placeholder="Paste AWS KMS CiphertextForRecipient here"
            />

            <button
              className="btn btn-primary"
              type="button"
              disabled={startRecipient.isPending || profile.mode !== "nitro" || !ciphertextBase64.trim()}
              onClick={handleStartRecipient}
            >
              {startRecipient.isPending ? "Starting..." : "Start Recipient Unlock"}
            </button>

            {orchestration.data.commands.startRecipient.length > 0 && (
              <pre className="overflow-x-auto rounded-lg bg-slate-950 p-3 text-xs text-slate-300">
                {orchestration.data.commands.startRecipient.join("\n")}
              </pre>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
