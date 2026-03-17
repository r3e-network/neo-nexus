import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

describe('SecurityClient content', () => {
  it('does not ship fake hardcoded firewall rows', () => {
    const file = path.join(process.cwd(), 'src/app/app/security/SecurityClient.tsx');
    const contents = fs.readFileSync(file, 'utf8');

    expect(contents).not.toContain('203.0.113.45');
    expect(contents).not.toContain('https://app.neodapp.io');
    expect(contents).not.toContain('Save Firewall Rules');
  });

  it('does not pretend stored api key hashes can be revealed as raw keys', () => {
    const file = path.join(process.cwd(), 'src/app/app/security/SecurityClient.tsx');
    const contents = fs.readFileSync(file, 'utf8');

    expect(contents).not.toContain('nk_live_...${key.keyHash.substring(0, 8)}');
    expect(contents).not.toContain('toggleVisibility');
  });
});
