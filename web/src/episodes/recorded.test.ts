import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { downloadText } from '../downloads';
import { obj, valueText } from './presentation';
import { afterEach, expect, it, vi } from 'vitest';
import { renderRecorded } from './recorded';
import type { Json } from './types';

vi.mock('../downloads', () => ({ downloadText: vi.fn() }));

// Minimal DOM boundary follows the repository's canvas/view test pattern.
class Element {
  children: (Element | string)[] = []; value = ''; className = ''; open = false;
  listeners: Record<string, () => void> = {}; attributes: Record<string, string> = {};
  constructor(readonly tag: string) {}
  get childNodes() { return this.children; }
  get textContent(): string { return this.children.map(c => typeof c === 'string' ? c : c.textContent).join(' '); }
  set textContent(value: string) { this.children = [value]; }
  append(...children: (Element | string)[]) { this.children.push(...children); if (this.tag === 'select' && !this.value) this.value = (this.children[0] as Element).value; }
  replaceChildren(...children: (Element | string)[]) { this.children = []; this.append(...children); }
  setAttribute(key: string, value: string) { this.attributes[key] = value; }
  addEventListener(type: string, callback: () => void) { this.listeners[type] = callback; }
}
function descendants(el: Element): Element[] { return [el, ...el.children.flatMap(c => typeof c === 'string' ? [] : descendants(c))]; }
afterEach(() => vi.unstubAllGlobals());
const ratio = (numerator: string, denominator = '1') => ({ numerator, denominator });
const summary: Json = { version: 'surface-discovery-summary-v1', report_hash: 'frozen-report', source_hash: 'measurement-source',
  settings: [{ setting: '47', protocol: { investigator: 'B', policy: 'InspectOnly' }, environment: 'DataFlip',
    aggregate: { valid_mass: ratio('0'), failed_mass: ratio('1'), spent: [ratio('5'), ratio('7')], reward: [null, null], net: [null, null], group_net: null },
    projection: { unavailable_mass: ratio('1'), accuracy: [null, null], calibration_rounds: [{ count: '0', mass: ratio('1') }] } }],
  comparisons: [{ investigator: 'A', environment: { InFamily: 'SharedPersistent' }, left_policy: 'RawAdaptive', right_policy: 'NoProbe', difference: { spent: [ratio('-10'), ratio('2')], reward: [ratio('0'), ratio('24')], net: [ratio('10'), ratio('22')], group_net: ratio('32') } }],
  role_expectations: [], role_projections: [], role_contrasts: [], compilations: [] };
it('renders the measured discovery summary separately from search runs and preserves exact partial outcomes', () => {
  vi.stubGlobal('document', { createElement: (tag: string) => new Element(tag), createElementNS: (_: string, tag: string) => new Element(tag) });
  const input = { kind: 'recorded_results', version: 1, studies: { surface_discovery: summary } } as Json;
  const before = JSON.stringify(input); const el = renderRecorded(input) as unknown as Element;
  for (const node of descendants(el)) if (node.tag === 'details') { node.open = true; node.listeners.toggle?.(); }
  expect(el.textContent).toContain('Full census');
  expect(el.textContent).toContain('frozen-report');
  expect(el.textContent).toContain('measurement-source');
  expect(el.textContent).toContain('7 / 1, 5 / 1'); // Investigator B, then helper A.
  expect(el.textContent).toContain('Unavailable');
  expect(el.textContent).toContain('32 / 1');
  expect(el.textContent).not.toContain('Search run');
  expect(JSON.stringify(input)).toBe(before);
});
it('renders Gate pure ranges, both private regrets, paid blocked attempts and non-delivery without search controls', () => {
  vi.stubGlobal('document', { createElement: (tag: string) => new Element(tag), createElementNS: (_: string, tag: string) => new Element(tag) });
  const classification = { holder_regret: '2', retriever_regret: '6', is_pure: false, team_gap: '12', outcome: { spent: ['0', '6'], reward: ['0', '0'], net: ['0', '-6'], group_net: '-6', delivery_round: null, holder_attempts: ['6', '0', '0'], holder_blocked: ['0', '0', '0'], retriever_attempts: ['6', '0', '0', '0'], retriever_blocked: ['6', '0', '0', '0'], retriever_waits: '0' } };
  const gate = { version: 'gate-assistance-summary-v1', report_hash: 'gate-report', source_hash: 'gate-source', settings: [{ stratum: { holding_cost: '1', holder_share: '3', distance: '0' }, equilibrium: { pure_count: '71', delivering_pure_count: '70', ranges: { holder: ['0', '1'], retriever: ['0', '5'], group: ['0', '6'] }, range_unavailable: null }, team: { profile: { holder: '540', retriever: '60' }, classification }, solo_best: '0', stats: { profiles: '46656' }, controls: [{ selector: 'SoloAttempt', profile: { holder: '0', retriever: '63' }, classification, unavailable: null }] }], contrasts: [] } as Json;
  const before = JSON.stringify(gate); const el = renderRecorded({ studies: { gate_assistance: gate } }) as unknown as Element;
  for (const node of descendants(el)) if (node.tag === 'details') { node.open = true; node.listeners.toggle?.(); }
  for (const text of ['Gate assistance', 'gate-report', 'gate-source', '71', '70', 'Holder regret', 'Retriever regret', 'Retriever blocked', 'No delivery', '12']) expect(el.textContent).toContain(text);
  expect(el.textContent).not.toContain('Search run');
  expect(JSON.stringify(gate)).toBe(before);
});

