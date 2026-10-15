import { readFileSync, mkdtempSync,writeFileSync,rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { afterAll, beforeAll, expect,it } from 'vitest';
import { initSync,experiment_run_json,experiment_validate_json } from '../experiments-pkg/experiments.js';
import { notesArms,notesRoutines } from './notes-comparison';
import { projectCheckpoint,resultPayload } from './projection';
import type { EpisodeRecord } from './types';
let preservedRecord:EpisodeRecord|undefined;
const cwd=fileURLToPath(new URL('../../../',import.meta.url));
const dir=mkdtempSync(`${tmpdir()}/notes-task5-parity-`);
beforeAll(()=>initSync({module:readFileSync(new URL('../experiments-pkg/experiments_bg.wasm',import.meta.url))}),60_000);
afterAll(()=>rmSync(dir,{recursive:true,force:true}));
const input=(arm:string,routine:string,request:string)=>({study:'persistent_notes',stratum:{acquisition_cost:1,write_cost:2,note_read_cost:1,manifest_inspection_cost:4,writer_share:3},arm,routine,request,ids:{writer:'Writer',reader:'Reader'}});
for(const arm of notesArms) for(const routine of notesRoutines) for(const request of ['Left','Right']) it(`${arm}/${routine}/${request} complete native/WASM parity`,()=>{
 const text=JSON.stringify(input(arm,routine,request));writeFileSync(`${dir}/input.json`,text);
 const options={cwd,encoding:'utf8' as const,timeout:60_000,maxBuffer:16*1024*1024};
 const native:EpisodeRecord=JSON.parse(execFileSync(`${cwd}/target/release/sugarscape`,['experiment-view','run','--input',`${dir}/input.json`],options));
 const start=performance.now();const wasm:EpisodeRecord=JSON.parse(experiment_run_json(text));
 expect(performance.now()-start).toBeLessThan(60_000);
 expect(wasm).toEqual(native);
 preservedRecord??=wasm;
 expect(wasm.checkpoints.length).toBeLessThanOrEqual(16);
 expect(resultPayload(wasm,wasm.checkpoints.length-1,{kind:'agent',agent:'Reader'})).toBeNull();
 expect(projectCheckpoint(wasm,0,{kind:'agent',agent:'Reader'}).researcher).toBeNull();
},60_000);
it('fresh import reconstructs results and rejects mutated or invalid wire',()=>{
 const record=JSON.parse(experiment_run_json(JSON.stringify(input('Retained','ReadAdaptive','Left')))) as EpisodeRecord;
 expect(JSON.parse(experiment_validate_json(JSON.stringify(record)))).toEqual(record);
 record.payload=null;expect(()=>experiment_validate_json(JSON.stringify(record))).toThrow();
 const selected=input('Retained','ReadAdaptive','Left');
 expect(()=>experiment_run_json(JSON.stringify({...selected,known_request:'Right'}))).toThrow();
 expect(()=>experiment_run_json(JSON.stringify({...selected,stratum:{...selected.stratum,write_cost:2.5}}))).toThrow();
},60_000);
it('passive receipt updates possession without refreshing the own map and prior snapshots stay owned',()=>{
 const record=JSON.parse(experiment_run_json(JSON.stringify(input('Retained','ReadAdaptive','Right')))) as EpisodeRecord;
 const initial=JSON.stringify(record.checkpoints[0]);
 const receipt=record.checkpoints.find(point=>point.kind!=='terminal' && (point.local.Reader as {view:{key:boolean}}).view.key);
 expect(receipt).toBeDefined();
 const view=(receipt!.local.Reader as {view:{clock:unknown,map:{sensed_at:unknown},key_receipt:unknown}}).view;
 expect(view.key_receipt).not.toBeNull();expect(view.map.sensed_at).not.toEqual(view.clock);
 expect(JSON.stringify(record.checkpoints[0])).toBe(initial);
},60_000);
it('rejects noncanonical selected and saved-input shapes before reconstruction',()=>{
 const selected=input('Retained','ReadAdaptive','Left');
 for(const [field,value] of [['stratum',[1,2,1,4,3]],['ids',['Writer','Reader']],['arm',{Retained:null}],['routine',{ReadAdaptive:null}],['request',{Left:null}]] as const){
  expect(()=>experiment_run_json(JSON.stringify({...selected,[field]:value}))).toThrow();
 }
 // Reuse a planned parity case; saved-input rejects must occur before reconstruction.
 expect(preservedRecord).toBeDefined();
 const rootRecord=preservedRecord!;
 const selectedInput=rootRecord.input as Record<string,unknown>;
 for(const [field,value] of [['stratum',[1,2,1,4,3]],['ids',['Writer','Reader']],['arm',{[String(selectedInput.arm)]:null}],['routine',{[String(selectedInput.routine)]:null}],['request',{[String(selectedInput.request)]:null}]] as const){
  const saved=structuredClone(rootRecord);(saved.input as Record<string,unknown>)[field]=value;
  let errors:unknown;
  try{experiment_validate_json(JSON.stringify(saved));}catch(error){errors=JSON.parse(String(error));}
  expect(errors).toContainEqual({field:'input',message:expect.any(String)});
 }
},60_000);
