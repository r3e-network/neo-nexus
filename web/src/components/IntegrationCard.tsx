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

export function nextSaveAndTestEnabledState(currentEnabled: boolean): boolean {
  return currentEnabled;
}

export function IntegrationCard({ integration, onSave, onTest, isSaving, isTesting }: IntegrationCardProps) {
  const [formValues, setFormValues] = useState<Record<string, string>>({});
  const [enabled, setEnabled] = useState(integration.enabled);
  const [testResult, setTestResult] = useState<{ success: boolean; error?: string } | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [revealedFields, setRevealedFields] = useState<Set<string>>(new Set());

  useEffect(() => {
    setFormValues(integration.configValues);
    setEnabled(integration.enabled);
  }, [integration]);

  const formatError = (error: unknown): string => {
    if (error instanceof Error) return error.message;
    return typeof error === 'string' ? error : 'Failed to save integration.';
  };

  const handleSaveAndTest = async () => {
    setTestResult(null);
    setSaveError(null);
    const nextEnabled = nextSaveAndTestEnabledState(enabled);
    try {
      await onSave(integration.id, formValues, nextEnabled);
    } catch (error) {
      setSaveError(formatError(error));
      return;
    }
    try {
      const result = await onTest(integration.id);
      setTestResult(result);
    } catch (error) {
      setTestResult({ success: false, error: formatError(error) });
    }
    setEnabled(nextEnabled);
  };

  const handleEnable = async () => {
    setTestResult(null);
    setSaveError(null);
    try {
      await onSave(integration.id, formValues, true);
      setEnabled(true);
    } catch (error) {
      setSaveError(formatError(error));
    }
  };

  const handleDisable = async () => {
    setTestResult(null);
    setSaveError(null);
    try {
      await onSave(integration.id, formValues, false);
      setEnabled(false);
    } catch (error) {
      setSaveError(formatError(error));
    }
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
    ? 'text-amber-700'
    : integration.enabled && integration.configured
      ? 'text-emerald-700'
      : 'text-slate-500';

  const statusLabel = integration.lastError
    ? 'Error'
    : integration.enabled && integration.configured
      ? 'Connected'
      : 'Not configured';

  return (
    <form
      className="card space-y-4"
      onSubmit={(event) => {
        event.preventDefault();
        if (!hasRequiredFields || isSaving || isTesting) {
          return;
        }
        void handleSaveAndTest();
      }}
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-lg font-semibold text-slate-950">{integration.name}</h3>
          <p className="text-sm text-slate-600 mt-1">{integration.description}</p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <span className={`w-2 h-2 rounded-full ${statusColor.replace('text-', 'bg-')}`} />
          <span className={`text-xs font-medium ${statusColor}`}>{statusLabel}</span>
        </div>
      </div>

      {integration.lastError && (
        <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          {integration.lastError}
        </div>
      )}

      <div className="space-y-3">
        {integration.configSchema.map(field => {
          const fieldId = `${integration.id}-${field.key}`;
          return (
          <div key={field.key}>
            <label htmlFor={fieldId} className="block text-sm font-medium text-slate-700 mb-1.5">{field.label}</label>
            <div className="relative">
              <input
                id={fieldId}
                type={(field.type === 'password' || field.sensitive) && !revealedFields.has(field.key) ? 'password' : 'text'}
                name={fieldId}
                className="input w-full pr-10"
                value={formValues[field.key] || ''}
                onChange={e => setFormValues(prev => ({ ...prev, [field.key]: e.target.value }))}
                placeholder={field.placeholder}
                autoComplete={field.type === 'password' || field.sensitive ? 'new-password' : 'off'}
              />
              {(field.type === 'password' || field.sensitive) && (
                <button
                  type="button"
                  onClick={() => toggleReveal(field.key)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-slate-500 hover:text-slate-950"
                  aria-label={revealedFields.has(field.key) ? 'Hide value' : 'Reveal value'}
                >
                  {revealedFields.has(field.key) ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              )}
            </div>
          </div>
          );
        })}
      </div>

      {saveError && (
        <div className="flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
          <XCircle className="mt-0.5 w-4 h-4 shrink-0" />
          <span>{saveError}</span>
        </div>
      )}

      {testResult && (
        <div className={`flex items-center gap-2 text-sm ${testResult.success ? 'text-emerald-700' : 'text-red-700'}`}>
          {testResult.success ? <CheckCircle2 className="w-4 h-4" /> : <XCircle className="w-4 h-4" />}
          {testResult.success ? 'Connection successful' : testResult.error || 'Connection failed'}
        </div>
      )}

      <div className="flex flex-col gap-3 pt-1 sm:flex-row sm:flex-wrap">
        <button
          type="submit"
          className="btn btn-primary justify-center sm:min-w-40 sm:flex-1"
          disabled={isSaving || isTesting || !hasRequiredFields}
        >
          {(isSaving || isTesting) && <Loader2 className="w-4 h-4 animate-spin" />}
          Save & Test
        </button>
        {enabled && (
          <button
            type="button"
            className="btn btn-secondary justify-center sm:w-auto"
            disabled={isSaving}
            onClick={handleDisable}
          >
            Disable
          </button>
        )}
        {!enabled && hasRequiredFields && (
          <button
            type="button"
            className="btn btn-secondary justify-center sm:w-auto"
            disabled={isSaving}
            onClick={handleEnable}
          >
            Enable
          </button>
        )}
      </div>

      {integration.lastTestAt && (
        <p className="text-xs text-slate-500">
          Last tested: {new Date(integration.lastTestAt).toLocaleString()}
        </p>
      )}
    </form>
  );
}
