import { useState } from "react";
import { HardDrive, Trash2 } from "lucide-react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { FeedbackBanner } from "../../components/FeedbackBanner";
import { useCleanLogs, useExportConfiguration, useRestoreConfiguration } from "../../hooks/useSystemActions";
import type { ConfigurationSnapshot } from "../../../../src/types";

export function StorageSection() {
  const [storageMessage, setStorageMessage] = useState("");
  const [storageError, setStorageError] = useState("");
  const [restoreFile, setRestoreFile] = useState<File | null>(null);
  const [replaceExisting, setReplaceExisting] = useState(false);
  const [pendingRestoreSnapshot, setPendingRestoreSnapshot] = useState<ConfigurationSnapshot | null>(null);
  const cleanLogs = useCleanLogs();
  const exportConfiguration = useExportConfiguration();
  const restoreConfiguration = useRestoreConfiguration();

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
      setPendingRestoreSnapshot(snapshot);
    } catch (error) {
      setStorageError(error instanceof Error ? error.message : "Failed to restore configuration.");
    }
  };

  const confirmRestoreConfiguration = async () => {
    if (!pendingRestoreSnapshot) {
      return;
    }

    try {
      const result = await restoreConfiguration.mutateAsync({
        snapshot: pendingRestoreSnapshot,
        replaceExisting,
      });
      setPendingRestoreSnapshot(null);
      setStorageMessage(
        `Restored ${result.restoredCount} nodes, skipped ${result.skippedCount}, failed ${result.failedCount}.`,
      );
    } catch (error) {
      setStorageError(error instanceof Error ? error.message : "Failed to restore configuration.");
    }
  };

  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-4">
        <div className="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center">
          <HardDrive className="w-5 h-5 text-amber-400" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-slate-950">Storage Management</h2>
          <p className="text-slate-600 text-sm">Manage node data and logs</p>
        </div>
      </div>

      <div className="space-y-4">
        <FeedbackBanner error={storageError} success={storageMessage} />

        <div className="flex flex-col gap-3 p-4 bg-slate-50 rounded-lg border border-slate-200 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="font-medium text-slate-950">Clean Old Logs</p>
            <p className="text-sm text-slate-600">Remove log files older than 30 days</p>
          </div>
          <button className="btn btn-secondary justify-center sm:w-auto" disabled={cleanLogs.isPending} onClick={handleCleanLogs} type="button">
            <Trash2 className="w-4 h-4" />
            {cleanLogs.isPending ? "Cleaning..." : "Clean"}
          </button>
        </div>

        <div className="flex flex-col gap-3 p-4 bg-slate-50 rounded-lg border border-slate-200 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="font-medium text-slate-950">Export Configuration</p>
            <p className="text-sm text-slate-600">Download all node configurations</p>
          </div>
          <button
            className="btn btn-secondary justify-center sm:w-auto"
            disabled={exportConfiguration.isPending}
            onClick={handleExportConfiguration}
            type="button"
          >
            {exportConfiguration.isPending ? "Exporting..." : "Export"}
          </button>
        </div>

        <div className="p-4 bg-slate-50 rounded-lg border border-slate-200 space-y-4">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p className="font-medium text-slate-950">Restore Configuration</p>
              <p className="text-sm text-slate-600">Import a previously exported NeoNexus JSON snapshot</p>
            </div>
            <button
              className="btn btn-secondary justify-center sm:w-auto"
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
              className="block w-full text-sm text-slate-600 file:mr-4 file:rounded-lg file:border-0 file:bg-slate-100 file:px-4 file:py-2 file:text-sm file:font-medium file:text-slate-700 hover:file:bg-slate-200"
            />

            <label className="flex items-center gap-3 text-sm text-slate-600">
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

      <ConfirmDialog
        open={Boolean(pendingRestoreSnapshot)}
        title="Restore configuration snapshot?"
        description={
          replaceExisting
            ? "This will restore the selected snapshot and replace existing node definitions first."
            : "This will restore the selected snapshot into the current installation."
        }
        confirmLabel="Restore snapshot"
        confirmVariant="primary"
        isConfirming={restoreConfiguration.isPending}
        onCancel={() => setPendingRestoreSnapshot(null)}
        onConfirm={() => void confirmRestoreConfiguration()}
      />
    </div>
  );
}
