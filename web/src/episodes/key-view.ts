import { h } from '../ui/dom';
import { keyDisplay, type KeyAgent } from './key-projection';
import { obj, list, fact, details, valueText } from './presentation';
import type { Checkpoint, Json } from './types';
import type { RenderDescriptor } from './catalog';
function graph(observation: Json, physical?: Record<string, Json>): HTMLElement {
  const seen = list(obj(observation).cells).map(obj);
  const root = h('div', { class: 'gate-graph key-graph', 'aria-label': physical ? 'Researcher current physical graph' : 'Dated local observation graph' });
  for (const [i, cell] of ['P', 'O', 'C'].entries()) {
    const row = seen.find(row => row.cell === cell);
    const occupants = physical ? list(physical.positions).flatMap((at, index) => at === cell ? [`${index === 0 ? 'Giver' : 'Receiver'}${physical.key_owner === (index === 0 ? 'Giver' : 'Receiver') ? ' · key carrier' : ''}${index === 1 && physical.receiver_cargo ? ' · carrying token' : ''}`] : []) : list(row?.occupants).map(entry => { const occupant = obj(entry); return `${valueText(occupant.id)} · ${valueText(occupant.role)}${occupant.key ? ' · key carrier' : ''}${occupant.cargo ? ' · carrying token' : ''}`; });
    const token = physical ? physical.resource === 'Cache' && cell === 'C' : row?.loose_token;
    root.append(h('div', { class: `gate-cell${!physical && !row ? ' unknown' : ''}`, 'data-cell': cell }, h('h4', {}, cell), h('p', {}, {P:'Approach',O:'Origin / delivery',C:'Cache'}[cell]), h('p', {}, !physical && !row ? 'Unobserved' : occupants.join(', ') || 'No observed Agent'), token ? h('p', {}, 'Loose token') : null));
    if (i < 2) root.append(h('div', { class: `gate-edge${i === 1 ? ' gated' : ''}` }, i === 0 ? 'P–O: traversable' : 'O–C: locked to noncarriers'));
  }
  return root;
}
function agentPane(agent: KeyAgent): HTMLElement {
  const root = h('section', { class: 'gate-agent key-agent', 'data-agent': agent.id }, h('h3', {}, `Agent ${agent.id}`));
  if (!agent.role) { root.append(h('p', {}, 'Local state unavailable at this checkpoint.'), graph(agent.observation)); return root; }
  const own = obj(agent.own), map = obj(agent.observation);
  root.append(fact('Own role', agent.role), fact('Own key possession now', own.own_key), fact('Carrier-only access', own.own_key ? 'Own O–C crossings permitted' : 'Own O–C crossings locked'), h('p', { class: 'hint' }, 'One key has one owner. Possession permits only the carrier to cross; there is no globally open edge.'), graph(map), fact('Observed map at', map.observed_at), h('p', { class: 'hint' }, 'This dated map may be stale. A passive ownership receipt updates own possession without refreshing the map or revealing peer finances. Playback does not sense.'), fact('Own current cell', own.own_cell), fact('Own cargo', own.own_cargo), fact('Own credits', own.credits), fact('Own reward', own.own_reward), fact('Own net', own.own_net));
  root.append(h('h4', {}, 'Own six-round schedule'), h('ol', { class: 'gate-word key-word' }, ...list(obj(own.word)[agent.role]).map((token, i) => h('li', {}, `Round ${i+1}: ${valueText(token)}`))));
  root.append(h('h4', {}, 'Own ownership receipts'), h('ol', { class: 'key-receipts' }, ...agent.receipts.map(receipt => { const r = obj(receipt); return h('li', {}, fact('Receipt clock', r.clock), fact('Counterpart', r.counterpart), fact('Own possession before', r.before), fact('Own possession after', r.after)); })));
  const history = h('ol', { class: 'episode-events gate-history key-history' });
  for (const entry of list(own.history)) {
    const row = obj(entry), action = obj(row.OwnAction);
    history.append(h('li', {}, 'OwnAction' in row ? h('div', {}, fact('Clock', action.clock), fact('Schedule token', action.token), fact('Executed primitive', action.intent), fact('Own result', action.result), fact('Paid credits', action.paid)) : valueText(entry)));
  }
  root.append(h('details', {}, h('summary', {}, `Committed own history (${history.childElementCount})`), history));
  return root;
}
const changedOwnership = () => h('p', { class: 'hint key-initial-control' }, 'Changed initial ownership: Receiver starts with the key. Main-game regrets and equilibrium classification: Unavailable. This is a physical control, not a matched causal transfer estimate.');
export function renderKey(projected: Checkpoint, _descriptor: RenderDescriptor): HTMLElement {
  const display = keyDisplay(projected), root = h('div', { class: 'key-view' }, fact('Public clock', display.clock), fact('Public status', display.status));
  if (display.researcher !== null) {
    const researcher = obj(display.researcher);
    root.append(h('section', { class: 'gate-researcher key-researcher' }, h('h3', {}, 'Researcher current physical state'), fact('Carrier-only access', 'Only the current key owner may cross O–C'), researcher.classification_scope === 'changed_initial_control' ? changedOwnership() : null, fact('Current key owner', obj(researcher.physical).key_owner), graph(null,obj(researcher.physical)), details('Native physical snapshot', researcher.physical), details('Researcher complete schedules', researcher.words), details('Native compiled classification', researcher.classification)));
  }
  root.append(h('div', { class: 'gate-agents key-agents' }, ...display.agents.map(agentPane)));
  return root;
}
export function renderKeyResult(payload: Json): HTMLElement {
  const result = obj(payload), classification = obj(result.classification);
  return h('div', { class: 'gate-result key-result' }, result.classification_scope === 'changed_initial_control' ? changedOwnership() : h('div', {}, fact('Pure equilibrium (native classification)', classification.is_pure), fact('Giver unilateral regret', classification.giver_regret), fact('Receiver unilateral regret', classification.receiver_regret), fact('Gap to team optimum', classification.team_gap)), details('Native outcome', obj(result.episode).outcome,true), details('Compiler statistics (main stratum)', result.compiler_stats), fact('Trace hash', result.trace_hash), details('Complete native episode', result.episode));
}
