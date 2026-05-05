// src/integrations/providers/errors/SentryProvider.ts
import type { ErrorProvider, ConfigField } from '../../types';
import { safeIntegrationFetch, validateLiteralIntegrationUrl } from '../../safeFetch';

// Note on outbound target protection:
// `testConnection` below uses `safeIntegrationFetch` and is therefore covered
// by the same DNS-rebind / private-target guards as other providers. The actual
// error reporting transport is owned by `@sentry/node` once `Sentry.init` runs,
// so constructor/config validation blocks literal private DSNs while DNS-level
// pinning is only available during explicit connection tests.
export const sentrySchema: ConfigField[] = [
  { key: 'dsn', label: 'DSN', type: 'url', placeholder: 'https://examplePublicKey@o0.ingest.sentry.io/0', required: true, sensitive: true },
];

// Module-level tracking prevents double Sentry.init() across provider reloads
let activeDsn: string | null = null;

export class SentryProvider implements ErrorProvider {
  readonly name = 'Sentry';

  constructor(private config: { dsn: string }) {
    validateLiteralIntegrationUrl(config.dsn);
    const url = new URL(config.dsn);
    if (url.protocol !== 'https:' || !url.username || url.pathname.length <= 1) {
      throw new Error('Sentry DSN must be an HTTPS DSN with a public key and project id');
    }
  }

  private async ensureInit(): Promise<typeof import('@sentry/node')> {
    const Sentry = await import('@sentry/node');
    if (activeDsn !== this.config.dsn) {
      if (activeDsn !== null) {
        await Sentry.close(2000);
      }
      Sentry.init({
        dsn: this.config.dsn,
        tracesSampleRate: 0,
        defaultIntegrations: false,
      });
      activeDsn = this.config.dsn;
    }
    return Sentry;
  }

  captureError(error: Error, context?: Record<string, unknown>): void {
    this.ensureInit().then(Sentry => {
      if (context) {
        Sentry.withScope(scope => {
          for (const [key, value] of Object.entries(context)) {
            scope.setExtra(key, value);
          }
          Sentry.captureException(error);
        });
      } else {
        Sentry.captureException(error);
      }
    }).catch(initError => {
      console.error('[integrations] Sentry initialization failed:', initError instanceof Error ? initError.message : initError);
    });
  }

  async testConnection(): Promise<boolean> {
    try {
      // Validate DSN format
      const url = new URL(this.config.dsn);
      if (url.protocol !== 'https:' || url.pathname.length <= 1) return false;

      // Attempt to reach the Sentry ingest endpoint
      const projectId = url.pathname.replace('/', '');
      const host = url.hostname;
      const publicKey = url.username;
      const storeUrl = `https://${host}/api/${projectId}/envelope/?sentry_key=${publicKey}&sentry_version=7`;

      const response = await safeIntegrationFetch(storeUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-sentry-envelope' },
        body: `{"dsn":"${this.config.dsn}"}\n{"type":"check_in"}\n{"status":"ok"}`,
        signal: AbortSignal.timeout(10_000),
      });
      // Sentry returns 200 on success or 400 for invalid payload, but both prove connectivity
      return response.status === 200 || response.status === 400;
    } catch {
      return false;
    }
  }

  shutdown(): void {
    if (activeDsn === this.config.dsn) {
      import('@sentry/node').then(Sentry => {
        Sentry.close(2000);
      }).catch(() => {});
      activeDsn = null;
    }
  }
}
