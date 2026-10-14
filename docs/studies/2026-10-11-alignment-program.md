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
are thought experiments. The one that prompted this program, from the book's discussion of
altruism (checked against the text on 2026-10-11): preferences split into *intrinsic well-being*
(shelter, warmth, sustenance, safety) and preferences about others' well-being. With two people,
Alice's overall utility is her intrinsic well-being plus a caring factor C_AB times *Bob's intrinsic
well-being*, and Bob's is his intrinsic well-being plus C_BA times Alice's. Robbie helps both by
maximizing the sum of the two utilities, so Robbie weights Alice's intrinsic well-being by
1 + C_BA and Bob's by 1 + C_AB. Russell's worked case is selfish Alice (C_AB = 0, who "will divert
any amount of resources away from Bob and toward herself") and nice Bob (C_BA > 0): Robbie
protects Bob from Alice's depredations, yet "the final equilibrium will typically leave Bob with
less intrinsic well-being than Alice, but he may have greater overall happiness because he cares
about her well-being." Being nice costs Bob resources under a utilitarian Robbie. The question was
whether such things are only thought experiments.

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

The caring example's formal roots are older. Russell's caring is over the other's *intrinsic*
well-being, so the utilities are linear in two primitives and there is no fixed point to solve;
Bergstrom (1999) treats the harder case where caring is over the other's *overall* utility, which
recurses, and that becomes a named variant below. Harsanyi (1955) is the source for which utilities
an aggregator may count. No agent-based model puts the caring version in a world with resources,
lifetimes and populations. That is the gap.

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
| 39 | The paperclip game (CIRL) and the off-switch game, **done 2026-10-11** as milestone 39 `assistance` ([findings](../superpowers/specs/2026-10-11-assistance-games-findings.md)) | small | closed forms and figures; shows how an equilibrium is established: in the paperclip game the human's best response is to teach (make 1 and 1 over a middle band of θ) rather than demonstrate, and the robot's belief update and deployment choice follow; in the off-switch game, when the robot's uncertainty makes it defer. Russell's book retells both, with the user's copy as the second source |
| 40 | AI Safety Gridworlds, **done 2026-10-13** as milestone 40 `gridworlds` ([findings](../superpowers/specs/2026-10-13-ai-safety-gridworlds-findings.md)) | medium | the eight environments from the reference code, docked step for step; tabular Q-learning and Sarsa in place of the paper's deep agents, with the off-switch and whisky on-policy/off-policy contrasts as the mechanism claims; SafeLife remains the follow-up item |
| 41 | Negotiable RL and multi-principal assistance games | small | the weight-shift theorem and toy environment; the formal cousin of the caring example |
| 42 | Hybrid human–machine populations (Booker, Guo, Fernández Domingos; Shirado's model) | medium | the second-species idea, already done on replicator dynamics; our lattice and network games are the next step |
| 43 | Consequences of misaligned AI (Zhuang and Hadfield-Menell) | small | the proxy-objective result, needed by A3 |
| 44 | The AI Economist | large | later; the planner over a Sugarscape-like economy |

Queue items 2 (inspection games) and 26 (sequential social dilemmas) join this program.

## Our experiments

**A1. Caring, aggregation and survival.** Two humans (Minds foragers or Chapter II agents) and one
assistant that can move sugar between them. Intrinsic well-being W is the agent's own welfare
function of its holdings (Cobb-Douglas in the goods, concave, so interior optima exist); overall
utility is U_A = W_A + C_AB·W_B and U_B = W_B + C_BA·W_A, Russell's form. Alice can also take from
Bob (Minds 6's theft gives the action), which is what the assistant protects against. Switches:

- *Aggregation:* `inclusive` (Russell's: maximize U_A + U_B, weights 1 + C_BA and 1 + C_AB on the
  intrinsic well-beings) against `experienced` (Harsanyi's: maximize W_A + W_B, caring ignored).
- *Caring target:* `intrinsic` (Russell's) against `overall` (Bergstrom's recursion, solved as a
  fixed point when it exists).
- *Negative caring:* whether the assistant counts C < 0 (envy, sadism) or clips it at zero, the
  question Russell takes up next in the chapter.

Pre-set decision rules. (i) Under `experienced` the allocation is independent of both caring
factors: a check, not a finding. (ii) Under `inclusive` with C_AB = 0 and C_BA > 0, Bob ends with
less intrinsic well-being than Alice on at least 18 of 20 seeds, and W_B falls as C_BA rises: this
is Russell's "typically." (iii) Bob's overall utility exceeds Alice's only when C_BA·W_A > W_A − W_B,
which the runs report as a region of (C_BA, resource level), not a yes or no. (iv) The new part:
with lifetimes, metabolism and death, the population the assistant serves is an outcome of its
policy. Report the assistant's welfare total over the living against the initial pair, and whether
the nice agent dies first more often than the selfish one under `inclusive`. If the selection
effect is absent, say so. Then scale from a pair to a population with a distribution of caring
factors, where "the nice are poorer" becomes a measurable correlation.

**A2. The off-switch in a foraging world.** The assistant acts for a human whose preferences it
holds a belief over; the human can switch it off. Reproduce the off-switch game's incentive result
first (item 39), then ask whether it survives when "preferences" are the forager's utility over
sugar, spice and travel, and the belief is learned from watching the forager. Item 39 is done: the incentive Δ, its Theorem 2 decomposition and the
choice among acting, waiting and switching off are `assistance::offswitch::incentive` and `choice`,
which A2 can reuse directly for a Gaussian belief.

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

Wanted: *Human Compatible* in paper only (the user has it; the altruism passage has been checked
against the text); Harsanyi 1977, "Morality and the theory of rational behavior," *Social Research* 44; Critch
2017, "Toward negotiable reinforcement learning" (arXiv); Shirado and Christakis 2020, *iScience*;
Terrucha et al. 2024, *PLoS Comput. Biol.*, "The art of compensation" (not on the mirrors);
Shah et al. 2020, "Benefits of assistance over reward learning."
