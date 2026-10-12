# Memory1 pricing: reconstructed recurrences and conditional feedback effects

**2026-10-12; one bounded native reconstruction with live cloned interventions.** The selected decision failures recur as both firms learn; they are not first-ever failures. At the adjacent checked points for case 3, the focal choice is the same while the rival environment and optimal first choice change. The two hold interventions have different effects across cases and can improve the first action while worsening the prescribed continuation. An extra 100k periods beyond native stopping does not eliminate either focal failure. These observations narrow the history without identifying a single necessary cause.

This follows the [component experiment](2026-10-11-memory1-component-refresh.md), [saved-data diagnostics](2026-10-11-memory1-saved-followup.md) and [operator probe](2026-10-11-memory1-operator-probe.md), under the [research method](../research-method.md). A separately approved [full saved-table panel](2026-10-12-memory1-full-panel.md) addresses generality; it repairs all eight sources under the tested full-coverage operator while retaining higher-price, all-state-certified endpoints in five cases. Native historical reconstruction and deterministic table counterfactuals remain different evidence.

## Faithful reconstruction and approved budget

Reconstruct only cases 3/4 from the preserved helper 300c9deb and exact source configurations: memory1/asynchronous Control prefixes of 1M, native binary64 alpha .15/gamma .95, beta4e-6, fifteen actions/225states and lowest epsilon-band ties. Each live prefix is cloned into original Control and Both continuations. Both switches to the documented synchronous mode only after the prefix gate. Compare prefix, branch+100k and terminal complete Q/best/greedy/R/B/counters/state/RNG/config/native fields against the component archive; also compare original Controls against the original raw run. All fidelity gates pass before intervention.

Original Both terminals are 1,941,103 and 1,988,907; Controls are 2M externally capped and 1,823,341 natively finished. Shared-prefix reconstruction plus both source/control continuations costs 5,753,351 periods. Eight live-clone treatment arms each complete100k periods (800k); two stopping extensions add 100k each (200k). Actual exposure equals the approved maximum: 6,753,351 periods, 95,546,982 Q writes and equal passive R writes. No treatment arm ends early. The original eight-case 32-branch campaign was not rerun or overwritten.

The source stop-period update is shared, charged once, with each extension. Clone after its actual score/RNG/counter updates but before termination finalization; advance only to the actual successor and suppress stopping for 100k additional periods. No exploration, stopping counter, learning rate, scores, statistics or RNG are reset. Native stopped controllers remain independently retained. Actual worlds use native Clone; JSON checkpoints are audit representations, not restart states.

## Fixed criterion, grids and recurrence

Focal firm 0/state 16 was selected before reconstruction. Selected first-action regret is `max_a[r+gamma*Vstar(next)]` minus the chosen action's same optimal-continuation value. This differs from one-shot deviation followed by the learned policy and from sustained gain Vstar-Vpi. All are evaluated against explicitly committed greedy rival maps, using the unchanged four-candidate Fraction/Decimal80/VI/Howard method and inherited gates. Raw upper <= 1e-8 is a pass; any lower > 1e-8 fails; crossings or uncertified values remain unresolved.

Prefix positions 0/250k/500k/750k/1M describe earlier history. Fork eligibility searches only the eleven-point 1M..1.1M grid, requiring the immediately adjacent certified PASS before its first FAIL; no skipping unresolved points, later convenient crossing or own-greedy-change requirement. Both cases are eligible. The native Clone forks are 1.05M (case 3) and 1M (case 4); the following grid points 1.06M and 1.01M fail.

| Case | Adjacent audited points | Focal selected action | Rival action at state 16 | Optimal first action | Raw first-action regret |
| --- | --- | --- | --- | --- | --- |
| 3 | 1.05M→1.06M | 2→2 | 2→1 | 2→3 | 0→.0196160786223 |
| 4 | 1M→1.01M | 3→2 | 1→1 | 3→0 | 0→.114025526804 |

