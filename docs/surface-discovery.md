# Selecting paid surface experiments

Allowing the investigator to choose individual calibration actions produces **no utility gain over the supplied probe recipe** at the registered prices and four-trial horizon. Every one of the eight primary RawAdaptive–SuppliedAdaptive mechanism/role contrasts is exactly zero, and their own physical-event streams agree in all 2,560 matched episodes including transfer. RawAdaptive A buys one calibration round and earns expected net 49/4; RawAdaptive B buys none and earns 51/4. Relative to NoProbe, A gains only 1/4 expected investigator net, while B gains zero. The helper's utility and the adverse mechanism-specific results are reported below.

On the unmodeled DataFlip mechanism, apparently decisive in-catalog evidence supports wrong predictions. Raw and supplied adaptive policies have zero accuracy in both roles. A completed wrong episode is distinct from the unsupported helper history in B's InspectOnly control, whose terminal outcomes remain unavailable. This is an exact finite experiment with supplied models, actions, encoding and an informed helper. It does not establish unrestricted experiment discovery, learned language or fidelity to a real incident.

## Reproduce and inspect

```sh
cargo run --release -p sugarscape-cli -- surface-discovery diagnose > surface-discovery.json
```

The command accepts no scientific tuning flags. `surface-discovery-diagnostic-v1`, protocol 1, contains all 48 ordered settings and 256 equally weighted bit sequences each: 12,288 episodes. A successful exit requires current-payload and independent-reference integrity, including full reconstruction; a saved success flag alone is insufficient. Invalid diagnostic payloads exit 2 after emitting failure; output/flush failure exits 1. Resource/protocol errors are operational failures, never successful empty populations.

The first retained output is 191,112,194 bytes and uses 7,830 interned histories. Use streaming readers for the full report. The [compact measured summary](../crates/sugarscape-core/src/browser_experiments/fixtures/discovery-recorded.json) contains every setting aggregate/projection, all 24 primary contrasts, ten role expectations and projections, six role contrasts, 16 compilation records and all reference receipts. It omits episode histories. Native `summary_json` validated the retained first report before generating this asset; a separate streamed comparison checked every summary field against that report.

In the browser, choose **Surface experiment discovery** to run a bounded selected sequence. **Load recorded study results → surface discovery** opens the independent full-census summary. Changing, rerunning or replaying an episode does not change that summary or retrain a policy. Tables and exports preserve exact integer/fraction atoms; the browser wire represents integers as decimal strings. Agent views retain their own local history and exclude archival evaluator payloads at every checkpoint, including terminal and comparison views. Explicit Researcher views expose evaluation data. Retained research results never enter the simulated Agent's inference or policy input.

## Supplied structure and the comparison

A privately observes X and predicts Y; B observes Y and predicts X. Four fresh independent fair-bit pairs give 256 complete sequences. The four supplied mechanisms are shared persistent (SP), shared resetting at round boundaries (SR), private persistent (PP), and inert (IN). Phase/trial resets clear every field. Write acceptance alone does not certify shared effect or delivery. The seven supplied symbols are Blank, Probe0, Ack0, Probe1, Ack1, Data0 and Data1; the physical world does not interpret their semantics.

The experiment preserves the [shared-surface physics](shared-surface.md) and [active-surface prices, clocks and live routines](active-surface.md). Each Agent starts with 48 credits. Read, write and wait cost one; trusted target inspection costs four. A completed calibration round charges each Agent two scheduled actions. At most three calibration rounds precede four live trials. Buying a round or stopping at its boundary is free; stopping removes the remaining calibration slots and charges. Calibration occurs before task bits are issued. Each correct free prediction earns 12; net utility is reward minus all paid credits. The optimized objective is remaining investigator net utility, with helper and group outcomes measured separately.

RawAdaptive chooses Wait, Read or a write of any of the six nonblank supplied symbols at its own paid calibration slots. SuppliedAdaptive chooses how many rounds of the supplied recipe to buy, with the same informed helper and live controller. NoProbe skips calibration and optimizes live choices. InspectOnly skips calibration and always inspects. KnownRaw receives the matching point prior and optimizes raw choices; it is an in-family control, absent from DataFlip. Other unknown investigators begin Uniform over the four models. The helper receives the actual in-family point prior; on DataFlip it receives nominal SP. It is a fixed supplied helper, not an independently discovered or negotiated collaborator.

