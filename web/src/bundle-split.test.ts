// The engine package must stay free of the provenance cluster; the six exports
// that reach it live only in the experiments package the episodes worker loads.
import { describe, expect, it } from 'vitest';
import * as engine from './wasm-pkg/sugarscape.js';
import * as experiments from './experiments-pkg/experiments.js';

const moved = [
  'experiment_catalog_json',
  'experiment_run_json',
  'experiment_validate_json',
  'experiment_recorded_json',
  'burrow_replay_json',
  'burrow_access_replay_json',
] as const;

describe('bundle split', () => {
  for (const name of moved) {
    it(`${name} is exported by the experiments package only`, () => {
      expect(name in engine, `${name} leaked into the engine package`).toBe(false);
      expect(typeof (experiments as Record<string, unknown>)[name]).toBe('function');
    });
  }
  it('the engine package keeps the playground exports', () => {
    for (const name of ['Sim', 'run_point', 'presets_json', 'model_schemas_json']) {
      expect(typeof (engine as Record<string, unknown>)[name], name).toBe('function');
    }
  });
});