it('every measured Gate stratum and control reaches the DOM and complete export unchanged', () => {
  vi.stubGlobal('document', { createElement: (tag: string) => new Element(tag), createElementNS: (_: string, tag: string) => new Element(tag) });
  const gate = JSON.parse(readFileSync(fileURLToPath(new URL('../../../crates/sugarscape-core/src/browser_experiments/fixtures/gate-recorded.json', import.meta.url)), 'utf8')) as Json;
  const el = renderRecorded({ studies: { gate_assistance: gate } }) as unknown as Element;
  for (const node of descendants(el)) if (node.tag === 'details') { node.open = true; node.listeners.toggle?.(); }
  const bodies = descendants(el).filter(node => node.tag === 'tbody');
  const strata = (obj(gate).settings as Json[]);
  expect(bodies.map(body => body.children.length)).toEqual([18, 90, 45, 18]);
  strata.forEach((raw, index) => {
    const row = obj(raw), eq = obj(row.equilibrium), ranges = obj(eq.ranges), st = obj(row.stratum);
    const cells = (bodies[0].children[index] as Element).children as Element[];
    expect(cells[8].textContent).toBe('Available');
    expect(cells.slice(0, 8).map(cell => cell.textContent)).toEqual([st.holding_cost, st.holder_share, st.distance, eq.pure_count, eq.delivering_pure_count, ranges.holder, ranges.retriever, ranges.group].map(valueText));
    (row.controls as Json[]).forEach((rawControl, controlIndex) => {
      const control = obj(rawControl), c = obj(control.classification), o = obj(c.outcome);
      const controlCells = (bodies[1].children[index * 5 + controlIndex] as Element).children as Element[];
      expect(controlCells.map(cell => cell.textContent)).toEqual([st.holding_cost, st.holder_share, st.distance, control.selector, control.profile, control.unavailable ?? 'Available', c.holder_regret, c.retriever_regret, c.is_pure, c.team_gap, o.spent, o.reward, o.net, o.group_net, o.delivery_round === null ? 'No delivery' : o.delivery_round, o.holder_attempts, o.holder_blocked, o.retriever_attempts, o.retriever_blocked, o.retriever_waits].map(valueText));
    });
  });
  descendants(el).find(node => node.tag === 'button' && node.textContent === 'Export Gate census summary')!.listeners.click();
  expect(downloadText).toHaveBeenLastCalledWith('gate-assistance-recorded-summary.json', JSON.stringify(gate), 'application/json');
});

