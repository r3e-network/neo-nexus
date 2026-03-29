import { useState } from 'react';
import { Plug, RefreshCw } from 'lucide-react';
import { CardSkeleton } from '../components/LoadingSkeleton';
import { EmptyState } from '../components/EmptyState';
import { IntegrationCard } from '../components/IntegrationCard';
import {
  useIntegrations,
  useSaveIntegration,
  useTestIntegration,
  type IntegrationCategory,
} from '../hooks/useIntegrations';

const CATEGORIES: { key: IntegrationCategory | 'all'; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'metrics', label: 'Metrics' },
  { key: 'logging', label: 'Logging' },
  { key: 'uptime', label: 'Uptime' },
  { key: 'alerting', label: 'Alerting' },
  { key: 'errors', label: 'Errors' },
];

export default function Integrations() {
  const { data: integrations = [], isLoading, refetch, isFetching } = useIntegrations();
  const saveIntegration = useSaveIntegration();
  const testIntegration = useTestIntegration();
  const [activeCategory, setActiveCategory] = useState<IntegrationCategory | 'all'>('all');
  const [activeId, setActiveId] = useState<string | null>(null);

  const filtered = activeCategory === 'all'
    ? integrations
    : integrations.filter(i => i.category === activeCategory);

  // Group by category when showing "all"
  const grouped = activeCategory === 'all'
    ? CATEGORIES.filter(c => c.key !== 'all').map(c => ({
        ...c,
        items: integrations.filter(i => i.category === c.key),
      })).filter(g => g.items.length > 0)
    : [{ key: activeCategory, label: CATEGORIES.find(c => c.key === activeCategory)?.label || '', items: filtered }];

  const handleSave = async (id: string, config: Record<string, string>, enabled: boolean) => {
    setActiveId(id);
    await saveIntegration.mutateAsync({ id, config, enabled });
  };

  const handleTest = async (id: string) => {
    setActiveId(id);
    return testIntegration.mutateAsync(id);
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-white">Integrations</h1>
          <p className="text-slate-400 mt-1">Connect NeoNexus to external monitoring, logging, and notification services.</p>
        </div>
        <button
          className="btn btn-secondary"
          onClick={() => refetch()}
          type="button"
        >
          <RefreshCw className={`w-4 h-4 ${isFetching ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>

      {/* Category tabs */}
      <div className="flex gap-1 overflow-x-auto pb-1">
        {CATEGORIES.map(cat => (
          <button
            key={cat.key}
            type="button"
            onClick={() => setActiveCategory(cat.key)}
            className={`px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors ${
              activeCategory === cat.key
                ? 'bg-blue-500/10 text-blue-400'
                : 'text-slate-400 hover:text-white hover:bg-slate-800'
            }`}
          >
            {cat.label}
          </button>
        ))}
      </div>

      {/* Content */}
      {isLoading ? (
        <CardSkeleton count={4} />
      ) : integrations.length === 0 ? (
        <EmptyState
          icon={Plug}
          title="No integrations available"
          description="Integration providers are not loaded."
        />
      ) : (
        <div className="space-y-8">
          {grouped.map(group => (
            <div key={group.key}>
              {activeCategory === 'all' && (
                <h2 className="text-lg font-semibold text-white mb-4 capitalize">{group.label}</h2>
              )}
              <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
                {group.items.map(integration => (
                  <IntegrationCard
                    key={integration.id}
                    integration={integration}
                    onSave={handleSave}
                    onTest={handleTest}
                    isSaving={saveIntegration.isPending && activeId === integration.id}
                    isTesting={testIntegration.isPending && activeId === integration.id}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
