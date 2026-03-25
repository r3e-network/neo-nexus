import { Settings, Database, HardDrive, Trash2, AlertTriangle } from 'lucide-react';
import { useSystemMetrics } from '../hooks/useNodes';

export default function SettingsPage() {
  const { data: systemMetrics } = useSystemMetrics();

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
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
              <div className="mt-3 h-2 bg-slate-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-blue-500 transition-all duration-500"
                  style={{ width: `${systemMetrics.cpu.usage}%` }}
                />
              </div>
            </div>

            <div className="p-4 bg-slate-800/50 rounded-lg">
              <p className="text-slate-400 text-sm mb-2">Memory</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-white">{systemMetrics.memory.percentage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">
                  {formatBytes(systemMetrics.memory.used)} / {formatBytes(systemMetrics.memory.total)}
                </span>
              </div>
              <div className="mt-3 h-2 bg-slate-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-emerald-500 transition-all duration-500"
                  style={{ width: `${systemMetrics.memory.percentage}%` }}
                />
              </div>
            </div>

            <div className="p-4 bg-slate-800/50 rounded-lg">
              <p className="text-slate-400 text-sm mb-2">Disk</p>
              <div className="flex items-end gap-2">
                <span className="text-2xl font-bold text-white">{systemMetrics.disk.percentage.toFixed(1)}%</span>
                <span className="text-sm text-slate-500 mb-1">
                  {formatBytes(systemMetrics.disk.used)} / {formatBytes(systemMetrics.disk.total)}
                </span>
              </div>
              <div className="mt-3 h-2 bg-slate-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-purple-500 transition-all duration-500"
                  style={{ width: `${systemMetrics.disk.percentage}%` }}
                />
              </div>
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
          <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
            <div>
              <p className="font-medium text-white">Clean Old Logs</p>
              <p className="text-sm text-slate-400">Remove log files older than 30 days</p>
            </div>
            <button className="btn btn-secondary">
              <Trash2 className="w-4 h-4" />
              Clean
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
            <div>
              <p className="font-medium text-white">Export Configuration</p>
              <p className="text-sm text-slate-400">Download all node configurations</p>
            </div>
            <button className="btn btn-secondary">
              Export
            </button>
          </div>
        </div>
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
          <div className="flex items-center justify-between p-4 bg-red-500/5 rounded-lg border border-red-500/10">
            <div>
              <p className="font-medium text-white">Stop All Nodes</p>
              <p className="text-sm text-slate-400">Immediately stop all running nodes</p>
            </div>
            <button className="btn btn-error">
              Stop All
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-red-500/5 rounded-lg border border-red-500/10">
            <div>
              <p className="font-medium text-white">Reset All Data</p>
              <p className="text-sm text-slate-400">Delete all nodes and configuration</p>
            </div>
            <button className="btn btn-error">
              Reset
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
          <p>Version: <span className="text-white">2.0.0</span></p>
          <p>License: <span className="text-white">MIT</span></p>
          <p>Repository: <a href="https://github.com/r3e-network/neonexus" className="text-blue-400 hover:underline" target="_blank" rel="noopener noreferrer">github.com/r3e-network/neonexus</a></p>
        </div>
      </div>
    </div>
  );
}
