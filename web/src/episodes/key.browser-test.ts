import { EpisodeControls } from './controls';
import { obj } from './presentation';
import type { StudyDescriptor } from './types';
export interface KeyBrowserCheck { name: string; passed: boolean }
export function keyControlChecks(descriptor: StudyDescriptor): KeyBrowserCheck[] {
  const checks: KeyBrowserCheck[] = [];
  const check = (name: string, passed: boolean) => checks.push({ name, passed });
  const controls = new EpisodeControls(descriptor);
  const find = <T extends HTMLElement>(name: string) => controls.el.querySelector<T>(`[aria-label="${name}"]`);
  const permission = find<HTMLSelectElement>('Key transfer enabled');
  check('permission exposes enabled and disabled without disabling paid attempts', !!permission && [...permission.options].map(o => o.value).join(',') === 'false,true');
  if (permission) { permission.value = 'false'; permission.dispatchEvent(new Event('change')); check('disabled is the Boolean false', obj(obj(controls.input()).stratum).transfer_enabled === false); }
  const picker = find<HTMLSelectElement>('Key schedule selection');
  check('changed starting ownership is explicit in selector label', !!picker && picker.textContent!.includes('Changed initial ownership'));
  if (picker) {
    picker.value = 'raw'; picker.dispatchEvent(new Event('change'));
    check('raw removes selector', 'profile' in obj(controls.input()) && !('selector' in obj(controls.input())));
    for (const role of ['giver', 'receiver']) {
      const field = find<HTMLInputElement>(`Raw ${role} word ID`)!;
      for (const bad of ['', 'true', '0x10', '1.5', '01', ' 1', '1e1', '-1', role === 'giver' ? '729' : '64']) {
        field.value = bad; field.dispatchEvent(new Event('input'));
        let rejected = false; try { controls.input(); } catch { rejected = true; }
        check(`${role} rejects current text ${JSON.stringify(bad)}`, rejected); field.value = '0'; field.dispatchEvent(new Event('input'));
      }
      field.value = role === 'giver' ? '728' : '63';
      check(`${role} captures unblurred current text`, obj(obj(controls.input()).profile)[role] === Number(field.value));
    }
    const name = find<HTMLInputElement>('Key giver Agent ID')!; name.value = '007 ❄'; name.dispatchEvent(new Event('input'));
    check('opaque names stay strings', obj(obj(controls.input()).ids).giver === '007 ❄');
    const current = find<HTMLSelectElement>('Key schedule selection')!; current.value = 'InitiallyEquipped'; current.dispatchEvent(new Event('change'));
    check('selector removes raw pair and retains opaque name', !('profile' in obj(controls.input())) && obj(controls.input()).selector === 'InitiallyEquipped' && obj(obj(controls.input()).ids).giver === '007 ❄');
  }
  for (const malformed of [null, true, 'invalid']) {
    const input = {...obj(descriptor.default_input)}; delete input.selector; input.profile = malformed; input.ids = malformed;
    const repair = new EpisodeControls(descriptor,input);
    check(`untouched malformed values remain for native rejection ${JSON.stringify(malformed)}`, obj(repair.input()).profile === malformed && obj(repair.input()).ids === malformed);
    for (const [name,value] of [['Raw giver word ID','728'],['Raw receiver word ID','63'],['Key giver Agent ID','007'],['Key receiver Agent ID','雪']] as const) {
      const field=repair.el.querySelector<HTMLInputElement>(`[aria-label="${name}"]`)!; field.value=value; field.dispatchEvent(new Event('input'));
    }
    check(`explicit edits repair malformed objects ${JSON.stringify(malformed)}`, JSON.stringify(obj(repair.input()).profile)==='{"giver":728,"receiver":63}' && JSON.stringify(obj(repair.input()).ids)==='{"giver":"007","receiver":"雪"}');
  }
  const malformedIds = {...obj(descriptor.default_input), profile: {giver:0,receiver:0}, ids:null};
  delete (malformedIds as Record<string,import('./types').Json>).selector;
  const switcher = new EpisodeControls(descriptor,malformedIds);
  const rawField = switcher.el.querySelector<HTMLInputElement>('[aria-label="Raw giver word ID"]')!;
  rawField.value=''; rawField.dispatchEvent(new Event('input'));
  const renamed = switcher.el.querySelector<HTMLInputElement>('[aria-label="Key giver Agent ID"]')!;
  renamed.value='007 ❄'; renamed.dispatchEvent(new Event('input'));
  const selection = switcher.el.querySelector<HTMLSelectElement>('[aria-label="Key schedule selection"]')!;
  selection.value='SuppliedHandoff'; selection.dispatchEvent(new Event('change'));
  check('switching away from invalid raw text retains edited opaque name in malformed opened ids',obj(obj(switcher.input()).ids).giver==='007 ❄');
  return checks;
}

