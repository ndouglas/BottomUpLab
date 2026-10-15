import { h } from '../ui/dom';
import { obj,list,label,valueText } from './presentation';
import type { Json,StudyDescriptor } from './types';
/** Hidden request edits are researcher setup, outside the masked Agent projection. */
export class NotesControls {
 readonly el=h('div',{class:'notes-controls'});
 private draft:Record<string,Json>;
 constructor(descriptor:StudyDescriptor,input:Json,changed:(input:Json)=>void){
  this.draft=structuredClone(obj(input));
  this.el.append(h('p',{class:'hint'},'Researcher setup · hidden request and treatment. Oracle receives supplied truth; other Agents must pay for permitted observations.'));
  for(const key of ['acquisition_cost','write_cost','note_read_cost','manifest_inspection_cost','writer_share','arm','routine','request']){
   const nested=key.endsWith('_cost')||key==='writer_share';
   const values=list(obj(descriptor.controls)[nested?key:key==='arm'?'arms':key==='routine'?'routines':'requests']);
   const selected=(nested?obj(this.draft.stratum):this.draft)[key];
   const picker=h('select',{'aria-label':`Researcher setup ${label(key)}`},...values.map(value=>h('option',{value:String(value),selected:value===selected},valueText(value))));
   if(!values.includes(selected)) picker.prepend(h('option',{value:'',selected:true,disabled:true},'Opened value · validation on run'));
   picker.addEventListener('change',()=>{const value=values.find(v=>String(v)===picker.value);if(value!==undefined){if(nested)this.draft.stratum={...obj(this.draft.stratum),[key]:value};else this.draft[key]=value;}changed(this.input());});
   this.el.append(h('label',{},`Researcher ${label(key)}`,picker));
  }
  for(const role of ['writer','reader']){
   const field=h('input',{type:'text',value:obj(this.draft.ids)[role]??'','aria-label':`Notes ${role} Agent ID`,maxLength:32});
   field.addEventListener('input',()=>{this.draft.ids={...obj(this.draft.ids),[role]:field.value};changed(this.input());});
   this.el.append(h('label',{},`${label(role)} Agent ID`,field));
  }
 }
 input():Json{return structuredClone(this.draft);}
}
