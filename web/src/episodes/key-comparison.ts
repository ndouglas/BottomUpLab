import { canonical, matchingCheckpoint } from './comparison';
import { obj, list, valueText } from './presentation';
import type { EpisodeRecord, Json, StudyDescriptor } from './types';
export const keySelectorLabels: Record<string, string> = {
  SoloAttempt: 'Solo attempt (fixed script)', SuppliedHandoff: 'Supplied minimal handoff (fixed script)',
  TeamOptimal: 'Team optimal', NashLex: 'Lexicographic pure equilibrium', NashBestGroup: 'Best-group pure equilibrium',
  InitiallyEquipped: 'Initially equipped · Changed initial ownership; main-game classification unavailable',
};

export function keyComparisonKey(input: Json): string {
  const copy = structuredClone(obj(input));
  if (copy.study !== 'key_delegation') throw new Error('Comparison study differs');
  delete copy.selector; delete copy.profile;
  return canonical(copy);
}
export function keyMatchedInputs(input: Json, descriptor: StudyDescriptor): Json[] {
  if (descriptor.family !== 'key' || obj(input).study !== descriptor.id) return [];
  const base = structuredClone(obj(input)); delete base.selector; delete base.profile;
  const selectors = list(obj(descriptor.controls).selectors).filter(s => typeof s === 'string' && s in keySelectorLabels);
  const choices: Json[] = [...selectors.map(selector => ({ ...base, selector })), { ...base, profile: { giver: 0, receiver: 0 } }];
  return choices.filter(candidate => canonical(candidate) !== canonical(input));
}
export function keySettingLabel(input: Json): string {
  const value = obj(input);
  return 'profile' in value ? `Fixed raw schedule: Giver ${valueText(obj(value.profile).giver)}, Receiver ${valueText(obj(value.profile).receiver)}` : keySelectorLabels[String(value.selector)] ?? 'Opened selector · validation on run';
}
export function matchKeyCheckpoint(primary: EpisodeRecord, secondary: EpisodeRecord, index: number): number | null {
  const at = primary.checkpoints[index];
  if (!at || primary.study !== 'key_delegation' || secondary.study !== primary.study || keyComparisonKey(primary.input) !== keyComparisonKey(secondary.input)) return null;
  const match = matchingCheckpoint(secondary.checkpoints, at);
  return match < 0 ? null : match;
}
