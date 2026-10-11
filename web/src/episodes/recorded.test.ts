import { afterEach, expect, it, vi } from 'vitest';
import { renderRecorded } from './recorded';
import type { Json } from './types';

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
