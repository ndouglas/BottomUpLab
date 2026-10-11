import { EpisodeView } from './view';
import { obj } from './presentation';
import { canonical } from './comparison';
import type { Json } from './types';

/** Mounted browser-only QA uses the production client and real WASM workers.
 * These helpers are not registered in the product navigation or Node Vitest.
 */
export async function mountDiscoveryHarness(): Promise<DiscoveryBrowserHarness> {
  const view = new EpisodeView();
  for (const node of [...document.body.children]) if (node instanceof HTMLElement) node.hidden = true;
  document.body.append(view.el);
  await view.ready;
  return new DiscoveryBrowserHarness(view);
}
export class DiscoveryBrowserHarness {
  constructor(readonly view: EpisodeView) {}
  open(input: Json): void { this.view.openInput(input); }
  state() {
    return { records: this.view.session.records.length, input: this.view.session.editing, shown: this.view.session.shown?.input ?? null,
      index: this.view.replay.index, perspective: this.view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')?.value,
      status: this.view.el.querySelector('[role="status"]')?.textContent, busy: this.view.el.querySelector<HTMLFieldSetElement>('.episode-setup')?.disabled };
  }
  captureAll() {
    const record = this.view.session.shown;
    if (!record) throw new Error('Run an actual episode before capture');
    const original = canonical(record as unknown as Json);
    const picker = this.view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
    const output: { index: number; perspective: string; facts: string[]; events: string[][]; text: string }[] = [];
    for (const checkpoint of record.checkpoints) {
      this.view.replay.seek(checkpoint.index);
      for (const perspective of [...Object.keys(checkpoint.local).map(id => `agent:${id}`), 'researcher']) {
        picker.value = perspective; picker.dispatchEvent(new Event('change'));
        const stage = this.view.el.querySelector('.episode-rendered')!;
        output.push({ index: checkpoint.index, perspective,
          facts: [...stage.querySelectorAll('.surface-view p')].map(p => p.textContent ?? ''),
          events: [...stage.querySelectorAll('.surface-events ol')].map(ol => [...ol.querySelectorAll('li')].map(li => li.textContent ?? '')),
          text: stage.textContent ?? '' });
      }
    }
    for (const index of [0, record.checkpoints.length - 1, 3, 0, 3]) this.view.replay.seek(Math.min(index, record.checkpoints.length - 1));
    if (canonical(record as unknown as Json) !== original) throw new Error('Playback changed the native record');
    this.view.replay.reset(); picker.value = `agent:${Object.keys(record.checkpoints[0].local)[0]}`; picker.dispatchEvent(new Event('change'));
    return output;
  }
  sentinelCheck() {
    const original = this.view.session.shown!;
    const sentinel = structuredClone(original);
    sentinel.checkpoints[0].researcher = { environment: 'MODEL_SENTINEL', topology: 'TOPOLOGY_SENTINEL', fields: [{symbol:'FIELD_SENTINEL',lineage:{symbol:'LINEAGE_SENTINEL'}}], event: 'EVENT_SENTINEL' };
    sentinel.checkpoints[0].local['Agent-B'] = { prefix: { ids: obj(obj(original.checkpoints[0].local['Agent-B']).prefix).ids, entries:['PEER_SENTINEL'] }, private_bit:'PEER_BIT_SENTINEL' };
    sentinel.checkpoints[1].local['Agent-A'] = {private_bit:'FUTURE_BIT_SENTINEL'};
    sentinel.payload = { metrics:[{reward:'REWARD_SENTINEL',correct:'GRADE_SENTINEL'}], failure:null };
    this.view.replay.load(sentinel);
    const stage = this.view.el.querySelector('.episode-rendered')!;
    const agentText = stage.textContent ?? '';
    const picker = this.view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
    picker.value = 'researcher'; picker.dispatchEvent(new Event('change'));
    const researcherText = stage.textContent ?? '';
    this.view.replay.load(original); picker.value = 'agent:Agent-A'; picker.dispatchEvent(new Event('change'));
    return { agentText, researcherText, agentHasSentinel:agentText.includes('SENTINEL'), researcherHasEvaluator:researcherText.includes('MODEL_SENTINEL') && researcherText.includes('REWARD_SENTINEL') && researcherText.includes('GRADE_SENTINEL') };
  }
  /** Actual DOM capture, including a temporary unavailable-owner/public-only boundary. */
  privacyCapture(index: number) {
    const record = this.view.session.shown!;
    const original = canonical(record as unknown as Json);
    this.view.replay.seek(index);
    const picker = this.view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
    const output: { perspective: string; mainText: string; comparisonText: string; publicHeader: string }[] = [];
    for (const perspective of ['agent:Agent-A', 'agent:Agent-B', 'agent:unavailable-public-view', 'researcher']) {
      if (perspective === 'agent:unavailable-public-view') {
        const option = document.createElement('option'); option.value = perspective; option.textContent = 'Unavailable Agent · QA public-only boundary'; picker.append(option);
      }
      picker.value = perspective; picker.dispatchEvent(new Event('change'));
      output.push({ perspective, mainText: this.view.el.querySelector('.episode-rendered')!.textContent ?? '',
        comparisonText: this.view.el.querySelector('.episode-comparison > div > div:last-child')?.textContent ?? '',
        publicHeader: this.view.el.querySelector('.episode-rendered .surface-clock')!.textContent ?? '' });
    }
    picker.value = 'agent:Agent-A'; picker.dispatchEvent(new Event('change'));
    if (canonical(record as unknown as Json) !== original) throw new Error('Privacy capture changed the actual record');
    return output;
  }
  /** Display-only evaluator sentinel fixture; never used as a native/worker outcome. */
  terminalPayloadSentinelCheck() {
    const records = this.view.session.records, shown = this.view.session.shown!;
    const supplied = records.map(record => ({ ...structuredClone(record), payload: {
      metrics: [{ spent: 'SPENT_SENTINEL', reward: 'REWARD_SENTINEL', net: 'NET_SENTINEL', correct: 'GRADE_SENTINEL', lineage_reads: 'LINEAGE_SENTINEL' }],
      histories: ['PEER_HISTORY_SENTINEL'], privileged: 'PRIVILEGED_SENTINEL',
      failure: { prefix: 'PEER_PREFIX_SENTINEL', private_bit: 'PEER_BIT_SENTINEL' }
    } }));
    const selected = supplied[records.indexOf(shown)];
    this.view.session.records = supplied; this.view.session.shown = selected; this.view.replay.load(selected);
    const captures = this.privacyCapture(selected.checkpoints.length - 1);
    this.view.session.records = records; this.view.session.shown = shown; this.view.replay.load(shown);
    return captures;
  }
  dispose(): void { this.view.dispose(); }
}