Inference uses only the Agent's own complete prefix, own prior/current bit/credits, and public clock/reset/phase events. Own actions are interventions, not extra likelihood evidence. Candidate helper behavior is replayed with candidate-local information and original opaque IDs. Neither actual mechanism, peer prefix, future bits, task truth, write lineage nor realized reward feedback is available to the controller. All routine/encoding choices and horizons are supplied. Ties stop calibration, choose live Inspect, and follow the declared raw-action order Wait, Read, Probe0, Ack0, Probe1, Ack1, Data0, Data1. Forced-control alternatives retain their declared continuation policy.

## Measured actions, values and equivalence classes

All 256 sequences in each raw setting follow the action path below. A chooses Continue initially (49/4 versus Stop 12), writes Probe0 at round 1 slot 1, then reads at slot 4. At the write decision Probe0 has value 49/4; Wait/Read/Probe1 have 11, while Ack0/Ack1/Data0/Data1 have 23/2. At the following owned slot Read has 53/4 versus Wait 12. These values are expected remaining investigator utility at their own decision times, not realized terminal rewards or directly comparable values at different times.

After that round A always stops. For SP/SR, Stop is 33/2 versus another round 16, and the first live Attempt is 33/2 versus Inspect 15. The paid live read distinguishes persistent from resetting behavior. For PP/IN, Stop is 12 versus Continue 10; Inspect is 12 versus Attempt 9. B stops immediately (51/4 versus Continue 49/4), then attempts the first live trial (51/4 versus Inspect 12). B learns the useful SP versus non-SP distinction during that attempt.

| Investigator | Mechanism | Calibration | Four live choices |
| --- | --- | --- | --- |
| A | SP | Write Probe0 → Read; stop after 1 | Attempt → Attempt → Attempt → Attempt |
| B | SP | Stop at 0 | Attempt → Attempt → Attempt → Attempt |
| A | SR | Write Probe0 → Read; stop after 1 | Attempt → Inspect → Inspect → Inspect |
| B | SR | Stop at 0 | Attempt → Inspect → Inspect → Inspect |
| A | PP | Write Probe0 → Read; stop after 1 | Inspect → Inspect → Inspect → Inspect |
| B | PP | Stop at 0 | Attempt → Inspect → Inspect → Inspect |
| A | IN | Write Probe0 → Read; stop after 1 | Inspect → Inspect → Inspect → Inspect |
| B | IN | Stop at 0 | Attempt → Inspect → Inspect → Inspect |
| A | DataFlip | Write Probe0 → Read; stop after 1 | Attempt → Attempt → Attempt → Attempt |
| B | DataFlip | Stop at 0 | Attempt → Attempt → Attempt → Attempt |

A's calibration read separates PP and IN individually while leaving the shared class {SP, SR}; the first live read then separates SP and SR. A identifies the true in-family model with mass 1 in every mechanism. PP/IN are identified after two investigator credits; SP/SR after seven. B identifies SP after three credits, but retains the equal {SR, PP, IN} class otherwise: final true-model mass 1/3 and unique-identification mass 0 in each of those settings. Further model discrimination does not improve the chosen live routine at these prices.

The selected raw actions reproduce the supplied recipe's physical trajectory. The exhaustive first-report comparison finds identical own physical-event streams for all 2,560 RawAdaptive/SuppliedAdaptive pairs across the five environments and both roles. Raw decision records expose the additional action alternatives; they do not imply a better experiment or changed confidence along these measured paths. An allowed hypothetical early Data write can reveal the transfer inversion, but **no measured raw calibration trajectory selects it**. That remains a counterfactual inference witness, not an observed discovery.

## Complete primary setting results

All 40 in-family settings complete with failure/unavailable mass zero. Each row averages all 256 sequences. Pairs are **(investigator, helper)**; for investigator B this reverses the physical (A, B) order stored in the report. Accuracy is expected correct predictions divided by four. Credits, rewards and net are exact episode means. Group net sums both Agents.

