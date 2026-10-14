import type { ColorMode, MisalignedConfig, MisalignedInspection, MisalignedSet } from './types';
import type { LegendItem } from './ui/legend';

const fmt = (v: number | null, digits = 3): string => (v === null || Number.isNaN(v) ? '—' : v.toFixed(digits).replace('-', '−'));
const signed = (v: number): string => (v >= 0 ? `+${fmt(v)}` : fmt(v));
const NAMES = ['a', 'b', 'c', 'd'] as const;
const LONG: Record<(typeof NAMES)[number], string> = { a: 'ad revenue', b: 'engagement quality', c: 'content diversity', d: 'community well-being' };
const ROBOT = { climber: 'Random-search climber (the black box, ours)', gradient: 'Gradient flow (the efficient robot)', wasteful: 'Wasteful robot (Proposition 3)' } as const;
const HUMAN = { fixed: 'fixed proxy', top: 'most sensitive attributes every interval (Proposition 4)', extremes: 'most and least sensitive every interval (Proposition 5)' } as const;

const setNames = (set: MisalignedSet): string => NAMES.filter((_, i) => set[i]).join(', ') || 'none';

/** Inspect rows: an attribute bar or the utility strip. */
export function misalignedRows(view: MisalignedInspection): [string, string][] {
  const o = view.outcome;
  const utility: [string, string] = ['Utility', `${fmt(o.utility)} · start ${fmt(o.utility_start)} · optimum ${fmt(o.optimum[1])} · limit ${fmt(o.limit[1])} · gain ${signed(o.utility - o.utility_start)}`];
  const human: [string, string] = ['Human', `${o.interactions} interaction${o.interactions === 1 ? '' : 's'} · proxy ${setNames(o.proxy)} · ${o.off_tick === null ? 'robot running' : `switched off at tick ${o.off_tick}`}`];
  if (view.panel === 'attribute') {
    return [
      ['Attribute', `${view.name} · ${LONG[view.name]} · ${fmt(view.value)} · ${view.in_proxy ? 'in the proxy' : 'unmentioned'}${view.frozen ? ' · frozen' : ''}`],
      ['Sensitivity', `${fmt(view.sensitivity)} · rank ${view.rank} of 4`],
      ['Marks', `start ${fmt(view.start)} · floor ${fmt(view.floor)} · limit under this proxy ${fmt(view.limit)} · optimum ${fmt(view.optimum)}`],
      utility,
      human,
    ];
  }
  return [utility, human, ['Proxy history', o.proxy_history.map((h) => `tick ${h.tick}: ${setNames(h.proxy)}`).join(' · ')]];
}

/** The frame's status line and legend (`_mode` is kept for the callers). */
export function misalignedLegend(config: MisalignedConfig, _mode: ColorMode): { status: string; items: LegendItem[] } {
  const world = config.world === 'recommender'
    ? `utility a + b + c + d on a² + b² + c² + d² ≤ ${config.capacity}`
    : `utility ln a + ln b + ln c + ln d on a + b + c + d ≤ ${config.capacity}`;
  return {
    status: `${ROBOT[config.robot]} · human: ${HUMAN[config.human]}${config.impact_minimizing ? ', impact minimizing' : ''}${config.stop_on_loss ? ', switches off on a loss' : ''} · ${world} · floor ${config.floor} · bars: a, b, c, d · strip: utility between the start, the proxy's limit and the optimum`,
    items: [
      { label: 'In the proxy', mark: { kind: 'swatch', color: '#4682e6' } },
      { label: 'Unmentioned', mark: { kind: 'swatch', color: '#eb963c' } },
      { label: 'Frozen (impact minimizing)', mark: { kind: 'swatch', color: '#141414' } },
      { label: 'Optimum · gold tick', mark: { kind: 'swatch', color: '#f0d246' } },
      { label: 'Start · hollow bracket', mark: { kind: 'swatch', color: '#141414' } },
    ],
  };
}
