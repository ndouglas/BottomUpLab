import { canonical, matchingCheckpoint } from './comparison';
import { obj } from './presentation';
import type { EpisodeRecord, Json } from './types';
export const notesArms = ['Retained','Erased','NoNote','NoAcquireNoNote'];
export const notesRoutines = ['BlindGuess','InspectFirst','ReadAdaptive','Oracle'];
export type NotesAxis = 'arm' | 'routine';
export function notesComparisonKey(input: Json, axis: NotesAxis): string {
 const base=structuredClone(obj(input));
 if(base.study !== 'persistent_notes') throw new Error('Comparison study differs');
 delete base[axis]; return canonical(base);
}
export function notesMatchedInputs(input: Json, axis: NotesAxis): Json[] {
 const base=structuredClone(obj(input));
 notesComparisonKey(input,axis);
 return (axis==='arm'?notesArms:notesRoutines).filter(value=>value!==base[axis]).map(value=>({...base,[axis]:value}));
}
/** A missing same-clock boundary leaves the independent comparison timeline available. */
export function matchNotesCheckpoint(primary: EpisodeRecord, secondary: EpisodeRecord, index: number, axis: NotesAxis): number | null {
 const at=primary.checkpoints[index];
 if(!at || primary.study!=='persistent_notes' || secondary.study!==primary.study || notesComparisonKey(primary.input,axis)!==notesComparisonKey(secondary.input,axis)) return null;
 const found=matchingCheckpoint(secondary.checkpoints,at); return found<0?null:found;
}
