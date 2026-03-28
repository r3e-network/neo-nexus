import { Settings, Database } from "lucide-react";
import { ProgressBar } from "../components/ProgressBar";
import { useSystemMetrics } from "../hooks/useNodes";
import { formatBytes } from "../utils/format";
import { PasswordSection } from "./settings/PasswordSection";
import { StorageSection } from "./settings/StorageSection";
import { SecureSignerSection } from "./settings/SecureSignerSection";
import { DangerZoneSection } from "./settings/DangerZoneSection";
import { UserManagement } from "./settings/UserManagement";
import { AuditLogSection } from "./settings/AuditLogSection";
import { useAuth } from "../hooks/useAuth";

export default function SettingsPage() {
  const { data: systemMetrics } = useSystemMetrics();
  const { user } = useAuth();

  return (
    <div className="space-y-6 animate-fade-in">
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

      <StorageSection />
      <SecureSignerSection />
      <PasswordSection />
      {user?.role === 'admin' && <UserManagement />}
      {user?.role === 'admin' && <AuditLogSection />}
      <DangerZoneSection />

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
