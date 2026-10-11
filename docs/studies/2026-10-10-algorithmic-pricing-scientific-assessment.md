# Algorithmic pricing and scientific interpretation

**Working research note and write-up outline.** Updated 2026-10-10. This assessment follows the [research method](../research-method.md): faithful reproduction, charitable source clarification where necessary, and claim-level scientific evaluation. It adds a primary-source reading and input-availability audit, not a new simulation or certified strategy-value result.

## What the current evidence supports

The original Calvano et al. computational baseline reproduces closely under the authors' code conventions. The project reports 100/100 period-for-period docking and recovered table values. That establishes the named computation; equilibrium and reward-punishment explanations remain separate claims. [Original paper and replication gateway](https://www.aeaweb.org/articles?id=10.1257/aer.20190623).

Our [memoryless diagnostic](2026-10-10-memoryless-pricing-diagnostic.md) and [cloned component study](2026-10-10-memoryless-component-refresh.md) establish high prices under a specified no-price-history state with retained discounting. Extra feedback substantially reduces the result; refreshing residual future credit is the stronger price reduction in eight cloned checkpoints. These are exploratory counterfactuals, not reproductions of the paper's original memoryless control or proof against its history-dependent mechanism.

The original stateless appendix jointly sets memory and discounting to zero and changes learning/exploration settings. A stationary single-state optimal-action ranking can be independent of discounting while finite asynchronous learning paths still depend on it. Lambin's retained-discount memoryless critique already addresses this distinction. Our bookkeeping and cloned interventions provide additional conditional evidence, rather than the first observation of that problem. [Lambin working paper identity](https://ssrn.com/abstract=4498926), local July 2024 revision as recorded in the source audit.

## Claim map

| Claim or question | Present evidence | Scientific disposition and next test |
| --- | --- | --- |
| Published pricing computation | Original-code docking and table recovery | Preserve the reproducible phenomenon; keep exact code conventions distinct from project defaults |
| Learned strategies are best responses | Legacy code/prose comparison raises a one-period versus optimal-continuation discrepancy | Independently certify learned-policy and best-response values with matching states, tolerances and denominators |
| Memorylessness implies competitive behavior | Our retained-discount/default-learning counterfactual gives high prices | Qualifies a broad inference; separately dock the appendix's bundled parameter changes |
| Exploration distinguishes genuine from spurious collusion | Authors' 2023 response acknowledges spurious outcomes and tests a different market/rate regime | Read its defense in its own scope; retrieve figure-specific inputs before testing source equivalence |
| A price cut followed by low prices proves enforcement | The response pattern reproduces; alternative learning dynamics also generate it | Compare discounted profitable deviations and recovery, not response shape alone |
| A price increase triggering punishment disproves collusion | Our earlier upward-deviation observations are real | This is insufficient: grim-trigger-style strategies can punish any deviation; test incentive support directly |
| Continuation-value feedback is a novel discovery | Substantial prior selective-learning and continuation-mechanism literature exists | Compare the precise intervention and regime before claiming a new contribution |
| The original memory1 mechanism fails | Current memoryless interventions do not establish this | Audit the saved history-dependent policies first; preserve legitimate action-dependent future targets in later interventions |

A one-action deviation test is not inherently invalid. Checking all relevant states against exactly evaluated policy continuation can certify optimality through policy improvement. Checking only an observed cycle can miss profitable improvements after entering off-path states. Conversely, our existing optimal-action predicate is mathematically meaningful: if prescribed actions are optimal throughout a closed cycle, that cycle attains the corresponding optimal value. A certified value gap would measure exploitability explicitly rather than treating the existing predicate as a coding error.

## Authors responses and the strongest fair interpretation

Calvano et al.'s 2023 response explicitly accepts that high prices can be spurious. Its homogeneous rectangular-demand market uses learning rate alpha 0.3 and exploration decay beta 4e-4, compared with our differentiated-goods alpha 0.15/beta 4e-6 defaults. Its notation also uses epsilon for exploitation probability, so random exploration is 1-epsilon; translations must preserve that convention. Its inspected memoryless figure does not establish a numerical discount setting for our comparison. Treating our regime as an exact failed replication of that figure would misstate the evidence. [Published response](https://doi.org/10.1016/j.ijindorg.2023.102973), [April 2023 accepted manuscript](https://cris.unibo.it/bitstream/11585/952593/4/True_Or_Fake_IJIO_Revision%2Bsubmitted.pdf).

The authors also address upward punishments: a strategy can punish any departure from cooperative play. That observation weakens the diagnostic specificity of a price-cut response but does not by itself establish that punishment is nonstrategic. The decisive question is whether following the learned policy remains preferable to an optimized sustained deviation in the stated evaluation setting.

