import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('operations drilldown actions', () => {
  it('ships actionable retry controls in provisioning and alert drilldowns', () => {
    const provisioning = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/provisioning/page.tsx'), 'utf8');
    const alerts = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/alerts/page.tsx'), 'utf8');

    expect(provisioning).toContain('RetryProvisioningQueueButton');
    expect(alerts).toContain('RetryAlertQueueButton');
  });
});