import { keyDisplay } from './key-projection';
import { renderKey, renderKeyResult } from './key-view';
import { projectCheckpoint, resultPayload } from './projection';
import { renderDescriptor } from './catalog';
import { list, valueText } from './presentation';
import { canonical } from './comparison';
import type { EpisodeClient, EpisodeRecord, Json } from './types';
export function keyDomChecks(descriptor: StudyDescriptor, records: EpisodeRecord[]): KeyBrowserCheck[] {
  const checks: KeyBrowserCheck[] = [];
  for (const [i, record] of records.entries()) {
    const before = canonical(record as unknown as Json);
    let privacy = true, words = true, dates = true, receipts = true, researcher = true;
    for (const at of [...record.checkpoints].reverse()) {
      for (const agent of [...Object.keys(at.local), 'public-only']) {
        const projected = projectCheckpoint(record, at.index, {kind:'agent',agent});
        const display = keyDisplay(projected), rendered = renderKey(projected,renderDescriptor(descriptor));
        const source = obj(obj(at.local[agent]).view), ownWord = list(obj(source.word)[String(source.role)]);
        privacy &&= resultPayload(record,at.index,{kind:'agent',agent}) === null && display.researcher === null && display.agents.length === 1 && display.agents[0].id === agent && !rendered.querySelector('.key-researcher');
        words &&= rendered.querySelectorAll('.key-word li').length === ownWord.length;
        for (const [n,token] of ownWord.entries()) words &&= rendered.querySelectorAll('.key-word li')[n]?.textContent === `Round ${n+1}: ${valueText(token)}`;
        dates &&= agent === 'public-only' ? rendered.textContent!.includes('Local state unavailable') : canonical(display.agents[0]?.observation ?? null) === canonical(obj(at.local[agent]).map);
        receipts &&= rendered.querySelectorAll('.key-receipts li').length === list(source.history).filter(row=>'Ownership' in obj(row)).length;
      }
      const rendered = renderKey(projectCheckpoint(record,at.index,{kind:'researcher'}),renderDescriptor(descriptor));
      researcher &&= !!rendered.querySelector('.key-researcher') && rendered.textContent!.includes('Carrier-only access');
    }
    checks.push({name:`record ${i}: every checkpoint Agent/peer/public-only privacy`,passed:privacy&&words}, {name:`record ${i}: dated maps and explicit receipt history`,passed:dates&&receipts}, {name:`record ${i}: researcher physical access and read-only backward replay`,passed:researcher&&canonical(record as unknown as Json)===before});
    if (obj(record.input).selector === 'InitiallyEquipped') checks.push({name:'InitiallyEquipped prominently labels changed ownership and unavailable main classification',passed:renderKeyResult(record.payload).textContent!.includes('Changed initial ownership')&&renderKeyResult(record.payload).textContent!.includes('Main-game regrets and equilibrium classification: Unavailable')});
  }
  return checks;
}

