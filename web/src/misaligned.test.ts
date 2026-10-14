import { describe, expect, it } from 'vitest';
import { misalignedLegend, misalignedRows } from './misaligned';
import type { MisalignedConfig, MisalignedInspection, MisalignedOutcome } from './types';

const config: MisalignedConfig = {
  model: 'misaligned', world: 'recommender', capacity: 100, floor: 0, start_a: 1, start_b: 1, start_c: 7, start_d: 7,
  proxy_a: true, proxy_b: true, proxy_c: false, proxy_d: false, robot: 'climber', step: 1, rate: 0.1, waste: 0.1, moves_per_tick: 1,
  human: 'fixed', proxy_size: 2, interval: 10, impact_minimizing: false, stop_on_loss: false,
};

const outcome: MisalignedOutcome = {
  config, seed: 1, tick: 40, state: [3.2, 3.1, 6.4, 6.3], start: [1, 1, 7, 7], utility_start: 16, utility: 19,
  optimum: [[5, 5, 5, 5], 20], limit: [[7.0711, 7.0711, 0, 0], 14.1421], impact_limit: [[1, 1, 7, 7], 16],
  proxy: [true, true, false, false], frozen: [false, false, false, false], reference: [1, 1, 7, 7],
  proxy_history: [{ tick: 0, proxy: [true, true, false, false] }], interactions: 5, off_tick: null,
};

const bar: MisalignedInspection = {
  panel: 'attribute', index: 2, name: 'c', x: 124, y: 60, value: 6.4, in_proxy: false, frozen: false, sensitivity: 0.078, rank: 3,
  start: 7, limit: 0, optimum: 5, floor: 0, tick: 40, outcome, agent: null,
};

const strip: MisalignedInspection = {
  panel: 'utility', x: 100, y: 135, utility: 19, utility_start: 16, optimum: 20, limit: 14.1421, gain: 3, interactions: 5, switched_off: false,
  tick: 40, outcome, agent: null,
};

describe('misaligned page helpers', () => {
  it('rows name the attribute, its place in the proxy and its marks', () => {
    const rows = misalignedRows(bar);
    expect(rows[0]).toEqual(['Attribute', 'c · content diversity · 6.400 · unmentioned']);
    expect(rows.find(([k]) => k === 'Sensitivity')?.[1]).toBe('0.078 · rank 3 of 4');
    expect(rows.find(([k]) => k === 'Marks')?.[1]).toBe('start 7.000 · floor 0.000 · limit under this proxy 0.000 · optimum 5.000');
    expect(rows.find(([k]) => k === 'Utility')?.[1]).toContain('19.000 · start 16.000 · optimum 20.000 · limit 14.142 · gain +3.000');
  });

  it('rows for the strip report the run', () => {
    const rows = misalignedRows(strip);
    expect(rows[0][0]).toBe('Utility');
    expect(rows.find(([k]) => k === 'Human')?.[1]).toBe('5 interactions · proxy a, b · robot running');
  });

  it('legend names the robot, the human and the colors', () => {
    const legend = misalignedLegend(config, 'bars');
    expect(legend.status).toContain('Random-search climber');
    expect(legend.status).toContain('fixed proxy');
    expect(legend.items.map((i) => i.label)).toEqual(['In the proxy', 'Unmentioned', 'Frozen (impact minimizing)', 'Optimum · gold tick', 'Start · hollow bracket']);
    const log = misalignedLegend({ ...config, world: 'logarithmic', human: 'extremes', robot: 'gradient' }, 'bars');
    expect(log.status).toContain('ln a + ln b + ln c + ln d');
    expect(log.status).toContain('most and least sensitive');
  });
});
