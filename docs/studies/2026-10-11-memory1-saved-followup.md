# Memory1 pricing: saved-data examination of deterrence and remaining weaknesses

**2026-10-11; retrospective exploratory follow-up.** The [component-feedback experiment](2026-10-11-memory1-component-refresh.md) produced genuinely defended high-price cycles, but its remaining weaknesses cannot all be explained by unvisited situations. Frequently refreshed estimates can still favor actions that the certified evaluator finds inferior. The saved evidence establishes these distinctions without identifying the particular learning event that caused each outcome.

The user approved three saved-data examinations: off-path exposure, the seed 4 and seed 3 exceptions, and the response paths behind all seven cycle passes. No new training, seeds, tuning, exploration restart or policy fitting occurred. The original experiment remains the single 34,044,310-period campaign. This follow-up uses its frozen checkpoints, certified values, component ledgers, sampled transitions and block summaries under the [research method](../research-method.md).

## What remained vulnerable, and how often it was encountered

Study the three Future passes (seeds 1/3/7) and four Both passes (1/5/7/8). All Future passes and Both seed 5 finished natively; Both seeds 1/7/8 are censored snapshots. A firm-state is vulnerable when its saved sustained-deviation raw-gain lower bound exceeds 1e-8. A state is counted once per controller if either firm is vulnerable. Terminal vulnerability is held fixed while examining earlier exposure; the analysis does not claim the same vulnerability existed at the time of each visit.

All **894 vulnerable state/controller entries** received treatment visits. Of these, 892 have at least 1,000 lifetime visits, often mostly inherited from the untreated prefix. Many receive only tens or hundreds of treatment visits. Lifetime exposure, treatment exposure, selected-action updates and all-action feedback writes are different quantities.

| Passing pair | Vulnerable states / states in shared reachable union | Treatment visits to those vulnerable states, as share of all treatment periods |
| --- | ---: | ---: |
| Future 1 | 14/93 | 0.4351% |
| Future 3 | 45/93 | 5.5572% |
| Future 7 | 28/95 | 8.4719% |
| Both 1, censored | 11/69 | 1.0173% |
| Both 5 | 60/69 | 2.4472% |
| Both 7, censored | 97/105 | 55.8512% |
| Both 8, censored | 38/82 | 3.1867% |

Both seed 7 is a useful counterexample to a blanket sparse-exposure explanation. Its worst shared-union witness for both firms, state 32, received **20,643 treatment visits**, with reward and future feedback refreshed for every action on every visit. Its last visit was 48,301 periods before the final snapshot. Conversely, Future seed 1's worst shared witnesses received only 76 and 144 treatment visits; the final learned actions there received only 4 and 26 selected reward updates. Frequency and recency differ across the cases.

The shared union combines states reachable through either firm's departures within one frozen controller. Every pair also has vulnerabilities within a firm's own reachable set; failure is not solely an artifact of evaluating one firm in states reachable only by the other. A positive gap conditional on reaching such a state does **not** establish a profitable departure from the original cycle: the earlier actions needed to get there can cost more than the later opportunity earns. Cycle-start deterrence and off-path weaknesses can coexist.

Visits are reconstructed from component count differences, checked across both firms and against elapsed periods. Both's all-action counts are divided by 15; extra action writes are not additional visits. Future's reward counts identify selected-action updates. Both's component counts do not identify which action was actually chosen on each visit. The native cumulative visit counters are separately available; convergence streaks are not treated as visit counts.

## The exceptions survive more careful comparisons

Cycle selection changes the evaluated population. To check whether it alone explains the exceptions, compare the original prefix cycle and all 225 states, both common across arms. An additional retrospective population takes the union of fixed-prefix reachable states across all arms and saved endpoints: 225 states for seed 3, 224 for seed 4. The original archive's within-controller shared union is not mislabeled as a common cross-arm population. Equal-period fork+100k comparisons are retained alongside endogenous terminal endpoints.

| Seed and terminal arm | Own-cycle mean E | Original-prefix-cycle mean E | All-state mean E |
| --- | ---: | ---: | ---: |
| 4 Control | .00728305 | .00728305 | .01485959 |
| 4 Future | .02066329 | .02486909 | .02097773 |
| 4 Both | .01029753 | .01003349 | .01021091 |
| 3 Future | 0, strict pass | .00004328 | .00065307 |
| 3 Both | .00572122 | .00557452 | .00588737 |

E retains the original definition: equally average `(1-native gamma)*(Vstar-Vpi)` over both firms and the stated state population. These comparisons hold starting states fixed, not the rival policy or future successor dynamics.

