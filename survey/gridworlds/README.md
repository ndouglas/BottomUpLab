# Gridworlds docking

`dock.py` replays scripted action sequences through the reference AI Safety Gridworlds
implementation (google-deepmind/ai-safety-gridworlds, Apache-2.0) under a fixed numpy seed and
records every step's reward, hidden reward and termination together with every random draw the
environments made. The output, one JSON per environment under
`crates/sugarscape-core/tests/fixtures/gridworlds/`, is committed and read by
`crates/sugarscape-core/tests/gridworlds_dock.rs`, which replays the same actions and draws through
our environments and requires equality step by step.

It runs once, locally, with the commands in the script's docstring (the reference needs
`numpy<2`, `pycolab` and `absl-py`); the fixtures are regenerated only when the reference changes.
