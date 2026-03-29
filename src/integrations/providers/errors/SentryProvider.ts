// src/integrations/providers/errors/SentryProvider.ts
import type { ErrorProvider, ConfigField } from '../../types';

export const sentrySchema: ConfigField[] = [
  { key: 'dsn', label: 'DSN', type: 'password', placeholder: 'https://examplePublicKey@o0.ingest.sentry.io/0', required: true },
];

export class SentryProvider implements ErrorProvider {
  readonly name = 'Sentry';
  private initialized = false;

  constructor(private config: { dsn: string }) {}

  private async ensureInit(): Promise<typeof import('@sentry/node')> {
    const Sentry = await import('@sentry/node');
    if (!this.initialized) {
      Sentry.init({
        dsn: this.config.dsn,
        tracesSampleRate: 0,
        defaultIntegrations: false,
      });
      this.initialized = true;
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

      const response = await fetch(storeUrl, {
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
    if (this.initialized) {
      import('@sentry/node').then(Sentry => {
        Sentry.close(2000);
      }).catch(() => {});
      this.initialized = false;
    }
  }
}
