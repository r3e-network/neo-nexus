import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('operations queue presets', () => {
  it('ships timing filters and quick preset links across the queue pages', () => {
    const provisioning = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/provisioning/page.tsx'), 'utf8');
    const alerts = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/alerts/page.tsx'), 'utf8');
    const incidents = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/incidents/page.tsx'), 'utf8');

    expect(provisioning).toContain('name="timing"');
    expect(provisioning).toContain('Due Now');
    expect(provisioning).toContain('Hetzner');

    expect(alerts).toContain('name="timing"');
    expect(alerts).toContain('Webhook');
    expect(alerts).toContain('Snoozed');

    expect(incidents).toContain('Critical');
    expect(incidents).toContain('Snoozed');
  });
});
