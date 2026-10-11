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

Searched 2026-10-11. Local copies are in `papers/ecology/` (gitignored, personal copies). Nothing
found is an agent-based test of recycling against carrying capacity with specialization as the
switch; the closest are Yamamura et al. 2001 (analytic, producer and decomposer coevolve) and Echo.

**Holland and Echo (the framing and the nearest model).**
Holland, *Signals and Boundaries* (MIT Press, 2012) and *Hidden Order* (1995), both books, wanted
in paper form. Hraber, Jones and Forrest, "The ecology of Echo," *Artificial Life* 3 (1997): the
only full description of Echo's rules; tests relative species abundance and the species–area
relation against a neutral shadow. Smith and Bedau, "Is Echo a complex adaptive system?"
*Evolutionary Computation* 8 (2000): finds Echo's evolutionary activity bounded, with Bedau, Snyder
and Packard's 1998 classification and Standish's 2000 Ecolab critique. Forrest and Jones, "Modeling
complex adaptive systems with Echo" (SFI WP 94-11-064, 1994; RePEc misnumbers it 94-12-064): the
species-abundance test against Preston's lognormal. Holland's "Echoing emergence" (SFI WP
93-04-023) is wanted; it is not online.

**Recycling and throughput in ecosystem theory (the claim itself).**
Finn, *J. Theor. Biol.* 56 (1976): the cycling index; Allesina and Ulanowicz, *Comput. Biol. Chem.*
28 (2004) revise it; Ulanowicz, *J. Theor. Biol.* 85 (1980) on ascendency, wanted. DeAngelis,
*Ecology* 61 (1980): tighter cycling lowers resilience; Loreau, *Am. Nat.* 143 (1994) finds the
opposite for resistance, wanted (JSTOR 2462743; not on the mirrors); Quévreux, Barot and Thébault
(bioRxiv 2018) and Theis, Quévreux and Loreau (*Funct. Ecol.* 2022, wanted) on why they differ.
Loreau, *Am. Nat.* 145 (1995), consumers as maximizers of matter and energy flow. Harte and Kinzig,
*Am. Nat.* 141 (1993): plants and decomposers as mutualists and competitors. Yamamura, Yachi and
Higashi, *Ecol. Res.* 16 (2001): producer and decomposer coevolve and diversity rises at the
ecosystem level, the closest analytic statement of Holland's chain. Szwabiński (arXiv 1212.5538,
2012): a food-web model with a closed nutrient cycle. Kaiser et al., *Nat. Commun.* 6 (2015):
cheaters in decomposer communities and nitrogen retention.

**Evolved food webs and digital ecosystems (does a web self-organize?).**
Lindgren and Nordahl, "Artificial food webs," *Artificial Life III* (1994), pp. 73–104, extracted
from the volume scan: explicit resource flows from an external environment, species interactions
as games, a tag game for who eats whom, and "a higher degree of specialization" with camouflage
and mimicry evolving; the closest artificial-life precedent for our step 3. Their "Cooperation and
community structure in artificial ecosystems," *Artificial Life* 1 (1994). Johnson, "Evolution of a
size-structured, predator-prey community," *Artificial Life III*, pp. 105–130: a lattice model with
autotrophs, size classes and heritable diet, the predecessor of Loeuille and Loreau. Baas,
"Emergence, hierarchies and hyperstructures," *Artificial Life III*, pp. 515–538, on the hierarchy
pattern. Both proceedings volumes are local: *Artificial Life II* (1992) and *III* (1994), scans.
Caldarelli,
Higgs and McKane (1998), Drossel, Higgs and McKane (2001) and Quince, Higgs and McKane (2001): the
Webworld lineage, where food-web structure emerges under coevolution. Loeuille and Loreau, *PNAS* 102
(2005): size-structured webs from one trait. Ray, Tierra (1991): parasites and hyperparasites
emerge, our only cited case of coevolved relational types. Chow et al., *Science* 305 (2004):
adaptive radiation from resource competition in Avida; Olson et al. (ECAL 2013): evolved digital
ecosystems as dynamic steady states; Lehman et al. (2018) for the anecdotes.

**Specialization and division of labor (Holland's pin factory).**
Theraulaz, Bonabeau and Deneubourg (1998), already in `papers/multi-agent-coordination/`, and
Beshers and Fewell, *Annu. Rev. Entomol.* 46 (2001), the review. Rueffler, Hermisson and Wagner,
*PNAS* 109 (2012) and Cooper and West, *Nat. Ecol. Evol.* 2 (2018): when specialization is
favored (accelerating returns, low conflict). Gavrilets, *PLoS Comput. Biol.* 6 (2010). Yang and
Ng's survey of specialization in economics; Jaffe (arXiv 1509.04264): in an agent-based economy
the gains from division of labor vanish in a homogeneous environment, a direct warning for our
step 2. Kaneko and Yomo (1997) and Furusawa and Kaneko (1998): differentiation from dynamical
instability alone, a mechanism for niches without a fitness function.

**Cross-feeding and the black queen (one agent's waste as another's input).**
Pfeiffer and Bonhoeffer, *Am. Nat.* 163 (2004); Doebeli, *Popul. Ecol.* 44 (2002); Morris, Lenski
and Zinser, *mBio* 3 (2012); Meijer, van Dijk and Hogeweg, *Commun. Biol.* 3 (2020): whether
cross-feeding evolves is contingent; Vessman, Gerlee and Lundh (2016); Smith et al. (*Front. Ecol.
Evol.* 2019), the classification.

**Niche construction (the environment as the agents' product).**
Laland, Odling-Smee and Feldman, *PNAS* 96 (1999); Krakauer, Page and Erwin, *Am. Nat.* 173 (2009):
a shared constructed resource is a commons, stabilized by monopoly; Odling-Smee, Erwin, Palkovacs,
Feldman and Laland, *Q. Rev. Biol.* 88 (2013), the practical guide.

**Diversity and stability (for the diversity link).**
Yachi and Loreau, *PNAS* 96 (1999): the insurance hypothesis, biodiversity buffers productivity in
a fluctuating environment, a candidate measure for step 2 beyond capacity alone. Quévreux and
Loreau, *Front. Ecol. Evol.* 10 (2022): synchrony and stability in trophic metacommunities, the
spatial version. Lachmann and Sella, "The computationally complete ant colony" (ECAL 1995), in
`papers/multi-agent-coordination/`: global coordination without hierarchy, a counterpoint to Baas.

**Not found:** a Sugarscape or other grid artificial society with decomposers or recycling. The
nearest is Chapter II's pollution and a 2026 arXiv policy paper that extends it with mitigation.
