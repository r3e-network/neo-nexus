import { describe, expect, it } from 'vitest';
import { docsNavigation, filterDocsNavigation } from './DocsIndex';

describe('DocsIndex', () => {
  it('returns matching docs entries for a search query', () => {
    const results = filterDocsNavigation(docsNavigation, 'oracle');
    expect(results.flatMap((section) => section.items.map((item) => item.label))).toContain('TEE Oracle');
  });

  it('returns all docs entries when the query is empty', () => {
    const results = filterDocsNavigation(docsNavigation, '');
    expect(results).toHaveLength(docsNavigation.length);
  });
});
