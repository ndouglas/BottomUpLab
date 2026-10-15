import type { Checkpoint } from './types';
import { obj, valueText } from './presentation';
import { notesActions,notesCaption,notesClock,notesSceneSvg,escapeSvg,notesAssets,notesAssetNames,type NotesAssets } from './notes-view';
/** Selection occurs after projection: private peer events cannot select panels. */
export function notesPanels(points:readonly Checkpoint[]):{title:string;point:Checkpoint|null}[]{
 const action=(p:Checkpoint,kinds:string[])=>notesActions(p).some(a=>kinds.includes(String(obj(a.feedback.primitive).kind)));
 const find=(predicate:(p:Checkpoint)=>boolean)=>points.find(predicate)??null;
 return [
  {title:'Acquisition',point:find(p=>action(p,['InspectManifest'])&&obj(p.clock).phase==='Writer')},
  {title:'Handoff / write',point:[...points].reverse().find(p=>action(p,['WriteNote','PassKey']))??null},
  {title:'Departure',point:find(p=>obj(p.clock).phase==='Departure')},
  {title:'Reader information action',point:find(p=>obj(p.clock).phase==='Reader'&&action(p,['ReadNote','InspectManifest']))},
  {title:'Pickup',point:find(p=>action(p,['Pickup']))},
  {title:'Settlement',point:find(p=>obj(p.public).terminal===true&&obj(p.clock).phase==='Terminal')},
 ];
}
function lines(text:string,width=92):string[]{const out:string[]=[];let line='';for(const word of text.split(' ')){if((line+' '+word).length>width){out.push(line);line=word;}else line+=(line?' ':'')+word;}if(line)out.push(line);return out;}
/** No metadata, raw record/input/result or hidden-state digest is embedded. */
export function notesComicSvg(points:readonly Checkpoint[],assets:NotesAssets,perspectiveLabel:string):string{
 let svg=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 1900" font-family="Georgia, serif" fill="#263e42"><rect width="1600" height="1900" fill="white"/><text x="24" y="32" font-size="22">Paid persistent notes · ${escapeSvg(perspectiveLabel)}</text><text x="24" y="55" font-size="14">Native committed boundaries; unavailable events are not reconstructed. Studio decorative assets v1.</text>`;
 for(const [i,panel]of notesPanels(points).entries()){
  const x=20+(i%2)*790,y=78+Math.floor(i/2)*580;svg+=`<g transform="translate(${x} ${y})"><text x="8" y="20" font-size="18">${i+1}. ${escapeSvg(panel.title)}</text>`;
  if(panel.point){svg+=`<g transform="translate(0 35)">${notesSceneSvg(panel.point,assets)}</g>`;let row=0;for(const text of [notesClock(panel.point.clock),notesCaption(panel.point),...Object.entries(panel.point.local).map(([id,local])=>{const v=obj(obj(local).view);return `${id}: own credits ${valueText(v.credits)}; spent ${valueText(v.spent)}; reward ${valueText(v.reward)}; program ${valueText(v.writer_program??v.reader_routine)}`;})])for(const line of lines(text))svg+=`<text x="8" y="${400+row++*19}" font-size="13">${escapeSvg(line)}</text>`;}
  else svg+='<text x="8" y="90" font-size="16">Unavailable in the projected native record.</text>';
  svg+='</g>';
 }
 return svg+'<text x="24" y="1850" font-size="12">Native source SHA-256: 28b90c51a6f8d126eae517f1292a32518207c888ff95f60abd9c2b45ee3c572e</text><text x="24" y="1872" font-size="12">Reviewed reference SHA-256: 2227ad21600c94e54245bd2743509b6aa65399f2f8eb86693c1aecd483225788</text></svg>';
}
export function notesComicHtml(svg:string):string{return `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Paid persistent notes · projected comic</title><style>body{margin:0}svg{display:block;width:100%;height:auto}@media print{svg{max-height:98vh}}</style><body>${svg}</body></html>`;}
export async function embeddedNotesAssets():Promise<NotesAssets>{return Object.fromEntries(await Promise.all(notesAssetNames.map(async name=>{
 const response=await fetch(notesAssets[name]);if(!response.ok)throw new Error(`Unable to load decorative asset ${name}`);
 const data=new Uint8Array(await response.arrayBuffer());let binary='';for(const byte of data)binary+=String.fromCharCode(byte);
 return [name,`data:image/png;base64,${btoa(binary)}`];
})));}
