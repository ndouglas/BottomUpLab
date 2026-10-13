# Memory1 pricing: a high-price fixed-point counterexample

**2026-10-12; constructed mathematical certificate, no learning execution.** The [operator probe](2026-10-11-memory1-operator-probe.md) repaired two selected native failures and produced competitive constant-price policies. That outcome cannot be generalized to every initialization: the same finite game and greedy full-row update rule admit a self-enforcing high-price policy, including from the probe's reference state 16.

This follows the user's request to investigate historical causation and generality, under the [research method](../research-method.md). It is an analytical construction from preserved native payoffs, not another training campaign, a discovered native outcome, or a practical initialization remedy. Historical causation remains a separate empirical question.

## Construction and exact certificate

Keep the native two-firm, 15-price, 225-state, memory-one game, binary64 gamma .95 and alpha .15. State records the previous joint action as `15*a0+a1`. Action 1 prices at 1.466472142857143, near the continuous competitive benchmark 1.47293. Action 6 prices at 1.6602078571428573.

Both firms prescribe action 6 at states 16=(1,1) and 96=(6,6), and action 1 in all other 223 states. From state 16, joint compliance gives state 96 and then stays there at the higher price. A unilateral departure from cooperation produces another state; one compliant low-price period returns to state 16, and cooperation resumes. Departing during that low-price response postpones recovery.

For each firm, let rH be its native payoff at (6,6) and rL its payoff at (1,1). These are approximately .292531963 and .220205447. The candidate discounted values are:

- At cooperative states, `VC=rH/(1-gamma)`, approximately 5.850639255.
- Elsewhere, `VP=rL+gamma*VC`, approximately 5.778312738.

For every firm, state and unilateral action, compute `Q*(s,a)=native_reward(s,a)+gamma*V(next(s,a))`. Native payoff and parameter floats are represented as exact binary rationals, rather than replacing gamma with mathematical 19/20. Direct checking of **450 firm-state rows and 6,750 action values** establishes `max_a Q*(s,a)=V(s)` exactly, with the prescribed action uniquely optimal everywhere.

The smallest cooperative action margin is **.05726774749**. The smallest low-response-state margin is **.06981093828**. These are prescribed-action value advantages over the best alternative, not solver-error estimates or observed market gains. They certify both cooperation and the off-path response in the finite coded game.

The Bellman inequalities also bound arbitrary unilateral history-dependent deviations: recursively summing them gives a discounted reward bound plus a bounded terminal value whose discounted contribution vanishes. Equality under the prescribed policy establishes a best response from every state. Both policies therefore support mutual best responses and high-price cooperation in this stated game.

## Full-row updating and binary64

Initialize the tables to this constructed Q*. With greedy opponent actions, every full-row target equals its existing exact value, so this is a mathematical fixed point of `Q + alpha*(reward + gamma*maxQ(next)-Q)` for coupled learners. Wider coverage does not select the competitive solution from every possible table.

An independent arithmetic bound strengthens the statement for rounded binary64 initialization. A sup-norm error tube of radius 1e-8 preserves the strict greedy choices. For cells inside the tube, the ideal relaxed update contracts error by `1-alpha*(1-gamma)`; its slack is about 7.5e-11. A conservative bound on the five rounded operations is about 1.11023e-12, smaller than that slack. Correctly rounded Q* starts inside the tube, and consistent row-max caches remain valid. The greedy policies are therefore invariant for arbitrarily many such updates. This is a policy-invariance proof; bitwise constant Q cells are not asserted, and no operator updates were executed.

The guarantee requires greedy realized actions, consistent caches, the stated native rewards/parameters, prewrite targets and binary64 round-to-nearest. Native exploratory actions can change a target's rival price and lie outside this guarantee. The initialization is deliberately Bellman-optimal and differs from the default initializer and the saved learned tables. The small invariant tube does not establish a typical basin of attraction or default-run frequency.

## What generalizes, and what remains open

The checked counterexample rejects universal competitive collapse across starting tables, including when evaluation starts at state 16. It does not contradict the two observed repairs, establish that native bots learn this policy, refute a claim restricted to default-generated tables, or estimate prevalence. Whether full coverage preserves or repairs high-price behavior across the remaining saved controllers is still unmeasured.

This is the familiar repeated-game role of a credible low-price response and recovery; no novelty is claimed. The original pricing paper reports learned high-price strategies with finite punishment and return to cooperation, whereas this account verifies a deliberately specified strategy in the project's finite grid. [Calvano et al. (2020)](https://pubs.aeaweb.org/doi/10.1257/aer.20190623). Exploration rate/mode and feedback protocol are separate issues in the subsequent literature. [Calvano et al. (2023)](https://cris.unibo.it/handle/11585/952593), [Asker, Fershtman and Pakes](https://www.nber.org/papers/w28535). Memoryless long-run convergence results concern another state regime; this construction retains price history. [Xu and Zhao, version 2](https://arxiv.org/abs/2409.01147v2).

An earlier, separately retained construction cooperates only at state 96 and prescribes competitive prices forever elsewhere. It also passes exact all-state checks, but its path from state 16 stays competitive. The finite-response construction above addresses that initial-state limitation. Both constructions and their scopes are retained; neither is relabeled as an observed native learning outcome.

The saved native chronology cannot presently identify the first failure or every successor-value ancestor: complete maps are saved only at selected checkpoints, while update samples are sparse. A faithful, passively instrumented replay with cloned interventions is a proposed next investigation, requiring a separately approved execution cap. No such replay, holdout or additional operator panel was run here.

## Evidence and verification

Independent calculations verified the compressed/decompressed checkpoint identities, native stage-game orientation, memory-one transitions, every rational action value and the conservative rounding bound. The source input is seed 3 Both's preserved terminal checkpoint; only its common native payoff/configuration is used, not its learned Q as the constructed initializer. The exact scripts and receipts are preserved at `survey/out/collusion-memory1-generality-proof-20261012/`, with file identities in its archive index. Earlier experimental archives remain unchanged.

The finite-response calculation SHA256 is `b1829c52bcbb12836861bcf7e9b6d0a38da0286055a0d12d139102381fd8027a`; its numerical certificate SHA256 is `1ed03b5a1760a3c23bbef2c418c6fb0cfc40a509f899ce87abd951ab932a387f`. Both proof scripts use direct rational arithmetic and analytic bounds, without a value solver or learner execution. No source helper, new training, native replay, publication, push or author contact is part of this result.
