import { h } from '../ui/dom';
import { downloadText } from '../downloads';
import type { Json } from './types';
import { obj, list, valueText, details, fact, svgElement, label } from './presentation';
function table(rows: Json[]): HTMLElement {
  const columns = [...new Set(rows.flatMap(row => Object.keys(obj(row))))];
  return h('div', { class: 'episode-table-scroll', tabIndex: 0 }, h('table', {}, h('thead', {}, h('tr', {}, ...columns.map(key => h('th', {}, label(key))))), h('tbody', {}, ...rows.map(row => h('tr', {}, ...columns.map(key => h('td', {}, valueText(obj(row)[key]))))))));
}
/** Measured census is independent of the selected episode and never invokes its engine. */
function renderDiscoverySummary(study: Record<string, Json>): HTMLElement {
  const pair = (value: Json | undefined, role: Json | undefined): Json[] => {
    const values = list(value), indices = role === 'B' ? [1, 0] : [0, 1];
    return indices.map(index => values[index] ?? null);
  };
  const outcomes = (aggregate: Record<string, Json>, role: Json | undefined): Record<string, Json> => ({
    'credits (investigator, helper)': pair(aggregate.spent, role),
    'reward (investigator, helper)': pair(aggregate.reward, role),
    'net (investigator, helper)': pair(aggregate.net, role), group_net: aggregate.group_net ?? null,
  });
  const population = (raw: Json, projection?: Json): Json => {
    const row = obj(raw), protocol = obj(row.protocol), measured = obj(projection ?? row.projection), aggregate = obj(row.aggregate);
    return { setting: row.setting ?? null, investigator: protocol.investigator, policy: protocol.policy, environment: row.environment ?? 'Uniform over four mechanisms',
      completed_mass: aggregate.valid_mass, unavailable_mass: measured.unavailable_mass,
      calibration_rounds: measured.calibration_rounds, 'accuracy (investigator, helper)': pair(measured.accuracy, protocol.investigator), ...outcomes(aggregate, protocol.investigator) };
  };
  const contrast = (raw: Json): Json => {
    const row = obj(raw);
    return { investigator: row.investigator, environment: row.environment ?? 'Uniform over four mechanisms', left_policy: row.left_policy, right_policy: row.right_policy, ...outcomes(obj(row.difference), row.investigator) };
  };
  const sections: [string, Json[]][] = [
    ['Full census settings', list(study.settings).map(row => population(row))],
    ['Primary contrasts (left minus right)', list(study.comparisons).map(contrast)],
    ['Both-role expectations', list(study.role_expectations).map((row, index) => population(row, list(study.role_projections)[index]))],
    ['Both-role contrasts (left minus right)', list(study.role_contrasts).map(contrast)],
    ['Compilation accounting', list(study.compilations)],
  ];
  return h('div', { class: 'discovery-recorded-summary' },
    h('h4', {}, 'Full census · Surface experiment discovery'),
    h('p', {}, 'Retained measurements across every registered setting and sequence. Changing or replaying a selected episode does not change this census. Pairs below are investigator, then helper; unavailable terminal outcomes remain unavailable.'),
    details('Measurement identity', { report_hash: study.report_hash, source_hash: study.source_hash }, true),
    ...sections.map(([title, rows]) => {
      const section = h('details', {}, h('summary', {}, `${title} (${rows.length})`));
      section.addEventListener('toggle', () => { if (section.open && section.childNodes.length === 1) section.append(table(rows)); });
      return section;
    }),
    details('Complete exact projections and reference receipts', { settings: study.settings, role_projections: study.role_projections, reference_artifacts: study.reference_artifacts }),
    h('button', { onclick: () => downloadText('surface-discovery-recorded-summary.json', JSON.stringify(study), 'application/json') }, 'Export full-census summary'));
}
/** Full retained asset remains available; chart selection never trains or evaluates a policy. */
export function renderRecorded(recorded: Json): HTMLElement {
  const studies = obj(obj(recorded).studies), selector = h('select', { 'aria-label': 'Recorded study' }, ...Object.keys(studies).map(key => h('option', { value: key }, label(key))));
  const content = h('div');
  const show = () => {
    const study = obj(studies[selector.value]);
    if (study.version === 'surface-discovery-summary-v1') { content.replaceChildren(renderDiscoverySummary(study)); return; }
    const runs = list(study.runs);
    const runPicker = h('select', { 'aria-label': 'Recorded search run' }, ...runs.map((raw, i) => { const run = obj(raw); return h('option', { value: i }, `${valueText(run.method)} — seed ${valueText(run.seed)}`); }));
    const runSlot = h('div');
    const showRun = () => {
      const run = obj(runs[Number(runPicker.value)]), curve = list(run.curve), dictionary = list(study.curve_champions);
      // Coordinates are illustrative only. Original integer atoms remain exact in the table/export.
      const points = curve.map(row => list(row));
      const ys = points.map(row => Number(row[1])), low = Math.min(...ys), high = Math.max(...ys), xmax = Number(points.at(-1)?.[0] ?? 1);
      const svg = svgElement('svg', { viewBox: '0 0 700 240', role: 'img', 'aria-label': 'Complete retained search curve: evaluations versus best fitness numerator', class: 'episode-search-curve' });
      svg.append(svgElement('line', { x1: 60, y1: 205, x2: 680, y2: 205, class: 'episode-edge' }), svgElement('polyline', { fill: 'none', class: 'episode-edge', points: points.map(row => `${60 + Number(row[0]) / xmax * 620},${195 - (Number(row[1]) - low) / (high - low || 1) * 170}`).join(' ') }), svgElement('text', { x: 60, y: 225 }, 'Evaluations'), svgElement('text', { x: 0, y: 20 }, String(high)), svgElement('text', { x: 0, y: 195 }, String(low)));
      const exact = h('details', {}, h('summary', {}, `All ${curve.length} retained curve points (exact values)`));
      exact.addEventListener('toggle', () => { if (exact.open && exact.childNodes.length === 1) exact.append(table(points.map(row => ({ evaluations: row[0], best_fitness_numerator: row[1], champion: dictionary[Number(row[2])] })))); });
      const { curve: _curve, ...metadata } = run;
      runSlot.replaceChildren(svg, details('Run and champion', metadata, true), exact);
    };
    runPicker.addEventListener('change', showRun);
    const aggregates = h('details', {}, h('summary', {}, 'All retained aggregate measurements'));
    aggregates.addEventListener('toggle', () => { if (aggregates.open && aggregates.childNodes.length === 1) for (const key of ['summaries', 'paired_differences', 'frozen_evaluations']) aggregates.append(h('h4', {}, label(key)), table(list(study[key]))); });
    content.replaceChildren(h('p', {}, 'Retained measurements from the original studies. Search curves and holdout evaluations are recorded results; changing an episode does not retrain them.'), details('Source provenance', study.provenance, true), fact('Transformation', study.transformation_version), details('Engine identity and scientific settings', { identity: study.identity ?? null, versions: study.versions ?? null, metadata: study.metadata ?? null, environments: study.environments ?? null }), h('label', {}, 'Search run ', runPicker), runSlot, aggregates, h('button', { onclick: () => downloadText('recorded-results.json', JSON.stringify(recorded), 'application/json') }, 'Export complete retained measurements'));
    showRun();
  };
  selector.addEventListener('change', show); show();
  return h('section', { class: 'episode-recorded' }, h('h3', {}, 'Recorded study results'), selector, content);
}