const keyWire = (value: Json): Json => Array.isArray(value) ? value.map(keyWire) : value !== null && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([k,v])=>[k,keyWire(v)])) : typeof value === 'number' ? String(value) : value;
const measuredKey = (): Json => keyWire(JSON.parse(readFileSync(fileURLToPath(new URL('../../../crates/sugarscape-core/src/browser_experiments/fixtures/key-recorded.json',import.meta.url)),'utf8')) as Json);
function openKey(study: Json): Element {
  vi.stubGlobal('document', { createElement:(tag:string)=>new Element(tag), createElementNS:(_:string,tag:string)=>new Element(tag) });
  const el=renderRecorded({studies:{key_delegation:study}}) as unknown as Element;
  for(const node of descendants(el))if(node.tag==='details'){node.open=true;node.listeners.toggle?.();}
  return el;
}
it('retains every measured Key stratum, control, contrast and accounting cell with exact export',()=>{
  const key=measuredKey(),before=JSON.stringify(key),el=openKey(key),nodes=descendants(el),bodies=nodes.filter(n=>n.tag==='tbody');
  expect(bodies.map(b=>b.children.length)).toEqual([36,216,84,36]);
  const settings=obj(key).settings as Json[];
  settings.forEach((raw,i)=>{
    const r=obj(raw),st=obj(r.stratum),eq=obj(r.equilibrium),ranges=obj(eq.ranges),team=obj(r.team);
    const cells=(bodies[0].children[i] as Element).children as Element[];
    expect(cells.map(c=>c.textContent)).toEqual([st.transfer_enabled,st.transfer_cost,st.giver_share,st.distance,eq.pure_count,eq.delivering_pure_count,ranges.giver,ranges.receiver,ranges.group,eq.range_unavailable??'Available',eq.delivery_round_range,eq.delivery_unavailable??'Available',eq.team_gap_range,eq.lex,eq.best_group,team.profile,team.classification,r.solo_best].map(valueText));
    (r.controls as Json[]).forEach((rawC,j)=>{
      const c=obj(rawC),sel=obj(c.selection),cl=obj(sel.classification),o=obj(c.outcome);
      const cs=(bodies[1].children[i*6+j] as Element).children as Element[];
      expect(cs.map(x=>x.textContent)).toEqual([st.transfer_enabled,st.transfer_cost,st.giver_share,st.distance,c.selector,c.classification_scope,sel.initial_owner,sel.profile,c.unavailable??'Available',cl.giver_regret,cl.receiver_regret,cl.is_pure,cl.team_gap,o.spent,o.reward,o.net,o.group_net,o.delivery_round===null?'No delivery':o.delivery_round,o.giver_attempts,o.giver_blocked,o.receiver_attempts,o.receiver_blocked,o.receiver_waits,o.transfer_count,o.approach_spent,o.pass_spent,o.unused_credits].map(valueText));
    });
    expect(((bodies[3].children[i] as Element).children as Element[]).map(x=>x.textContent)).toEqual([st.transfer_enabled,st.transfer_cost,st.giver_share,st.distance,...Object.values(obj(r.stats))].map(valueText));
  });
  (obj(key).contrasts as Json[]).forEach((raw,i)=>expect(((bodies[2].children[i] as Element).children as Element[]).map(x=>x.textContent)).toEqual(Object.values(obj(raw)).map(valueText)));
  for(const field of ['source_receipts','reference_receipts','limits','checks'])expect(el.textContent).toContain(valueText(obj(key)[field]));
  expect(el.textContent).toContain('InitiallyEquipped changes starting ownership');
  expect(el.textContent).not.toContain('Search run');
  nodes.find(n=>n.tag==='button'&&n.textContent==='Export Key census summary')!.listeners.click();
  expect(downloadText).toHaveBeenLastCalledWith('key-delegation-recorded-summary.json',before,'application/json');
  expect(JSON.stringify(key)).toBe(before);
});
it('shape-only unavailable Key selector keeps its reason and absent incentives separate from real changed-owner outcome',()=>{
  const key=structuredClone(measuredKey()),r=obj((obj(key).settings as Json[])[0]),controls=r.controls as Json[];
  controls[3]={selector:'NashLex',classification_scope:'main',selection:null,outcome:null,unavailable:'engineering_no_pure_equilibrium'};
  const el=openKey(key),bodies=descendants(el).filter(n=>n.tag==='tbody');
  const unavailable=(bodies[1].children[3] as Element).children as Element[];
  expect(unavailable.slice(8,13).map(c=>c.textContent)).toEqual(['engineering_no_pure_equilibrium','Unavailable','Unavailable','Unavailable','Unavailable']);
  const equipped=(bodies[1].children[5] as Element).children as Element[];
  expect(equipped.slice(5,7).map(c=>c.textContent)).toEqual(['changed_initial_control','Receiver']);
  expect(equipped.slice(9,13).map(c=>c.textContent)).toEqual(['Unavailable','Unavailable','Unavailable','Unavailable']);
  expect(equipped[17].textContent).toBe('4');
});

