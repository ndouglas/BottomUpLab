import { expect, it } from 'vitest';
import { resultPayload, projectCheckpoint } from './projection';
import { record as fixture } from './record.fixture';
it('keeps notes terminal results researcher-only', () => {
 const record = {...fixture, study: 'persistent_notes' as const};
 expect(resultPayload(record, record.checkpoints.length-1, {kind:'agent',agent:'reader'})).toBeNull();
 expect(projectCheckpoint(record, 0, {kind:'agent',agent:'reader'}).researcher).toBeNull();
});
it('notes display receives only the masked own snapshot',async()=>{
 const {notesDisplay}=await import('./notes-projection');
 const at={...fixture.checkpoints[0],local:{Writer:{view:{known_request:'Right'}},Reader:{view:{known_request:null}}},researcher:{request:'Right'}};
 const record={...fixture,study:'persistent_notes' as const,checkpoints:[at]};
 const display=notesDisplay(projectCheckpoint(record,0,{kind:'agent',agent:'Reader'}));
 expect(display.agents).toEqual([{id:'Reader',view:{known_request:null}}]);
 expect(display.researcher).toBeNull();
 expect(JSON.stringify(display)).not.toContain('Right');
});