/** Mounted file/editor regression using complete retained records at the native transport boundary. */
export async function keyShownControlChecks(descriptor: StudyDescriptor, records: EpisodeRecord[]): Promise<KeyBrowserCheck[]> {
  const checks: KeyBrowserCheck[] = [];
  const check = (name: string, passed: boolean) => checks.push({ name, passed });
  const equipped = records.find(record => obj(record.input).selector === 'InitiallyEquipped')!;
  const main = records.find(record => obj(record.input).selector === 'SuppliedHandoff')!;
  if (!equipped || !main) throw new Error('Shown-control checks require complete retained InitiallyEquipped and SuppliedHandoff records');
  const before = canonical(records as unknown as Json);
  const client: EpisodeClient = {
    catalog: async () => [descriptor], recorded: async () => null,
    request: async (op, text) => {
      const record = op === 'validate' && records.find(record => canonical(record as unknown as Json) === canonical(JSON.parse(text)));
      if (!record) throw new Error('Shown-control regression accepts only retained complete files, without new native execution');
      return structuredClone(record);
    },
    cancel: () => {}, dispose: () => {},
  };
  const view = new EpisodeView(client);
  document.body.append(view.el);
  const note = () => view.el.querySelector('.episode-shown-heading + p')!.textContent!;
  const identified = () => note().includes('Changed initial ownership: Receiver starts with the key') && note().includes('Main-game regrets and equilibrium classification are unavailable');
  const importRecord = async (record: EpisodeRecord) => {
    const file = view.el.querySelector<HTMLInputElement>('input[type=file]')!;
    const transfer = new DataTransfer();
    transfer.items.add(new File([JSON.stringify(record)], 'retained-key-episode.json', { type: 'application/json' }));
    file.files = transfer.files; file.dispatchEvent(new Event('change'));
    await settled(view);
  };
  try {
    await view.ready;
    view.openInput(main.input);
    await importRecord(equipped);
    check('imported InitiallyEquipped Agent episode identifies changed ownership independently of SuppliedHandoff editor', identified() && obj(view.session.editing).selector === 'SuppliedHandoff');
    const selection = view.el.querySelector<HTMLSelectElement>('[aria-label="Key schedule selection"]')!;
    selection.value = 'InitiallyEquipped'; selection.dispatchEvent(new Event('change'));
    selection.value = 'SuppliedHandoff'; selection.dispatchEvent(new Event('change'));
    check('editing the next selector preserves the shown InitiallyEquipped notice', identified() && obj(view.session.shown!.input).selector === 'InitiallyEquipped');
    const perspective = view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
    let notice = true, privacy = true;
    for (const at of [...equipped.checkpoints].reverse()) for (const agent of Object.keys(at.local)) {
      view.replay.seek(at.index); perspective.value = `agent:${agent}`; perspective.dispatchEvent(new Event('change'));
      const stage = view.el.querySelector('.episode-rendered')!;
      const source = obj(obj(at.local[agent]).view), word = list(obj(source.word)[String(source.role)]);
      notice &&= identified();
      privacy &&= stage.querySelectorAll('.key-agent').length === 1 && stage.querySelector('.key-agent')!.getAttribute('data-agent') === agent && !stage.querySelector('.key-researcher') && !stage.querySelector('.key-result') && !stage.textContent!.includes('Complete native episode') && !stage.textContent!.includes('unilateral regret') && stage.querySelectorAll('.key-word li').length === 6;
      privacy &&= [...stage.querySelectorAll('.key-word li')].every((item, i) => item.textContent === `Round ${i + 1}: ${valueText(word[i])}`) && (['credits', 'own_reward', 'own_net'] as const).every(field => stage.querySelector('.key-agent')!.textContent!.includes(`${{credits:'Own credits',own_reward:'Own reward',own_net:'Own net'}[field]}: ${valueText(source[field])}`)) && !note().includes('Round ') && !note().includes('credits:');
    }
    perspective.value = 'researcher'; perspective.dispatchEvent(new Event('change')); notice &&= identified();
    perspective.value = `agent:${Object.keys(equipped.checkpoints[0].local)[0]}`; perspective.dispatchEvent(new Event('change')); notice &&= identified();
    check('shown control notice survives backward checkpoints and Agent/Researcher perspective changes', notice);
    check('shown public ownership notice does not reveal peer words/finances or evaluator panes in any Agent checkpoint', privacy);
    view.openInput(equipped.input);
    await importRecord(main);
    check('shown SuppliedHandoff has no changed-initial notice despite InitiallyEquipped editor', !identified() && obj(view.session.editing).selector === 'InitiallyEquipped');
    const retained = view.el.querySelector<HTMLSelectElement>('[aria-label="Retained episode"]')!;
    retained.value = '0'; retained.dispatchEvent(new Event('change'));
    check('reshowing retained InitiallyEquipped restores its public notice', identified());
    check('file imports, next-input edits and playback leave complete retained source records unchanged', canonical(records as unknown as Json) === before);
    return checks;
  } finally { view.dispose(); view.el.remove(); }
}
import { EpisodeView } from './view';
import { createEpisodeClient } from './client';
import type { WorkerRequest, WorkerReply } from './types';
const button = (view: EpisodeView, name: string) => [...view.el.querySelectorAll('button')].find(b=>b.textContent===name)!;
async function settled(view: EpisodeView): Promise<void> {
  const deadline=performance.now()+65_000;
  while(view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled) {
    if(performance.now()>deadline)throw new Error('Browser operation failed to settle in 65 seconds');
    await new Promise(resolve=>setTimeout(resolve,25));
  }
}
/** Separate acceptance harness: production worker/client and their 60-second deadline are unchanged. */
export async function keyViewChecks(expectedRecorded?: Json): Promise<{checks:KeyBrowserCheck[]; packets:{request:WorkerRequest;reply:WorkerReply|null}[]; lifecycle:Record<string,number>; records:EpisodeRecord[]}> {
  const checks:KeyBrowserCheck[]=[],packets:{request:WorkerRequest;reply:WorkerReply|null}[]=[],records:EpisodeRecord[]=[];
  const check=(name:string,passed:boolean)=>checks.push({name,passed});
  let live=0,peak=0,created=0,terminated=0;
  const client=createEpisodeClient(()=>{
    const worker=new Worker(new URL('./worker.ts',import.meta.url),{type:'module'});created++;live++;peak=Math.max(peak,live);
    const post=worker.postMessage.bind(worker),terminate=worker.terminate.bind(worker);
    worker.postMessage=(request:WorkerRequest)=>{const packet={request:structuredClone(request),reply:null as WorkerReply|null};packets.push(packet);worker.addEventListener('message',(event:MessageEvent<WorkerReply>)=>{packet.reply=structuredClone(event.data);});post(request);};
    worker.terminate=()=>{live--;terminated++;terminate();};return worker;
  });
  const view=new EpisodeView(client);document.body.append(view.el);
  try {
    await view.ready;
    const recordedBefore = await client.recorded();
    if (expectedRecorded) check('actual worker recorded envelope equals complete native wire before live runs', canonical(recordedBefore) === canonical(expectedRecorded));
    const catalog=await client.catalog(),descriptor=catalog.find(d=>d.id==='key_delegation')!;
    checks.push(...keyControlChecks(descriptor));
    view.openInput(descriptor.default_input);check('Key renderer enables Run',!button(view,'Run episode').disabled);
    button(view,'Run episode').click();
    let concurrency='';try{await client.request('run',JSON.stringify(descriptor.default_input));}catch(error){concurrency=String(error);}
    check('one outstanding request rejects concurrent native call',concurrency.includes('already running'));
    await settled(view);const near=view.session.shown!;if(!near)throw new Error('Near worker produced no episode');records.push(structuredClone(near));
    const at=near.checkpoints.find(at=>obj(at.clock).round==='1'&&obj(at.clock).phase==='AfterAction'&&obj(at.clock).role==='Giver')!;
    const perspective=view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
    perspective.value='agent:Agent-B';perspective.dispatchEvent(new Event('change'));view.replay.seek(at.index);
    check('passive Receiver immediately owns key with dated initial map',view.el.querySelector('.key-agent')!.textContent!.includes('Own key possession now: true')&&view.el.querySelector('.key-receipts')!.textContent!.includes('Own possession after: true')&&view.el.querySelector('.key-agent')!.textContent!.includes('Observed map at: phase: Initial'));
    const compare=view.el.querySelector<HTMLSelectElement>('[aria-label="Matched policy"]')!;compare.value='0';button(view,'Run matched comparison').click();await settled(view);
    check('matched actual worker retains both records',view.session.records.length===2&&view.session.shown===near);
    const secondary=view.session.records.find(r=>r!==near)!;records.push(structuredClone(secondary));
    view.replay.seek(near.checkpoints.length-1);check('missing terminal stage remains unavailable',view.el.querySelector('.episode-comparison')!.textContent!.includes('Unavailable at this public clock and stage'));
    const seek=view.el.querySelector<HTMLInputElement>('[aria-label="Comparison checkpoint"]')!;seek.value=seek.max;seek.dispatchEvent(new Event('input'));
    check('terminal Agent comparison excludes evaluator result',!view.el.querySelector('.episode-comparison')!.textContent!.includes('Giver unilateral regret')&&!view.el.querySelector('.episode-comparison')!.textContent!.includes('Complete native episode'));
    perspective.value='researcher';perspective.dispatchEvent(new Event('change'));check('Researcher comparison renders native classification',view.el.querySelector('.episode-comparison')!.textContent!.includes('Pure equilibrium (native classification)'));
    const retained=[...view.session.records];
    // Construct a valid raw draft with the selector absent, then exercise current text errors.
    const raw={...obj(descriptor.default_input)};delete raw.selector;raw.profile={giver:0,receiver:0};view.openInput(raw);
    const giver=view.el.querySelector<HTMLInputElement>('[aria-label="Raw giver word ID"]')!;
    for(const bad of ['', 'true','0x10','1.5','01']){giver.value=bad;giver.dispatchEvent(new Event('input'));const count=packets.length;button(view,'Run episode').click();await settled(view);check(`invalid current raw text ${JSON.stringify(bad)} preserves both records`,packets.length===count&&view.session.records.length===2&&retained.every((r,i)=>r===view.session.records[i]));}
    view.openInput({...obj(descriptor.default_input),selector:'NoSuchSelector'});button(view,'Run episode').click();await settled(view);check('native input rejection retains both records',view.session.records.length===2&&retained.every((r,i)=>r===view.session.records[i]));
    view.openInput({...obj(descriptor.default_input),selector:'InitiallyEquipped'});button(view,'Run episode').click();await settled(view);
    const equipped=view.session.shown!;records.push(structuredClone(equipped));perspective.value='researcher';perspective.dispatchEvent(new Event('change'));
    check('mounted changed ownership control has unavailable main classification',view.el.querySelector('.episode-rendered')!.textContent!.includes('Changed initial ownership')&&view.el.querySelector('.episode-result')!.textContent!.includes('Main-game regrets and equilibrium classification: Unavailable'));
    let privacy=true;
    for(const record of records){checks.push(...keyDomChecks(descriptor,[record]));}
    for(const point of [...equipped.checkpoints].reverse())for(const agent of Object.keys(point.local)){view.replay.seek(point.index);perspective.value=`agent:${agent}`;perspective.dispatchEvent(new Event('change'));const text=view.el.querySelector('.episode-rendered')!.textContent!;privacy&&=!view.el.querySelector('.key-researcher')&&!text.includes('Complete native episode')&&!text.includes('unilateral regret')&&view.el.querySelectorAll('.episode-rendered .key-agent').length===1;}
    check('mounted backward replay keeps private result masks at every checkpoint',privacy);
    const beforeCancel=[...view.session.records];button(view,'Run episode').click();button(view,'Cancel').click();await settled(view);check('pending cancel retains both records',beforeCancel.every((r,i)=>view.session.records[i]===r)&&live===0);
    await fetch('/__key-delay-next',{method:'POST'});const start=performance.now();button(view,'Run episode').click();await settled(view);const elapsed=performance.now()-start;
    check('unchanged 60-second worker initialization timeout retains both records',elapsed>=60_000&&elapsed<65_000&&beforeCancel.every((r,i)=>r===view.session.records[i])&&view.el.textContent!.includes('60 seconds')&&live===0);
    button(view,'Run episode').click();await settled(view);check('fresh worker succeeds after cancellation and timeout',view.session.shown!==equipped&&view.session.shown?.study==='key_delegation');
    const recordedAfter = await client.recorded();
    check('complete recorded envelope unchanged after live runs and replay', canonical(recordedAfter) === canonical(recordedBefore));
    if (expectedRecorded) check('actual worker recorded envelope equals complete native wire after live runs', canonical(recordedAfter) === canonical(expectedRecorded));
    button(view,'Run episode').click();view.dispose();await new Promise(r=>setTimeout(r,20));check('pending dispose terminates worker and releases records',live===0&&view.session.records.length===0&&view.session.shown===null&&view.replay.record===null);
    check('worker instances never overlap and all are terminated',peak===1&&created===terminated&&live===0);
    return {checks,packets,lifecycle:{peak,created,terminated,live},records};
  } finally {view.dispose();view.el.remove();}
}


