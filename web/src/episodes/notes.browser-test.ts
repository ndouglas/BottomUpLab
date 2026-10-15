import { NotesControls } from './notes-controls';
import { renderNotes, notesSceneSvg, notesCaption, notesActions, notesAssets } from './notes-view';
import { notesComicSvg,notesComicHtml,notesPanels } from './notes-comic';
import { projectCheckpoint } from './projection';
import { obj,list,valueText } from './presentation';
import type { StudyDescriptor,EpisodeRecord } from './types';
export function notesControlChecks(descriptor:StudyDescriptor):{name:string;passed:boolean}[]{
 const checks:{name:string;passed:boolean}[]=[],check=(name:string,passed:boolean)=>checks.push({name,passed});let changes=0;
 const controls=new NotesControls(descriptor,descriptor.default_input,()=>changes++),before=JSON.stringify(descriptor.default_input);
 for(const key of ['acquisition_cost','write_cost','note_read_cost','manifest_inspection_cost','writer_share','arm','routine','request']){
  const picker=controls.el.querySelector<HTMLSelectElement>(`[aria-label="Researcher setup ${key.replace(/_/g,' ')}"]`)!;
  check(`${key} exposes native bounded values`,!!picker&&picker.options.length>1);
  picker.value=picker.options[picker.options.length-1].value;picker.dispatchEvent(new Event('change'));
  const nested=key.endsWith('_cost')||key==='writer_share',value=(nested?obj(obj(controls.input()).stratum):obj(controls.input()))[key];
  check(`${key} commits the exact native value`,String(value)===picker.value);
 }
 const field=controls.el.querySelector<HTMLInputElement>('[aria-label="Notes reader Agent ID"]')!;field.value='007 <雪>';field.dispatchEvent(new Event('input'));check('opaque ID stays a string',obj(obj(controls.input()).ids).reader==='007 <雪>');
 check('control edits keep descriptor unchanged',before===JSON.stringify(descriptor.default_input));check('all explicit edits notify session',changes===9);
 const copy=controls.input();obj(copy).arm='Retained';check('input returns an owned copy',obj(controls.input()).arm!=='Retained');
 return checks;
}
/** Runs against actual native records, every frame and both masks, in the real browser DOM. */
export function notesDomChecks(descriptor:StudyDescriptor,records:EpisodeRecord[]):{name:string;passed:boolean}[]{
 const checks:{name:string;passed:boolean}[]=[];
 for(const [n,record]of records.entries()){
  const before=JSON.stringify(record);let masked=true,exact=true,departed=true,selection=true,exported=true;
  for(const at of record.checkpoints)for(const perspective of [{kind:'researcher'} as const,...Object.keys(at.local).map(agent=>({kind:'agent' as const,agent}))]){
   const projected=projectCheckpoint(record,at.index,perspective),root=renderNotes(projected,descriptor),svg=notesSceneSvg(projected),all=root.outerHTML;
   const history=Object.values(projected.local).flatMap(local=>list(obj(obj(local).view).history));
   exact&&=root.textContent!.includes(notesCaption(projected))&&notesActions(projected).every(a=>history.some(f=>JSON.stringify(f)===JSON.stringify(a.feedback))||perspective.kind==='researcher');
   if(obj(at.public).writer_active===false)departed&&=!svg.includes('aria-label="Writer Flump"');
   if(perspective.kind==='agent'){
    const v=obj(obj(projected.local[perspective.agent]).view);masked&&=projected.researcher===null&&root.querySelectorAll('.notes-ledger').length===1&&!all.includes('Researcher original-cohort');
    masked&&=svg.includes(`Known target: ${valueText(v.known_request)}`)&&svg.includes(`Raw note: ${valueText(obj(v.note_read).symbol)}`);
    if(v.known_request===null)masked&&=!svg.includes('Known target: Right')&&!svg.includes('Known target: Left');
   }
  }
  for(const agent of Object.keys(record.checkpoints[0].local)){
   const points=record.checkpoints.map((_,i)=>projectCheckpoint(record,i,{kind:'agent',agent})),comic=notesComicSvg(points,notesAssets,`Agent ${agent}`),html=notesComicHtml(comic);
   selection&&=notesPanels(points).every(p=>!p.point||points.includes(p.point));
   exported&&=!comic.includes(String(obj(record.payload).trace_hash))&&!html.includes('"input":')&&!comic.includes('<metadata')&&!comic.includes('Researcher original-cohort');
   for(const p of notesPanels(points))if(p.point)exported&&=comic.includes(notesCaption(p.point).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;'));
  }
  checks.push({name:`native ${n}: all-frame masks and raw symbols`,passed:masked},{name:`native ${n}: exact native actions/costs`,passed:exact},{name:`native ${n}: departed actors removed`,passed:departed},{name:`native ${n}: projected-only panel selection/export`,passed:selection&&exported},{name:`native ${n}: source record unchanged`,passed:before===JSON.stringify(record)});
 }
 for(const left of records.filter(r=>obj(r.input).request==='Left')){
  const right=records.find(r=>obj(r.input).request==='Right'&&JSON.stringify({...obj(r.input),request:'Left'})===JSON.stringify(left.input));
  if(!right)continue;
  for(const agent of Object.keys(left.checkpoints[0].local)){
   let invariant=true,count=0;
   for(const at of left.checkpoints){const other=right.checkpoints.find(p=>JSON.stringify(p.clock)===JSON.stringify(at.clock));if(!other)continue;
    const lv=obj(obj(at.local[agent]).view),rv=obj(obj(other.local[agent]).view);
    if(lv.known_request===null&&rv.known_request===null){count++;invariant&&=notesSceneSvg(projectCheckpoint(left,at.index,{kind:'agent',agent}))===notesSceneSvg(projectCheckpoint(right,other.index,{kind:'agent',agent}));}
   }
   checks.push({name:`${String(obj(left.input).arm)}/${String(obj(left.input).routine)}/${agent}: ${count} pre-reveal SVGs invariant across H`,passed:invariant});
  }
 }
 return checks;
}