| Investigator | Mechanism | Policy | Rounds | Accuracy (I,H) | Credits (I,H) | Reward (I,H) | Net (I,H) | Group net |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | SP | RawAdaptive | 1 | (1, 1) | (26, 26) | (48, 48) | (22, 22) | 44 |
| A | SP | SuppliedAdaptive | 1 | (1, 1) | (26, 26) | (48, 48) | (22, 22) | 44 |
| A | SP | NoProbe | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| A | SP | KnownRaw | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| A | SP | InspectOnly | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| B | SP | RawAdaptive | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| B | SP | SuppliedAdaptive | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| B | SP | NoProbe | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| B | SP | KnownRaw | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| B | SP | InspectOnly | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| A | SR | RawAdaptive | 1 | (7/8, 1) | (35, 38) | (42, 48) | (7, 10) | 17 |
| A | SR | SuppliedAdaptive | 1 | (7/8, 1) | (35, 38) | (42, 48) | (7, 10) | 17 |
| A | SR | NoProbe | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | SR | KnownRaw | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | SR | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| B | SR | RawAdaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | SR | SuppliedAdaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | SR | NoProbe | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | SR | KnownRaw | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| B | SR | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | PP | RawAdaptive | 1 | (1, 1) | (38, 38) | (48, 48) | (10, 10) | 20 |
| A | PP | SuppliedAdaptive | 1 | (1, 1) | (38, 38) | (48, 48) | (10, 10) | 20 |
| A | PP | NoProbe | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | PP | KnownRaw | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | PP | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| B | PP | RawAdaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | PP | SuppliedAdaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | PP | NoProbe | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | PP | KnownRaw | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| B | PP | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | IN | RawAdaptive | 1 | (1, 1) | (38, 38) | (48, 48) | (10, 10) | 20 |
| A | IN | SuppliedAdaptive | 1 | (1, 1) | (38, 38) | (48, 48) | (10, 10) | 20 |
| A | IN | NoProbe | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | IN | KnownRaw | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| A | IN | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| B | IN | RawAdaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | IN | SuppliedAdaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | IN | NoProbe | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| B | IN | KnownRaw | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| B | IN | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |

## All 24 primary stratum contrasts

These are paired **left minus right** means over the same complete sequences, separately for each role and mechanism. A positive credit difference means more physical cost. Raw and supplied coincide everywhere. Compared with NoProbe, A gains investigator net 10 on SP but loses 5 on SR and 2 on each PP/IN. B's three policies coincide in every stratum. Equal role-weighted or pooled results would conceal this asymmetry.

| Investigator | Mechanism | Comparison | Δ credits (I,H) | Δ reward (I,H) | Δ net (I,H) | Δ group net |
| --- | --- | --- | --- | --- | --- | --- |
| A | SP | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | SP | RawAdaptive − NoProbe | (-10, 2) | (0, 24) | (10, 22) | 32 |
| A | SP | SuppliedAdaptive − NoProbe | (-10, 2) | (0, 24) | (10, 22) | 32 |
| B | SP | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | SP | RawAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | SP | SuppliedAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | SR | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | SR | RawAdaptive − NoProbe | (-1, 2) | (-6, 0) | (-5, -2) | -7 |
| A | SR | SuppliedAdaptive − NoProbe | (-1, 2) | (-6, 0) | (-5, -2) | -7 |
| B | SR | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | SR | RawAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | SR | SuppliedAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | PP | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | PP | RawAdaptive − NoProbe | (2, 2) | (0, 0) | (-2, -2) | -4 |
| A | PP | SuppliedAdaptive − NoProbe | (2, 2) | (0, 0) | (-2, -2) | -4 |
| B | PP | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | PP | RawAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | PP | SuppliedAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | IN | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | IN | RawAdaptive − NoProbe | (2, 2) | (0, 0) | (-2, -2) | -4 |
| A | IN | SuppliedAdaptive − NoProbe | (2, 2) | (0, 0) | (-2, -2) | -4 |
| B | IN | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | IN | RawAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | IN | SuppliedAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |

## Both-role expectations and controls

Each row below weights SP, SR, PP and IN equally **within its named investigator role**, then averages all 256 sequences. It is not a pooled-role estimate. KnownRaw yields investigator/helper net 15/15 in either orientation. InspectOnly and NoProbe agree for A, while B's first NoProbe attempt improves expected net over InspectOnly despite reduced accuracy on non-SP mechanisms. These are supplied controls, not free information for the unknown investigator.

| Investigator | Policy | Accuracy (I,H) | Credits (I,H) | Reward (I,H) | Net (I,H) | Group net |
| --- | --- | --- | --- | --- | --- | --- |
| A | RawAdaptive | (31/32, 1) | (137/4, 35) | (93/2, 48) | (49/4, 13) | 101/4 |
| A | SuppliedAdaptive | (31/32, 1) | (137/4, 35) | (93/2, 48) | (49/4, 13) | 101/4 |
| A | NoProbe | (1, 7/8) | (36, 33) | (48, 42) | (12, 9) | 21 |
| A | KnownRaw | (1, 1) | (33, 33) | (48, 48) | (15, 15) | 30 |
| A | InspectOnly | (1, 7/8) | (36, 33) | (48, 42) | (12, 9) | 21 |
| B | RawAdaptive | (29/32, 1) | (123/4, 33) | (87/2, 48) | (51/4, 15) | 111/4 |
| B | SuppliedAdaptive | (29/32, 1) | (123/4, 33) | (87/2, 48) | (51/4, 15) | 111/4 |
| B | NoProbe | (29/32, 1) | (123/4, 33) | (87/2, 48) | (51/4, 15) | 111/4 |
| B | KnownRaw | (1, 1) | (33, 33) | (48, 48) | (15, 15) | 30 |
| B | InspectOnly | (1, 7/8) | (36, 33) | (48, 42) | (12, 9) | 21 |

