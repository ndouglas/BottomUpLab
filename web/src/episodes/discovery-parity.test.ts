import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { initSync, experiment_catalog_json, experiment_run_json, experiment_validate_json } from '../wasm-pkg/sugarscape.js';
import { canonical } from './comparison';
import { obj, list } from './presentation';
import type { EpisodeRecord, Json, StudyDescriptor } from './types';
initSync({ module: readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const root = fileURLToPath(new URL('../../../', import.meta.url));
const descriptor = (JSON.parse(experiment_catalog_json()) as StudyDescriptor[]).find(d => d.id === 'surface_discovery')!;
const reference = JSON.parse(readFileSync(`${root}/crates/sugarscape-core/src/surface_discovery/tests/fixtures/evaluation-reference.json`, 'utf8')) as { episodes: { protocol: Json; environment: Json; sequence: number; episode: Json }[] };
const wire = (value: Json): Json => Array.isArray(value) ? value.map(wire) : value !== null && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([key, v]) => [key, wire(v)])) : typeof value === 'number' && Number.isInteger(value) ? String(value) : value;
function native(text: string): string {
  const scratch = mkdtempSync(`${tmpdir()}/discovery-parity-`);
  try {
    const path = `${scratch}/input.json`; writeFileSync(path, text);
    const options = { cwd: root, encoding: 'utf8' as const, maxBuffer: 16 * 1024 * 1024 + 1, timeout: 60_000 };
    return execFileSync(`${root}/target/release/sugarscape`, ['experiment-view', 'run', '--input', path], options).trimEnd();
  } finally { rmSync(scratch, { recursive: true, force: true }); }
}
describe('discovery complete native CLI and actual Node WASM parity', () => {
  it('registers all 48 unchanged native settings', () => expect(list(obj(descriptor.controls).settings)).toHaveLength(48));
  list(obj(descriptor.controls).settings).forEach((setting, index) => {
    for (const sequence of [0, 255]) it(`setting ${index}, sequence ${sequence}: every export and imported field`, () => {
      const text = JSON.stringify({ study: descriptor.id, ...obj(setting), sequence });
      const actual = experiment_run_json(text);
      expect(actual).toBe(native(text));
      expect(experiment_validate_json(actual)).toBe(actual);
      const record = JSON.parse(actual) as EpisodeRecord;
      expect(record.checkpoints.length).toBeLessThanOrEqual(4096);
      expect(new TextEncoder().encode(actual).byteLength).toBeLessThanOrEqual(16 * 1024 * 1024);
    }, 60_000);
  });
  for (const [index, witness] of reference.episodes.entries()) it(`sealed independent complete payload witness ${index}`, () => {
    const actual = JSON.parse(experiment_run_json(JSON.stringify({ study: descriptor.id, protocol: witness.protocol, environment: witness.environment, sequence: witness.sequence }))) as EpisodeRecord;
    expect(canonical(actual.payload)).toBe(canonical(wire(witness.episode)));
  }, 60_000);
});