Asker, Fershtman and Pakes show that chosen-action and counterfactual learning can yield different prices. Xu and Zhao's 2026 revision explicitly analyzes immediate-payoff and continuation-learning contributions to memoryless dynamics. Their update decomposition and limits differ from our reward-EMA/residual-history accounting; neither distinction licenses a broad priority claim. [Asker et al. working paper](https://www.nber.org/papers/w28535), [Xu and Zhao version2](https://arxiv.org/abs/2409.01147v2).

The strongest current account is therefore conditional: reproducible high prices may coexist with genuine enforcement, biased valuation or both, depending on policy, state and learning regime. Mechanism assessment belongs to specified sessions and states, with aggregate prevalence and uncertainty reported separately.

## A feasible next audit using saved policies

The eight saved memory1/asynchronous controllers contain two complete 225-by15 score tables per run: six ended under the native convergence criterion and two are horizon-censored snapshots. Preserve those labels during frozen-policy evaluation. Read-only input inspection found 3,600 policy rows with unique maxima beyond the implementation's floating tie tolerance; the smallest first/second gap was 3.0407839e-5. Recovered current-state actions match all stored current actions. This establishes recoverability, not optimality.

The native payoff table is also recoverable without new training: 67 archived frames contain all 450 firm/action/rival-action payoff entries, with repeated entries consistent. Input files and checks are retained in the local audit evidence. These controllers use project RNG and lowest ties; conclusions about them are conditional baseline-variant evidence. Authors-code RAN2/random-tie coverage remains a separately specified comparison.

The proposed next question is whether the saved high-price, history-dependent policies actually deter sustained unilateral deviations. Freeze both firms' policy maps, evaluate the learned-policy discounted value, independently solve each firm's optimal discounted Markov decision problem against its frozen rival, and certify numerical error using Bellman residuals and an independent solution method. Report the value difference at observed cycles, deviation-reachable states and all states separately. Compare one-period departures followed by resumption of the learned policy with sustained optimized departures, and retain explicit deviation witnesses. Resuming the policy need not return to the original cycle.

The proposed numerical classification tolerance is 1e-8 in absolute discounted-profit model units. Each learned-policy and optimal-value solution must have certified error at most 1e-10, accounting for floating evaluation and converting the maximum Bellman residual into a value-error bound using 1/(1-delta). Add both errors for a gap bound of at most 2e-10. Classify an improvement only when the gap interval lies above 1e-8; classify no improvement above tolerance only when its upper bound is at most 1e-8; otherwise retain an unresolved boundary. Relative and stationary-equivalent profit gains should accompany raw value differences, without conflating their denominators with the original paper's Q-loss. Unresolved numerical or provenance gates remain unresolved. Recovered policies and native payoff entries must be validated before evaluation; no convenient tie breaking or silent parameter changes are allowed.

This is a new deterministic saved-data probe, not yet implemented or executed. User approval of its question and scope is required before implementation or execution. It requires no new stochastic training episodes. If a value audit exposes profitable sustained deviations, it strengthens the interpretation issue directly. If the policy genuinely deters them, history-dependent enforcement survives a stronger test and memoryless findings remain a separate mechanism. If mixed, report the mixture rather than a verdict on the whole paper.

Only after that audit should a new memory1 component campaign be considered. Such interventions must retain each action's counterfactual next-state continuation. Substituting a common future target would remove the strategic mechanism being investigated. Compute/feedback-cost matching and practical corrective algorithms are subsequent questions.

## Potential write-up

A defensible account can follow five parts: original reproducible phenomenon; source/code and charitable interpretation boundaries; existing spurious-learning literature; the differentiated retained-discount memoryless diagnostics and cloned score-history evidence; and a separately certified history-dependent policy audit. Its contribution should be described as a precise comparative diagnosis and reproducible evaluation, with novelty qualified against the literature.

Publication readiness requires completed original-policy assessment, verified source versions and metrics, clear code/data provenance, numerical error certification, appropriate robustness and independent review. The present note is an internal working synthesis; it does not claim publication-ready findings, authorize publication, or authorize additional empirical execution.

Local evidence: `survey/out/collusion-scientific-audit-20261010/` and its source copy `/tmp/sugarscape-collusion-scientific-audit-20261010/`, including eight source-claim entries, identified primary-paper revisions, the independent next-probe assessment, policy and payoff availability checks. Original numerical archives remain authoritative and unchanged. Personal source PDFs stay local and are not redistributed in the repository.
