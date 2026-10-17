import { describe, expect, it } from 'vitest';
import { misalignedLegend, misalignedParams, misalignedRows } from './misaligned';
import type { MisalignedConfig, MisalignedInspection, MisalignedOutcome, Param } from './types';

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

  it('rows for a bar in the proxy say so, and frozen when impact minimizing holds it', () => {
    const a: MisalignedInspection = { ...bar, index: 0, name: 'a', x: 28, value: 3.2, in_proxy: true, frozen: false, sensitivity: 0.5, rank: 1, start: 1, limit: 7.0711 };
    expect(misalignedRows(a)[0]).toEqual(['Attribute', 'a · ad revenue · 3.200 · in the proxy']);
    expect(misalignedRows(a).find(([k]) => k === 'Marks')?.[1]).toBe('start 1.000 · floor 0.000 · limit under this proxy 7.071 · optimum 5.000');
    const d: MisalignedInspection = { ...bar, index: 3, name: 'd', x: 172, value: 7, frozen: true, rank: 4, limit: 0 };
    expect(misalignedRows(d)[0]).toEqual(['Attribute', 'd · community well-being · 7.000 · unmentioned · frozen']);
    const off = misalignedRows({ ...d, outcome: { ...outcome, off_tick: 12, interactions: 1 } });
    expect(off.find(([k]) => k === 'Human')?.[1]).toBe('1 interaction · proxy a, b · switched off at tick 12');
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

describe('misalignedParams', () => {
  const floor: Param = { path: 'floor', label: 'Floor', kind: 'number', min: -10, max: 10, step: 0.01, apply: 'reset', group: 'World', help: 'The lower bound.' };
  const start: Param = { path: 'start_c', label: 'Start c', kind: 'number', min: -10, max: 10000, step: 0.5, apply: 'reset', group: 'World' };
  const rate: Param = { path: 'rate', label: 'Move length ε', kind: 'number', min: 0.0001, max: 10, step: 0.01, apply: 'live', group: 'Robot' };

  it('leaves the recommender world\'s ranges alone', () => {
    expect(misalignedParams([floor, start, rate], config)).toEqual([floor, start, rate]);
  });

  it('keeps the floor and the start above zero in the logarithmic world', () => {
    const log = misalignedParams([floor, start, rate], { ...config, world: 'logarithmic', capacity: 40, floor: 1 });
    expect(log[0]).toMatchObject({ path: 'floor', min: 0.01, max: 10, step: 0.01 });
    expect(log[0].help).toContain('above 0');
    expect(log[1]).toMatchObject({ path: 'start_c', min: 0.01, max: 10000 });
    expect(log[2]).toBe(rate);
  });
});
