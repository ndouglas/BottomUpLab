import { readFileSync } from 'node:fs';
import { beforeAll, expect, it } from 'vitest';
import { initSync, experiment_run_json, experiment_validate_json } from '../experiments-pkg/experiments.js';
import { EpisodeSession } from './file';
import { projectCheckpoint, resultPayload } from './projection';
import type { EpisodeClient, EpisodeRecord } from './types';

beforeAll(() => initSync({ module: readFileSync(new URL('../experiments-pkg/experiments_bg.wasm', import.meta.url)) }));
it('real release Notes imports and exports through fresh reconstruction while failed import retains playback', async () => {
  const input = { study: 'persistent_notes', stratum: { acquisition_cost: 1, write_cost: 2, note_read_cost: 1, manifest_inspection_cost: 4, writer_share: 3 }, arm: 'Retained', routine: 'ReadAdaptive', request: 'Right', ids: { writer: 'Writer', reader: 'Reader' } };
  const wire = experiment_run_json(JSON.stringify(input));
  let validations = 0;
  const client: EpisodeClient = {
    request: async (op, text) => {
      expect(op).toBe('validate');
      validations++;
      return JSON.parse(experiment_validate_json(text)) as EpisodeRecord;
    },
    catalog: async () => [], recorded: async () => { throw new Error('selected delivery must not load recorded data'); },
    cancel() {}, dispose() {},
  };
  const session = new EpisodeSession(client);
  const record = await session.import(wire);
  expect(JSON.parse(await session.export())).toEqual(JSON.parse(wire));
  expect(validations).toBe(2);
  const initial = JSON.stringify(record.checkpoints[0]);
  for (let checkpoint = 0; checkpoint < record.checkpoints.length; checkpoint++) {
    expect(projectCheckpoint(record, checkpoint, { kind: 'agent', agent: 'Reader' }).researcher).toBeNull();
    expect(resultPayload(record, checkpoint, { kind: 'agent', agent: 'Reader' })).toBeNull();
  }
  const tampered = JSON.parse(wire);
  tampered.payload = null;
  await expect(session.import(JSON.stringify(tampered))).rejects.toBeDefined();
  expect(session.shown).toEqual(record);
  expect(session.records).toHaveLength(1);
  expect(JSON.stringify(record.checkpoints[0])).toBe(initial);
});