These are adjacent snapshot comparisons, not assertions that no policy changed and returned inside an interval. Complete own/rival map Hamming distances are 61/43 (case 3) and 79/74 (case 4). A rival can change the optimal benchmark through off-path responses even when its state 16 action is unchanged. Case 3 directly demonstrates a changed frozen-rival benchmark with the same selected action at the two checked endpoints; case 4 changes both selected action and complete rival environment. Saved fork-reference values sharpen that distinction: case 3's action 2 still has zero first-action regret against the old rival map at 1.06M. In case 4, changing 3→2 costs .1570746048 against the old fork map, while the evolved rival map reduces the new action's regret to .1140255268. Retaining old action 3 under the evolved map would have regret .0030207290. These are descriptive contrasts from already saved action-value arrays; assigning a unique causal increment depends on the reference order and is not inferred.

Both cases already fail the focal frozen-greedy criterion at saved 0/250k/500k/750k, then pass at 1M. The eligible transitions are recurrences within the approved window, not first-ever onset or exact failure periods. In particular, early greedy audits abstract from substantial native exploration and ongoing adaptation; they do not establish that actual early stochastic choices were suboptimal. No intervention before 1M or unbounded per-event solver search was performed.

## Four cloned conditions and outcomes

Native retains both learning/behavior sources. Bootstrap held replaces only firm 0's successor maxima with its complete learned fork best vector; own Q/cache/greedy learning continues, and values are learned estimates rather than oracle values. Rival held uses firm 1's complete fork greedy map only for nonexploratory decisions, while its scores/cache continue learning. Both combines those two changes.

Native predecay exploration coin u<=epsilon remains inclusive; a random exploratory action is preserved even when it equals a learned/held greedy action. Keep actual exploration coins, native action!=learned-greedy statistics and imposed-behavior deviations separately. Native stability counts learned changes at the visited row. Current-reference audits evaluate imposed own and rival maps; original-fork rival references are separately retained. Endogenous visits/rewards can diverge after the fork and are treatment consequences rather than matched realized experiences.

The following raw discounted gains concern focal firm 0/state 16 after 100k actual treatment periods, using current imposed rival maps. A zero first-action point has a certified raw upper below 1e-8. Every listed sustained gain is positive above tolerance.

| Case | Condition | First-action regret | First-action judgment | Sustained gain Vstar-Vpi |
| --- | --- | ---: | --- | ---: |
| 3 | native | .03119114598 | fail | .3199091895 |
| 3 | bootstrap held | 0 | pass | .3047517692 |
| 3 | rival held | .00006820515 | fail | .00006820515 |
| 3 | both held | .00316824516 | fail | .3819744618 |
| 4 | native | .03618650687 | fail | .3711436602 |
| 4 | bootstrap held | .01195704692 | fail | 1.0986059343 |
| 4 | rival held | 0 | pass | .1639228261 |
| 4 | both held | 0 | pass | .1639228261 |

Passing the first action does not make the prescribed continuation optimal. Case 3's bootstrap hold passes first-action regret yet leaves sustained gain about.305. In case 4, bootstrap hold reduces first-action regret by.02422946 but increases sustained gain by.72746227. Rival and both holds pass the first action while still leaving sustained gain about.164. None of these eight focal endpoints passes the sustained criterion.

Effects depend on the other hold. Case 3's bootstrap effect on first-action regret is−.03119115 with a live rival and+.00310004 with a held rival, interaction+.03429119. Its sustained effects are−.01515742 and+.38190626. Case 4's first-action bootstrap effect is−.02422946 live and zero held; its sustained effect is+.72746227 live and zero held. These conditional factorial differences do not identify a universally harmful feedback component or unique historical necessity.

Current and fork references can disagree. Case 4 bootstrap-held retains original action 3 and passes the first-action test against the original-fork rival map, but fails against the evolved current rival map. Preserve both environments and the full 0/25k/50k/100k trajectories rather than selecting the favorable reference or endpoint. All-state judgments remain distinct from these selected focal outcomes.

## Native stopping extensions and source chronology

| Case | Native stop→extra 100k, current reference | First-action regret | Sustained focal gain |
| --- | --- | ---: | ---: |
| 3 | 1,941,103→2,041,103 | .01144244291→.00582912600 | .1173583888→.0597859077 |
| 4 | 1,988,907→2,088,907 | .0277906670→.03618650687 | .285032482→.3711436602 |

