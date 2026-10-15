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
/** Complete measured Gate census; selected runs never replace these retained values. */
function renderGateSummary(study: Record<string, Json>): HTMLElement {
  const settings = list(study.settings);
  const stratum = (raw: Json | undefined): Record<string, Json> => {
    const value = obj(raw); return { k: value.holding_cost, b: value.holder_share, d: value.distance };
  };
  const outcome = (raw: Json | undefined): Record<string, Json> => {
    const c = obj(raw), o = obj(c.outcome);
    return { 'Holder regret': c.holder_regret, 'Retriever regret': c.retriever_regret, pure: c.is_pure,
      team_gap: c.team_gap, spent: o.spent, reward: o.reward, net: o.net, group_net: o.group_net,
      delivery: o.delivery_round === null ? 'No delivery' : o.delivery_round,
      'Holder attempts': o.holder_attempts, 'Holder blocked': o.holder_blocked,
      'Retriever attempts': o.retriever_attempts, 'Retriever blocked': o.retriever_blocked, retriever_waits: o.retriever_waits };
  };
  const equilibria = settings.map(raw => {
    const row = obj(raw), eq = obj(row.equilibrium), ranges = obj(eq.ranges), team = obj(row.team);
    return { ...stratum(row.stratum), pure_count: eq.pure_count, delivering_pure_count: eq.delivering_pure_count,
      holder_net_range: ranges.holder, retriever_net_range: ranges.retriever, group_net_range: ranges.group,
      range_status: eq.range_unavailable ?? 'Available', team_profile: team.profile,
      team_group_net: obj(obj(team.classification).outcome).group_net, solo_best: row.solo_best };
  });
  const controls = settings.flatMap(raw => { const row = obj(raw); return list(row.controls).map(rawControl => {
    const control = obj(rawControl); return { ...stratum(row.stratum), selector: control.selector,
      profile: control.profile, availability: control.unavailable ?? 'Available', ...outcome(control.classification) };
  }); });
  const sections: [string, Json[]][] = [
    ['Every stratum and pure-equilibrium payoff range', equilibria],
    ['All control outcomes and private incentives', controls],
    ['Registered contrasts (upper minus lower; controls reselected)', list(study.contrasts)],
    ['Compilation accounting', settings.map(raw => { const row = obj(raw); return { ...stratum(row.stratum), ...obj(row.stats) }; })],
  ];
  return h('div', { class: 'gate-recorded-summary' },
    h('h4', {}, 'Full census · Gate assistance'),
    h('p', {}, 'All 18 registered strata and 839,808 raw schedule pairs. Pure equilibria retain every private-payoff tie. Representatives describe the set; they do not predict equilibrium selection or learned cooperation.'),
    h('p', {}, 'k is the cost of Hold, b is the Holder reward share, and d is the initial Holder distance. Pairs are Holder, then Retriever. Holder attempt arrays are Wait / Travel / Hold; Retriever arrays are outbound Move / Pickup / return Move / Deliver. Blocked attempts are paid. No delivery is a completed deadline outcome.'),
    details('Measurement identity', { report_hash: study.report_hash, source_hash: study.source_hash }, true),
    ...sections.map(([title, rows]) => {
      const section = h('details', {}, h('summary', {}, `${title} (${rows.length})`));
      section.addEventListener('toggle', () => { if (section.open && section.childNodes.length === 1) section.append(table(rows)); });
      return section;
    }),
    details('Complete measured summary, caps and source receipts', study),
    h('button', { onclick: () => downloadText('gate-assistance-recorded-summary.json', JSON.stringify(study), 'application/json') }, 'Export Gate census summary'));
}
/** Complete accepted Key census. Selection classification and physical outcome are distinct. */
function renderKeySummary(study: Record<string, Json>): HTMLElement {
  const settings = list(study.settings);
  const stratum = (raw: Json | undefined): Record<string, Json> => {
    const s = obj(raw); return { e: s.transfer_enabled, t: s.transfer_cost, b: s.giver_share, d: s.distance };
  };
  const equilibria = settings.map(raw => {
    const r = obj(raw), eq = obj(r.equilibrium), ranges = obj(eq.ranges), team = obj(r.team);
    return { ...stratum(r.stratum), pure_count: eq.pure_count, delivering_pure_count: eq.delivering_pure_count,
      giver_net_range: ranges.giver, receiver_net_range: ranges.receiver, group_net_range: ranges.group,
      range_status: eq.range_unavailable ?? 'Available', delivery_round_range: eq.delivery_round_range,
      delivery_status: eq.delivery_unavailable ?? 'Available', team_gap_range: eq.team_gap_range,
      lex: eq.lex, best_group: eq.best_group, team_profile: team.profile, team_classification: team.classification, solo_best: r.solo_best };
  });
  const controls = settings.flatMap(raw => {
    const r = obj(raw); return list(r.controls).map(rawControl => {
      const c = obj(rawControl), selection = obj(c.selection), classification = obj(selection.classification), o = obj(c.outcome);
      return { ...stratum(r.stratum), selector: c.selector, classification_scope: c.classification_scope,
        initial_owner: selection.initial_owner, profile: selection.profile, availability: c.unavailable ?? 'Available',
        'Giver regret': classification.giver_regret, 'Receiver regret': classification.receiver_regret,
        pure: classification.is_pure, team_gap: classification.team_gap, spent: o.spent, reward: o.reward,
        net: o.net, group_net: o.group_net, delivery: o.delivery_round === null ? 'No delivery' : o.delivery_round,
        'Giver attempts': o.giver_attempts, 'Giver blocked': o.giver_blocked,
        'Receiver attempts': o.receiver_attempts, 'Receiver blocked': o.receiver_blocked,
        receiver_waits: o.receiver_waits, transfer_count: o.transfer_count, approach_spent: o.approach_spent,
        pass_spent: o.pass_spent, unused_credits: o.unused_credits };
    });
  });
  const sections: [string, Json[]][] = [
    ['Every stratum and pure-equilibrium payoff range', equilibria],
    ['All control outcomes and private incentives', controls],
    ['Registered contrasts (upper minus lower; fixed words and reselected controls)', list(study.contrasts)],
    ['Compilation accounting', settings.map(raw => { const r = obj(raw); return { ...stratum(r.stratum), ...obj(r.stats) }; })],
  ];
  return h('div', { class: 'key-recorded-summary' },
    h('h4', {}, 'Full census · Key delegation'),
    h('p', {}, 'All 36 registered strata and 1,679,616 raw schedule pairs. Pure equilibria retain every private-payoff tie and raw suffix. Representatives describe a set; they do not predict equilibrium selection or learned cooperation.'),
    h('p', {}, 'e is transfer permission, t the paid Pass attempt cost, b the Giver reward share, and d the initial handoff distance. Pairs are Giver, then Receiver. Giver arrays are Wait / Approach / Pass; Receiver arrays are outbound Move / Pickup / return Move / Deliver. Blocked attempts are paid. No delivery is a completed deadline outcome.'),
    h('p', {}, 'InitiallyEquipped changes starting ownership to Receiver. It is an access/cost ceiling control with unavailable main-game regrets and equilibrium classification, not a matched causal transfer estimate. Fixed-word contrasts remain distinct from reselected controls.'),
    details('Measurement identity', { report_hash: study.report_hash, source_hash: study.source_hash }, true),
    ...sections.map(([title, rows]) => {
      const section = h('details', {}, h('summary', {}, `${title} (${rows.length})`));
      section.addEventListener('toggle', () => { if (section.open && section.childNodes.length === 1) section.append(table(rows)); });
      return section;
    }),
    details('Complete measured summary, caps and source receipts', study),
    h('button', { onclick: () => downloadText('key-delegation-recorded-summary.json', JSON.stringify(study), 'application/json') }, 'Export Key census summary'));
}
/** Full retained asset remains available; chart selection never trains or evaluates a policy. */
function decodeNotesSummary(wire: Record<string, Json>): Record<string, Json> {
  if (wire.wire_encoding === undefined) return wire;
  if (wire.wire_encoding !== 'persistent-notes-metrics-dictionary-v1') throw new Error('Unknown Notes wire encoding');
  if (!Array.isArray(wire.metrics_dictionary)) throw new Error('Invalid Notes metrics dictionary');
  const dictionary = wire.metrics_dictionary;
  const decoded = { ...wire };
  for (const field of ['means', 'request_contrasts', 'contrasts']) {
    if (!Array.isArray(wire[field])) throw new Error('Invalid Notes row array');
    decoded[field] = wire[field].map(raw => {
      const row = obj(raw), text = row.metrics;
      if (typeof text !== 'string' || !/^(0|[1-9][0-9]*)$/.test(text)) throw new Error('Invalid Notes metrics dictionary index');
      const index = Number(text);
      if (!Number.isSafeInteger(index) || index >= dictionary.length) throw new Error('Invalid Notes metrics dictionary index');
      return { ...row, metrics: dictionary[index] };
    });
  }
  delete decoded.metrics_dictionary;
  delete decoded.wire_encoding;
  return decoded;
}
/** Complete saved Notes reductions; opening this view never evaluates a policy. */
function renderNotesSummary(wire: Record<string, Json>): HTMLElement {
  const study = decodeNotesSummary(wire);
  const sections: [string, Json[]][] = [
    ['Every request-mean outcome and whole production cost', list(study.means)],
    ['Fixed-routine request contrasts', list(study.request_contrasts)],
    ['Fixed-routine mean contrasts', list(study.contrasts)],
    ['Offline selected-routine envelope and all ties', list(study.envelopes)],
  ];
  return h('div', { class: 'notes-recorded-summary' },
    h('h4', {}, 'Paid persistent notes · complete retained census'),
    h('p', {}, 'Exactly two frozen collections. These are supplied roles, routines and note encodings. Fixed-routine contrasts differ from the offline envelope, which selects on expected outcomes before the hidden request. All negative and wrong outcomes and all ties remain included.'),
    h('p', {}, 'Costs include acquisition, writing, reading, inspection, movement, handoff, pickup, delivery and waits. Reader read savings are separate from whole information-production cost and from the original departed Writer’s private net outcome. No equilibrium, volunteering, emergent communication, novelty or real-incident fidelity is established.'),
    details('Frozen measurement identity', { report_sha256: study.report_sha256, source: study.source, reference: study.reference }, true),
    ...sections.map(([title, rows]) => {
      const section = h('details', {}, h('summary', {}, `${title} (${rows.length})`));
      section.addEventListener('toggle', () => { if (section.open && section.childNodes.length === 1) section.append(table(rows)); });
      return section;
    }),
    details('Complete resource counters', study.counters),
    h('button', { onclick: () => downloadText('persistent-notes-recorded-summary.json', JSON.stringify(study), 'application/json') }, 'Export complete Notes summary'));
}
export function renderRecorded(recorded: Json): HTMLElement {
  const studies = obj(obj(recorded).studies), selector = h('select', { 'aria-label': 'Recorded study' }, ...Object.keys(studies).map(key => h('option', { value: key }, label(key))));
  const content = h('div');
  const show = () => {
    const study = obj(studies[selector.value]);
    if (study.version === 'surface-discovery-summary-v1') { content.replaceChildren(renderDiscoverySummary(study)); return; }
    if (study.version === 'persistent-notes-summary-v1') { content.replaceChildren(renderNotesSummary(study)); return; }
    if (study.version === 'key-delegation-summary-v1') { content.replaceChildren(renderKeySummary(study)); return; }
    if (study.version === 'gate-assistance-summary-v1') { content.replaceChildren(renderGateSummary(study)); return; }
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
