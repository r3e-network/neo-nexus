import { useState, useEffect } from 'react';
import { CheckCircle2, XCircle, Loader2, Eye, EyeOff } from 'lucide-react';
import type { IntegrationStatus } from '../hooks/useIntegrations';

interface IntegrationCardProps {
  integration: IntegrationStatus;
  onSave: (id: string, config: Record<string, string>, enabled: boolean) => Promise<void>;
  onTest: (id: string) => Promise<{ success: boolean; error?: string }>;
  isSaving: boolean;
  isTesting: boolean;
}

export function IntegrationCard({ integration, onSave, onTest, isSaving, isTesting }: IntegrationCardProps) {
  const [formValues, setFormValues] = useState<Record<string, string>>({});
  const [enabled, setEnabled] = useState(integration.enabled);
  const [testResult, setTestResult] = useState<{ success: boolean; error?: string } | null>(null);
  const [revealedFields, setRevealedFields] = useState<Set<string>>(new Set());

  useEffect(() => {
    setFormValues(integration.configValues);
    setEnabled(integration.enabled);
  }, [integration]);

  const handleSaveAndTest = async () => {
    setTestResult(null);
    await onSave(integration.id, formValues, true);
    const result = await onTest(integration.id);
    setTestResult(result);
    if (result.success) {
      setEnabled(true);
    }
  };

  const handleDisable = async () => {
    setTestResult(null);
    await onSave(integration.id, formValues, false);
    setEnabled(false);
  };

  const toggleReveal = (key: string) => {
    setRevealedFields(prev => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  const hasRequiredFields = integration.configSchema
    .filter(f => f.required)
    .every(f => !!formValues[f.key]?.trim());

  const statusColor = integration.lastError
    ? 'text-amber-400'
    : integration.enabled && integration.configured
      ? 'text-emerald-400'
      : 'text-slate-500';

  const statusLabel = integration.lastError
    ? 'Error'
    : integration.enabled && integration.configured
      ? 'Connected'
      : 'Not configured';

  return (
    <div className="card space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-lg font-semibold text-white">{integration.name}</h3>
          <p className="text-sm text-slate-400 mt-1">{integration.description}</p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <span className={`w-2 h-2 rounded-full ${statusColor.replace('text-', 'bg-')}`} />
          <span className={`text-xs font-medium ${statusColor}`}>{statusLabel}</span>
        </div>
      </div>

      {integration.lastError && (
        <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-2 text-sm text-amber-300">
          {integration.lastError}
        </div>
      )}

      <div className="space-y-3">
        {integration.configSchema.map(field => (
          <div key={field.key}>
            <label className="block text-sm font-medium text-slate-300 mb-1.5">{field.label}</label>
            <div className="relative">
              <input
                type={(field.type === 'password' || field.sensitive) && !revealedFields.has(field.key) ? 'password' : 'text'}
                className="input w-full pr-10"
                value={formValues[field.key] || ''}
                onChange={e => setFormValues(prev => ({ ...prev, [field.key]: e.target.value }))}
                placeholder={field.placeholder}
              />
              {(field.type === 'password' || field.sensitive) && (
                <button
                  type="button"
                  onClick={() => toggleReveal(field.key)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-slate-500 hover:text-slate-300"
                  aria-label={revealedFields.has(field.key) ? 'Hide value' : 'Reveal value'}
                >
                  {revealedFields.has(field.key) ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      {testResult && (
        <div className={`flex items-center gap-2 text-sm ${testResult.success ? 'text-emerald-400' : 'text-red-400'}`}>
          {testResult.success ? <CheckCircle2 className="w-4 h-4" /> : <XCircle className="w-4 h-4" />}
          {testResult.success ? 'Connection successful' : testResult.error || 'Connection failed'}
        </div>
      )}

      <div className="flex gap-3 pt-1">
        <button
          type="button"
          className="btn btn-primary flex-1 justify-center"
          disabled={isSaving || isTesting || !hasRequiredFields}
          onClick={handleSaveAndTest}
        >
          {(isSaving || isTesting) && <Loader2 className="w-4 h-4 animate-spin" />}
          Save & Test
        </button>
        {enabled && (
          <button
            type="button"
            className="btn btn-secondary"
            disabled={isSaving}
            onClick={handleDisable}
          >
            Disable
          </button>
        )}
      </div>

      {integration.lastTestAt && (
        <p className="text-xs text-slate-500">
          Last tested: {new Date(integration.lastTestAt).toLocaleString()}
        </p>
      )}
    </div>
  );
}
