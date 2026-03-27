import { vi, describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// Unmock fs so we can use real file operations
vi.unmock('node:fs');

import { computeFileSha256 } from '../../src/utils/checksum';

describe('computeFileSha256', () => {
  it('computes the correct SHA256 hash for a file with "hello world" content', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'neonexus-checksum-'));
    const file = join(dir, 'test.txt');
    writeFileSync(file, 'hello world');

    const expected = createHash('sha256').update('hello world').digest('hex');
    const actual = await computeFileSha256(file);

    expect(actual).toBe(expected);
  });

  it('computes the correct SHA256 for an empty file', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'neonexus-checksum-'));
    const file = join(dir, 'empty.txt');
    writeFileSync(file, '');

    const expected = createHash('sha256').update('').digest('hex');
    const actual = await computeFileSha256(file);

    expect(actual).toBe(expected);
  });

  it('rejects for a non-existent file', async () => {
    await expect(computeFileSha256('/tmp/this-file-does-not-exist-12345.txt')).rejects.toThrow();
  });
});
