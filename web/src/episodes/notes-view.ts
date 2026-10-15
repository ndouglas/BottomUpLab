import { h } from '../ui/dom';
import { obj, list, fact, details, valueText } from './presentation';
import type { Checkpoint, Json } from './types';
import type { RenderDescriptor } from './catalog';
/** Decorative image URLs/data URLs only; no input or episode identity belongs here. */
export type NotesAssets = Readonly<Record<string,string>>;
export const notesAssetNames = ['flump-a-empty.png','flump-a-pencil.png','flump-a-magnifier.png','flump-b-empty.png','flump-b-magnifier.png','shared-board-blank.png','flump-giver-key.png','flump-receiver-key.png','flump-receiver-key-coin.png','gate-closed.png'] as const;
export const notesAssets:NotesAssets=Object.fromEntries(notesAssetNames.map(name=>[name,`${import.meta.env.BASE_URL}flumps/persistent-notes-v1/${name}`]));
export const escapeSvg=(value:unknown):string=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&apos;'}[c]!));
export const notesClock=(clock:Json|undefined):string=>{const c=obj(clock);return `boundary ${valueText(c.boundary)} / ${valueText(c.phase)} / Writer ${valueText(c.writer_slot)} / Reader ${valueText(c.reader_slot)}`;};
/** A stale final history entry is not a new action. */
export function notesActions(point:Checkpoint):{role:string;feedback:Record<string,Json>}[]{
 if(point.researcher!==null){const event=obj(obj(point.researcher).event);return event.feedback? [{role:valueText(event.actor),feedback:obj(event.feedback)}]:[];}
 return Object.values(point.local).flatMap(local=>{const v=obj(obj(local).view);return list(v.history).map(obj).filter(f=>notesClock(f.clock)===notesClock(point.clock)).map(feedback=>({role:valueText(v.role),feedback}));});
}
export function notesCaption(point:Checkpoint):string {
 const actions=notesActions(point).map(({role,feedback:f})=>`${role} ${valueText(obj(f.primitive).kind)} (${valueText(f.status)}); cost ${valueText(f.cost)}; reward ${valueText(f.reward)}${f.read_symbol!==null&&f.read_symbol!==undefined?`; read ${valueText(f.read_symbol)}`:''}${f.observed_request?`; observed target ${valueText(f.observed_request)}`:''}${f.picked_good?`; picked ${valueText(f.picked_good)}`:''}${obj(f.primitive).note?`; wrote ${valueText(obj(f.primitive).note)}`:''}`);
 if(actions.length)return actions.join(' | ');
 return obj(point.clock).phase==='Departure'?'Writer departure (public boundary)':obj(point.public).terminal?`Settlement: ${valueText(obj(point.public).terminal_reason)}`:point.kind==='initial'?'Initial committed boundary':'Private action unavailable in this perspective';
}
/** Every visible atom comes from this already projected checkpoint. Never accepts Config/raw record. */
export function notesSceneSvg(point:Checkpoint,assets:NotesAssets=notesAssets):string {
 const privileged=point.researcher!==null,physical=obj(obj(point.researcher).physical),own=obj(obj(Object.values(point.local)[0]).view),status=obj(point.public),map=obj(own.map),cells=list(map.cells).map(obj);
 const text=(x:number,y:number,t:string,size=13)=>`<text x="${x}" y="${y}" font-size="${size}">${escapeSvg(t)}</text>`;
 const image=(name:string,x:number,y:number,w:number,height:number,aria='')=>assets[name]?`<image href="${escapeSvg(assets[name])}" x="${x}" y="${y}" width="${w}" height="${height}"${aria?` aria-label="${escapeSvg(aria)}"`:''}/>`:'';
 let out=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 780 340" width="780" height="340" role="img" aria-label="${privileged?'Researcher physical map':'Own position and dated local map'}" font-family="Georgia, serif" fill="#263e42"><rect width="780" height="340" rx="12" fill="#edf5f5"/>`;
 out+=text(18,23,privileged?'Researcher: current physical state':`Dated map: ${notesClock(map.sensed_at)}`,12)+text(18,44,`Public ${notesClock(point.clock)}`,12);
 const positions=privileged?list(physical.participants).map(obj).filter(p=>p.active):own.cell&&status[own.role==='Writer'?'writer_active':'reader_active']!==false?[own]:[];
 for(const [i,cell] of ['P','O','C'].entries()){
  const x=125+i*260,row=cells.find(r=>r.cell===cell);
  out+=`<path d="M${x-100} 250 H${x+100}" stroke="#8aacae" stroke-width="3"/>`+text(x-12,275,cell,20)+text(x-78,296,({P:'Unused staging',O:'Origin / note / delivery',C:'Goods / manifest'}[cell]!),13);
  if(!privileged)out+=text(x-104,317,row?`Dated b${valueText(obj(row.sensed_at).boundary)} occupants: ${list(row.occupants).map(valueText).join(', ')||'none'}`:'Unobserved',11);
  const goods=privileged?(cell==='C'?list(physical.goods).flatMap((g,j)=>g?[j===0?'Left':'Right']:[]):[]):list(row?.goods).map(valueText);
  if(cell==='C')for(const [j,good]of goods.entries())out+=`<rect x="${x-70+j*78}" y="91" width="68" height="33" rx="${good==='Left'?0:15}" fill="${good==='Left'?'#c9a0be':'#87bcc4'}"/>`+text(x-65+j*78,113,good);
  if(cell==='O')out+=image('shared-board-blank.png',x-56,60,112,90);
  if(i<2)out+=text(x+106,269,i===0?'P–O':'O–C',11);
 }
 out+=image('gate-closed.png',471,155,54,50)+text(464,226,'Carrier only',11);
 const raw=privileged?physical.field:obj(own.note_read).symbol,request=privileged?physical.request:own.known_request;
 out+=text(280,162,`Raw note: ${valueText(raw)}`,12)+text(20,68,`${privileged?'Target card':'Known target'}: ${valueText(request)}`,12);
 if(!privileged&&own.note_read)out+=text(273,179,`Read at boundary ${valueText(obj(obj(own.note_read).clock).boundary)}`,10);
 if(Object.values(point.local).some(local=>obj(obj(local).view).reader_routine==='Oracle'))out+=text(20,86,'Oracle: supplied target knowledge',12);
 for(const p of positions){
  const i=['P','O','C'].indexOf(String(p.cell));if(i<0)continue;
  const role=valueText(p.role),x=125+i*260+(privileged&&positions.filter(q=>q.cell===p.cell).length>1?(role==='Writer'?-45:45):0),action=notesActions(point).find(a=>a.role===role),kind=obj(action?.feedback.primitive).kind;
  const tool=kind==='InspectManifest'||kind==='ReadNote'?'magnifier':kind==='WriteNote'?'pencil':null;
  const name=tool?`flump-${role==='Writer'?'a':'b'}-${tool}.png`:p.cargo&&role==='Reader'?'flump-receiver-key-coin.png':p.key?`flump-${role==='Writer'?'giver':'receiver'}-key.png`:`flump-${role==='Writer'?'a':'b'}-empty.png`;
  const emptyA=name==='flump-a-empty.png',emptyB=name==='flump-b-empty.png';
  const width=tool?720:emptyB?360:480,height=emptyB?288:400,footX=tool?360:emptyA?240:emptyB?179:212,footY=tool?337.3188257217407:emptyA?357:emptyB?273:337;
  const scale=(emptyA?.181:.208)*.65;
  out+=image(name,x-footX*scale,250-footY*scale,width*scale,height*scale,`${role} Flump`);
  if(kind==='InspectManifest'){const cardX=x+(role==='Writer'?62:-100);out+=`<rect x="${cardX}" y="180" width="46" height="33" fill="#fff" stroke="#345a61"/>`+text(cardX+3,201,valueText(request),10);}
  out+=text(x-34,196,`${role}${p.key?' · key':''}`,11);
  if(p.cargo)out+=text(x+38,244,`Cargo ${valueText(p.cargo)}`,11);
 }
 if(status.writer_active===false)out+=text(18,332,'Writer departed; dated observations are not present actors.',11);
 return out+'</svg>';
}
export function renderNotes(point:Checkpoint,_descriptor:RenderDescriptor):HTMLElement{
 const root=h('div',{class:'notes-view'}),scene=h('div',{class:'notes-map'});scene.innerHTML=notesSceneSvg(point);root.append(scene,h('p',{class:'notes-caption'},notesCaption(point)),h('p',{class:'hint'},'The O–C barrier permits only its key carrier. Decorative tools show the committed action; playback adds no observations. Target card, raw note and Left/Right goods are distinct.'));
 for(const [id,local]of Object.entries(point.local)){
  const v=obj(obj(local).view),panel=h('section',{class:'notes-ledger'},h('h3',{},`Agent ${id} · ${valueText(v.role)}`));
  if(!v.role){panel.append(h('p',{},'Local state unavailable.'));root.append(panel);continue;}
  if(v.reader_routine==='Oracle')panel.append(h('p',{class:'notes-oracle'},'Oracle: target knowledge is supplied explicitly.'));
  for(const [title,val]of Object.entries({'Own clock':notesClock(v.clock),'Own current cell':v.cell,'Own key':v.key,'Own cargo':v.cargo,'Own credits':v.credits,'Own spent':v.spent,'Own reward':v.reward,'Known target':v.known_request,'Own belief Left / Right':v.belief,'Dated raw note read':v.note_read,'Passive key receipt':v.key_receipt,'Own program':v.writer_program??v.reader_routine}))panel.append(fact(title,val));
  panel.append(details('Own public costs and settings',v.public),details('Own dated map (not current peer positions)',v.map),h('details',{},h('summary',{},`Committed own history (${list(v.history).length})`),h('ol',{},...list(v.history).map(f=>h('li',{},`${notesClock(obj(f).clock)}: ${valueText(f)}`)))));root.append(panel);
 }
 if(point.researcher!==null)root.append(details('Researcher original-cohort physical ledger and event',point.researcher));
 return root;
}
