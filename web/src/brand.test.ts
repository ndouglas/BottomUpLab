// The page names itself from one place: the header, the tab title and saved
// file names all come from brand.ts, and no source spells the old name.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { FILE_PREFIX, SITE_NAME } from './brand';

const SRC = new URL('.', import.meta.url).pathname;
const SKIP = new Set(['wasm-pkg', 'experiments-pkg', 'test-fixtures']);

function sources(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) {
      if (!SKIP.has(name)) out.push(...sources(path));
    } else if (name.endsWith('.ts') && !name.includes('.test.') && !name.includes('fixture') && !name.includes('browser-test')) {
      out.push(path);
    }
  }
  return out;
}

describe('brand', () => {
  it('is the renamed repository', () => {
    expect(SITE_NAME).toBe('BottomUpLab');
    expect(FILE_PREFIX).toBe('bottomuplab');
  });

  it('is the tab title', () => {
    const html = readFileSync(join(SRC, '..', 'index.html'), 'utf8');
    expect(html).toContain(`<title>${SITE_NAME}</title>`);
    expect(html).not.toContain('SugarScape');
  });

  it('is spelled nowhere else in the page source', () => {
    // 'Sugarscape' (the model) is fine; 'SugarScape' (the old site) and the
    // old file-name prefix are not.
    const offenders = sources(SRC)
      .filter((p) => /SugarScape|`sugarscape-|'sugarscape-/.test(readFileSync(p, 'utf8')))
      .map((p) => p.slice(SRC.length));
    expect(offenders).toEqual([]);
  });
});
