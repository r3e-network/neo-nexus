import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('dashboard client UX behavior', () => {
  it('avoids browser alerts and full-page reloads in billing and security clients', () => {
    const files = [
      path.join(process.cwd(), 'src/app/app/security/SecurityClient.tsx'),
      path.join(process.cwd(), 'src/app/app/billing/BillingClient.tsx'),
      path.join(process.cwd(), 'src/app/app/endpoints/[id]/EndpointDetailsClient.tsx'),
    ];

    for (const file of files) {
      const contents = fs.readFileSync(file, 'utf8');
      expect(contents).not.toContain('alert(');
      expect(contents).not.toContain('window.location.reload(');
      expect(contents).not.toContain('window.confirm(');
    }
  });

  it('does not ship dead Stripe billing buttons', () => {
    const file = path.join(process.cwd(), 'src/app/app/billing/BillingClient.tsx');
    const contents = fs.readFileSync(file, 'utf8');

    expect(contents).not.toContain('cursor-not-allowed opacity-50');
    expect(contents).not.toContain('Cancel Subscription');
  });
});
