import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import { keyDisplay } from './key-projection';
import { obj, list } from './presentation';
import { canonical } from './comparison';
import { projectCheckpoint, resultPayload } from './projection';
import type { EpisodeRecord } from './types';
import { initSync, experiment_run_json } from '../experiments-pkg/experiments.js';
initSync({ module: readFileSync(new URL('../experiments-pkg/experiments_bg.wasm', import.meta.url)) });
describe('enabled near', () => {
  let record: EpisodeRecord;
  beforeAll(() => {
    record = JSON.parse(experiment_run_json(JSON.stringify({
      study: 'key_delegation',
      stratum: { transfer_enabled: true, transfer_cost: 1, giver_share: 3, distance: 0 },
      ids: { giver: 'Agent-A', receiver: 'Agent-B' },
      selector: 'SuppliedHandoff',
    }))) as EpisodeRecord;
  }, 60_000);
  it('keeps evaluator data outside every Agent projection', () => {
    for (const at of record.checkpoints) for (const agent of ['Agent-A', 'Agent-B', 'public-only']) {
      const projected = projectCheckpoint(record, at.index, { kind: 'agent', agent });
      expect(keyDisplay(projected).researcher).toBeNull();
      expect(keyDisplay(projected).agents.map(a=>a.id)).toEqual([agent]);
      expect(Object.keys(projected.local)).toEqual([agent]);
      expect(resultPayload(record, at.index, { kind: 'agent', agent })).toBeNull();
    }
  });
  it('passive receipt changes own possession before Receiver senses while dated map stays stale',()=>{
    const at = record.checkpoints.find(at=>obj(at.clock).round==='1'&&obj(at.clock).phase==='AfterAction'&&obj(at.clock).role==='Giver')!;
    const agent = keyDisplay(projectCheckpoint(record,at.index,{kind:'agent',agent:'Agent-B'})).agents[0];
    expect(obj(agent.own).own_key).toBe(true);
    expect(agent.receipts).toEqual([{clock:at.clock,counterpart:'Agent-A',before:false,after:true}]);
    expect(agent.observation).toEqual(obj(record.checkpoints[0].local['Agent-B']).map);
    expect(obj(agent.observation).observed_at).not.toEqual(at.clock);
    expect(list(obj(agent.own).history).filter(row=>'OwnAction' in obj(row))).toHaveLength(0);
    const before=canonical(record as unknown as import('./types').Json);
    for(const at of [...record.checkpoints].reverse())keyDisplay(projectCheckpoint(record,at.index,{kind:'agent',agent:'Agent-B'}));
    expect(canonical(record as unknown as import('./types').Json)).toBe(before);
  });
});

describe.each([
 ['disabled deadline',false,0,'SuppliedHandoff'],
 ['enabled far',true,1,'SuppliedHandoff'],
 ['changed ownership',false,0,'InitiallyEquipped'],
] as const)('%s', (_name,transfer_enabled,distance,selector)=>{
 let record: EpisodeRecord;
 beforeAll(()=>{record=JSON.parse(experiment_run_json(JSON.stringify({study:'key_delegation',stratum:{transfer_enabled,transfer_cost:1,giver_share:3,distance},ids:{giver:'Agent-A',receiver:'Agent-B'},selector})));},60_000);
 it('all actual checkpoint perspectives retain only registered own state and never evaluator results',()=>{
  for(const at of record.checkpoints)for(const agent of ['Agent-A','Agent-B','public-only']){
   const view=projectCheckpoint(record,at.index,{kind:'agent',agent});const display=keyDisplay(view);
   expect(display.researcher).toBeNull();expect(resultPayload(record,at.index,{kind:'agent',agent})).toBeNull();
   expect(display.agents.map(a=>a.id)).toEqual([agent]);
   expect(obj(display.agents[0].own)).not.toHaveProperty('classification');
   expect(display.agents[0].receipts).toEqual(list(obj(obj(at.local[agent]).view).history).filter(row=>'Ownership' in obj(row)).map(row=>obj(row).Ownership));
  }
  expect(keyDisplay(projectCheckpoint(record,0,{kind:'researcher'})).researcher).toEqual(record.checkpoints[0].researcher);
 });
});
