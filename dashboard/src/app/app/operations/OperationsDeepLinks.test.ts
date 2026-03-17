import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('operations deep links', () => {
  it('links summary and run detail surfaces into the dedicated drilldown pages', () => {
    const client = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/OperationsClient.tsx'), 'utf8');
    const runDetail = fs.readFileSync(path.join(process.cwd(), 'src/app/app/operations/[id]/page.tsx'), 'utf8');

    expect(client).toContain('/app/operations/provisioning/');
    expect(client).toContain('buildOperationsQueueHref');
    expect(client).toContain('View Due Queue');
    expect(client).toContain('/app/operations/alerts/');
    expect(runDetail).toContain('/app/operations/provisioning/');
  });
});
