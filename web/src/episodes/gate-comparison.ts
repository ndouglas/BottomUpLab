import type { Json, StudyDescriptor } from './types';
import { canonical } from './comparison';
import { obj, list, valueText } from './presentation';
export const gateSelectorLabels: Record<string, string> = {
  SoloAttempt: 'Solo attempt (fixed script)', SuppliedHelp: 'Supplied minimal assistance (fixed script)',
  TeamOptimal: 'Team optimal', NashLex: 'Lexicographic pure equilibrium', NashBestGroup: 'Best-group pure equilibrium',
};
/** Excludes only the declared choice; IDs, stratum and every other field must agree. */
export function gateComparisonKey(input: Json): string {
  const copy = structuredClone(obj(input));
  if (copy.study !== 'gate_assistance') throw new Error('Comparison study differs');
  delete copy.selector; delete copy.profile;
  return canonical(copy);
}
export function gateMatchedInputs(input: Json, descriptor: StudyDescriptor): Json[] {
  if (descriptor.family !== 'gate' || obj(input).study !== descriptor.id) return [];
  const base = structuredClone(obj(input)); delete base.selector; delete base.profile;
  const selectors = list(obj(descriptor.controls).selectors).filter(s => typeof s === 'string' && s in gateSelectorLabels);
  const choices: Json[] = [...selectors.map(selector => ({ ...base, selector })), { ...base, profile: { holder: 0, retriever: 0 } }];
  return choices.filter(candidate => canonical(candidate) !== canonical(input));
}
export function gateSettingLabel(input: Json): string {
  const value = obj(input);
  return 'profile' in value ? `Fixed raw schedule: Holder ${valueText(obj(value.profile).holder)}, Retriever ${valueText(obj(value.profile).retriever)}` : gateSelectorLabels[String(value.selector)] ?? 'Opened selector · validation on run';
}
