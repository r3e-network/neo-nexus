import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('operations provisioning actions', () => {
  it('ships a batch retry control in the provisioning queue view', () => {
    const provisioningPage = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/provisioning/page.tsx'), 'utf8');

    expect(provisioningPage).toContain('RetryProvisioningOrdersButton');
  });
});
