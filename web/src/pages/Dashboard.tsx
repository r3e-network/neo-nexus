import { Activity, Server, Cpu, Play, AlertCircle, AlertTriangle, Settings, ShieldCheck } from "lucide-react";
import { ProgressBar } from "../components/ProgressBar";
import { SignerStatus } from "../components/SignerStatus";
import { NodeProtectionLabel } from "../components/NodeProtectionLabel";
import { StatSkeleton } from "../components/LoadingSkeleton";
import { EmptyState } from "../components/EmptyState";
import { useNodes, useSystemMetrics } from "../hooks/useNodes";
import { useAuth } from "../hooks/useAuth";
import { Link } from "react-router-dom";
import { countProtectedNodes } from "../utils/signerVisibility";

export default function Dashboard() {
  const { data: nodes = [], isLoading } = useNodes();
  const { data: systemMetrics } = useSystemMetrics();
  const { user } = useAuth();

  // Check if using default credentials
  const isDefaultPassword = user?.usingDefaultPassword === true;

  const runningNodes = nodes.filter((n) => n.process.status === "running");
  const errorNodes = nodes.filter((n) => n.process.status === "error");
  const protectedNodes = countProtectedNodes(nodes);

  const stats = [
    {
      label: "Total Nodes",
      value: nodes.length,
      icon: Server,
      color: "text-blue-400",
      bgColor: "bg-blue-500/10",
    },
    {
      label: "Running",
      value: runningNodes.length,
      icon: Play,
      color: "text-emerald-400",
      bgColor: "bg-emerald-500/10",
    },
    {
      label: "Errors",
      value: errorNodes.length,
      icon: AlertCircle,
      color: errorNodes.length > 0 ? "text-red-400" : "text-slate-400",
      bgColor: errorNodes.length > 0 ? "bg-red-500/10" : "bg-slate-500/10",
    },
    {
      label: "Protected",
      value: protectedNodes,
      icon: ShieldCheck,
      color: protectedNodes > 0 ? "text-cyan-400" : "text-slate-400",
      bgColor: protectedNodes > 0 ? "bg-cyan-500/10" : "bg-slate-500/10",
    },
  ];

  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-2xl font-bold text-white">Dashboard</h1>
        <p className="text-slate-400 mt-1">Overview of your Neo nodes</p>
      </div>

      {/* Security Warning - Default Password */}
      {isDefaultPassword && (
        <div className="p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
          <div className="flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-yellow-500 shrink-0 mt-0.5" />
            <div className="flex-1">
              <h3 className="text-sm font-medium text-yellow-400">Security Warning: Default Password in Use</h3>
              <p className="text-sm text-yellow-400/80 mt-1">
                You are currently using the default password "admin". For security, please change your password
                immediately.
              </p>
              <Link
                to="/settings"
                className="inline-flex items-center gap-2 mt-3 text-sm font-medium text-yellow-400 hover:text-yellow-300"
              >
                <Settings className="w-4 h-4" />
                Change Password
              </Link>
            </div>
          </div>
        </div>
      )}

      {protectedNodes > 0 && (
        <div className="p-4 bg-cyan-500/10 border border-cyan-500/20 rounded-lg">
          <div className="flex items-start gap-3">
            <ShieldCheck className="w-5 h-5 text-cyan-400 shrink-0 mt-0.5" />
            <div>
              <h3 className="text-sm font-medium text-cyan-300">Secure Signer Protection Active</h3>
              <p className="text-sm text-cyan-200/80 mt-1">
                {protectedNodes} node{protectedNodes === 1 ? "" : "s"} currently use external secure-signer protection. Review signer readiness from the node cards below.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Stats */}
      {isLoading ? (
        <StatSkeleton />
      ) : (
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          {stats.map((stat, i) => (
            <div key={stat.label} className={`card-interactive animate-fade-in-up stagger-${i + 1} group cursor-default`}>
              <div className="flex items-center justify-between relative z-10">
                <div>
                  <p className="text-slate-400 text-sm font-medium group-hover:text-slate-300 transition-colors">{stat.label}</p>
                  <p className="text-3xl font-bold text-white mt-1 group-hover:text-blue-50 transition-colors">{stat.value}</p>
                </div>
                <div className={`w-12 h-12 rounded-xl ${stat.bgColor} flex items-center justify-center ring-1 ring-inset ring-white/5 group-hover:scale-110 transition-transform duration-300`}>
                  <stat.icon className={`w-6 h-6 ${stat.color}`} />
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* System Metrics */}
      {systemMetrics && (
        <div className="card">
          <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
            <Cpu className="w-5 h-5 text-blue-400" />
            System Resources
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="text-slate-400">CPU Usage</span>
                <span className="text-white">{systemMetrics.cpu.usage.toFixed(1)}%</span>
              </div>
              <ProgressBar value={systemMetrics.cpu.usage} color="bg-blue-500" />
            </div>
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="text-slate-400">Memory</span>
                <span className="text-white">{systemMetrics.memory.percentage.toFixed(1)}%</span>
              </div>
              <ProgressBar value={systemMetrics.memory.percentage} color="bg-emerald-500" />
            </div>
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="text-slate-400">Disk</span>
                <span className="text-white">{systemMetrics.disk.percentage.toFixed(1)}%</span>
              </div>
              <ProgressBar value={systemMetrics.disk.percentage} color="bg-purple-500" />
            </div>
          </div>
        </div>
      )}

      {/* Node List */}
      <div className="card">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Nodes</h2>
          <Link to="/nodes/create" className="btn btn-primary">
            Create Node
          </Link>
        </div>

        {isLoading ? (
          <div className="space-y-3">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="animate-pulse flex items-center justify-between p-4 bg-slate-800/30 backdrop-blur-md rounded-xl border border-slate-700/30">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 rounded-lg bg-slate-700/50"></div>
                  <div className="space-y-2">
                    <div className="h-4 w-32 bg-slate-700/50 rounded"></div>
                    <div className="h-3 w-48 bg-slate-700/50 rounded"></div>
                  </div>
                </div>
                <div className="flex items-center gap-4 hidden sm:flex">
                  <div className="space-y-2 text-right">
                    <div className="h-4 w-24 bg-slate-700/50 rounded ml-auto"></div>
                    <div className="h-3 w-16 bg-slate-700/50 rounded ml-auto"></div>
                  </div>
                  <div className="w-20 h-6 bg-slate-700/50 rounded-full"></div>
                </div>
              </div>
            ))}
          </div>
        ) : nodes.length === 0 ? (
          <EmptyState
            icon={Server}
            title="No nodes yet"
            description="Deploy a new Neo node or import an existing installation"
            actions={[
              { label: "Create Node", href: "/nodes/create", variant: "primary" },
              { label: "Import Existing", href: "/nodes/import", variant: "secondary" },
            ]}
          />
        ) : (
          <div className="space-y-3">
            {nodes.map((node) => (
              <Link
                key={node.id}
                to={`/nodes/${node.id}`}
                className="animate-fade-in flex items-center justify-between p-4 bg-slate-800/30 backdrop-blur-md rounded-xl hover:bg-slate-800/50 transition-all duration-300 hover:shadow-lg border border-slate-700/50 hover:border-blue-500/50"
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                      node.type === "neo-cli" ? "bg-blue-500/10" : "bg-emerald-500/10"
                    }`}
                  >
                    <Activity className={`w-5 h-5 ${node.type === "neo-cli" ? "text-blue-400" : "text-emerald-400"}`} />
                  </div>
                  <div>
                    <h3 className="font-medium text-white">{node.name}</h3>
                    <div className="mt-1 flex flex-wrap items-center gap-2">
                      <p className="text-sm text-slate-400">
                        {node.type} • {node.network} • v{node.version}
                      </p>
                      <NodeProtectionLabel node={node} />
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  {node.metrics && (
                    <div className="text-right hidden sm:block">
                      <p className="text-sm text-slate-400">Block {node.metrics.blockHeight.toLocaleString()}</p>
                      <p className="text-xs text-slate-500">{node.metrics.connectedPeers} peers</p>
                    </div>
                  )}
                  <span className={`status-badge status-${node.process.status}`}>
                    {node.process.status === "running" && (
                      <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                    )}
                    {node.process.status}
                  </span>
                  {node.settings?.keyProtection?.mode === "secure-signer" && (
                    <SignerStatus nodeId={node.id} showPrefix textSize="text-[11px]" />
                  )}
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
