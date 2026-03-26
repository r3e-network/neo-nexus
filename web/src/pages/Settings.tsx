import { useState, type FormEvent } from "react";
import { Settings, Database, HardDrive, Trash2, AlertTriangle, Lock, Shield } from "lucide-react";
import { FeedbackBanner } from "../components/FeedbackBanner";
import { ProgressBar } from "../components/ProgressBar";
import { useSystemMetrics } from "../hooks/useNodes";
import { useAuth } from "../hooks/useAuth";
import { useCleanLogs, useExportConfiguration, useResetAllData, useRestoreConfiguration, useStopAllNodes } from "../hooks/useSystemActions";
import {
  useCreateSecureSigner,
  useDeleteSecureSigner,
  useFetchSecureSignerAttestation,
  useSecureSignerOrchestration,
  useSecureSigners,
  useStartSecureSignerRecipient,
  useTestSecureSigner,
  useUpdateSecureSigner,
} from "../hooks/useSecureSigners";
import type { ConfigurationSnapshot, CreateSecureSignerRequest, SecureSignerMode, SecureSignerProfile, SecureSignerUnlockMode } from "../../../src/types";
import { formatBytes } from "../utils/format";
import { signerReadinessColor } from "../utils/signerVisibility";

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

export default function SettingsPage() {
  const { data: systemMetrics } = useSystemMetrics();
  const { changePassword, isChangingPassword } = useAuth();
  const secureSigners = useSecureSigners();
  const createSecureSigner = useCreateSecureSigner();
  const updateSecureSigner = useUpdateSecureSigner();
  const deleteSecureSigner = useDeleteSecureSigner();
  const testSecureSigner = useTestSecureSigner();
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [passwordError, setPasswordError] = useState("");
  const [passwordSuccess, setPasswordSuccess] = useState("");
  const [storageMessage, setStorageMessage] = useState("");
  const [storageError, setStorageError] = useState("");
  const [restoreFile, setRestoreFile] = useState<File | null>(null);
  const [replaceExisting, setReplaceExisting] = useState(false);
  const [dangerMessage, setDangerMessage] = useState("");
  const [dangerError, setDangerError] = useState("");
  const [secureSignerForm, setSecureSignerForm] = useState<CreateSecureSignerRequest>(createEmptySignerForm);
  const [editingSecureSignerId, setEditingSecureSignerId] = useState<string | null>(null);
  const [secureSignerMessage, setSecureSignerMessage] = useState("");
  const [secureSignerError, setSecureSignerError] = useState("");
  const cleanLogs = useCleanLogs();
  const exportConfiguration = useExportConfiguration();
  const restoreConfiguration = useRestoreConfiguration();
  const stopAllNodes = useStopAllNodes();
  const resetAllData = useResetAllData();

  const handlePasswordSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setPasswordError("");
    setPasswordSuccess("");

    if (!currentPassword || !newPassword || !confirmPassword) {
      setPasswordError("All password fields are required.");
      return;
    }

    if (newPassword.length < 8) {
      setPasswordError("New password must be at least 8 characters.");
      return;
    }

    if (newPassword !== confirmPassword) {
      setPasswordError("New passwords do not match.");
      return;
    }

    try {
      await changePassword(currentPassword, newPassword);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      setPasswordSuccess("Password updated successfully.");
    } catch (error) {
      setPasswordError(error instanceof Error ? error.message : "Failed to update password.");
    }
  };

  const handleCleanLogs = async () => {
    setStorageError("");
    setStorageMessage("");

    try {
      const result = await cleanLogs.mutateAsync(30);
      setStorageMessage(`Removed ${result.cleanedFiles} log files across ${result.nodesAffected} nodes.`);
    } catch (error) {
      setStorageError(error instanceof Error ? error.message : "Failed to clean old logs.");
    }
  };

  const handleExportConfiguration = async () => {
    setStorageError("");
    setStorageMessage("");

    try {
      const result = await exportConfiguration.mutateAsync();
      setStorageMessage(`Exported configuration snapshot with ${result.nodes.length} nodes.`);
    } catch (error) {
      setStorageError(error instanceof Error ? error.message : "Failed to export configuration.");
    }
  };

  const handleRestoreConfiguration = async () => {
    setStorageError("");
    setStorageMessage("");

    if (!restoreFile) {
      setStorageError("Choose an exported NeoNexus JSON snapshot first.");
      return;
    }

    try {
      const fileContents = await restoreFile.text();
      const snapshot = JSON.parse(fileContents) as ConfigurationSnapshot;

      if (!snapshot || !Array.isArray(snapshot.nodes)) {
        setStorageError("The selected file does not contain a valid NeoNexus snapshot.");
        return;
      }

      const confirmed = window.confirm(
        replaceExisting
          ? "Restore this snapshot and replace existing nodes? This will remove current node definitions first."
          : "Restore this snapshot into the current installation?",
      );

      if (!confirmed) {
        return;
      }

      const result = await restoreConfiguration.mutateAsync({
        snapshot,
        replaceExisting,
      });
      setStorageMessage(
        `Restored ${result.restoredCount} nodes, skipped ${result.skippedCount}, failed ${result.failedCount}.`,
      );
    } catch (error) {
      setStorageError(error instanceof Error ? error.message : "Failed to restore configuration.");
    }
  };

  const handleStopAllNodes = async () => {
    if (!window.confirm("Stop all running nodes now?")) {
      return;
    }

    setDangerError("");
    setDangerMessage("");

    try {
      const result = await stopAllNodes.mutateAsync();
      setDangerMessage(`Stopped ${result.stoppedCount} nodes. ${result.alreadyStoppedCount} were already stopped.`);
    } catch (error) {
      setDangerError(error instanceof Error ? error.message : "Failed to stop nodes.");
    }
  };

  const handleResetAllData = async () => {
    if (!window.confirm("Reset all node data? This removes NeoNexus-managed node records and managed node directories.")) {
      return;
    }

    setDangerError("");
    setDangerMessage("");

    try {
      const result = await resetAllData.mutateAsync();
      setDangerMessage(
        `Deleted ${result.deletedNodeCount} nodes and removed ${result.removedDirectoryCount} managed directories.`,
      );
    } catch (error) {
      setDangerError(error instanceof Error ? error.message : "Failed to reset data.");
    }
  };

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
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-slate-400 mt-1">Manage system settings and resources</p>
      </div>

      {/* System Resources */}
      {systemMetrics && (
        <div className="card">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
              <Database className="w-5 h-5 text-blue-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">System Resources</h2>
              <p className="text-slate-400 text-sm">Current system utilization</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="p-4 bg-slate-800/50 rounded-lg">
              <p className="text-slate-400 text-sm mb-2">CPU Usage</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-white">{systemMetrics.cpu.usage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">{systemMetrics.cpu.cores} cores</span>
              </div>
              <ProgressBar value={systemMetrics.cpu.usage} color="bg-blue-500" className="mt-3" />
            </div>

            <div className="p-4 bg-slate-800/50 rounded-lg">
              <p className="text-slate-400 text-sm mb-2">Memory</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-white">{systemMetrics.memory.percentage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">
                  {formatBytes(systemMetrics.memory.used)} / {formatBytes(systemMetrics.memory.total)}
                </span>
              </div>
              <ProgressBar value={systemMetrics.memory.percentage} color="bg-emerald-500" className="mt-3" />
            </div>

            <div className="p-4 bg-slate-800/50 rounded-lg">
              <p className="text-slate-400 text-sm mb-2">Disk</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-white">{systemMetrics.disk.percentage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">
                  {formatBytes(systemMetrics.disk.used)} / {formatBytes(systemMetrics.disk.total)}
                </span>
              </div>
              <ProgressBar value={systemMetrics.disk.percentage} color="bg-purple-500" className="mt-3" />
            </div>
          </div>
        </div>
      )}

      {/* Storage Management */}
      <div className="card">
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center">
            <HardDrive className="w-5 h-5 text-purple-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-white">Storage Management</h2>
            <p className="text-slate-400 text-sm">Manage node data and logs</p>
          </div>
        </div>

        <div className="space-y-4">
          <FeedbackBanner error={storageError} success={storageMessage} />

          <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
            <div>
              <p className="font-medium text-white">Clean Old Logs</p>
              <p className="text-sm text-slate-400">Remove log files older than 30 days</p>
            </div>
            <button className="btn btn-secondary" disabled={cleanLogs.isPending} onClick={handleCleanLogs} type="button">
              <Trash2 className="w-4 h-4" />
              {cleanLogs.isPending ? "Cleaning..." : "Clean"}
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
            <div>
              <p className="font-medium text-white">Export Configuration</p>
              <p className="text-sm text-slate-400">Download all node configurations</p>
            </div>
            <button
              className="btn btn-secondary"
              disabled={exportConfiguration.isPending}
              onClick={handleExportConfiguration}
              type="button"
            >
              {exportConfiguration.isPending ? "Exporting..." : "Export"}
            </button>
          </div>

          <div className="p-4 bg-slate-800/50 rounded-lg space-y-4">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="font-medium text-white">Restore Configuration</p>
                <p className="text-sm text-slate-400">Import a previously exported NeoNexus JSON snapshot</p>
              </div>
              <button
                className="btn btn-secondary"
                disabled={restoreConfiguration.isPending}
                onClick={handleRestoreConfiguration}
                type="button"
              >
                {restoreConfiguration.isPending ? "Restoring..." : "Restore"}
              </button>
            </div>

            <div className="space-y-3">
              <input
                type="file"
                accept="application/json,.json"
                onChange={(event) => setRestoreFile(event.target.files?.[0] || null)}
                className="block w-full text-sm text-slate-300 file:mr-4 file:rounded-lg file:border-0 file:bg-slate-700 file:px-4 file:py-2 file:text-sm file:font-medium file:text-white hover:file:bg-slate-600"
              />

              <label className="flex items-center gap-3 text-sm text-slate-300">
                <input
                  type="checkbox"
                  checked={replaceExisting}
                  onChange={(event) => setReplaceExisting(event.target.checked)}
                  className="h-4 w-4"
                />
                Replace existing node definitions before restore
              </label>
            </div>
          </div>
        </div>
      </div>

      {/* Secure Signers */}
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

      {/* Password Management */}
      <div className="card">
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center">
            <Lock className="w-5 h-5 text-emerald-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-white">Change Password</h2>
            <p className="text-slate-400 text-sm">Update the administrator password used to access NeoNexus</p>
          </div>
        </div>

        <FeedbackBanner error={passwordError} success={passwordSuccess} />

        <form className="grid grid-cols-1 gap-4 md:grid-cols-3" onSubmit={handlePasswordSubmit}>
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Current Password</label>
            <input
              type="password"
              value={currentPassword}
              onChange={(event) => setCurrentPassword(event.target.value)}
              className="input"
              placeholder="Current password"
            />
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">New Password</label>
            <input
              type="password"
              value={newPassword}
              onChange={(event) => setNewPassword(event.target.value)}
              className="input"
              placeholder="At least 8 characters"
            />
          </div>
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">Confirm Password</label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(event) => setConfirmPassword(event.target.value)}
              className="input"
              placeholder="Repeat new password"
            />
          </div>

          <div className="md:col-span-3 flex justify-end">
            <button className="btn btn-primary" disabled={isChangingPassword} type="submit">
              <Lock className="w-4 h-4" />
              {isChangingPassword ? "Updating..." : "Change Password"}
            </button>
          </div>
        </form>
      </div>

      {/* Danger Zone */}
      <div className="card border-red-500/20">
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center">
            <AlertTriangle className="w-5 h-5 text-red-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-white">Danger Zone</h2>
            <p className="text-slate-400 text-sm">Irreversible actions</p>
          </div>
        </div>

        <div className="space-y-4">
          <FeedbackBanner error={dangerError} success={dangerMessage} />

          <div className="flex items-center justify-between p-4 bg-red-500/5 rounded-lg border border-red-500/10">
            <div>
              <p className="font-medium text-white">Stop All Nodes</p>
              <p className="text-sm text-slate-400">Immediately stop all running nodes</p>
            </div>
            <button className="btn btn-error" disabled={stopAllNodes.isPending} onClick={handleStopAllNodes} type="button">
              {stopAllNodes.isPending ? "Stopping..." : "Stop All"}
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-red-500/5 rounded-lg border border-red-500/10">
            <div>
              <p className="font-medium text-white">Reset All Data</p>
              <p className="text-sm text-slate-400">Delete all nodes and configuration</p>
            </div>
            <button className="btn btn-error" disabled={resetAllData.isPending} onClick={handleResetAllData} type="button">
              {resetAllData.isPending ? "Resetting..." : "Reset"}
            </button>
          </div>
        </div>
      </div>

      {/* About */}
      <div className="card">
        <div className="flex items-center gap-3 mb-4">
          <Settings className="w-6 h-6 text-slate-400" />
          <div>
            <h2 className="text-lg font-semibold text-white">About NeoNexus</h2>
          </div>
        </div>
        <div className="space-y-2 text-sm text-slate-400">
          <p>Version: <span className="text-white">{__APP_VERSION__}</span></p>
          <p>License: <span className="text-white">MIT</span></p>
          <p>Repository: <a href="https://github.com/r3e-network/neonexus" className="text-blue-400 hover:underline" target="_blank" rel="noopener noreferrer">github.com/r3e-network/neonexus</a></p>
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
