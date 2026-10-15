import { obj } from './presentation';
import type { Checkpoint, Json } from './types';
/** The shell must mask this checkpoint before display. No input or physical state is accepted. */
export function notesDisplay(projected: Checkpoint): { clock:Json; public:Json; agents:{id:string; view:Json}[]; researcher:Json } {
 return {clock:projected.clock,public:projected.public,agents:Object.entries(projected.local).map(([id,local])=>({id,view:obj(local).view??null})),researcher:projected.researcher};
}
