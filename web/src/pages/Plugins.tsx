import { Puzzle, Info, Check } from 'lucide-react';

const PLUGINS = [
  {
    id: 'RpcServer',
    name: 'RPC Server',
    description: 'Provides the standard JSON-RPC interface for the Neo node.',
    category: 'API',
    installed: true,
  },
  {
    id: 'RestServer',
    name: 'REST Server',
    description: 'Provides a RESTful API interface for interacting with the Neo node.',
    category: 'API',
    installed: false,
  },
  {
    id: 'ApplicationLogs',
    name: 'Application Logs',
    description: 'Synchronizes smart contract execution logs into node storage.',
    category: 'Core',
    installed: false,
  },
  {
    id: 'DBFTPlugin',
    name: 'dBFT Consensus',
    description: 'Provides the dBFT consensus algorithm for consensus nodes.',
    category: 'Core',
    installed: false,
  },
  {
    id: 'LevelDBStore',
    name: 'LevelDB Store',
    description: 'Uses LevelDB for the underlying node storage engine.',
    category: 'Storage',
    installed: true,
  },
  {
    id: 'RocksDBStore',
    name: 'RocksDB Store',
    description: 'Uses RocksDB for the underlying node storage engine.',
    category: 'Storage',
    installed: false,
  },
  {
    id: 'OracleService',
    name: 'Oracle Service',
    description: 'Enables the node to participate in the native Oracle network.',
    category: 'Core',
    installed: false,
  },
  {
    id: 'StateService',
    name: 'State Service',
    description: 'Provides MPT state root tracking and validation.',
    category: 'Core',
    installed: false,
  },
  {
    id: 'TokensTracker',
    name: 'Tokens Tracker',
    description: 'Tracks NEP-11 and NEP-17 token transfers and balances.',
    category: 'API',
    installed: false,
  },
  {
    id: 'SQLiteWallet',
    name: 'SQLite Wallet',
    description: 'Allows Neo-CLI to open and manage SQLite based NEP-6 wallets.',
    category: 'Tooling',
    installed: false,
  },
  {
    id: 'StorageDumper',
    name: 'Storage Dumper',
    description: 'Provides tools for dumping and migrating Neo node storage states.',
    category: 'Tooling',
    installed: false,
  },
  {
    id: 'SignClient',
    name: 'Sign Client',
    description: 'Allows node to securely communicate with a remote multi-sig wallet.',
    category: 'Tooling',
    installed: false,
  },
];

const CATEGORIES: Record<string, string> = {
  Core: 'bg-blue-500/10 text-blue-400',
  Storage: 'bg-emerald-500/10 text-emerald-400',
  API: 'bg-purple-500/10 text-purple-400',
  Tooling: 'bg-orange-500/10 text-orange-400',
};

export default function Plugins() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Plugins</h1>
        <p className="text-slate-400 mt-1">
          Manage plugins for your neo-cli nodes. Plugins extend the functionality of your Neo nodes.
        </p>
      </div>

      <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <Info className="w-5 h-5 text-blue-400 shrink-0 mt-0.5" />
          <div>
            <p className="text-blue-300 text-sm">
              Plugins are only available for neo-cli nodes. Neo-go has all features built-in.
              Manage plugins from the individual node page.
            </p>
          </div>
        </div>
      </div>

      <div className="card">
        <h2 className="text-lg font-semibold text-white mb-4">Available Plugins</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {PLUGINS.map((plugin) => (
            <div 
              key={plugin.id}
              className="flex items-start gap-4 p-4 bg-slate-800/50 rounded-lg border border-slate-700/50"
            >
              <div className="w-10 h-10 rounded-lg bg-slate-700 flex items-center justify-center shrink-0">
                <Puzzle className="w-5 h-5 text-slate-400" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <h3 className="font-medium text-white">{plugin.name}</h3>
                  <span className={`text-xs px-2 py-0.5 rounded-full ${CATEGORIES[plugin.category]}`}>
                    {plugin.category}
                  </span>
                </div>
                <p className="text-sm text-slate-400 mt-1">{plugin.description}</p>
              </div>
              {plugin.installed && (
                <div className="flex items-center gap-1 text-emerald-400 text-sm">
                  <Check className="w-4 h-4" />
                  <span>Default</span>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
