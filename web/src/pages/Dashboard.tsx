import { 
  Activity, 
  Server, 
  Cpu, 
   
  Play, 
  
  AlertCircle,
  CheckCircle,
  AlertTriangle,
  Settings
} from 'lucide-react';
import { useNodes, useSystemMetrics } from '../hooks/useNodes';
import { useAuth } from '../hooks/useAuth';
import { Link } from 'react-router-dom';

export default function Dashboard() {
  const { data: nodes = [] } = useNodes();
  const { data: systemMetrics } = useSystemMetrics();
  const { user } = useAuth();

  // Check if using default credentials
  const isDefaultPassword = user?.username === 'admin';

  const runningNodes = nodes.filter(n => n.process.status === 'running');
  const errorNodes = nodes.filter(n => n.process.status === 'error');
  const totalBlocks = nodes.reduce((sum, n) => sum + (n.metrics?.blockHeight || 0), 0);

  const stats = [
    { 
      label: 'Total Nodes', 
      value: nodes.length, 
      icon: Server,
      color: 'text-blue-400',
      bgColor: 'bg-blue-500/10'
    },
    { 
      label: 'Running', 
      value: runningNodes.length, 
      icon: Play,
      color: 'text-emerald-400',
      bgColor: 'bg-emerald-500/10'
    },
    { 
      label: 'Errors', 
      value: errorNodes.length, 
      icon: AlertCircle,
      color: errorNodes.length > 0 ? 'text-red-400' : 'text-slate-400',
      bgColor: errorNodes.length > 0 ? 'bg-red-500/10' : 'bg-slate-500/10'
    },
    { 
      label: 'Blocks Synced', 
      value: totalBlocks.toLocaleString(), 
      icon: CheckCircle,
      color: 'text-purple-400',
      bgColor: 'bg-purple-500/10'
    },
  ];

  return (
    <div className="space-y-6">
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
                You are currently using the default password "admin". For security, please change your password immediately.
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

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {stats.map((stat) => (
          <div key={stat.label} className="card">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-slate-400 text-sm">{stat.label}</p>
                <p className="text-2xl font-bold text-white mt-1">{stat.value}</p>
              </div>
              <div className={`w-12 h-12 rounded-lg ${stat.bgColor} flex items-center justify-center`}>
                <stat.icon className={`w-6 h-6 ${stat.color}`} />
              </div>
            </div>
          </div>
        ))}
      </div>

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
              <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-blue-500 transition-all duration-500"
                  style={{ width: `${systemMetrics.cpu.usage}%` }}
                />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="text-slate-400">Memory</span>
                <span className="text-white">{systemMetrics.memory.percentage.toFixed(1)}%</span>
              </div>
              <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-emerald-500 transition-all duration-500"
                  style={{ width: `${systemMetrics.memory.percentage}%` }}
                />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span className="text-slate-400">Disk</span>
                <span className="text-white">{systemMetrics.disk.percentage.toFixed(1)}%</span>
              </div>
              <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-purple-500 transition-all duration-500"
                  style={{ width: `${systemMetrics.disk.percentage}%` }}
                />
              </div>
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

        {nodes.length === 0 ? (
          <div className="text-center py-12">
            <Server className="w-12 h-12 text-slate-600 mx-auto mb-4" />
            <p className="text-slate-400">No nodes yet</p>
            <Link to="/nodes/create" className="btn btn-primary mt-4 inline-flex">
              Create your first node
            </Link>
          </div>
        ) : (
          <div className="space-y-3">
            {nodes.map((node) => (
              <Link
                key={node.id}
                to={`/nodes/${node.id}`}
                className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg hover:bg-slate-800 transition-colors"
              >
                <div className="flex items-center gap-4">
                  <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                    node.type === 'neo-cli' ? 'bg-blue-500/10' : 'bg-emerald-500/10'
                  }`}>
                    <Activity className={`w-5 h-5 ${
                      node.type === 'neo-cli' ? 'text-blue-400' : 'text-emerald-400'
                    }`} />
                  </div>
                  <div>
                    <h3 className="font-medium text-white">{node.name}</h3>
                    <p className="text-sm text-slate-400">
                      {node.type} • {node.network} • v{node.version}
                    </p>
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
                    {node.process.status === 'running' && <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />}
                    {node.process.status}
                  </span>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
