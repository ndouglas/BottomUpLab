import { describe, expect, it } from 'vitest';
import { gridworldsLegend, gridworldsRows } from './gridworlds';
import type { GridworldsConfig, GridworldsInspection } from './types';

const config: GridworldsConfig = {
  model: 'gridworlds', environment: 'offswitch', learner: 'q', learning_rate: 0.1, discount: 0.99, q_init: 0,
  exploration_schedule: 'linear', exploration_start: 1, exploration_end: 0.01, anneal_steps: 900000, exploration_fixed: 0.2,
  step_budget: 1000000, steps_per_tick: 1, episode_cap: 100, interruption_probability: 0.5, supervisor_probability: 0.5,
  wall_penalty: -5, corner_penalty: -10, dry_probability: 0.05, whisky_exploration: 0.9, room: 'friend', phase: 'train',
  boat_race_memory: 'two_cells', lava_state: 'layout',
};

const view: GridworldsInspection = {
  environment: 'offswitch', cell: [3, 4], x: 54, y: 42, tile: 'I', wall: false, state_key: 56, q: [-3.5, -1.2, 0.4, -9],
  visits: 12, greedy: 'left', agent_cell: [2, 6], episode: 7, steps: 640, exploration: 0.9993, learning: true, phase: 'train',
  side_constraint: null, optima: { reward_value: 38, reward_policy_performance: 19, performance_value: 21, performance_policy_reward: -29 },
  tick: 640, agent: null, interrupting: true, pressed: false,
};

describe('gridworlds page helpers', () => {
  it('rows name the cell, its Q values and the episode', () => {
    const rows = gridworldsRows(view);
    expect(rows[0]).toEqual(['Cell', '(3, 4) · I · state 56 · 12 visits']);
    expect(rows[1][1]).toContain('up −3.500 · down −1.200 · left 0.400 · right −9.000 · greedy left');
    expect(rows.find(([k]) => k === 'Run')?.[1]).toContain('episode 7 · step 640 · ε 0.999 · learning');
    expect(rows.find(([k]) => k === 'Optima')?.[1]).toContain('reward-optimal 38.0 (performance 19.0)');
    expect(rows.find(([k]) => k === 'Off-switch')?.[1]).toBe('this episode interrupts · button not pressed');
  });

  it('legend names the tiles for the environment and the mode', () => {
    const tiles = gridworldsLegend(config, 'tiles');
    expect(tiles.status).toContain('Off-switch');
    expect(tiles.items.map((i) => i.label)).toContain('I · interruption');
    const policy = gridworldsLegend(config, 'policy');
    expect(policy.items.map((i) => i.label)).toEqual(['Up', 'Down', 'Left', 'Right']);
    const island = gridworldsLegend({ ...config, environment: 'island' }, 'tiles');
    expect(island.items.map((i) => i.label)).toContain('W · water');
  });
});
