import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import { projectCheckpoint, resultPayload } from './projection';
import { gateDisplay } from './gate-projection';
import type { EpisodeRecord } from './types';
import { initSync, experiment_run_json } from '../wasm-pkg/sugarscape.js';
initSync({ module: readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
describe.each([
  ['supplied near', 1, 3, 0, 540, 60],
  ['supplied far', 2, 6, 1, 423, 30],
  ['non-equilibrium delayed deadline', 1, 3, 0, 540, 15],
  ['idle deadline', 1, 0, 0, 0, 0],
] as const)('%s', (name, holding_cost, holder_share, distance, holder, retriever) => {
  let record: EpisodeRecord;
  // Each request compiles its public stratum: bound one request per hook,
  // as in gate-parity, rather than charging four requests to one deadline.
  beforeAll(() => {
    record = JSON.parse(experiment_run_json(JSON.stringify({ study: 'gate_assistance', stratum: { holding_cost, holder_share, distance }, ids: { holder: 'Agent-A', retriever: 'Agent-B' }, profile: { holder, retriever } })));
  }, 60_000);
  it('all actual WASM checkpoints keep evaluator results and peer words outside Agent projections', () => {
    for (const at of record.checkpoints) {
      for (const agent of ['Agent-A', 'Agent-B', 'public-only']) {
        const own = projectCheckpoint(record, at.index, { kind: 'agent', agent });
        expect(resultPayload(record, at.index, { kind: 'agent', agent })).toBeNull();
        expect(own.researcher).toBeNull();
        expect(Object.keys(own.local)).toEqual([agent]);
        const display = gateDisplay(own);
        expect(display.researcher).toBeNull();
        expect(display.agents.map(a => a.id)).toEqual([agent]);
      }
      expect(resultPayload(record, at.index, { kind: 'researcher' })).toEqual(record.payload);
      expect(gateDisplay(projectCheckpoint(record, at.index, { kind: 'researcher' })).researcher).toEqual(at.researcher);
    }
  });
  if (name === 'supplied near') it('native passive checkpoints retain dated observations and seeking cannot mutate records', () => {
    const before = JSON.stringify(record);
    const initial = gateDisplay(projectCheckpoint(record, 0, { kind: 'agent', agent: 'Agent-A' })).agents[0];
    const reset = gateDisplay(projectCheckpoint(record, 1, { kind: 'agent', agent: 'Agent-A' })).agents[0];
    expect(reset.map).toEqual(initial.map);
    expect(reset.map.observed_at).not.toEqual(record.checkpoints[1].clock);
    expect(initial.map.gate).toBeNull();
    expect(initial.gateInference).toBeNull();
    expect(reset.map.gate).toBeNull();
    expect(reset.gateInference).toBe('Closed by public reset / termination rule');
    for (const at of [...record.checkpoints].reverse()) gateDisplay(projectCheckpoint(record, at.index, { kind: 'agent', agent: 'Agent-A' }));
    expect(JSON.stringify(record)).toBe(before);
  });
});
