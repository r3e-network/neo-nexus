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
      <section className="page-hero pb-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="console-kicker">External systems</p>
          <h1 className="mt-2 text-3xl font-semibold text-slate-950">Integrations</h1>
          <p className="text-slate-600 mt-2 max-w-3xl text-sm leading-6">Connect NeoNexus to monitoring, logging, uptime, alerting, and error services.</p>
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
      </section>

      {/* Category filters */}
      <div className="grid grid-cols-2 gap-2 sm:flex sm:flex-wrap" role="group" aria-label="Integration categories">
        {CATEGORIES.map((cat) => (
          <button
            key={cat.key}
            type="button"
            aria-pressed={activeCategory === cat.key}
            onClick={() => setActiveCategory(cat.key)}
            className={`inline-flex justify-center px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors ${
              activeCategory === cat.key
                ? 'border border-teal-200 bg-teal-50 text-teal-950'
                : 'border border-transparent text-slate-600 hover:text-slate-950 hover:bg-slate-100'
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
        <div
          className="space-y-8"
        >
          {grouped.map(group => (
            <div key={group.key}>
              {activeCategory === 'all' && (
                <h2 className="text-lg font-semibold text-slate-950 mb-4 capitalize">{group.label}</h2>
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