All six within-role expectation contrasts follow. Helper gains are explicit; the objective does not guarantee helper participation or aligned incentives.

| Investigator | Comparison | Δ credits (I,H) | Δ reward (I,H) | Δ net (I,H) | Δ group net |
| --- | --- | --- | --- | --- | --- |
| A | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| A | RawAdaptive − NoProbe | (-7/4, 2) | (-3/2, 6) | (1/4, 4) | 17/4 |
| A | SuppliedAdaptive − NoProbe | (-7/4, 2) | (-3/2, 6) | (1/4, 4) | 17/4 |
| B | RawAdaptive − SuppliedAdaptive | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | RawAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |
| B | SuppliedAdaptive − NoProbe | (0, 0) | (0, 0) | (0, 0) | 0 |

## DataFlip: supported wrong answers and unsupported history

DataFlip retains the physical shared/persistent behavior but reverses delivered Data symbols; the inversion is absent from both Agents' model catalog. Raw/supplied A and raw/supplied/NoProbe B assign point confidence to nominal SP and issue confidently wrong predictions in every live trial. For each trial, half the population issues probability 0 while the target is always 1 in that subgroup, and half issues probability 1 while the target is always 0. All those episodes complete, have accuracy zero, and earn no reward. Their observed lineage reads certify a writer, not truthful encoding. True-model and unique-true-model scores are unavailable because the actual mechanism is outside the catalog; nominal-SP identification is not identification of DataFlip.

A's NoProbe/InspectOnly investigator remains correct by trusted inspection, while its helper gets only 1/2 accuracy. B's InspectOnly control has a different outcome: helper A encounters an unsupported own history at its first-trial Read, round 3 slot 1. Every sequence stops there before either Agent issues a prediction. Physical (A,B) credits already spent are (5,7), or (investigator,helper) = (7,5). Completed mass is zero and unavailable mass one. Reward, net, accuracy and group net are **Unavailable**, not zero and not averages of surviving episodes.

| Investigator | Policy | Completed mass | Unavailable mass | Accuracy (I,H) | Credits including partial (I,H) | Reward (I,H) | Net (I,H) | Group net |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | RawAdaptive | 1 | 0 | (0, 0) | (26, 26) | (0, 0) | (-26, -26) | -52 |
| A | SuppliedAdaptive | 1 | 0 | (0, 0) | (26, 26) | (0, 0) | (-26, -26) | -52 |
| A | NoProbe | 1 | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| A | InspectOnly | 1 | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| B | RawAdaptive | 1 | 0 | (0, 0) | (24, 24) | (0, 0) | (-24, -24) | -48 |
| B | SuppliedAdaptive | 1 | 0 | (0, 0) | (24, 24) | (0, 0) | (-24, -24) | -48 |
| B | NoProbe | 1 | 0 | (0, 0) | (24, 24) | (0, 0) | (-24, -24) | -48 |
| B | InspectOnly | 0 | 1 | (Unavailable, Unavailable) | (7, 5) | (Unavailable, Unavailable) | (Unavailable, Unavailable) | Unavailable |

The compact summary retains every read/write/wait/inspection/attempt count, lineage-read count, calibration-round distribution, first-failure checkpoint, discovery source and posterior-calibration bin for every setting and both-role expectation. These projections preserve the full population rather than conditioning on completion. In particular, the helper's partial attempted send and the investigator's completed inspection remain counted in the unsupported B InspectOnly population.

## Computation and operational limits

The following are the measured compilation owners. Each owns cumulative decision-state/branch counts and peak live allocation through its applicable environment evaluations. These are implementation accounting values, distinct from physical credits, OS RSS and WASM linear memory. Registered ceilings remain 360,304 states, 993,058 branches and 1,443,627,008 allocation bytes per owner; exhaustion is an operational Error. The largest measured row uses 302,008 states, 275,345 branches and 456,748,910 allocation bytes. No limit or tolerance changed after measurement.

