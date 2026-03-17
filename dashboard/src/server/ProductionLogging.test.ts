import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('production logging hygiene', () => {
  it('avoids console.log in live server paths', () => {
    const files = [
      path.join(process.cwd(), 'src/app/app/billing/actions.ts'),
      path.join(process.cwd(), 'src/app/app/endpoints/pluginActions.ts'),
      path.join(process.cwd(), 'src/services/billing/StripeService.ts'),
    ];

    for (const file of files) {
      const contents = fs.readFileSync(file, 'utf8');
      expect(contents).not.toContain('console.log(');
    }
  });
});
