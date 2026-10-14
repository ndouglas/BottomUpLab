import { expect, it } from 'vitest';
import { keyComparisonKey, keyMatchedInputs, keySettingLabel, matchKeyCheckpoint } from './key-comparison';
import type { EpisodeRecord, StudyDescriptor } from './types';
const input = {study:'key_delegation',stratum:{transfer_enabled:true,transfer_cost:1,giver_share:3,distance:0},ids:{giver:'Agent-A',receiver:'Agent-B'},selector:'SuppliedHandoff'};
const descriptor = {title:'',supplied:'',question:'',default_input:input,id:'key_delegation',family:'key',controls:{selectors:['SoloAttempt','SuppliedHandoff','TeamOptimal','NashLex','NashBestGroup','InitiallyEquipped']}} as StudyDescriptor;
it('declares every alternative with fixed stratum and opaque IDs',()=>{
 const choices=keyMatchedInputs(input,descriptor);expect(choices).toHaveLength(6);
 for(const choice of choices) expect(keyComparisonKey(choice)).toBe(keyComparisonKey(input));
 expect(keySettingLabel(choices[4])).toContain('Changed initial ownership');
 expect(keySettingLabel(choices[5])).toContain('Fixed raw schedule');
 expect(keyComparisonKey({...input,stratum:{...input.stratum,transfer_enabled:false}})).not.toBe(keyComparisonKey(input));
});
it('matches exact public phase, role and round with explicit missing terminal',()=>{
 const primary={kind:'experiment_episode',version:1,rules_identity:'test',semantics:'trajectory',payload:null,study:'key_delegation',input,checkpoints:[{public:null,local:{},researcher:null,index:0,kind:'after_action',clock:{round:'1',phase:'AfterAction',role:'Giver'}},{public:null,local:{},researcher:null,index:1,kind:'finished',clock:{round:'4',phase:'Finished',role:null}}]} as EpisodeRecord;
 const secondary={...primary,checkpoints:[{...primary.checkpoints[0],clock:{round:'1',phase:'AfterAction',role:'Receiver'}}, {...primary.checkpoints[1],clock:{round:'6',phase:'Finished',role:null}}, primary.checkpoints[0]]};
 expect(matchKeyCheckpoint(primary,secondary,0)).toBe(2);
 expect(matchKeyCheckpoint(primary,secondary,1)).toBeNull();
 expect(matchKeyCheckpoint(primary,secondary,2)).toBeNull();
});
