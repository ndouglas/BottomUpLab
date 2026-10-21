import { describe, expect, it } from 'vitest';
import { hybridLegend, hybridParams, hybridRows } from './hybrid';
import type { HybridConfig, HybridInspection, Param } from './types';

const param = (path: string, group: string): Param => ({ path, label: path, kind: 'number', apply: 'live', group });
const PARAMS = [
  param('world', 'World'), param('structure', 'Structure'), param('bot_count', 'Bots'), param('beta', 'Imitation'),
  param('b', 'Booker'), param('t', 'Guo'), param('r', 'Si'), param('noise', 'Coloring'),
];
const config = (world: HybridConfig['world']): HybridConfig => ({ model: 'hybrid', world, framework: 'mixed', bot_kind: 'samaritan', structure: 'lattice', beta: 10 } as HybridConfig);

describe('hybrid page helpers', () => {
  it('shows the shared groups and only the chosen world’s own', () => {
    expect(hybridParams(PARAMS, config('booker')).map((p) => p.path)).toEqual(['world', 'structure', 'bot_count', 'beta', 'b']);
    expect(hybridParams(PARAMS, config('si')).map((p) => p.path)).toEqual(['world', 'structure', 'bot_count', 'beta', 'r']);
    expect(hybridParams(PARAMS, config('coloring')).map((p) => p.path)).toEqual(['world', 'structure', 'bot_count', 'noise']);
  });

  it('offers the coloring game only the graphs it validates', () => {
    const shared: Param[] = [
      { ...param('structure', 'Structure'), kind: 'choice', choices: [{ value: 'lattice', label: 'Square lattice' }, { value: 'barabasi_albert', label: 'Barabási–Albert' }] },
      { ...param('nodes', 'Structure'), kind: 'integer', min: 2, max: 200000 },
      { ...param('links', 'Structure'), kind: 'integer', min: 1, max: 10 },
    ];
    const [structure, nodes, links] = hybridParams(shared, config('coloring'));
    expect(structure.choices?.map((c) => c.value)).toEqual(['barabasi_albert']);
    expect([nodes.min, nodes.max, links.min, links.max]).toEqual([3, 60, 2, 2]);
    expect(hybridParams(shared, config('guo'))).toEqual(shared);
  });

  it('rows describe a node, a coloring node and an empty cell', () => {
    const node: HybridInspection = { panel: 'node', world: 'guo', tick: 3, x: 1, y: 2, reference: 0.4123, agent: null, node: 9, bot: false, strategy: 1, action: 'C', degree: 4, payoff: 2.5, cooperation: 0.61, replicator: null, phase: 'coexistence' };
    const rows = hybridRows(node);
    expect(rows[0]).toEqual(['Player', 'node 9 · human · plays C']);
    expect(rows.find(([k]) => k === 'Payoff')?.[1]).toBe('2.500 over 4 games');
    expect(rows.find(([k]) => k === 'Reference')?.[1]).toBe('highest stable master-equation root 0.412 · replicator phase: coexistence');
    const booker: HybridInspection = { ...node, world: 'booker', reference: 0.98, replicator: 1, phase: null };
    expect(hybridRows(booker).find(([k]) => k === 'Reference')?.[1]).toBe('all-C weight of the well-mixed chain 0.980 · replicator rest point 1.000');
    const si: HybridInspection = { ...node, world: 'si', reference: null, bot: true, strategy: 0.9, action: 'D', phase: null };
    expect(hybridRows(si)[0]).toEqual(['Player', 'node 9 · bot · plays D']);
    expect(hybridRows(si).find(([k]) => k === 'Strategy')?.[1]).toBe('cooperates with probability 0.900');
    expect(hybridRows(si).find(([k]) => k === 'Reference')?.[1]).toBe('none (no closed form)');
    const color: HybridInspection = { panel: 'node', world: 'coloring', tick: 12, x: 0, y: 0, reference: 48, agent: null, node: 0, bot: true, color: 2, degree: 7, conflicts_here: 1, solutions: 48, conflicts: 3, time: 12.4, solved_at: null };
    const crow = hybridRows(color);
    expect(crow[0]).toEqual(['Player', 'node 0 · bot · purple']);
    expect(crow.find(([k]) => k === 'Conflicts')?.[1]).toBe('1 here · 3 in all');
    expect(crow.find(([k]) => k === 'Clock')?.[1]).toBe('12.4 s, unsolved');
    expect(crow.find(([k]) => k === 'Solutions')?.[1]).toBe('48 proper 3-colorings');
    expect(hybridRows({ panel: 'empty', world: 'guo', tick: 0, x: 3, y: 2, reference: null, agent: null })).toEqual([['Cell', 'no node here']]);
  });

  it('legend names the world and its colors', () => {
    const booker = hybridLegend({ ...config('booker'), bot_kind: 'discriminatory' }, 'strategy');
    expect(booker.status).toContain('Booker et al.');
    expect(booker.items.map((i) => i.label)).toContain('Discriminatory bot');
    expect(hybridLegend(config('si'), 'strategy').status).toContain('as the published code runs it');
    expect(hybridLegend(config('coloring'), 'strategy').items.map((i) => i.label)).toEqual(['Green', 'Orange', 'Purple', 'Bots: darker']);
  });
});
