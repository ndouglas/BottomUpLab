import type { Checkpoint, Json } from './types';
import { obj, list } from './presentation';
export interface KeyAgent { id: string; role: string; own: Json; observation: Json; receipts: Json[] }
export interface KeyDisplay { clock: Json; status: Json; agents: KeyAgent[]; researcher: Json }
/** Only registered own fields from the shell's already masked native checkpoint. No sensing on replay. */
export function keyDisplay(projected: Checkpoint): KeyDisplay {
  const agents = Object.entries(projected.local).map(([id, local]) => {
    const source = obj(local), native = obj(source.view), map = obj(source.map);
    const own: Record<string, Json> = {};
    for (const field of ['word', 'own_cell', 'own_cargo', 'own_key', 'credits', 'own_reward', 'own_net', 'clock', 'history']) if (field in native) own[field] = native[field];
    const receipts = list(native.history).filter(entry => 'Ownership' in obj(entry)).map(entry => obj(entry).Ownership);
    return { id, role: typeof native.role === 'string' ? native.role : '', own, observation: local === null ? null : { cells: list(map.cells), observed_at: map.observed_at ?? null }, receipts };
  });
  return { clock: projected.clock, status: obj(projected.public).status ?? null, agents, researcher: projected.researcher };
}
