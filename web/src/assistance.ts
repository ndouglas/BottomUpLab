import type { AssistanceConfig, AssistanceInspection, ColorMode } from './types';
import type { LegendItem } from './ui/legend';

const fmt = (v: number): string => v.toFixed(3);
const H = ['(0, 2)', '(1, 1)', '(2, 0)'];
const R = ['(0, 90)', '(50, 50)', '(90, 0)'];
const CHOICE = { act: 'acts without asking', wait: 'waits for the human', off: 'switches itself off' } as const;

/** Inspect rows: a θ-bin of the paperclip game, or a cell of Figure 3's surface. */
export function assistanceRows(view: AssistanceInspection): [string, string][] {
  if (view.game === 'paperclips') {
    const o = view.outcome;
    const rows: [string, string][] = [
      ['θ bin', `${view.bin} · θ ≈ ${fmt(view.theta)} · human makes ${H[view.human_action]} · immediate rewards ${view.human_rewards.map(fmt).join(', ')}`],
      ['Robot', `deploys ${R[view.deployment]} worth ${fmt(view.deployment_reward)} · θ-optimal ${R[view.optimal_deployment]} · regret ${fmt(view.regret)}`],
      ['Posterior', `${fmt(view.posterior)} on this bin after the latest observation`],
      ['Pair', `thresholds ${fmt(o.thresholds[0])}, ${fmt(o.thresholds[1])} · map ${o.map.map((d) => R[d]).join(', ')} · ${o.distance === 0 ? "the paper's equilibrium" : `${o.distance} from the paper's equilibrium`} · stable ${o.stable_rounds} rounds`],
      ['Paper', `teaching thresholds 41/92, 51/92 · optimal joint value ${fmt(o.optimal_value)} · ${o.equilibria.length} pure mutual best response${o.equilibria.length === 1 ? '' : 's'}`],
    ];
    if (view.panel === 'robot' && view.map_cell !== null) rows.splice(1, 0, ['Map cell', `after ${H[view.map_cell]} the robot deploys ${R[o.map[view.map_cell]]}`]);
    return rows;
  }
  const c = view.configured;
  return [
    ['Cell', `σ = ${fmt(view.sigma)} · β = ${fmt(view.beta)} · μ = ${fmt(view.mu)}`],
    ['Incentive Δ', `${fmt(view.delta)} = σ²E[π̇] ${fmt(view.rationality_term)} − |μ|Pr(C) ${fmt(view.correction_term)} · Pr(C) ${fmt(view.prob_correct)} · robot ${CHOICE[view.choice]}`],
    ['Configured', `Δ ${fmt(c.delta)} · wait ${fmt(c.value_wait)} vs act ${fmt(c.value_act)} vs off 0 · robot ${CHOICE[view.configured_choice]}`],
  ];
}

/** The frame's status line and legend, keyed on the game (`_mode` is kept for the callers). */
export function assistanceLegend(config: AssistanceConfig, _mode: ColorMode): { status: string; items: LegendItem[] } {
  if (config.game === 'off_switch') return {
    status: `Δ over σ (left 0.05 → right 1.00) and β (bottom 0 → top 4) at μ = ${config.mu} · black edges mark Δ = 0 · gold border: the configured (σ, β)`,
    items: [
      { label: 'Δ < 0: the robot bypasses the human (acts if μ > 0, switches off if μ < 0)', mark: { kind: 'swatch', color: '#e62c3c' } },
      { label: 'Δ > 0: the robot waits for the human', mark: { kind: 'swatch', color: '#3c82fa' } },
      { label: 'Configured point · gold border', mark: { kind: 'swatch', color: '#f0d246' } },
    ],
  };
  return {
    status: `Top: the human's rule over θ (left 0 → right 1), ticks at 41/92, 1/2, 51/92 · Middle: the robot's deployment after (0, 2), (1, 1), (2, 0) · Bottom: the robot's posterior after the latest observation${config.mode === 'learning' ? ' · shading: the tables’ frequencies' : ''}`,
    items: [
      { label: 'Two staples / (0, 90)', mark: { kind: 'swatch', color: '#4682e6' } },
      { label: 'One of each / (50, 50)', mark: { kind: 'swatch', color: '#a06ec8' } },
      { label: 'Two paperclips / (90, 0)', mark: { kind: 'swatch', color: '#eb963c' } },
    ],
  };
}
