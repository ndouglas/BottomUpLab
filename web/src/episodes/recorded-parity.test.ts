import { readFileSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import { initSync, experiment_recorded_json, experiment_run_json } from '../experiments-pkg/experiments.js';
import type { Json } from './types';

initSync({ module: readFileSync(new URL('../experiments-pkg/experiments_bg.wasm', import.meta.url)) });
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
}, 60_000);
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

const keyCases = [
  ['SuppliedHandoff', {study:'key_delegation',stratum:{transfer_enabled:true,transfer_cost:1,giver_share:3,distance:0},selector:'SuppliedHandoff',ids:{giver:'Agent-A',receiver:'Agent-B'}}],
  ['raw profile', {study:'key_delegation',stratum:{transfer_enabled:false,transfer_cost:4,giver_share:0,distance:1},profile:{giver:0,receiver:63},ids:{giver:'Agent-A',receiver:'Agent-B'}}],
  ['InitiallyEquipped', {study:'key_delegation',stratum:{transfer_enabled:true,transfer_cost:2,giver_share:3,distance:1},selector:'InitiallyEquipped',ids:{giver:'Agent-A',receiver:'Agent-B'}}],
] as const;
describe.each(keyCases)('%s recorded export preservation', (_name, input) => {
  const dir = mkdtempSync(`${tmpdir()}/key-recorded-parity-`);
  const options = {cwd:root,encoding:'utf8' as const,maxBuffer:16*1024*1024,timeout:60_000};
  let nativeBefore:string, nativeAfter:string, wasmBefore:string, wasmAfter:string;
  beforeAll(() => {
    nativeBefore = execFileSync(`${root}/target/release/sugarscape`,['experiment-view','recorded'],options).trimEnd();
    writeFileSync(`${dir}/input.json`,JSON.stringify(input));
    execFileSync(`${root}/target/release/sugarscape`,['experiment-view','run','--input',`${dir}/input.json`],options);
    nativeAfter = execFileSync(`${root}/target/release/sugarscape`,['experiment-view','recorded'],options).trimEnd();
  },60_000);
  // Match the existing Key parity allowance for one full WASM selected-record setup.
  // The production worker timeout remains 60 seconds.
  beforeAll(() => {
    wasmBefore = experiment_recorded_json();
    experiment_run_json(JSON.stringify(input));
    wasmAfter = experiment_recorded_json();
  },120_000);
  afterAll(() => rmSync(dir,{recursive:true,force:true}));
  it('preserves the complete accepted Key projection and full native/WASM envelope', () => {
    const measured = JSON.parse(readFileSync(`${root}/crates/sugarscape-core/src/browser_experiments/fixtures/key-recorded.json`,'utf8')) as Json;
    expect(JSON.parse(wasmBefore).studies.key_delegation).toEqual(wire(measured));
    expect(wasmBefore).toBe(nativeBefore);
  });
  it('leaves the complete recorded envelope unchanged after both selected runs', () => {
    expect(wasmAfter).toBe(wasmBefore);
    expect(nativeAfter).toBe(wasmBefore);
  });
});

it('actual WASM Notes recorded dictionary retains the complete saved summary within the unchanged cap', () => {
  const text = experiment_recorded_json();
  expect(new TextEncoder().encode(text).length).toBeLessThanOrEqual(16 * 1024 * 1024);
  const notes = JSON.parse(text).studies.persistent_notes;
  expect(notes.wire_encoding).toBe('persistent-notes-metrics-dictionary-v1');
  const dictionary = notes.metrics_dictionary;
  for (const field of ['means', 'request_contrasts', 'contrasts']) for (const row of notes[field]) row.metrics = dictionary[Number(row.metrics)];
  delete notes.metrics_dictionary; delete notes.wire_encoding;
  const measured = JSON.parse(readFileSync(`${root}/crates/sugarscape-core/src/browser_experiments/fixtures/notes-recorded.json`, 'utf8')) as Json;
  expect(notes).toEqual(wire(measured));
});
