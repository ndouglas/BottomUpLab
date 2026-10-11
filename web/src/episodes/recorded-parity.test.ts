import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { expect, it } from 'vitest';
import { initSync, experiment_recorded_json } from '../wasm-pkg/sugarscape.js';
import type { Json } from './types';

initSync({ module: readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const root = fileURLToPath(new URL('../../../', import.meta.url));
const summary = JSON.parse(readFileSync(`${root}/crates/sugarscape-core/src/browser_experiments/fixtures/discovery-recorded.json`, 'utf8')) as Json;
const wire = (value: Json): Json => Array.isArray(value) ? value.map(wire) : value !== null && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([key, item]) => [key, wire(item)])) : typeof value === 'number' && Number.isInteger(value) ? String(value) : value;
it('actual WASM and native recorded exports retain the complete first-report summary losslessly', () => {
  const actual = experiment_recorded_json();
  const recorded = JSON.parse(actual).studies.surface_discovery;
  expect(recorded?.version).toBe('surface-discovery-summary-v1');
  expect(recorded).toEqual(wire(summary));
  const options = { cwd: root, encoding: 'utf8' as const, maxBuffer: 16 * 1024 * 1024, timeout: 60_000 };
  const native = execFileSync(`${root}/target/release/sugarscape`, ['experiment-view', 'recorded'], options).trimEnd();
  expect(actual).toBe(native);
});
