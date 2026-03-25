import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { FolderOpen, Search, CheckCircle, AlertCircle, Loader2, Server } from 'lucide-react';
import { api } from '../utils/api';

interface DetectedConfig {
  type: 'neo-cli' | 'neo-go';
  network: 'mainnet' | 'testnet' | 'private';
  version: string;
  ports: {
    rpc?: number;
    p2p?: number;
  };
  dataPath: string;
  configPath: string;
  isRunning: boolean;
}

export default function ImportNode() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [path, setPath] = useState('');
  const [name, setName] = useState('');
  const [pid, setPid] = useState('');
  const [detected, setDetected] = useState<DetectedConfig | null>(null);
  const [scanResults, setScanResults] = useState<Array<{ path: string; type: string }> | null>(null);
  const [error, setError] = useState('');

  const detectMutation = useMutation({
    mutationFn: async (detectPath: string) => {
      const response = await api.post('/nodes/detect', { path: detectPath });
      return response.data;
    },
    onSuccess: (data) => {
      if (data.detected) {
        setDetected(data.detected);
        setError('');
      }
    },
    onError: (err: any) => {
      setError(err.response?.data?.error || 'Failed to detect node configuration');
      setDetected(null);
    },
  });

  const scanMutation = useMutation({
    mutationFn: async (scanPath: string) => {
      const response = await api.post('/nodes/scan', { path: scanPath });
      return response.data;
    },
    onSuccess: (data) => {
      setScanResults(data.nodes);
      setError('');
    },
    onError: (err: any) => {
      setError(err.response?.data?.error || 'Failed to scan directory');
      setScanResults(null);
    },
  });

  const importMutation = useMutation({
    mutationFn: async () => {
      if (!detected || !name || !path) return;
      
      const response = await api.post('/nodes/import', {
        name,
        type: detected.type,
        existingPath: path,
        pid: pid ? parseInt(pid, 10) : undefined,
        network: detected.network,
        version: detected.version,
        ports: detected.ports,
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['nodes'] });
      navigate('/nodes');
    },
    onError: (err: any) => {
      setError(err.response?.data?.error || 'Failed to import node');
    },
  });

  const handleDetect = () => {
    if (!path.trim()) {
      setError('Please enter a path');
      return;
    }
    detectMutation.mutate(path);
  };

  const handleScan = () => {
    if (!path.trim()) {
      setError('Please enter a path to scan');
      return;
    }
    scanMutation.mutate(path);
  };

  const handleSelectScanResult = (resultPath: string) => {
    setPath(resultPath);
    detectMutation.mutate(resultPath);
    setScanResults(null);
  };

  const handleImport = () => {
    if (!name.trim()) {
      setError('Please enter a name for the node');
      return;
    }
    importMutation.mutate();
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white">Import Existing Node</h1>
        <p className="text-slate-400 mt-2">
          Import an existing neo-cli or neo-go installation into NeoNexus for management.
        </p>
      </div>

      {error && (
        <div className="mb-6 bg-red-500/10 border border-red-500/50 rounded-lg p-4 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 text-red-400" />
          <span className="text-red-400">{error}</span>
        </div>
      )}

      <div className="space-y-6">
        {/* Path Input */}
        <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
          <label className="block text-sm font-medium text-slate-300 mb-2">
            Node Directory Path
          </label>
          <div className="flex gap-3">
            <div className="flex-1 relative">
              <FolderOpen className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
              <input
                type="text"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="/path/to/neo-node (e.g., /home/user/neo-cli)"
                className="w-full pl-10 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
              />
            </div>
            <button
              onClick={handleDetect}
              disabled={detectMutation.isPending}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-700 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center gap-2 transition-colors"
            >
              {detectMutation.isPending ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Search className="w-4 h-4" />
              )}
              Detect
            </button>
          </div>
          <p className="text-slate-500 text-sm mt-2">
            Enter the full path to the node directory containing the executable and config files.
          </p>
        </div>

        {/* Scan Directory Option */}
        <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
          <label className="block text-sm font-medium text-slate-300 mb-2">
            Or Scan Directory for Nodes
          </label>
          <div className="flex gap-3">
            <button
              onClick={handleScan}
              disabled={scanMutation.isPending}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 disabled:bg-slate-800 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center gap-2 transition-colors"
            >
              {scanMutation.isPending ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Search className="w-4 h-4" />
              )}
              Scan Parent Directory
            </button>
          </div>

          {/* Scan Results */}
          {scanResults && scanResults.length > 0 && (
            <div className="mt-4 space-y-2">
              <p className="text-sm text-slate-400">Found {scanResults.length} node installations:</p>
              {scanResults.map((result, index) => (
                <button
                  key={index}
                  onClick={() => handleSelectScanResult(result.path)}
                  className="w-full text-left p-3 bg-slate-800 hover:bg-slate-700 rounded-lg border border-slate-700 transition-colors"
                >
                  <div className="flex items-center gap-3">
                    <Server className="w-5 h-5 text-blue-400" />
                    <div>
                      <p className="text-white font-medium">{result.type || 'Unknown'}</p>
                      <p className="text-slate-400 text-sm">{result.path}</p>
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}

          {scanResults && scanResults.length === 0 && (
            <p className="mt-4 text-amber-400 text-sm">No node installations found in this directory.</p>
          )}
        </div>

        {/* Detected Configuration */}
        {detected && (
          <div className="bg-slate-900 rounded-xl p-6 border border-slate-800">
            <div className="flex items-center gap-3 mb-4">
              <CheckCircle className="w-6 h-6 text-emerald-400" />
              <h2 className="text-xl font-semibold text-white">Configuration Detected</h2>
            </div>

            <div className="grid grid-cols-2 gap-4 mb-6">
              <div className="bg-slate-800 rounded-lg p-4">
                <p className="text-slate-400 text-sm">Node Type</p>
                <p className="text-white font-medium capitalize">{detected.type}</p>
              </div>
              <div className="bg-slate-800 rounded-lg p-4">
                <p className="text-slate-400 text-sm">Network</p>
                <p className="text-white font-medium capitalize">{detected.network}</p>
              </div>
              <div className="bg-slate-800 rounded-lg p-4">
                <p className="text-slate-400 text-sm">Version</p>
                <p className="text-white font-medium">{detected.version}</p>
              </div>
              <div className="bg-slate-800 rounded-lg p-4">
                <p className="text-slate-400 text-sm">Status</p>
                <p className={`font-medium ${detected.isRunning ? 'text-emerald-400' : 'text-slate-400'}`}>
                  {detected.isRunning ? 'Running' : 'Stopped'}
                </p>
              </div>
              <div className="bg-slate-800 rounded-lg p-4">
                <p className="text-slate-400 text-sm">RPC Port</p>
                <p className="text-white font-medium">{detected.ports.rpc || 'Auto'}</p>
              </div>
              <div className="bg-slate-800 rounded-lg p-4">
                <p className="text-slate-400 text-sm">P2P Port</p>
                <p className="text-white font-medium">{detected.ports.p2p || 'Auto'}</p>
              </div>
            </div>

            {/* Import Configuration */}
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Node Name *
                </label>
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="My Neo Node"
                  className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Process ID (PID) - Optional
                </label>
                <input
                  type="text"
                  value={pid}
                  onChange={(e) => setPid(e.target.value)}
                  placeholder={detected.isRunning ? 'Auto-detected' : 'e.g., 12345'}
                  className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
                />
                <p className="text-slate-500 text-sm mt-1">
                  If the node is running, you can specify its PID to attach to the existing process.
                </p>
              </div>
            </div>

            {/* Import Button */}
            <button
              onClick={handleImport}
              disabled={importMutation.isPending || !name.trim()}
              className="w-full mt-6 px-4 py-3 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-700 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center justify-center gap-2 transition-colors"
            >
              {importMutation.isPending ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Importing...
                </>
              ) : (
                <>
                  <CheckCircle className="w-4 h-4" />
                  Import Node
                </>
              )}
            </button>
          </div>
        )}

        {/* Help Text */}
        <div className="bg-slate-800/50 rounded-lg p-4 border border-slate-700">
          <h3 className="text-sm font-medium text-slate-300 mb-2">What gets imported?</h3>
          <ul className="text-sm text-slate-400 space-y-1 list-disc list-inside">
            <li>Node type (neo-cli or neo-go)</li>
            <li>Network configuration (mainnet/testnet/private)</li>
            <li>Port settings (RPC, P2P)</li>
            <li>Data directory location</li>
            <li>Existing blockchain data (no re-sync needed)</li>
            <li>Running process attachment (optional)</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
