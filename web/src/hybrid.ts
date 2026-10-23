import type { ColorMode, HybridConfig, HybridInspection, Param } from './types';
import type { LegendItem } from './ui/legend';

const fmt = (v: number | null | undefined, digits = 3): string => (v === null || v === undefined || Number.isNaN(v) ? '—' : v.toFixed(digits).replace('-', '−'));
const WORLD_GROUP: Record<HybridConfig['world'], string> = { booker: 'Booker', guo: 'Guo', si: 'Si', coloring: 'Coloring' };
const WORLD_GROUPS = Object.values(WORLD_GROUP);
const COLOR_NAMES = ['green', 'orange', 'purple'];
const swatch = (label: string, color: string): LegendItem => ({ label, mark: { kind: 'swatch', color } });

/** The coloring game validates a Barabási–Albert graph of 3 to 40 nodes with two links per new node; offer nothing else. */
const narrowForColoring = (p: Param): Param => {
  if (p.path === 'structure' && p.choices) return { ...p, choices: p.choices.filter((c) => c.value === 'barabasi_albert') };
  if (p.path === 'nodes') return { ...p, min: 3, max: 40 };
  if (p.path === 'links') return { ...p, min: 2, max: 2 };
  return p;
};

/** The Rules panel for one world: the shared groups and the world's own; the coloring game has no imitation rule. */
export function hybridParams(params: Param[], config: HybridConfig): Param[] {
  const own = WORLD_GROUP[config.world];
  const kept = params.filter((p) => (WORLD_GROUPS.includes(p.group) ? p.group === own : !(config.world === 'coloring' && p.group === 'Imitation')));
  return config.world === 'coloring' ? kept.map(narrowForColoring) : kept;
}

const PHASES = { full_c: 'full cooperation', full_d: 'full defection', coexistence: 'coexistence', bistable: 'bistable' } as const;

const referenceText = (view: HybridInspection): string => {
  const node = view.panel === 'node' && 'phase' in view ? view : null;
  switch (view.world) {
    case 'booker': return `all-C weight of the well-mixed chain ${fmt(view.reference)} · replicator rest point ${fmt(node?.replicator)}`;
    case 'guo': return `highest stable master-equation root ${fmt(view.reference)} · replicator phase: ${node?.phase ? PHASES[node.phase] : '—'}`;
    case 'coloring': return `${fmt(view.reference, 0)} proper 3-colorings`;
    default: return 'none (no closed form)';
  }
};

/** Inspect rows: a population node, a coloring node, or an empty cell. */
export function hybridRows(view: HybridInspection): [string, string][] {
  if (view.panel === 'empty') return [['Cell', 'no node here']];
  if ('color' in view) {
    return [
      ['Player', `node ${view.node} · ${view.bot ? 'bot' : 'human (ours)'} · ${COLOR_NAMES[view.color]}`],
      ['Conflicts', `${view.conflicts_here} here · ${view.conflicts} in all`],
      ['Clock', view.solved_at === null ? `${fmt(view.time, 1)} s, unsolved` : `solved at ${fmt(view.solved_at, 1)} s`],
      ['Solutions', `${view.solutions} proper 3-colorings`],
    ];
  }
  return [
    ['Player', `node ${view.node} · ${view.bot ? 'bot' : 'human'} · plays ${view.action}`],
    ['Strategy', `cooperates with probability ${fmt(view.strategy)}`],
    ['Payoff', `${fmt(view.payoff)} over ${view.degree} games`],
    ['Humans cooperating', fmt(view.cooperation)],
    ['Reference', referenceText(view)],
  ];
}

/** The frame's status line and legend. */
export function hybridLegend(config: HybridConfig, _mode: ColorMode): { status: string; items: LegendItem[] } {
  if (config.world === 'coloring') {
    return {
      status: `Shirado & Christakis's coloring game · ${config.bot_count} bots, noise ${config.noise} · humans are our reconstruction (errant rate ${config.errant}) · one tick is one second, up to ${config.time_limit} s`,
      items: [swatch('Green', '#50b450'), swatch('Orange', '#f09628'), swatch('Purple', '#965ac8'), swatch('Bots: darker', '#303030')],
    };
  }
  const who = { booker: 'Booker et al.', guo: 'Guo et al.', si: 'Si et al., as the published code runs it' }[config.world];
  const game = config.world === 'booker' ? `donation game b ${config.b}, c ${config.c}` : config.world === 'guo' ? `T ${config.t}, S ${config.s}, bots cooperate with φ ${config.phi}` : `r ${config.r}, ${config.framework} framework`;
  const bots = config.world === 'booker' && config.bot_kind === 'discriminatory'
    ? [swatch('Discriminatory bot', '#a064dc')]
    : [swatch('Bot acting C', '#5ac878'), swatch('Bot acting D', '#f5a532')];
  return {
    status: `${who} · ${config.structure.replace('_', ' ')} · ${game} · β ${config.beta} · humans blend from red (defect) to blue (cooperate)`,
    items: [swatch('Human cooperating', '#4682e6'), swatch('Human defecting', '#e64646'), ...bots],
  };
}
