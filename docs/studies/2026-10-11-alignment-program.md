# Assistance and alignment in artificial societies: a program note

**Date:** 2026-10-11
**Status:** a stub, persisted after the user approved the direction. Nothing is scheduled or
built; each item gets its own brainstorm, spec and plan when picked up. The reproductions go on the
queue in `docs/papers.md` (items 39–44) and keep their own milestone numbers; the experiments below
are numbered A1, A2, … like Minds.
**Separate from:** the reproductions (`docs/papers.md`), the war program
(`2026-09-26-war-and-society.md`), Minds (`2026-09-27-minds.md`) and swarm coordination
(`2026-09-27-swarm-coordination.md`), which this program borrows from: Minds foragers are the
humans, and swarm coordination's oversight experiments are the multi-agent end of the same
question. Like those, these are our own experiments, built on models we have reproduced.
**Serves:** the AI-safety coordination direction in the research notes.

## Where it came from

Stuart Russell's *Human Compatible* (2019) argues for machines whose objective is the satisfaction
of human preferences under uncertainty about what those preferences are. Several of its arguments
are thought experiments. The one that prompted this program: Alice and Bob each want to be happy;
Robbie, the machine, wants both happy; Alice cares about Bob less than Bob cares about Alice; a
Robbie that counts the caring ends up favoring Alice and neglecting Bob. (The user's paraphrase
from the book; check the chapter's exact statement before anything is built. The user owns the
book.) The question was whether such things are only thought experiments.

They are not, and the literature falls into four strands:

1. **Formal assistance games from Russell's group.** Cooperative inverse reinforcement learning
   (Hadfield-Menell et al. 2016), the off-switch game (2017), obedience (Milli et al. 2017), inverse
   reward design (2017), negotiable reinforcement learning (Desai, Critch and Russell 2018: a
   Pareto-optimal agent serving principals with different beliefs must shift weight over time toward
   the one whose beliefs prove more accurate, shown in a toy environment) and multi-principal
   assistance games (Fickinger et al. 2020, humans who can misreport). Zhuang and Hadfield-Menell
   (2020) show a proxy objective that omits attributes can be arbitrarily bad.
2. **Thought experiments as gridworlds.** AI Safety Gridworlds (Leike et al. 2017: off-switch, side
   effects, reward gaming, distributional shift) and SafeLife (Wainwright and Eckersley 2020).
3. **Machine agents inserted into human social dilemmas.** Shirado and Christakis (2017, human
   subjects: noisy bots improve network coordination); Crandall et al. (2018) and Ishowo-Oloko et al.
   (2019) on cooperating with machines and the "machine penalty"; evolutionary game theory of hybrid
   populations (Fernández Domingos et al. 2022, Booker et al. 2023, Guo et al. 2023, and a 2024
   cross-game study): unconditionally helpful bots raise cooperation more than discriminating ones,
   and bots help in Stag Hunt but barely in the Prisoner's Dilemma.
4. **A machine planner over an artificial economy.** The AI Economist (Zheng et al. 2022).

The caring example's formal root is older: Bergstrom (1999) on systems of benevolent utility
functions, and Harsanyi (1955) on which utilities an aggregator may count. No agent-based model
puts the caring version in a world with resources, lifetimes and populations. That is the gap.

## Principle

- **Originator first.** Reproduce the papers with stated rules and numbers before any experiment of
  our own, with every gap a named switch and every failure reported.
- **A claim that can fail.** Models that illustrate a philosophical point are cheap to build and
  worthless to publish. Every experiment below names its decision rule before a run.
- **Nothing here builds capability.** The machine agents are rule-based or tabular learners in a
  grid world; "misalignment" is defined by the simulation's own welfare measures.
- **A second kind of agent, not a smarter one.** The machine agent differs from the humans in the
  world the way the user put it: no immune string or an orthogonal one, different or no metabolism,
  different lifetime, an objective defined over the humans' welfare rather than its own. Each
  difference is a switch, so the control is a machine agent that is a human in all but objective.
- **Human-subject papers cannot be reproduced here.** For Shirado and Christakis and the
  machine-penalty studies we reproduce the authors' models and simulations and treat the human data
  as targets, as with Bali and the Anasazi.
- In code, pages and docs the kind is `assistant`; "Robbie" stays in this note as Russell's name.

## Reproduction queue (on `docs/papers.md`, items 39–44)

