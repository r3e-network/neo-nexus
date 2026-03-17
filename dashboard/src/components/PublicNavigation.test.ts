import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('public navigation', () => {
  it('does not ship dead placeholder href links in public surfaces', () => {
    const dashboardRoot = process.cwd();
    const files = [
      path.join(dashboardRoot, 'src/components/Footer.tsx'),
      path.join(dashboardRoot, 'src/app/(marketing)/docs/page.tsx'),
      path.join(dashboardRoot, 'src/app/(marketing)/developers/page.tsx'),
    ];

    for (const file of files) {
      const contents = fs.readFileSync(file, 'utf8');
      expect(contents).not.toContain('href="#"');
    }
  });
});
