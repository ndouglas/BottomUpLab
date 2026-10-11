import type { Checkpoint, Json } from './types';
import { obj, list } from './presentation';
export interface GateAgent {
  id: string;
  view: Record<string, Json>;
  map: Record<string, Json>;
  available: boolean;
  gateInference: string | null;
}
/** Receives only the shell's perspective projection; allowlist prevents incidental data from becoming a local View. */
export function gateDisplay(at: Checkpoint): { clock: Json; status: Json; agents: GateAgent[]; researcher: Json } {
  const agents = Object.entries(at.local).map(([id, local]) => {
    const source = obj(local), native = obj(source.view), map = obj(source.map);
    const view: Record<string, Json> = {};
    for (const key of ['role', 'word', 'own_cell', 'own_cargo', 'credits', 'own_reward', 'clock', 'history', 'last_observation']) if (key in native) view[key] = native[key];
    const phase = obj(at.clock).phase;
    const gateInference = phase === 'Reset' || phase === 'Finished' ? 'Closed by public reset / termination rule' : null;
    return { id, view, gateInference, available: local !== null && 'role' in native, map: { cells: list(map.cells), gate: map.gate ?? null, observed_at: map.observed_at ?? null } };
  });
  return { clock: at.clock, status: obj(at.public).status ?? null, agents, researcher: at.researcher };
}
