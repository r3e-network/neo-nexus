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
    }).catch(() => {
      // Silently fail — Sentry is best-effort
    });
  }

  async testConnection(): Promise<boolean> {
    try {
      // Validate DSN format
      const url = new URL(this.config.dsn);
      return url.protocol === 'https:' && url.pathname.length > 1;
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
