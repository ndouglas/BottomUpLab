import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { gateComparisonKey, gateMatchedInputs, gateSettingLabel } from './gate-comparison';
import { matchingCheckpoint } from './comparison';
import type { EpisodeRecord, StudyDescriptor } from './types';
import { beforeAll } from 'vitest';
import { initSync, experiment_run_json } from '../wasm-pkg/sugarscape.js';
initSync({ module: readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
let near: EpisodeRecord, idle: EpisodeRecord;
beforeAll(() => {
 const run = (holder:number,retriever:number):EpisodeRecord => JSON.parse(experiment_run_json(JSON.stringify({study:'gate_assistance',stratum:{holding_cost:1,holder_share:3,distance:0},profile:{holder,retriever},ids:{holder:'Agent-A',retriever:'Agent-B'}})));
 near=run(540,60);idle=run(0,0);
}, 60_000);
const descriptor = { id: 'gate_assistance', family: 'gate', title: '', question: '', supplied: '', default_input: null, controls: { selectors: ['SoloAttempt', 'SuppliedHelp', 'TeamOptimal', 'NashLex', 'NashBestGroup'] } } as StudyDescriptor;
it('raw versus selector pairs preserve stratum, IDs and fixed protocol exactly', () => {
  const candidates = gateMatchedInputs(near.input, descriptor);
  expect(candidates).toHaveLength(6);
  for (const input of candidates) expect(gateComparisonKey(input)).toBe(gateComparisonKey(near.input));
  expect(gateSettingLabel(near.input)).toContain('Fixed raw schedule');
  expect(gateSettingLabel(candidates[2])).toContain('Team optimal');
});
it('supplied selectors also offer the explicitly declared raw alternative', () => {
  const input = gateMatchedInputs(near.input, descriptor)[1];
  expect(gateMatchedInputs(input, descriptor).some(value => gateSettingLabel(value).includes('Fixed raw schedule'))).toBe(true);
});
it('clock joins require exact round, role and stage and never invent waits after delivery', () => {
  const deadline = idle.checkpoints.at(-1)!;
  expect(matchingCheckpoint(near.checkpoints, deadline)).toBe(-1);
  const before = near.checkpoints.find(at => at.kind === 'before_action')!;
  expect(matchingCheckpoint(near.checkpoints.filter(at => at.kind !== 'before_action'), before)).toBe(-1);
});
