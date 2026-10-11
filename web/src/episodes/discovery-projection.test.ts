import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { initSync, experiment_catalog_json, experiment_run_json } from '../wasm-pkg/sugarscape.js';
import { projectCheckpoint, resultPayload } from './projection';
import { surfaceDisplay, surfaceChoiceLabel } from './surface-projection';
import { obj, list } from './presentation';
import type { EpisodeRecord, StudyDescriptor } from './types';
initSync({ module: readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const descriptor = (JSON.parse(experiment_catalog_json()) as StudyDescriptor[]).find(d => String(d.id) === 'surface_discovery')!;
const record = JSON.parse(experiment_run_json(JSON.stringify(descriptor.default_input))) as EpisodeRecord;
it('selected own checkpoint excludes evaluator, peer and future sentinels', () => {
  const supplied = structuredClone(record);
  supplied.checkpoints[0].researcher = { actual_model: 'MODEL_SENTINEL', lineage: 'LINEAGE_SENTINEL', reward: 'REWARD_SENTINEL', grade: 'GRADE_SENTINEL' };
  supplied.checkpoints[0].local['Agent-B'] = { prefix: { entries: ['PEER_SENTINEL'] }, private_bit: 'PEER_BIT_SENTINEL' };
  supplied.checkpoints[1].local['Agent-A'] = { private_bit: 'FUTURE_BIT_SENTINEL' };
  supplied.payload = { reward: 'REWARD_SENTINEL', grade: 'GRADE_SENTINEL' };
  const projected = projectCheckpoint(supplied, 0, { kind: 'agent', agent: 'Agent-A' });
  expect(JSON.stringify(surfaceDisplay(projected))).not.toContain('SENTINEL');
  expect(resultPayload(supplied, 0, { kind: 'agent', agent: 'Agent-A' })).toBeNull();
  expect(surfaceDisplay(projectCheckpoint(supplied, 0, { kind: 'researcher' }))).toMatchObject({ researcher: supplied.checkpoints[0].researcher });
});
it('raw owned paid choices are distinct from free round purchase', () => {
  expect(surfaceChoiceLabel({ Calibration: { Write: 'Probe0' } })).toBe('Owned paid calibration action');
  expect(surfaceChoiceLabel('Continue')).toBe('Free round-boundary choice');
  expect(surfaceChoiceLabel('Inspect')).toBe('Live routine choice');
});
it('raw alternatives and continuation values retain native order and exact fractions', () => {
  const at = record.checkpoints.find(p => list(obj(obj(p.local['Agent-A']).decision).alternatives).length === 8)!;
  const decision = obj(obj(list(obj(surfaceDisplay(projectCheckpoint(record, at.index, { kind: 'agent', agent: 'Agent-A' }))).agents)[0]).decision);
  expect(list(decision.alternatives).map(a => obj(a).choice)).toEqual([{ Calibration: 'Wait' }, { Calibration: 'Read' }, ...['Probe0', 'Ack0', 'Probe1', 'Ack1', 'Data0', 'Data1'].map(symbol => ({ Calibration: { Write: symbol } }))]);
  expect(decision).toEqual(obj(at.local['Agent-A']).decision);
  expect(obj(at.local['Agent-A']).decision_label).toBe('Expected remaining investigator net task utility under the declared continuation policy');
});
it('seeking and switching perspective preserve complete recorded decisions and costs', () => {
  const before = JSON.stringify(record);
  for (const index of [record.checkpoints.length - 1, 0, 3, 1, 3]) for (const agent of ['Agent-A', 'Agent-B']) surfaceDisplay(projectCheckpoint(record, index, { kind: 'agent', agent }));
  expect(JSON.stringify(record)).toBe(before);
});

const unsupportedInput = { ...obj(descriptor.default_input), environment: 'DataFlip', protocol: { ...obj(obj(descriptor.default_input).protocol), investigator: 'B', policy: 'InspectOnly' } };
const unsupported = JSON.parse(experiment_run_json(JSON.stringify(unsupportedInput))) as EpisodeRecord;
for (const [name, source] of [['completed', record], ['unsupported', unsupported]] as const) {
  it(`${name} discovery keeps evaluator and peer payload Researcher-only at every checkpoint`, () => {
    const sentinel = structuredClone(source);
    sentinel.payload = { actual_model: 'MODEL_SENTINEL', histories: 'PEER_HISTORY_SENTINEL', failure: { prefix: 'PEER_PREFIX_SENTINEL', private_bit: 'PEER_BIT_SENTINEL' }, metrics: [{ reward: 'REWARD_SENTINEL', correct: 'GRADE_SENTINEL' }], privileged: 'LINEAGE_SENTINEL' };
    for (const at of sentinel.checkpoints) {
      for (const agent of ['Agent-A', 'Agent-B', 'unavailable-public-view']) {
        expect(resultPayload(sentinel, at.index, { kind: 'agent', agent })).toBeNull();
        expect(projectCheckpoint(sentinel, at.index, { kind: 'agent', agent }).public).toEqual(at.public);
      }
      expect(resultPayload(sentinel, at.index, { kind: 'researcher' })).toBe(sentinel.payload);
    }
  });
}
it('unsupported discovery final Agent retains own observations without the peer failure prefix', () => {
  const index = unsupported.checkpoints.length - 1;
  const projected = projectCheckpoint(unsupported, index, { kind: 'agent', agent: 'Agent-B' });
  expect(Object.keys(projected.local)).toEqual(['Agent-B']);
  expect(projected.researcher).toBeNull();
  expect(resultPayload(unsupported, index, { kind: 'agent', agent: 'Agent-B' })).toBeNull();
  expect(Object.keys(obj(projected.public)).sort()).toEqual(['probe_stop', 'reset_phase', 'status']);
});
it('matched discovery records never supply terminal comparison payload to an Agent', () => {
  const supplied = JSON.parse(experiment_run_json(JSON.stringify({ ...obj(descriptor.default_input), protocol: { ...obj(obj(descriptor.default_input).protocol), policy: 'SuppliedAdaptive' } }))) as EpisodeRecord;
  for (const compared of [record, supplied]) {
    expect(resultPayload(compared, compared.checkpoints.length - 1, { kind: 'agent', agent: 'Agent-A' })).toBeNull();
    expect(resultPayload(compared, compared.checkpoints.length - 1, { kind: 'researcher' })).toBe(compared.payload);
  }
});
it('legacy final Agent result availability remains unchanged', () => {
  const activeDescriptor = (JSON.parse(experiment_catalog_json()) as StudyDescriptor[]).find(d => d.id === 'active_surface')!;
  const active = JSON.parse(experiment_run_json(JSON.stringify(activeDescriptor.default_input))) as EpisodeRecord;
  expect(resultPayload(active, active.checkpoints.length - 1, { kind: 'agent', agent: 'Agent-A' })).toBe(active.payload);
  expect(resultPayload(active, 0, { kind: 'agent', agent: 'Agent-A' })).toBeNull();
});
