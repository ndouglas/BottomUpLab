import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { expect, it } from 'vitest';
import { initSync, experiment_recorded_json, experiment_run_json } from '../wasm-pkg/sugarscape.js';
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
it('actual WASM recorded export includes the measured Gate census with every stratum and contrast', () => {
  const gate = JSON.parse(experiment_recorded_json()).studies.gate_assistance;
  expect(gate?.version).toBe('gate-assistance-summary-v1');
  expect(gate?.settings).toHaveLength(18);
  expect(gate?.contrasts).toHaveLength(45);
  const measured = JSON.parse(readFileSync(`${root}/crates/sugarscape-core/src/browser_experiments/fixtures/gate-recorded.json`, 'utf8')) as Json;
  expect(gate).toEqual(wire(measured));
  const before = experiment_recorded_json();
  experiment_run_json(JSON.stringify({ study: 'gate_assistance', stratum: { holding_cost: 1, holder_share: 3, distance: 0 }, profile: { holder: 540, retriever: 60 }, ids: { holder: 'Agent-A', retriever: 'Agent-B' } }));
  expect(experiment_recorded_json()).toBe(before);
}, 60_000);
