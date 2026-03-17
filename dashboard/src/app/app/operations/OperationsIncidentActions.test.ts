import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('operations incident actions', () => {
  it('ships quick resolve and snooze controls in the incident queue view', () => {
    const incidentsPage = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/incidents/page.tsx'), 'utf8');

    expect(incidentsPage).toContain('ResolveIncidentButton');
    expect(incidentsPage).toContain('SnoozeAlertRuleButton');
    expect(incidentsPage).toContain('snoozedUntil');
  });
});
