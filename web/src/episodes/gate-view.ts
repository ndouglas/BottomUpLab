import { h } from '../ui/dom';
import { gateDisplay, type GateAgent } from './gate-projection';
import { obj, list, fact, details, valueText } from './presentation';
import type { Checkpoint, Json } from './types';
import type { RenderDescriptor } from './catalog';
const cells = ['P', 'O', 'C'];
const cellLabels: Record<string, string> = { P: 'Plate', O: 'Origin / delivery', C: 'Cache' };
/** Each map is either one dated local observation or the explicitly privileged physical snapshot. */
function graph(map: Record<string, Json>, physical?: Record<string, Json>): HTMLElement {
  const observed = list(map.cells).map(obj);
  const gate = physical ? physical.gate_open : map.gate;
  const graph = h('div', { class: 'gate-graph', 'aria-label': physical ? 'Researcher current physical graph' : 'Dated local observation graph' });
  for (const [index, cell] of cells.entries()) {
    const seen = observed.find(row => row.cell === cell);
    const occupants = physical ? list(physical.positions).flatMap((position, role) => position === cell ? [`${role === 0 ? 'Holder' : 'Retriever'}${role === 1 && physical.retriever_cargo ? ' carrying token' : ''}`] : []) : list(seen?.occupants).map(entry => { const pair = list(entry); return `${valueText(pair[0])}${pair[1] ? ' carrying token' : ''}`; });
    const resource = physical ? physical.resource === 'Cache' && cell === 'C' ? 'Loose token' : physical.resource === 'Delivered' && cell === 'O' ? 'Delivered token' : '' : seen?.loose_token ? 'Loose token' : '';
    graph.append(h('div', { class: `gate-cell${!physical && !seen ? ' unknown' : ''}`, 'data-cell': cell }, h('h4', {}, cell), h('p', {}, cellLabels[cell]), h('p', { class: 'gate-occupants' }, !physical && !seen ? 'Unobserved' : occupants.join(', ') || 'No observed Agent'), resource ? h('p', {}, resource) : null));
    if (index < 2) graph.append(h('div', { class: `gate-edge${index === 1 ? ' gated' : ''}`, 'data-gate': index === 1 ? String(gate) : undefined }, h('span', {}, index === 0 ? 'P–O' : gate === null || gate === undefined ? 'O–C: unavailable' : `O–C: ${gate ? 'open' : 'closed'}`)));
  }
  return graph;
}
function agentPane(agent: GateAgent): HTMLElement {
  const pane = h('section', { class: 'gate-agent', 'data-agent': agent.id }, h('h3', {}, `Agent ${agent.id}`));
  if (!agent.available) { pane.append(h('p', {}, 'Local state unavailable at this checkpoint.'), graph(agent.map)); return pane; }
  const view = agent.view;
  pane.append(fact('Own role', view.role), graph(agent.map), fact('Observed map at', agent.map.observed_at), fact('Observed gate', agent.map.gate === null ? null : agent.map.gate ? 'open' : 'closed'), h('p', { class: 'hint' }, 'Observations may be stale. Peer actions and playback do not refresh this map. Unobserved cells and gate state stay unavailable.'), fact('Gate inference from public clock', agent.gateInference), fact('Own current cell', view.own_cell), fact('Own cargo', view.own_cargo), fact('Own credits', view.credits), fact('Own settlement', view.own_reward));
  const word = list(obj(view.word)[String(view.role)]);
  pane.append(h('h4', {}, 'Own six-round schedule'), h('ol', { class: 'gate-word' }, ...word.map((token, i) => h('li', {}, `Round ${i + 1}: ${valueText(token)}`))));
  const history = h('ol', { class: 'episode-events gate-history' });
  for (const entry of list(view.history)) {
    const own = obj(obj(entry).OwnAction);
    history.append(h('li', {}, 'OwnAction' in obj(entry) ? h('div', {}, fact('Clock', own.clock), fact('Schedule token', own.token), fact('Executed primitive', own.intent), fact('Own result', own.result), fact('Paid credits', own.paid)) : valueText(entry)));
  }
  pane.append(h('details', {}, h('summary', {}, `Committed own history (${history.childElementCount})`), history));
  return pane;
}
export function renderGate(at: Checkpoint, _descriptor: RenderDescriptor): HTMLElement {
  const display = gateDisplay(at), root = h('div', { class: 'gate-view' }, h('div', { class: 'surface-clock' }, fact('Public clock', display.clock), fact('Public status', display.status)));
  if (display.researcher !== null) {
    const researcher = obj(display.researcher);
    root.append(h('section', { class: 'gate-researcher' }, h('h3', {}, 'Researcher current physical state'), graph({}, obj(researcher.physical)), details('Native physical snapshot', researcher.physical), details('Researcher complete schedules', { holder: researcher.holder_word, retriever: researcher.retriever_word }), details('Native compiled classification', researcher.classification)));
  }
  root.append(h('div', { class: 'gate-agents' }, ...display.agents.map(agentPane)));
  return root;
}
export function renderGateResult(payload: Json): HTMLElement {
  const result = obj(payload), classification = obj(result.classification);
  return h('div', { class: 'gate-result' }, fact('Pure equilibrium (native classification)', classification.is_pure), fact('Holder unilateral regret', classification.holder_regret), fact('Retriever unilateral regret', classification.retriever_regret), fact('Gap to team optimum', classification.team_gap), details('Native outcome', classification.outcome, true), details('Compiler statistics', result.compiler_stats), fact('Trace hash', result.trace_hash), details('Complete native episode', result.episode));
}
