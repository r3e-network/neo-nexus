import { useState } from "react";
import { AlertTriangle } from "lucide-react";
import { FeedbackBanner } from "../../components/FeedbackBanner";
import { useResetAllData, useStopAllNodes } from "../../hooks/useSystemActions";

export function DangerZoneSection() {
  const [dangerMessage, setDangerMessage] = useState("");
  const [dangerError, setDangerError] = useState("");
  const stopAllNodes = useStopAllNodes();
  const resetAllData = useResetAllData();

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

  return (
    <div className="card border-red-500/30 border-t-2 border-t-red-500/60">
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
  );
}