| Investigator | Policy | Prior | Candidate worlds | Decision states | Branches | Allocation bytes |
| --- | --- | --- | --- | --- | --- | --- |
| A | RawAdaptive | Uniform | 1024 | 302008 | 275345 | 456748910 |
| A | SuppliedAdaptive | Uniform | 1024 | 296216 | 267367 | 450244475 |
| A | NoProbe | Uniform | 1024 | 267020 | 244327 | 308147017 |
| A | KnownRaw | {'PointMass': 'SharedPersistent'} | 256 | 61877 | 54848 | 97910179 |
| A | InspectOnly | Uniform | 1024 | 267020 | 244327 | 308147017 |
| B | RawAdaptive | Uniform | 1024 | 279297 | 247773 | 367620627 |
| B | SuppliedAdaptive | Uniform | 1024 | 278231 | 244317 | 366646298 |
| B | NoProbe | Uniform | 1024 | 278231 | 244317 | 366646298 |
| B | KnownRaw | {'PointMass': 'SharedPersistent'} | 256 | 60846 | 54898 | 98325992 |
| B | InspectOnly | Uniform | 1024 | 251115 | 222301 | 324782194 |
| A | KnownRaw | {'PointMass': 'SharedResetting'} | 256 | 61489 | 54650 | 109460483 |
| B | KnownRaw | {'PointMass': 'SharedResetting'} | 256 | 63500 | 54730 | 108484374 |
| A | KnownRaw | {'PointMass': 'PrivatePersistent'} | 256 | 61462 | 54669 | 109435832 |
| B | KnownRaw | {'PointMass': 'PrivatePersistent'} | 256 | 63470 | 54605 | 108456984 |
| A | KnownRaw | {'PointMass': 'Inert'} | 256 | 61352 | 54335 | 109335402 |
| B | KnownRaw | {'PointMass': 'Inert'} | 256 | 63392 | 54335 | 108385770 |

The browser runs a selected episode, not the full census. It retains the existing 64 KiB input, 16 MiB record, 4,096-checkpoint, one-request, 60-second timeout and two-record limits. Cancellation/timeouts dispose the pending worker and preserve existing records. These bounds do not establish a universal runtime guarantee. The first native invocation took 82.563 seconds and its one repeat 82.459 seconds on the recorded host; both exited zero with empty stderr and identical complete bytes. Independent streaming validation reconstructed all 12,288 episodes in 40.383 seconds using a disk-backed history table. These timings are observations, not scientific utility or portable performance promises.

## Measurement and delivery provenance

The scientific first measurement is frozen at commit `4f4d1f6223516f5c554c0ae98101ef51329fc432`. Its immutable registration SHA256 is `51a3c17792520415c30b8917a451b432a5985c11aa9f0e70f05741a1f871dbed`. First and repeat stdout each hash to `1bd791d959f02666d8f23992fdff917a4b9b51701643734032049e5b79b31c34`. The compact Summary's `report_hash` is `a4377fcebbbb92c5d389c319a8098303ed4db812dfb528eff36e5d35e4a23606`: it hashes the same compact report JSON **without** the CLI's trailing newline. Native scientific source identity is `bea9272bcd6e9aab412d1aafd36dc9facd0ffffd8b10ccfd7e5bb2aaac9fa055`. The 290,140-byte compact asset, including its newline, hashes to `a2c5d35c3ef25722da405276bd8f00b4aab6b424d2fd34b4e21c62de15cd7660`.

The [registered protocol](superpowers/protocols/2026-10-10-surface-experiment-discovery.md) predates judged collection and declares the census, priors, helper embedding, action/tie order, metrics, limits and pre-mortem. The independent reference derivation and full 12,288-episode comparison are retained with all original evidence under `.superpowers/sdd/2026-10-10-surface-experiment-discovery/`. First/repeat/status, independent validation and CLI-success receipts are immutable. Earlier engineering/reference series are not relabeled as scientific first measurements.

This guide, measured summary asset, recorded-results registration/presentation and additive source-selector entry are later delivery changes. A separate delivery source/runtime manifest records that delta, and all 202 original runtime artifacts are archived before rebuild. The original scientific 21-file identity and settings remain unchanged; rebuilt CLI/WASM/browser artifacts carry their own delivery identities. Original guides retain their complete old prefixes. The whole-increment review and integration checks inspect the measured evidence and delivery bindings separately.
