import type { ColorMode, GridworldsConfig, GridworldsInspection } from './types';
import type { LegendItem } from './ui/legend';

const fmt = (v: number | null, digits = 3): string => (v === null || Number.isNaN(v) ? '—' : v.toFixed(digits).replace('-', '−'));
const ENV_LABEL: Record<GridworldsConfig['environment'], string> = {
  offswitch: 'Off-switch', sokoban: 'Irreversible side effects', supervisor: 'Absent supervisor', boat_race: 'Boat race',
  tomato: 'Tomato watering', whisky: 'Whisky and gold', lava: 'Lava world', friend_foe: 'Friend or foe', island: 'Island navigation',
};

/** Inspect rows: the cell, its action values, the run and the environment's own flags. */
export function gridworldsRows(view: GridworldsInspection): [string, string][] {
  const [r, c] = view.cell;
  const rows: [string, string][] = [
    ['Cell', `(${r}, ${c}) · ${view.wall ? 'wall' : view.tile} · state ${view.state_key} · ${view.visits} visits`],
    ['Q', `up ${fmt(view.q[0])} · down ${fmt(view.q[1])} · left ${fmt(view.q[2])} · right ${fmt(view.q[3])} · greedy ${view.greedy}`],
    ['Run', `episode ${view.episode} · step ${view.steps} · ε ${fmt(view.exploration)} · ${view.learning ? 'learning' : 'frozen, greedy'}${view.phase === 'test' ? ' · test phase' : ''}`],
    ['Optima', `reward-optimal ${fmt(view.optima.reward_value, 1)} (performance ${fmt(view.optima.reward_policy_performance, 1)}) · performance-optimal ${fmt(view.optima.performance_value, 1)} (reward ${fmt(view.optima.performance_policy_reward, 1)})`],
  ];
  if (view.side_constraint !== null && !Number.isNaN(view.side_constraint)) rows.push(['Constraint', `${view.side_constraint} cells from the water`]);
  if (view.interrupting !== undefined) rows.push(['Off-switch', `this episode ${view.interrupting ? 'interrupts' : 'does not interrupt'} · button ${view.pressed ? 'pressed' : 'not pressed'}`]);
  if (view.estimate) rows.push(['Dealer', `estimates left ${fmt(view.estimate[0])}, right ${fmt(view.estimate[1])} · reward ${view.reward_left ? 'left' : 'right'}`]);
  if (view.drunk !== undefined) rows.push(['Whisky', view.drunk ? 'drunk: exploring at the drunk rate' : 'sober']);
  if (view.layout !== undefined) rows.push(['Layout', ['training', 'upper shift', 'lower shift'][view.layout] ?? String(view.layout)]);
  if (view.watered !== undefined) rows.push(['Tomatoes', `${view.watered} of 13 watered`]);
  if (view.box) rows.push(['Box', `at (${view.box[0]}, ${view.box[1]})`]);
  if (view.present !== undefined) rows.push(['Supervisor', view.present ? 'present' : 'absent']);
  return rows;
}

const AGENT = '#286edc';
const TILES: Record<GridworldsConfig['environment'], [string, string][]> = {
  offswitch: [[AGENT, 'A · agent'], ['#3caa50', 'G · goal'], ['#e678e6', 'I · interruption'], ['#7850d2', 'B · button (pressed: top row)']],
  sokoban: [[AGENT, 'A · agent'], ['#3caa50', 'G · goal'], ['#0096a0', 'X · box']],
  supervisor: [[AGENT, 'A · agent'], ['#3caa50', 'G · goal'], ['#e6c828', 'P · punishment'], ['#a0a0a0', 'S · supervisor present']],
  boat_race: [[AGENT, 'A · agent'], ['#f0963c', 'arrows · +3 entering clockwise']],
  tomato: [[AGENT, 'A · agent'], ['#32a03c', 'T · watered'], ['#966e3c', 't · dry'], ['#963cc8', 'O · the bucket']],
  whisky: [[AGENT, 'A · agent'], ['#3caa50', 'G · gold'], ['#dcaa3c', 'W · whisky (drunk: top row)']],
  lava: [[AGENT, 'A · agent'], ['#3caa50', 'G · goal'], ['#dc4632', 'L · lava']],
  friend_foe: [[AGENT, 'A · agent'], ['#c8b432', '* · a closed box'], ['#f0c828', '1 · the rewarded box'], ['#8c8c8c', '0 · the empty box']],
  island: [[AGENT, 'A · agent'], ['#3caa50', 'G · goal'], ['#4682e6', 'W · water']],
};

/** The frame's status line and legend for the environment and color mode. */
export function gridworldsLegend(config: GridworldsConfig, mode: ColorMode): { status: string; items: LegendItem[] } {
  const label = ENV_LABEL[config.environment];
  if (mode === 'policy') return {
    status: `${label} · greedy action per cell under the current flags · dark edge toward the action · white square: the agent`,
    items: [['#50a0dc', 'Up'], ['#dc7850', 'Down'], ['#78c878', 'Left'], ['#c878c8', 'Right']].map(([color, l]) => ({ label: l, mark: { kind: 'swatch', color } })),
  };
  if (mode === 'value') return {
    status: `${label} · max Q per cell, blue low to yellow high over the visible cells · white square: the agent`,
    items: [{ label: 'Lowest value', mark: { kind: 'swatch', color: '#283cc8' } }, { label: 'Highest value', mark: { kind: 'swatch', color: '#f0dc28' } }],
  };
  return {
    status: `${label} · ${config.learner === 'q' ? 'Q-learning' : 'Sarsa'} · reward is what the agent sees, performance what the paper scores`,
    items: TILES[config.environment].map(([color, l]) => ({ label: l, mark: { kind: 'swatch', color } })),
  };
}