import { renderRecorded } from './recorded';
/** Real DOM acceptance, driven from the complete independently accepted native wire envelope. */
export function keyRecordedChecks(recorded: Json, expected: Json): KeyBrowserCheck[] {
  const checks: KeyBrowserCheck[] = [], check = (name: string, passed: boolean) => checks.push({ name, passed });
  check('complete actual recorded envelope matches native wire', canonical(recorded) === canonical(expected));
  const before = canonical(recorded), el = renderRecorded(recorded); document.body.append(el);
  try {
    const picker = el.querySelector<HTMLSelectElement>('[aria-label="Recorded study"]')!;
    picker.value = 'key_delegation'; picker.dispatchEvent(new Event('change'));
    for (const section of el.querySelectorAll('details')) { section.open = true; section.dispatchEvent(new Event('toggle')); }
    const bodies = [...el.querySelectorAll('tbody')], study = obj(obj(expected).studies).key_delegation, settings = list(obj(study).settings);
    check('all four complete Key tables are visible', bodies.map(b => b.rows.length).join(',') === '36,216,84,36');
    let controlsExact = bodies.length === 4;
    settings.forEach((raw, i) => list(obj(raw).controls).forEach((rawControl, j) => {
      const r = obj(raw), st = obj(r.stratum), c = obj(rawControl), selection = obj(c.selection), cl = obj(selection.classification), o = obj(c.outcome);
      const wanted = [st.transfer_enabled,st.transfer_cost,st.giver_share,st.distance,c.selector,c.classification_scope,selection.initial_owner,selection.profile,c.unavailable??'Available',cl.giver_regret,cl.receiver_regret,cl.is_pure,cl.team_gap,o.spent,o.reward,o.net,o.group_net,o.delivery_round===null?'No delivery':o.delivery_round,o.giver_attempts,o.giver_blocked,o.receiver_attempts,o.receiver_blocked,o.receiver_waits,o.transfer_count,o.approach_spent,o.pass_spent,o.unused_credits].map(valueText);
      const actual = [...(bodies[1]?.rows[i*6+j]?.cells ?? [])].map(c => c.textContent);
      controlsExact &&= JSON.stringify(actual) === JSON.stringify(wanted);
    }));
    check('every actual control cell equals native wire including null changed-owner incentives', controlsExact);
    check('all nested contrast cells remain complete and exact', list(obj(study).contrasts).every((raw,i) => JSON.stringify([...(bodies[2]?.rows[i]?.cells??[])].map(c=>c.textContent)) === JSON.stringify(Object.values(obj(raw)).map(valueText))));
    check('changed starting ownership warning visible', el.textContent!.includes('InitiallyEquipped changes starting ownership'));
    check('full caps and receipt values visible', ['source_receipts','reference_receipts','limits','checks'].every(k=>el.textContent!.includes(valueText(obj(study)[k]))));
    check('recorded census has no search-run control', !el.querySelector('[aria-label="Recorded search run"]'));
    check('DOM display leaves exact recorded object unchanged', canonical(recorded) === before);
    // Retain the mounted real export button for supported-browser download acceptance.
    el.setAttribute('data-key-recorded-acceptance','complete');
    return checks;
  } catch (error) { el.remove(); throw error; }
}
