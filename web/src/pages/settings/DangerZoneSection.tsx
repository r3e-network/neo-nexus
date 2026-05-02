import { useState } from "react";
import { AlertTriangle } from "lucide-react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { FeedbackBanner } from "../../components/FeedbackBanner";
import { useResetAllData, useStopAllNodes } from "../../hooks/useSystemActions";

export function DangerZoneSection() {
  const [pendingAction, setPendingAction] = useState<"stop-all" | "reset-all" | null>(null);
  const [dangerMessage, setDangerMessage] = useState("");
  const [dangerError, setDangerError] = useState("");
  const stopAllNodes = useStopAllNodes();
  const resetAllData = useResetAllData();

  const handleStopAllNodes = async () => {
    setDangerError("");
    setDangerMessage("");

    try {
      const result = await stopAllNodes.mutateAsync();
      setPendingAction(null);
      setDangerMessage(`Stopped ${result.stoppedCount} nodes. ${result.alreadyStoppedCount} were already stopped.`);
    } catch (error) {
      setDangerError(error instanceof Error ? error.message : "Failed to stop nodes.");
    }
  };

  const handleResetAllData = async () => {
    setDangerError("");
    setDangerMessage("");

    try {
      const result = await resetAllData.mutateAsync();
      setPendingAction(null);
      setDangerMessage(
        `Deleted ${result.deletedNodeCount} nodes and removed ${result.removedDirectoryCount} managed directories.`,
      );
    } catch (error) {
      setDangerError(error instanceof Error ? error.message : "Failed to reset data.");
    }
  };

  return (
    <div className="card border-red-500/30 border-t-2 border-t-red-500/60">
      <div className="flex items-center gap-3 mb-4">
        <div className="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center">
          <AlertTriangle className="w-5 h-5 text-red-400" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-slate-950">Danger Zone</h2>
          <p className="text-slate-600 text-sm">Irreversible actions</p>
        </div>
      </div>

      <div className="space-y-4">
        <FeedbackBanner error={dangerError} success={dangerMessage} />

        <div className="flex flex-col gap-3 p-4 bg-red-50 rounded-lg border border-red-200 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="font-medium text-slate-950">Stop All Nodes</p>
            <p className="text-sm text-slate-600">Immediately stop all running nodes</p>
          </div>
          <button className="btn btn-error justify-center sm:w-auto" disabled={stopAllNodes.isPending} onClick={() => setPendingAction("stop-all")} type="button">
            {stopAllNodes.isPending ? "Stopping..." : "Stop All"}
          </button>
        </div>

        <div className="flex flex-col gap-3 p-4 bg-red-50 rounded-lg border border-red-200 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="font-medium text-slate-950">Reset All Data</p>
            <p className="text-sm text-slate-600">Delete all nodes and configuration</p>
          </div>
          <button className="btn btn-error justify-center sm:w-auto" disabled={resetAllData.isPending} onClick={() => setPendingAction("reset-all")} type="button">
            {resetAllData.isPending ? "Resetting..." : "Reset"}
          </button>
        </div>
      </div>

      <ConfirmDialog
        open={pendingAction !== null}
        title={pendingAction === "reset-all" ? "Reset all node data?" : "Stop all running nodes?"}
        description={
          pendingAction === "reset-all"
            ? "This removes NeoNexus-managed node records and managed node directories."
            : "This immediately sends stop commands to every running node."
        }
        confirmLabel={pendingAction === "reset-all" ? "Reset all data" : "Stop all nodes"}
        isConfirming={stopAllNodes.isPending || resetAllData.isPending}
        onCancel={() => setPendingAction(null)}
        onConfirm={() => {
          if (pendingAction === "reset-all") {
            void handleResetAllData();
            return;
          }
          void handleStopAllNodes();
        }}
      />
    </div>
  );
}
