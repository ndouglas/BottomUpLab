# Recirculation, niches and carrying capacity: a study stub

**Date:** 2026-10-11
**Status:** a stub, persisted for later. Nothing is scheduled or built; it needs a brainstorm and a
spec before any code. The user may drive the science.
**Separate from:** the reproductions (`docs/papers.md`). This would be our own experiment on top of
the N-goods Sugarscape, not a reproduction. Holland's Echo (below) is the nearest published model;
if a reproducible Echo paper with stated rules and numbers turns up, a reproduction goes on the
queue first and this study follows it.

## Where it came from

Holland's *Signals and Boundaries* (MIT Press, 2012) names four patterns that recur in complex
adaptive systems: diversity, recirculation, niche and hierarchy, and coevolution. His argument is a
chain, not a list. Diversity of signal and boundary types lets niches form; niches close
recirculation loops, in which the same resource is used over and over; and closed loops are what
raise throughput and carrying capacity, as in his reading of Adam Smith's pin factory.

An audit of this repository on 2026-10-11 found:

- **Diversity:** present (vision, metabolism, endowment, up to eight goods; culture, tags, opinions
  test whether it persists), but every agent shares one goal shape, so goals never "interlock."
- **Recirculation:** absent. Sugar grows back from an exogenous rule, metabolized sugar vanishes,
  and pollution diffuses and decays without fertilizing anything. Trade and credit redistribute;
  Bali's water and pests and Minds 8's scrounging are one-step flows. No loop closes anywhere.
- **Niche and hierarchy:** thin. N-goods sorts agents to peaks by endowment, and the
  `n-goods-carrying-capacity` sweep found peak spacing matters more than the number of goods. Minds 8
  found no stable producer/scrounger mix. Theraulaz and Bonabeau's response thresholds (queue 25)
  are the direct specialization test, unbuilt.
- **Coevolution:** our strongest, but almost all adversarial (disease, tags, image scoring, norms,
  punishment, the pilfering campaign). Mutualism exists only by construction (rule T requires both to
  gain). No predator–prey with population dynamics, no commensalism.

## The question

Does closing a material loop raise carrying capacity in a Sugarscape world, and if so, through
specialization (Holland's claim) or through plain extra supply? And does anything resembling a
recirculation web self-organize when agents can specialize?

## The mechanism to test

Holland's chain, one link at a time:

1. **A closed loop raises capacity.** Add a decomposer step: pollution (already produced by
   harvesting and metabolism in Chapter II-8 and the pollutant matrices) becomes future growback of
   some good instead of merely decaying. Compare carrying capacity with the loop open against closed,
   holding total exogenous inflow fixed, so any gain is from recycling and not from a hidden subsidy.
2. **Specialization closes more loops.** Let metabolism specialize (heritable or learned weights
   over goods, with a cost to generalism) so that one agent's waste is another's input. Compare
   capacity with specialization allowed against forbidden, loop closed in both.
3. **A web emerges.** Measure whether the flow network among specialist types develops cycles and
   chains without being built in.

## What the model needs (Sugarscape has some of it)

- **Have:** N goods (1–8), pollutants (1–4) with production and consumption matrices, diffusion,
  per-good growback, heritable metabolism, the sweep machinery and `gini_total`-style measures.
- **Need:** a decomposer rule mapping a pollutant to a good's growback (a named switch, off by
  default so every existing preset runs unchanged); a conservation accounting that tracks every
  unit from inflow to sink; specialization of metabolism with a generalist cost (a switch); an
  optional second trophic level (agents that consume agents or their stores) only if steps 1–2 earn
  it. Predator–prey is a separate study, not this one.

## What to measure

- Carrying capacity (mean population over a late window), as in Figure II-5 and the N-goods sweep.
- Total and recycled throughput, and a cycling index: the share of flow that passes through a loop
  (Finn's cycling index from ecosystem flow analysis is the standard, to be read before adopting).
- Specialization: entropy of each agent's metabolism weights and of the population's type mix.
- The flow network among types: edges, chain length, cycles; whether it is stable across seeds.
- Inequality and lifetimes alongside, since `2026-09-27-trade-and-inequality.md` showed trade's
  gains and its inequality arrive together.

## Pre-set decision rules (to fix in the spec before any run)

- Capacity "rises" only if the closed-loop median exceeds the open-loop median on at least 18 of 20
  seeds with total inflow equal; otherwise the loop adds nothing that supply doesn't explain.
- Holland's specialization claim "holds" only if closed-loop capacity is higher with specialization
  allowed than forbidden under the same inflow; if capacity rises with the loop closed but not with
  specialization, that is a supply effect, reported as such.
- A "web" requires a cycle in the type-flow network in a majority of seeds without being wired in.

## Candidate findings

- Recycling raises capacity but specialization adds nothing: the pin-factory story fails in a
  foraging world where travel, not processing, is the cost.
- Specialization rises with the loop closed but the mix is unstable, as Minds 8's scroungers were.
- The N-goods spacing result returns: geometry of the peaks dominates any trophic effect.
- Diversity of goals (step 2) does more for capacity than diversity of goods (the existing sweep).

## Sources to read first

John H. Holland, *Signals and Boundaries: Building Blocks for Complex Adaptive Systems* (MIT Press,
2012), and *Hidden Order* (1995), whose Echo model has resource-string agents, trade, combat and
mating and is the nearest existing design (wanted: a paper version with stated rules; Echo's own
reproducibility is in doubt). Kristian Lindgren and Mats Nordahl, "Artificial food webs," in
*Artificial Life III* (1994), on evolved trophic structure. Robert Ulanowicz, *Growth and
Development: Ecosystems Phenomenology* (1986) and J. T. Finn (1976, *J. Theor. Biol.*) for cycling
and throughput measures. Theraulaz, Bonabeau and Deneubourg (1998) in `papers/multi-agent-coordination/`
for specialization by response thresholds. Epstein and Axtell, Chapter II (pollution) and Chapter IV
footnote 7 (N goods and pollutants). Search for an agent-based test of recycling and carrying capacity
before building; none is cited here, and nothing in `papers/` covers it.
