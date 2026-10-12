import { describe, expect, it } from 'vitest';
import { finishesUnpredictably, modelOf, ticksLeft, COLOR_MODES } from './models';
import { defaultForm } from './experiments/form';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import { MODEL_CHARTS } from './ui/series-data';
import { assistanceLegend, assistanceRows } from './assistance';
import type { AssistanceConfig, AssistanceInspection, ModelConfig, Preset } from './types';

const paperclips = { model: 'assistance', game: 'paperclips', mode: 'iterated', mu: 0, sigma: 1, beta: 0 } as unknown as AssistanceConfig;
const offSwitch = { ...paperclips, game: 'off_switch', mu: 0.25 } as AssistanceConfig;
const outcome = {
  tick: 3, rule: [0, 1, 2], map: [0, 1, 2], thresholds: [41 / 92, 51 / 92] as [number, number], rounds: 3, stable_rounds: 2, distance: 0,
  history: [], equilibria: [[[0, 1, 2], [0, 1, 2]]] as [number[], number[]][], optimal_map: [0, 1, 2], optimal_rule: [], optimal_value: 6373 / 92,
  teaching_thresholds: [41 / 92, 51 / 92] as [number, number], expert_posterior_mean: 0.25,
  incentive: { delta: 0.2864, decomposition: 0.2864, rationality: 0.3867, prob_correct: 0.4013, value_act: 0.25, value_wait: 0.5364, value_off: 0 }, surface: [],
};

describe('Assistance Games integration', () => {
  it('is a model that never finishes', () => {
    expect(modelOf(paperclips)).toBe('assistance');
    expect([ticksLeft(paperclips, 5), finishesUnpredictably(paperclips)]).toEqual([Infinity, false]);
    expect(COLOR_MODES.assistance.map(([m]) => m)).toEqual(['bands', 'surface']);
  });
  it('suggests Figure 3 for the off-switch game and a fictitious-play sweep for paperclips', () => {
    expect(defaultForm('assistance', offSwitch)).toMatchObject({ x: { path: 'sigma', values: '0.05:1:0.05' }, ticks: 1, metric: { kind: 'final', series: 'delta_exact' } });
    expect(defaultForm('assistance', paperclips)).toMatchObject({ set: { mode: 'learning' }, x: { path: 'fictitious_weight' }, ticks: 200, metric: { kind: 'final', series: 'distance' } });
  });
  it('compares demonstrating against teaching', () => {
    const preset = (id: string, fields: object): Preset => ({ id, title: id, name: id, source: '', description: '', config: { ...paperclips, mode: 'static', ...fields } as ModelConfig });
    const presets = [preset('assistance-paperclips-expert', { human_rule: 'expert' }), preset('assistance-paperclips-teaching', { human_rule: 'equilibrium' })];
    const pair = COMPARE_PRESETS.find((p) => p.id === 'assistance-demonstrate-vs-teach')!;
    expect(comparePresetStates(presets, pair, 3)?.b.config).toMatchObject({ human_rule: 'equilibrium' });
  });
  it('shows each game its own charts', () => {
    const charts = MODEL_CHARTS.assistance;
    expect(charts.map((c) => c.title)).toEqual(['Rewards', 'Human thresholds', 'Robot map', 'Convergence', 'Incentive', 'Decomposition', 'Values', 'Choices']);
    expect(charts.map((c) => c.shown!(paperclips))).toEqual([true, true, true, true, false, false, false, false]);
    expect(charts.map((c) => c.shown!(offSwitch))).toEqual([false, false, false, false, true, true, true, true]);
  });
  it('explains a paperclip bin and an off-switch cell', () => {
    const bin: AssistanceInspection = { game: 'paperclips', panel: 'robot', x: 100, y: 30, bin: 50, theta: 50.5 / 92, human_action: 1, human_rewards: [0.902, 1, 1.098], map_cell: 1, deployment: 1, deployment_reward: 50, optimal_deployment: 2, regret: 50 * 50.5 / 92 - 50 + 0.0, posterior: 0.1, rule: [], map: [0, 1, 2], tick: 3, outcome, agent: null };
    const rows = assistanceRows(bin);
    expect(rows[0][1]).toContain('human makes (1, 1)');
    expect(rows.find(([k]) => k === 'Map cell')?.[1]).toBe('after (1, 1) the robot deploys (50, 50)');
    expect(rows.find(([k]) => k === 'Pair')?.[1]).toContain("the paper's equilibrium");
    const cell: AssistanceInspection = { game: 'off_switch', panel: 'surface', x: 18, y: 0, sigma: 0.1, beta: 4, mu: 0.25, delta: -0.12, rationality_term: 0.0006, correction_term: 0.121, prob_correct: 0.484, rationality: 0.062, choice: 'act', configured: outcome.incentive, configured_choice: 'wait', tick: 0, outcome, agent: null };
    expect(assistanceRows(cell).map(([k]) => k)).toEqual(['Cell', 'Incentive Δ', 'Configured']);
    expect(assistanceRows(cell)[1][1]).toContain('acts without asking');
  });
  it('legends name the bands and the surface axes', () => {
    expect(assistanceLegend(paperclips, 'bands').status).toContain('41/92');
    expect(assistanceLegend(offSwitch, 'surface').status).toContain('μ = 0.25');
    expect(assistanceLegend(offSwitch, 'surface').items.length).toBe(3);
  });
});
