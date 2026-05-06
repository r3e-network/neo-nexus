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
import { PROJECT_LINKS } from "../config/constants";

export default function SettingsPage() {
  const { data: systemMetrics } = useSystemMetrics();
  const { user } = useAuth();
  const isAdmin = user?.role === "admin";

  return (
    <div className="space-y-7 animate-fade-in">
      <section className="page-hero pb-5">
        <div>
          <p className="console-kicker">Control plane settings</p>
          <h1 className="mt-2 text-3xl font-semibold text-slate-950">Settings</h1>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
            {isAdmin
              ? "Configure storage, secure signer profiles, users, audit trails, and guarded destructive operations."
              : "Manage your account password and review current control plane resources."}
          </p>
        </div>
      </section>

      {/* System Resources */}
      {systemMetrics && (
        <div className="card">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
              <Database className="w-5 h-5 text-blue-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-slate-950">System Resources</h2>
              <p className="text-slate-600 text-sm">Current system utilization</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="metric-tile">
              <p className="text-slate-600 text-sm mb-2">CPU Usage</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-slate-950">{systemMetrics.cpu.usage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">{systemMetrics.cpu.cores} cores</span>
              </div>
              <ProgressBar value={systemMetrics.cpu.usage} color="bg-blue-500" className="mt-3" />
            </div>

            <div className="metric-tile">
              <p className="text-slate-600 text-sm mb-2">Memory</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-slate-950">{systemMetrics.memory.percentage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">
                  {formatBytes(systemMetrics.memory.used)} / {formatBytes(systemMetrics.memory.total)}
                </span>
              </div>
              <ProgressBar value={systemMetrics.memory.percentage} color="bg-emerald-500" className="mt-3" />
            </div>

            <div className="metric-tile">
              <p className="text-slate-600 text-sm mb-2">Disk</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-slate-950">{systemMetrics.disk.percentage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">
                  {formatBytes(systemMetrics.disk.used)} / {formatBytes(systemMetrics.disk.total)}
                </span>
              </div>
              <ProgressBar value={systemMetrics.disk.percentage} color="bg-amber-500" className="mt-3" />
            </div>
          </div>
        </div>
      )}

      {isAdmin && <StorageSection />}
      {isAdmin && <SecureSignerSection />}
      <PasswordSection />
      {isAdmin && <UserManagement />}
      {isAdmin && <AuditLogSection />}
      {isAdmin && <DangerZoneSection />}

      {/* About */}
      <div className="card">
        <div className="flex items-center gap-3 mb-4">
          <Settings className="w-6 h-6 text-slate-500" />
          <div>
            <h2 className="text-lg font-semibold text-slate-950">About NeoNexus</h2>
          </div>
        </div>
        <div className="space-y-2 text-sm text-slate-600">
          <p>Version: <span className="text-slate-950">{__APP_VERSION__}</span></p>
          <p>License: <span className="text-slate-950">MIT</span></p>
          <p>Repository: <a href={PROJECT_LINKS.repositoryUrl} className="text-teal-700 hover:text-teal-900 hover:underline" target="_blank" rel="noopener noreferrer">{PROJECT_LINKS.repositoryLabel}</a></p>
        </div>
      </div>
    </div>
  );
}