**Seed 4 Future worsens beyond its selected cycle.** It is worse on the original state 144 and all states as well; the difference is already present at 100k. Both worsens on its selected and original cycles but improves the all-state mean. Thus the two interventions' exceptions are heterogeneous; describing Both as uniformly worse would be incorrect. All four seed 4 endpoints finished natively at different periods.

**Seed 3 Future outperforms Both beyond its selected cycle.** Both fail at 100k, with Future already less exploitable. At terminal, Future passes its fixed-point cycle at state 95, while Both's two-state cycle 16/96 fails. Future also has lower exploitability on the original prefix states and all states, while still failing those broader strict tests.

Both's failing seed 3 cycle is not simply an untouched row. At state 16, every action received **320,649 treatment refreshes of each component**, most recently at the final event. Both firms' learned scores favor action 6 over certified alternative 3, despite the reward component favoring 3; the residual future component offsets that reward advantage. Seed 4 Both has a similar fresh, heavily refreshed row at state 16. This locates ranking disagreements in the saved accounting. It does not identify a unique causal mechanism: the residual component, changing rivals, successor estimates, visits and endogenous stopping evolve together.

Stride-1000 samples identify differences between observed local action choices, not the first policy transition or its cause. Whole controller maps exist only at the prefix, 100k and terminal checkpoints. No intermediate global-optimality classification is invented.

## How the high-price cycles deter departures

All seven passing endpoints are fixed points. The three native-finished Future pairs and native-finished Both seed 5 share actions (6,5), prices approximately **(1.66021, 1.62146)**, above the competitive benchmark approximately 1.47293. The three censored Both passes have different fixed-point prices.

For every pair, both firms and all 14 nonprescribed first actions, evaluate immediate profit plus discounted continuation under either the learned own policy or the saved optimal sustained response to the rival's full committed map. **All 196 alternatives have strictly negative total gains even with optimal continuation**, with saved numerical error bounds retained. Deterministic response paths are evaluated as finite prefixes plus exact geometric cycle tails using the native binary64 rewards and gamma.

For example, Future seed 1 firm 0 can cut its price from about 1.66021 to 1.54397 and raise current profit from .272305 to .287081: a gain of **.014776**. The next price pair becomes approximately (1.46647, 1.42773), reducing its profit to .204057, before the original price pair returns. The discounted continuation loss is **.064836**, so the overall departure loses **.050060**. Even optimal sustained continuation cannot improve that example. Firm 1 has different temptations and losses; the other passing maps have distinct response paths despite sometimes sharing the same baseline prices.

The examples are finite low-profit responses and recovery, not permanent price wars. The best immediate undercut need not be the closest alternative in total value. Some closest alternatives instead lose profit immediately and restore the baseline without an additional continuation loss. Thus not every departure should be described as deterred by punishment.

The calculation establishes incentives for a departing firm **against a committed rival map**. It does not establish that the rival would find every off-path response optimal, or that the pair is a sequential or global equilibrium. The original shared-support and all-state failures remain.

## What this adds, and what remains unresolved

The original intervention established a substantial conditional effect of continuation feedback. This follow-up adds a concrete account of successful deterrence and rejects two overly simple explanations: all remaining weaknesses being unvisited, and the Future exceptions being solely a change in which cycle is scored. It also retains the meaningful benefit of better feedback without treating fresh feedback as a guarantee of certified optimal action rankings.

The remaining question is how learned action-ranking disagreements and frozen-controller vulnerabilities persist under repeated feedback while policies change. The present saved record narrows that question but cannot identify its cause uniquely. Another scientific campaign would need a separately fixed design that distinguishes competing explanations; no such campaign is launched or authorized by this account. These findings do not estimate prevalence, establish novelty, or support publication or external validity.

The original archive at `survey/out/collusion-memory1-components-20261011/` remains frozen. Follow-up scripts, input hashes, numeric results, failures, reviews and completed tracker are preserved separately at `survey/out/collusion-memory1-saved-followup-20261011/`. New calculations use saved values and frozen deterministic paths, without rerunning the original numerical solvers or training executable. This is a retrospective working study rather than a publication-ready finding.

Independent review verified 93 distinct original inputs, reconstructed reachability and native visit-count comparisons, recomputed all 196 departure paths and the common-state contrasts, and checked component rankings and counts. Synthetic and saved-data arithmetic checks passed. Separate saved-only regenerations produced nine byte-identical numeric, hash and report outputs. These checks verify the derivations from the frozen evidence; they do not reproduce the original training or supply an external trust anchor for the input manifest. The corrected common-population assertion and exposure-total correction remain recorded in the follow-up evidence.