| Item | Papers | Size | Why first |
|---|---|---|---|
| 39 | Off-switch game and CIRL | small | closed-form results and figures; the base assistance game |
| 40 | AI Safety Gridworlds | medium | the environments and the paper's baseline agents' failures; SafeLife as the follow-up |
| 41 | Negotiable RL and multi-principal assistance games | small | the weight-shift theorem and toy environment; the formal cousin of the caring example |
| 42 | Hybrid human–machine populations (Booker, Guo, Fernández Domingos; Shirado's model) | medium | the second-species idea, already done on replicator dynamics; our lattice and network games are the next step |
| 43 | Consequences of misaligned AI (Zhuang and Hadfield-Menell) | small | the proxy-objective result, needed by A3 |
| 44 | The AI Economist | large | later; the planner over a Sugarscape-like economy |

Queue items 2 (inspection games) and 26 (sequential social dilemmas) join this program.

## Our experiments

**A1. Caring, aggregation and survival.** Two humans (Minds foragers or Chapter II agents) and one
assistant that can move sugar between them. Caring coefficients C_AB and C_BA enter the humans'
utilities as in Bergstrom. Two aggregation rules as a switch: *experienced* (Harsanyi's: the
assistant sums the humans' own-consumption utilities) and *inclusive* (it sums their full utilities,
caring included). Pre-set decision rules: under experienced aggregation the allocation is
independent of C_BA (a check, not a finding); under inclusive aggregation Bob's experienced welfare
falls as C_BA rises, on at least 18 of 20 seeds, which is Russell's claim; and the new part, with
lifetimes and death, the population the assistant serves becomes an outcome of its policy, measured
as the difference between the assistant's welfare total over the living and over the initial
population. If the selection effect is absent, say so.

**A2. The off-switch in a foraging world.** The assistant acts for a human whose preferences it
holds a belief over; the human can switch it off. Reproduce the off-switch game's incentive result
first (item 39), then ask whether it survives when "preferences" are the forager's utility over
sugar, spice and travel, and the belief is learned from watching the forager.

**A3. Proxy objectives in Sugarscape.** The assistant optimizes a proxy that omits one attribute of
human welfare (spice, health, or leisure), per item 43. Measure the omitted attribute against the
assistant's optimization pressure, with the paper's prediction as the target.

**A4. A second species in the cooperation models.** Assistants placed in the spatial games, norms,
tags, culture and demographic PD worlds with Shirado-style noise or Samaritan and discriminating
rules; the machine penalty as a switch on how humans treat them. Targets from item 42: bots help in
Stag Hunt and barely in the PD; Samaritans beat discriminators under weak selection. Chapter V gives
a cheap version of orthogonality: assistants that cannot catch the disease but can carry it.

**A5. Power-seeking in a resource world.** Turner et al.'s result that optimal policies tend to seek
power, tested as whether an assistant with a long horizon accumulates sugar and options beyond what
its objective needs. Flagged as the easiest to overclaim; last.

## What the model needs

- An agent `kind` in the Sugarscape model with per-kind switches for metabolism, immune string,
  lifetime and reproduction, defaulting to identical kinds so every existing preset runs unchanged.
- An objective over other agents' welfare, with the aggregation rule as a switch, and an action that
  transfers resources (trade's bookkeeping already conserves units).
- A belief over a human's preferences, updated from observed choices (Minds 3's memory and Minds 8's
  watching are the parts).
- Per-kind welfare, survival and inequality series, and sweeps over caring and aggregation.

## Sources

Local copies in `papers/alignment/` (gitignored): the Russell-group papers above, Leike et al. 2017,
Wainwright and Eckersley 2020, Turner et al. 2021, Amodei et al. 2016, Conitzer et al. 2024 (social
choice for alignment), Dafoe et al. 2020 (cooperative AI), Leibo et al. 2021 (Melting Pot), Zheng et
al. 2022, Shirado and Christakis 2017, Crandall et al. 2018, Ishowo-Oloko et al. 2019, Fernández
Domingos et al. 2022, Booker et al. 2023, Guo et al. 2023, the 2024 cross-game bots paper, Bergstrom
1999 and Harsanyi 1955. Leibo et al. 2017 and Hughes et al. 2018 are in
`papers/multi-agent-coordination/`.

Wanted: *Human Compatible* itself (the user has it; the chapter with the caring example, to quote
exactly); Harsanyi 1977, "Morality and the theory of rational behavior," *Social Research* 44; Critch
2017, "Toward negotiable reinforcement learning" (arXiv); Shirado and Christakis 2020, *iScience*;
Terrucha et al. 2024, *PLoS Comput. Biol.*, "The art of compensation" (not on the mirrors);
Shah et al. 2020, "Benefits of assistance over reward learning."