it('keeps the complete paid-notes arrays, costs and ties available without running an episode', () => {
  vi.stubGlobal('document', { createElement: (tag: string) => new Element(tag), createElementNS: (_: string, tag: string) => new Element(tag) });
  const notes = { version: 'persistent-notes-summary-v1', report_sha256: 'measured-first', source: { sha256: 'frozen-native' }, reference: { sha256: 'sealed-reference' },
    means: [{ key: 'all-rows', metrics: { spent: [ratio('9'), ratio('4')], net: [ratio('-9'), ratio('8')] } }],
    request_contrasts: [{ difference: ratio('-3') }], contrasts: [{ difference: ratio('0') }], envelopes: [{ routines: ['InspectFirst', 'ReadAdaptive'] }], counters: { episode_rows: '4608' } };
  const before = JSON.stringify(notes);
  const el = renderRecorded({ studies: { persistent_notes: notes } } as Json) as unknown as Element;
  for (const node of descendants(el)) if (node.tag === 'details') { node.open = true; node.listeners.toggle?.(); }
  expect(el.textContent).toContain('Paid persistent notes');
  expect(el.textContent).toContain('Fixed-routine');
  expect(el.textContent).toContain('Offline selected-routine');
  expect(el.textContent).toContain('measured-first');
  expect(el.textContent).toContain('InspectFirst');
  expect(el.textContent).toContain('ReadAdaptive');
  expect(el.textContent).not.toContain('Search run');
  const exportButton = descendants(el).find(node => node.tag === 'button' && node.textContent === 'Export complete Notes summary')!;
  exportButton.listeners.click();
  expect(downloadText).toHaveBeenLastCalledWith('persistent-notes-recorded-summary.json', before, 'application/json');
  expect(JSON.stringify(notes)).toBe(before);
});

it('reconstructs every exact Notes row from its versioned metrics dictionary and rejects malformed indices', async () => {
  vi.stubGlobal('document', { createElement: (tag: string) => new Element(tag), createElementNS: (_: string, tag: string) => new Element(tag) });
  const original = JSON.parse(readFileSync(fileURLToPath(new URL('../../../crates/sugarscape-core/src/browser_experiments/fixtures/notes-recorded.json', import.meta.url)), 'utf8')) as Record<string, Json>;
  const encoded = JSON.parse(JSON.stringify(original)) as Record<string, Json>;
  const dictionary: Json[] = []; const lookup = new Map<string, number>();
  for (const field of ['means', 'request_contrasts', 'contrasts']) for (const raw of encoded[field] as Record<string, Json>[]) {
    const text = JSON.stringify(raw.metrics);
    if (!lookup.has(text)) { lookup.set(text, dictionary.length); dictionary.push(raw.metrics); }
    raw.metrics = String(lookup.get(text));
  }
  encoded.metrics_dictionary = dictionary; encoded.wire_encoding = 'persistent-notes-metrics-dictionary-v1';
  const el = renderRecorded({ studies: { persistent_notes: encoded } } as Json) as unknown as Element;
  descendants(el).find(node => node.tag === 'button' && node.textContent === 'Export complete Notes summary')!.listeners.click();
  const digest = async (value: unknown) => Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(JSON.stringify(value))))).join(',');
  expect(await digest(JSON.parse(vi.mocked(downloadText).mock.lastCall![1]))).toBe(await digest(original));
  const bad = JSON.parse(JSON.stringify(encoded)); bad.means[0].metrics = '999999';
  expect(() => renderRecorded({ studies: { persistent_notes: bad } } as Json)).toThrow('Invalid Notes metrics dictionary index');
  bad.means[0].metrics = '0'; bad.wire_encoding = 'unknown';
  expect(() => renderRecorded({ studies: { persistent_notes: bad } } as Json)).toThrow('Unknown Notes wire encoding');
});