Case 3 improves and case 4 worsens, with both still failing the focal first-action and sustained tests. Stopping alone is not a sufficient explanation or repair within these 100k extensions. This does not prove that no longer continuation could help; none was authorized or executed. The current rival map changes during the extension, so the original-stop rival references are also retained in the evidence.

The complete journal records every actual Q/R assignment, prewrite cached maximum and source version, native cache rescan identity, exact maximizing parents/masks and global learned-map changes. Parent ties remain explicit; behavior-selected greedy may differ from an exact maximizing cell under the native epsilon tie band. Histories include initialization and branch lineage, actual exploration and imposed/learned divergence.

Some read sources are old: maximum cache-read ages in faithful Both continuations are 801,890/809,461 periods, and in extensions 921,180/880,540. These are maxima over recorded reads rather than all cells' age at the endpoint. They show delayed refresh in actual histories; age alone does not establish a source value is wrong or uniquely causes a decision. Complete chronology and targeted witnesses support conditional explanation without replacing the certified benchmarks or cloned contrasts.

## Verification and scope

The frozen private Rust tree passed workspace tests 3,539 passed, zero failed and 106 existing ignored, seven native helper tests included, four example tests, formatter, warning-free core all-target clippy and release build. Final postrun Python verification has 33 passing tests. During saved decoding, a Rust receipt filename serialized as Unix bytes initially caused a parser error before journal scanning. A reviewed Python-only compatibility fix and real top-level synthetic tests corrected it; the failed log, original executed source and all actual data remain preserved. No Rust/kernel/numerical method, scientific output, learning, exploration or stopping change followed the launch.

Root verification covers all 16 complete journals/all 95,546,982 writes, every scored intermediate snapshot, state continuity, clone/stop/hold boundaries and 94 checkpoint hashes. Journals total 3,137,544,208 uncompressed bytes (within 4GB) and 1,014,327,842 stored bytes. Independent verification additionally decodes 32,000 periods/736,000 writes and separately checks all 333 saved firm/reference audits, 1,332 candidate arrays, 92 contexts and 184 score entries using exact saved-array arithmetic, without a solver or second learning trajectory. All numerical audits are certified; no failed or unresolved audit is suppressed. Caps remain 100 checkpoint contexts/400 firm-reference audits. Two aggregate files are byte-identical, reusing saved certificates with zero additional solver calls.

The study supports recurrence chronology and case-specific sensitivity to evolving estimates and rival behavior. It does not identify a single original cause, prove off-path/sequential optimality of native learners, establish universal necessity/sufficiency, infer population prevalence, or assert ongoing stochastic-policy best responses. All native source and prior cycle-level deterrence results keep their original scope. Broader saved-table repair and supported higher-price endpoint behavior belong to the separate full-panel counterfactual.

## Evidence and lifecycle

Executed source `466d6c2f6c87f201e85b3a130f6ae1a03bb40892` (Rust build 674a59d, unchanged Rust at launch); later saved-verifier source `2322600d00ac4a3907e31bee45154c5586741066`. Executable SHA-256: `53a36856ae84bd841922e72342ede39f71f21aa476b61406d461e908331b8778`. Frozen launch manifest SHA-256: `d58186bafc4273e7b6506936dd2bc774437b627abdabd260a87af09c64ad2f16`. Both aggregates SHA-256: `64b45f5b04874c5fdbfa5db318c6e538a09a1ae76d49fa79daf45d5c5a4a6e2b`.

The complete source/bundle with public prerequisite `989f0689e557e73300f45b46fc55560b64ac0540`, exact executable, frozen launched source, later verifier, journals/checkpoints/analyses/test logs/reviews/receipts are indexed in `survey/out/collusion-memory1-native-history-20261012/`. Bundle restoration matched both the executed and final tracked trees, all 24 retained launch-source files and all 26 final source files before cleanup. Only the owned clean history worktree was removed normally, then its archived branch was retired. This helper was never merged into maintained code. Generated Cargo cache is excluded from the archive and retained under the original `/tmp` evidence root. Other worktrees, the 96 original raw pricing files, prior archives, foraging evidence and `/tmp` evidence remain untouched. Documentation stays local; no push, publication, author contact or additional campaign is part of this study.
