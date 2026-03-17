import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('operations alert queue actions', () => {
  it('ships snooze state and quick actions in the alert queue views', () => {
    const alertsPage = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/alerts/page.tsx'), 'utf8');

    expect(alertsPage).toContain('SnoozeAlertRuleButton');
    expect(alertsPage).toContain('RecoverAlertRuleButton');
    expect(alertsPage).toContain('snoozedUntil');
  });
});
