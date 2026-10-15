import { expect,it } from 'vitest';
import { notesMatchedInputs, notesComparisonKey } from './notes-comparison';
it('changes only one treatment while preserving request, stratum and ids',()=>{
 const input={study:'persistent_notes',stratum:{acquisition_cost:1},arm:'Retained',routine:'ReadAdaptive',request:'Right',ids:{writer:'A',reader:'B'}};
 const choices=notesMatchedInputs(input,'arm');
 expect(choices).toHaveLength(3);
 for(const choice of choices) expect(notesComparisonKey(choice,'arm')).toEqual(notesComparisonKey(input,'arm'));
});
it('preserves routine when changing arm and arm when changing routine',()=>{
 const input={study:'persistent_notes',stratum:{writer_share:3},arm:'Retained',routine:'ReadAdaptive',request:'Right',ids:{writer:'A',reader:'B'}};
 for(const choice of notesMatchedInputs(input,'routine')) expect((choice as typeof input).arm).toBe('Retained');
});
it('provides independent availability when branch clock boundaries differ',async()=>{
 const {matchNotesCheckpoint}=await import('./notes-comparison');
 const {record}=await import('./record.fixture');
 const input={study:'persistent_notes',stratum:{writer_share:3},arm:'Retained',routine:'ReadAdaptive',request:'Right',ids:{writer:'A',reader:'B'}};
 const first={...record,study:'persistent_notes' as const,input};
 const second={...first,input:{...input,arm:'Erased'},checkpoints:[{...record.checkpoints[0],clock:{boundary:'99'}}]};
 expect(matchNotesCheckpoint(first,second,0,'arm')).toBeNull();
 expect(second.checkpoints).toHaveLength(1);
});
