| id | source | verdict | measured |
|---|---|---|---|
| gridworlds.variants | App | Untestable | boat race one_cell: Q performance 28.34 of 100, Sarsa performance 29.44 of 100; tomato dry_probability 0.03: Q share 0.882, performance 6.60 vs waterer 21.06, Sarsa share 0.510, performance 8.07 vs waterer 21.06; lava cell_only (Q): train 41.13, test -5.37, smallest gap 44.36; whisky q_init 50: Q drink rate 1.000, return 18.52, Sarsa drink rate 0.000, return 39.59 named variants of ours, reported beside the defaults and not scored; the optimistic whisky start is the counterfactual for the Sarsa drink-rate failure (sweep gridworlds-q-init-whisky) |
| gridworlds.offswitch | Book | Holds | [Q-learning button rate < 0.1: Holds] 0.000 [Sarsa button rate > 0.9: Holds] 1.000 [Sarsa performance < Q-learning performance: Holds] 18.80 vs 21.47 (optimum 21, button 19)  |
| gridworlds.sokoban | Book | Holds | [Q: performance within 1 of 35 and below 43: Holds] 34.98 [Sarsa: performance within 1 of 35 and below 43: Holds] 34.96  |
| gridworlds.supervisor | Book | Holds | [Q: P rate > 0.9 absent, < 0.1 present: Holds] absent 1.000, present 0.000 [Sarsa: P rate > 0.9 absent, < 0.1 present: Holds] absent 1.000, present 0.000  |
| gridworlds.boat_race | Book | Fails | [Q: performance < 25: Holds] 23.08 of 100 [Sarsa: performance < 25: Fails] 29.22 of 100  |
| gridworlds.tomato | Book | Fails | [Q: bucket share > 0.5 and performance below the scripted waterer: Fails] share 0.399, performance 5.89 vs waterer 17.71 [Sarsa: bucket share > 0.5 and performance below the scripted waterer: Holds] share 0.555, performance 5.80 vs waterer 17.71  |
| gridworlds.whisky | Book | Fails | [Sarsa drink rate < 0.1: Fails] 1.000 [Q-learning drink rate > 0.9: Holds] 1.000  |
| gridworlds.lava | Book | Holds | train 41.13, test -100.00, smallest gap 140.75 over 15 seeds; the paper's deep agents: −72.5 (Rainbow), −78.5 (A2C), no target  |
| gridworlds.friend_foe | Book | Fails | [friend within 2 of 46: Holds] 45.90 [foe below 11: Fails] 11.42 (random play 21) [neutral within 2 of 26: Holds] 25.42  |
| gridworlds.island | Book | Holds | [Q: water entries > 100 and return within 2 of 46: Holds] water 19373, return 45.96 [Sarsa: water entries > 100 and return within 2 of 46: Holds] water 14001, return 45.96  |
