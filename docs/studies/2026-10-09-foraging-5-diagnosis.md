# F5 post hoc diagnosis — 2026-10-09

This investigation follows the [registered full run](2026-10-09-foraging-5-full-run.md). It uses the unchanged 600 saved records and code from runtime `a129f06`; it adds no World runs, seed extensions, tuning, exclusions or altered scientific endpoints. Its descriptive diagnostics were selected after seeing outcomes and are not additional registered contrasts.

## AlreadyOpen: structural distance and traffic throughput differ

AlreadyOpen preserves the original Detour/Twisting route and adds the straight corridor; it is not a replacement of the original terrain. See [scenario construction](../../survey/src/claims/foraging_shortcuts/scenario.rs#L87). Adding that corridor changes exploration choices, meetings and congestion as well as initial shortest distance.

The navigation rule computes shortest routes through each worker's privately KnownOpen cells, then allows only immediate distance-decreasing steps with fewer than two occupants. If those steps are full, it returns Blocked and the controller spends an opportunity on a congestion wait. It retains the target and supplies no detour, yielding, swap or displacement. Sources: [route filtering](../../crates/sugarscape-core/src/foraging/construction/navigation.rs#L289), [wait dispatch](../../crates/sugarscape-core/src/foraging/construction/decision.rs#L79). This matches the approved F4 rule and the existing `full_nearest_steps_wait_instead_of_longer_escape` test. No discrepancy with that contract was found.

| Route condition | Mean delivered | Mean empty congestion waits / 4096 opportunities | Episodes with no pickup / 40 |
| --- | ---: | ---: | ---: |
| `route.straight.protected` | 6.950 | 2089.475 | 13 |
| `route.straight.already_open` | 6.950 | 2089.475 | 13 |
| `route.detour.protected` | 12.000 | 1040.475 | 9 |
| `route.detour.already_open` | 4.150 | 2617.300 | 21 |
| `route.twisting.protected` | 11.600 | 1098.775 | 9 |
| `route.twisting.already_open` | 4.150 | 2617.300 | 21 |

A concrete example is `route.detour.already_open/10001`: zero delivery and 3799 empty congestion waits, versus all 16 delivered in its matched Protected episode. At completed tick 128, cells (3,19) and (3,20) each contain two workers. The empty returner at (3,19) needs the adjacent nest cell (3,20); both workers at (3,20) retain frontier (3,19). Their immediate desired steps are occupied by opposing workers. The free cell (3,18) and spare capacity at (4,20) do not supply an allowed decreasing-distance step for those opposing destinations.

Workers 3, 4 and 7 each record exactly 128 additional congestion waits and zero moves between ticks 128 and 256. Their positions are unchanged. Disjoint action counters establish continuous immobility for these workers over that interval, beyond equality of coarse positions. This does not assert every worker is blocked forever: phase changes and other movements can still occur.

The strong supported mechanism is sustained traffic blocking at narrow passage mouths, consistent with the controller's shortest-step-only rule. The average wait pattern and example do not identify a pure causal effect of path length: geometry changes exploration and random-draw histories too. No counterfactual yielding policy was executed. Shorter physical distance cannot by itself predict throughput under these rules.

Detour and Twisting AlreadyOpen have identical delivery and aggregate action counters at every saved checkpoint for all 40 matched seeds. Complete worker views match in 36 pairs; the other four differ in worker diagnostics/knowledge counts. Equal sampled work is not proof of full private-map or RNG trajectory identity.

## Paid excavation: permission is not a shortcut-building policy

Searching attempts food pickup, then pursues a private open frontier, and only considers a dig face if no reachable open frontier remains. A geometrically reachable frontier whose next step is occupied still takes priority and produces a wait; it does not trigger excavation as a congestion escape. Uninformed departure does not dig, and carried food returns directly to the nest. Sources: [searching priority](../../crates/sugarscape-core/src/foraging/construction/decision.rs#L247) and the [approved policy](../superpowers/specs/2026-10-06-foraging-4-construction-design.md).

| Paid route geometry | Episodes with digs / 40 | Food already delivered at first dig | Zero-delivery episodes with digs |
| --- | ---: | --- | ---: |
| straight | 17 | 12: 2 episodes, 13: 1 episodes, 15: 9 episodes, 16: 5 episodes | 0 |
| detour | 30 | 12: 1 episodes, 13: 8 episodes, 14: 9 episodes, 15: 12 episodes | 0 |
| twisting | 28 | 12: 2 episodes, 13: 4 episodes, 14: 9 episodes, 15: 5 episodes, 16: 8 episodes | 0 |

Across all 120 Paid route episodes, only 75 dig at all, and every first dig occurs after at least 12 of the 16 tokens are delivered. All 42 zero-delivery Paid route episodes have zero digs. Paid and Protected have identical pickup counts, first-pickup clocks and first-delivery clocks in all 120 matched pairs. All-food completion clocks match in 119 pairs; the remaining Straight pair differs while retaining the same final delivery. Therefore the zero primary endpoint is not evidence of full behavioral identity or literally no timing effect.

The sole positive sampled route gain is `route.twisting.paid/10038`: all food is delivered at processing tick 419; its first excavation is tick 425; final minimum distance shortens by six cells. The checkpoint window remains (384,512], but the first-excavation milestone proves this episode's shortening could only start after food delivery was complete. It cannot improve that episode's already-completed delivery endpoint.

This explains the observed separation between excavation capability and realized food benefit: open exploration generally uses the existing path first, and face selection is not an explicit assessment of shortcut benefit. Stuck frontier traffic also prevents reaching the digging fallback. The investigation identifies an intentional policy limitation, not an omitted excavation action or an archive/reporting error.

## What remains to be designed

Traffic resolution is the first controller question to address: the saved evidence shows blocking before discovery in many AlreadyOpen episodes. A behavior change needs a separate design, regression cases and subsequent registration; this run must remain unchanged. Then examine when construction should compete with open exploration and how a worker can value a possible shortcut using only its private information. Dedicated roles and relay logistics remain separate future choices; neither is necessary to establish the present diagnosis.

## Evidence and limitations

`/tmp/sugarscape-f5-diagnostics-20261009/diagnose.py` is the reproducible standard-library saved-only check; `diagnosis.json` retains the full condition summaries, paired clocks, excavation contexts and example. It verifies the pinned index/analysis identities and all 600 raw SHA/size/key/provenance bindings, then rechecks all hashes after analysis. Exit zero; archive unchanged. No product/controller source was modified and no stochastic run was performed.

The raw archive is untracked in the main checkout, not ignored by its current Git rules. This corrects the earlier full-run document's storage wording. A complete hash-verified local copy of raw evidence and both reports is preserved at `/tmp/sugarscape-f5-execution-20261009/official-data/`; the originals remain in `survey/out/foraging-shortcuts/`. Neither local copy is a remote backup.

Full private maps, individual action traces and RNG states are not in these snapshots. The source trace and action-counter proof support the mechanisms above without reconstructing those missing histories. A causal comparison of an altered traffic or excavation policy remains future work. Biological calibration and empirical-source reproduction remain outside these computational observations.
