import { EpisodeView } from './view';
import { createEpisodeClient } from './client';
import { EpisodeControls } from './controls';
import { renderDescriptor } from './catalog';
import { renderGate } from './gate-view';
import { gateDisplay } from './gate-projection';
import { projectCheckpoint, resultPayload } from './projection';
import { canonical } from './comparison';
import { obj, list, valueText } from './presentation';
import { decodeEpisode } from './share';
import type { EpisodeRecord, StudyDescriptor, Json, WorkerRequest, WorkerReply } from './types';
export interface GateBrowserCheck { name: string; passed: boolean }
/** Only complete native/WASM exports enter these masks; never manufactured Agent Views. */
export function gateDomChecks(descriptor: StudyDescriptor, records: EpisodeRecord[]): GateBrowserCheck[] {
  const checks: GateBrowserCheck[] = [];
  for (const [recordIndex, record] of records.entries()) {
    const before = canonical(record as unknown as Json);
    let masks = true, positives = true, unavailable = true, dates = true;
    for (const at of record.checkpoints) {
      for (const agent of [...Object.keys(at.local), 'public-only']) {
        const projected = projectCheckpoint(record, at.index, {kind:'agent',agent});
        const display = gateDisplay(projected), rendered = renderGate(projected,renderDescriptor(descriptor));
        const text = rendered.textContent!;
        const source = obj(obj(at.local[agent]).view), word = list(obj(source.word)[String(source.role)]);
        masks &&= projected.researcher === null && resultPayload(record,at.index,{kind:'agent',agent}) === null && display.agents.length === 1 && display.agents[0].id === agent && !rendered.querySelector('.gate-researcher') && !text.includes('classification') && !text.includes('unilateral regret') && rendered.querySelectorAll('.gate-word li').length === word.length;
        for (const [index, token] of word.entries()) masks &&= rendered.querySelectorAll('.gate-word li')[index]?.textContent === `Round ${index+1}: ${valueText(token)}`;
        unavailable &&= agent !== 'public-only' || text.includes('Local state unavailable');
        dates &&= agent === 'public-only' || canonical(display.agents[0].map.observed_at) === canonical(obj(obj(at.local[agent]).map).observed_at);
      }
      const research = renderGate(projectCheckpoint(record,at.index,{kind:'researcher'}),renderDescriptor(descriptor));
      positives &&= !!research.querySelector('.gate-researcher') && research.textContent!.includes('Native compiled classification') && resultPayload(record,at.index,{kind:'researcher'}) === record.payload;
    }
    checks.push({name:`native record ${recordIndex}: all ${record.checkpoints.length} checkpoints / both Agents / public-only privacy`,passed:masks&&unavailable}, {name:`native record ${recordIndex}: Researcher positives and dated maps`,passed:positives&&dates}, {name:`native record ${recordIndex}: projection and DOM never mutate source`,passed:canonical(record as unknown as Json)===before});
  }
  return checks;
}
const button = (view: EpisodeView,name:string) => [...view.el.querySelectorAll('button')].find(b=>b.textContent===name)!;
async function idle(view: EpisodeView): Promise<void> {
  const deadline = performance.now()+65_000;
  while (view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled) {
    if(performance.now()>deadline) throw new Error('Gate browser operation did not settle within 65 seconds');
    await new Promise(resolve=>setTimeout(resolve,25));
  }
}
/** Actual worker bodies stay production bytes. Captures page request/reply packets, not worker network bodies. */
export async function gateViewChecks(records: EpisodeRecord[]): Promise<{checks:GateBrowserCheck[];packets:{request:WorkerRequest;reply:WorkerReply|null}[];records:EpisodeRecord[];share:Json|null;lifecycle:Record<string,number>}> {
  const checks:GateBrowserCheck[]=[], packets:{request:WorkerRequest;reply:WorkerReply|null}[]=[],actual:EpisodeRecord[]=[];
  const check=(name:string,passed:boolean)=>checks.push({name,passed});
  let live=0,peak=0,created=0,terminated=0;
  const client=createEpisodeClient(()=>{
    const worker=new Worker(new URL('./worker.ts',import.meta.url),{type:'module'});live++;created++;peak=Math.max(peak,live);
    const post=worker.postMessage.bind(worker),terminate=worker.terminate.bind(worker);
    worker.postMessage=(request:WorkerRequest)=>{const packet={request:structuredClone(request),reply:null as WorkerReply|null};packets.push(packet);worker.addEventListener('message',(event:MessageEvent<WorkerReply>)=>{packet.reply=structuredClone(event.data);});post(request);};
    worker.terminate=()=>{live--;terminated++;terminate();};return worker;
  });
  const view=new EpisodeView(client);document.body.append(view.el);let share:Json|null=null;
  const originalWrite=navigator.clipboard.writeText;const copied:string[]=[];
  try {
    await view.ready;const catalog=await client.catalog(),descriptor=catalog.find(d=>d.id==='gate_assistance')!;
    check('current native catalog enables Gate renderer',view.el.querySelector('[aria-label="Episode study"]')!.textContent!.includes(descriptor.title));
    for(const expected of records) {
      view.openInput(expected.input);button(view,'Run episode').click();await idle(view);
      const record=view.session.shown!;actual.push(structuredClone(record));check(`real worker ${valueText(obj(expected.input).profile)} equals complete native export`,canonical(record as unknown as Json)===canonical(expected as unknown as Json));
      const before=canonical(record as unknown as Json),perspective=view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
      let privacy=true,positive=true;
      for(const at of record.checkpoints){view.replay.seek(at.index);for(const agent of Object.keys(at.local)){perspective.value=`agent:${agent}`;perspective.dispatchEvent(new Event('change'));const rendered=view.el.querySelector('.episode-rendered')!;privacy&&=!rendered.querySelector('.gate-researcher')&&!rendered.textContent!.includes('unilateral regret')&&!rendered.textContent!.includes('Complete native episode')&&rendered.querySelectorAll('.gate-agent').length===1;}
        perspective.value='researcher';perspective.dispatchEvent(new Event('change'));positive&&=!!view.el.querySelector('.gate-researcher')&&view.el.querySelector('.episode-rendered')!.textContent!.includes('Pure equilibrium (native classification)');}
      check('actual mounted all-checkpoint both-Agent privacy and Researcher positives',privacy&&positive);check('actual mounted perspective / seek does not mutate native record',canonical(record as unknown as Json)===before);
    }
    const original=view.session.shown!,retained=[...view.session.records];
    const exact=await view.session.export();check('actual export validates complete native record',canonical(JSON.parse(exact))===canonical(original as unknown as Json));
    const file=view.el.querySelector<HTMLInputElement>('input[type=file]')!,transfer=new DataTransfer();transfer.items.add(new File([exact],'gate-qa-native.json',{type:'application/json'}));file.files=transfer.files;file.dispatchEvent(new Event('change'));await idle(view);check('actual file import retains exact validated native record',canonical(view.session.shown as unknown as Json)===canonical(original as unknown as Json));
    const successful=view.session.shown!,saved=[...view.session.records];const duplicate=exact.replace('"version":1','"version":1,"version":1'),bad=new DataTransfer();bad.items.add(new File([duplicate],'gate-qa-duplicate.json',{type:'application/json'}));file.files=bad.files;file.dispatchEvent(new Event('change'));await idle(view);check('duplicate raw file rejects without replacing successful records',view.session.shown===successful&&saved.every((r,i)=>r===view.session.records[i]));
    navigator.clipboard.writeText=async text=>{copied.push(text);};
    view.openInput({...obj(records[0].input),profile:{holder:0,retriever:0}});
    const holder=view.el.querySelector<HTMLInputElement>('[aria-label="Raw holder word ID"]')!,retriever=view.el.querySelector<HTMLInputElement>('[aria-label="Raw retriever word ID"]')!;
    holder.value='540';retriever.value='15';holder.dispatchEvent(new Event('input'));retriever.dispatchEvent(new Event('input'));
    button(view,'Share input link').click();const limit=performance.now()+5000;while(!copied.length&&performance.now()<limit)await new Promise(r=>setTimeout(r,10));share=copied.length?await decodeEpisode(copied[0].split('#e=')[1]):null;check('Share captures unblurred raw IDs',obj(obj(share).profile).holder===540&&obj(obj(share).profile).retriever===15);
    button(view,'Run episode').click();await idle(view);check('Run captures unblurred raw IDs in actual packet and native reply',obj(obj(view.session.shown!.input).profile).holder===540&&obj(obj(view.session.shown!.input).profile).retriever===15);
    const raw=view.session.shown!;holder.value='1e2';holder.dispatchEvent(new Event('input'));const count=packets.length,copyCount=copied.length;button(view,'Run episode').click();await idle(view);button(view,'Share input link').click();await new Promise(r=>setTimeout(r,20));check('invalid raw input cannot create worker or Share or replace shown records',packets.length===count&&copied.length===copyCount&&view.session.shown===raw);
    view.openInput(records[0].input);button(view,'Run episode').click();await idle(view);const near=view.session.shown!;
    const compare=view.el.querySelector<HTMLSelectElement>('[aria-label="Matched policy"]')!;compare.value='0';button(view,'Run matched comparison').click();await idle(view);check('sequential native comparison retains exactly two records',view.session.records.length===2&&view.session.shown===near);
    const perspective=view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;perspective.value='agent:Agent-A';perspective.dispatchEvent(new Event('change'));view.replay.seek(near.checkpoints.length-1);check('Agent terminal comparison has no compiled classification or evaluator payload',!view.el.querySelector('.episode-comparison')!.textContent!.includes('unilateral regret')&&!view.el.querySelector('.episode-comparison')!.textContent!.includes('Complete native episode'));
    check('exact missing terminal round / stage stays unavailable',view.el.querySelector('.episode-comparison')!.textContent!.includes('Unavailable at this public clock and stage'));
    const comparisonSeek=view.el.querySelector<HTMLInputElement>('[aria-label="Comparison checkpoint"]')!;comparisonSeek.value=comparisonSeek.max;comparisonSeek.dispatchEvent(new Event('input'));
    perspective.value='researcher';perspective.dispatchEvent(new Event('change'));check('Researcher comparison has complete native classification',view.el.querySelector('.episode-comparison')!.textContent!.includes('Pure equilibrium (native classification)'));
    const shown=view.session.shown!;button(view,'Run episode').click();button(view,'Cancel').click();await idle(view);check('actual pending cancel retains successful native record',view.session.shown===shown&&view.replay.record===shown);
    button(view,'Run episode').click();await idle(view);check('fresh production worker succeeds after cancel',view.session.shown!==shown&&view.session.shown?.study==='gate_assistance');
    const preTimeout=view.session.shown!;await fetch('/__gate-delay-next',{method:'POST'});const start=performance.now();button(view,'Run episode').click();await idle(view);const elapsed=performance.now()-start;
    check('actual 60-second initialization timeout retains records (production bytes availability delay)',elapsed>=60_000&&elapsed<65_000&&view.session.shown===preTimeout&&view.el.querySelector(':scope > .row > [role=status]')!.textContent!.includes('60 seconds'));
    button(view,'Run episode').click();await idle(view);check('fresh production worker succeeds after initialization timeout',view.session.shown!==preTimeout);
    button(view,'Run episode').click();view.dispose();await new Promise(r=>setTimeout(r,20));check('actual pending dispose releases native records and active worker',view.session.records.length===0&&view.session.shown===null&&view.replay.record===null&&live===0);
    check('worker lifecycle has at most one outstanding instance and no leaks',peak===1&&created===terminated&&live===0);
    check('pre-error retained records were native complete',retained.length===2&&original.study==='gate_assistance');
    const controls=new EpisodeControls(descriptor,{...obj(descriptor.default_input),selector:null});check('malformed opened choice is retained for native validation',obj(controls.input()).selector===null);
    return {checks,packets,records:actual,share,lifecycle:{peak,created,terminated,live}};
  } finally { navigator.clipboard.writeText=originalWrite;view.dispose();view.el.remove(); }
}
