import { ArrowRight, CheckCircle2, Database, FolderOpen, Network, Plus, ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";

const firstActions = [
  {
    title: "Create native node",
    description: "Best first step for a managed mainnet or testnet node with role presets.",
    href: "/nodes/create",
    icon: Plus,
    primary: true,
  },
  {
    title: "Plan private network",
    description: "Generate a single, 4-node, or 7-node N3 network with addresses and ports.",
    href: "/private-networks",
    icon: Network,
    primary: false,
  },
  {
    title: "Import observe-only",
    description: "Attach an existing node without changing its files until you adopt management.",
    href: "/nodes/import",
    icon: FolderOpen,
    primary: false,
  },
];

const readinessItems = [
  "Choose a role preset for consensus, state, oracle, or custom operations.",
  "Pick storage and sync before first start so data remains isolated and predictable.",
  "Confirm block height sync status before using node data for operations.",
];

export function FirstRunGuide() {
  return (
    <div className="space-y-5">
      <div className="rounded-lg border border-teal-200 bg-teal-50 p-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="max-w-2xl">
            <p className="console-kicker text-teal-800">First run</p>
            <h3 className="mt-2 text-lg font-semibold text-teal-950">Start with a safe node</h3>
            <p className="mt-2 text-sm leading-6 text-teal-900">
              NeoNexus is ready. Create a managed node, plan an isolated private network, or import an existing node in observe-only mode.
            </p>
          </div>
          <Link to="/nodes/create" className="btn btn-primary shrink-0 justify-center">
            Create native node <ArrowRight className="h-4 w-4" />
          </Link>
        </div>
      </div>

      <div className="grid gap-3 lg:grid-cols-3">
        {firstActions.map((action) => (
          <Link
            key={action.title}
            to={action.href}
            className={`group rounded-lg border p-4 transition-colors ${
              action.primary
                ? "border-blue-200 bg-blue-50 hover:bg-blue-100/70"
                : "border-slate-200 bg-white hover:border-teal-200 hover:bg-teal-50/50"
            }`}
          >
            <div className="flex items-start gap-3">
              <span className="rounded-lg border border-white/70 bg-white p-2 shadow-sm">
                <action.icon className={`h-5 w-5 ${action.primary ? "text-blue-700" : "text-slate-700"}`} />
              </span>
              <div>
                <p className="font-medium text-slate-950">{action.title}</p>
                <p className="mt-1 text-sm leading-5 text-slate-600">{action.description}</p>
              </div>
            </div>
          </Link>
        ))}
      </div>

      <div className="grid gap-4 rounded-lg border border-slate-200 bg-slate-50 p-4 lg:grid-cols-[minmax(0,1fr)_240px]">
        <div>
          <div className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-teal-700" />
            <h4 className="font-semibold text-slate-950">Before the first start</h4>
          </div>
          <ul className="mt-3 space-y-2 text-sm leading-6 text-slate-600">
            {readinessItems.map((item) => (
              <li key={item} className="flex items-start gap-2">
                <CheckCircle2 className="mt-1 h-4 w-4 shrink-0 text-emerald-600" />
                <span>{item}</span>
              </li>
            ))}
          </ul>
        </div>
        <div className="rounded-lg border border-slate-200 bg-white p-3">
          <div className="flex items-center gap-2">
            <Database className="h-4 w-4 text-blue-700" />
            <p className="text-sm font-semibold text-slate-950">Good default</p>
          </div>
          <p className="mt-2 text-xs leading-5 text-slate-600">
            Start with Neo CLI on testnet, LevelDB storage, and a role preset. Switch to RocksDB or private-network data contexts when the workload needs it.
          </p>
        </div>
      </div>
    </div>
  );
}
