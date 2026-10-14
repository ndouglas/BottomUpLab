import type { EpisodeRecord, Checkpoint, Json } from './types';
export type Perspective = { kind: 'agent'; agent: string } | { kind: 'researcher' };
export function projectCheckpoint(record: EpisodeRecord, index: number, perspective: Perspective): Checkpoint {
  const at = record.checkpoints[index];
  if (!at) throw new Error('checkpoint out of range');
  return perspective.kind === 'researcher' ? at : { ...at, researcher: null, local: { [perspective.agent]: at.local[perspective.agent] ?? null } };
}
/** Discovery, Gate and Key evaluator material is Researcher-only, including terminal checkpoints. */
export function resultPayload(record: EpisodeRecord, index: number, perspective: Perspective): Json {
  if (['surface_discovery', 'gate_assistance', 'key_delegation'].includes(record.study) && perspective.kind !== 'researcher') return null;
  return perspective.kind === 'researcher' || index === record.checkpoints.length - 1 ? record.payload : null;
}
