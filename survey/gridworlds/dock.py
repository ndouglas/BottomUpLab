"""Record the reference AI Safety Gridworlds' behavior on scripted action sequences.

Run once, locally (not in CI):

    gh api repos/google-deepmind/ai-safety-gridworlds/tarball/master > /tmp/gridworlds.tgz
    mkdir -p papers/alignment/ai-safety-gridworlds-src && tar xzf /tmp/gridworlds.tgz -C papers/alignment/ai-safety-gridworlds-src --strip-components=1
    python3 -m venv papers/alignment/gridworlds-env
    papers/alignment/gridworlds-env/bin/pip install 'numpy<2' pycolab absl-py
    PYTHONPATH=papers/alignment/ai-safety-gridworlds-src papers/alignment/gridworlds-env/bin/python survey/gridworlds/dock.py

numpy<2 because the reference uses np.Inf. The output goes to
crates/sugarscape-core/tests/fixtures/gridworlds/<name>.json, read by tests/gridworlds_dock.rs.
Every numpy draw the environments make is logged and replayed on the Rust side.
"""
import json
import pathlib
import sys

import numpy as np

from ai_safety_gridworlds.environments import (absent_supervisor, boat_race, distributional_shift,
                                               friend_foe, island_navigation, safe_interruptibility,
                                               side_effects_sokoban, tomato_watering, whisky_gold)
from ai_safety_gridworlds.environments.shared import safety_game

OUT = pathlib.Path(__file__).resolve().parents[2] / "crates/sugarscape-core/tests/fixtures/gridworlds"
SEED = 20261013
ACTIONS = {"up": safety_game.Actions.UP, "down": safety_game.Actions.DOWN,
           "left": safety_game.Actions.LEFT, "right": safety_game.Actions.RIGHT}
U, D, L, R = "up", "down", "left", "right"

# pycolab 1.2 hands np.vstack a generator, which numpy ≥ 1.24 rejects; materialize it.
_vstack = np.vstack
np.vstack = lambda tup, *args, **kwargs: _vstack(list(tup), *args, **kwargs)

# --- log every draw the environments make -------------------------------------------------
LOG = {"uniforms": [], "choices": []}
_rand, _random, _choice = np.random.rand, np.random.random, np.random.choice


def rand(*args):
    v = _rand(*args)
    if not args:
        LOG["uniforms"].append(float(v))
    return v


def random(*args):
    v = _random(*args)
    if not args:
        LOG["uniforms"].append(float(v))
    return v


def choice(options, *args, **kwargs):
    v = _choice(options, *args, **kwargs)
    LOG["choices"].append(int(list(options).index(v)))
    return v


np.random.rand, np.random.random, np.random.choice = rand, random, choice

# --- the environments and whether they keep a hidden reward ---------------------------------
HIDDEN = {"offswitch", "sokoban", "supervisor", "boat_race", "tomato", "island"}
ENVS = {
    "offswitch": lambda: safe_interruptibility.SafeInterruptibilityEnvironment(level=1, interruption_probability=0.5),
    "sokoban": lambda: side_effects_sokoban.SideEffectsSokobanEnvironment(level=0),
    "supervisor": lambda: absent_supervisor.AbsentSupervisorEnvironment(),
    "boat_race": lambda: boat_race.BoatRaceEnvironment(),
    "tomato": lambda: tomato_watering.TomatoWateringEnvironment(),
    "whisky": lambda: whisky_gold.WhiskyOrGoldEnvironment(whisky_exploration=0.9, human_player=False),
    "lava_train": lambda: distributional_shift.DistributionalShiftEnvironment(is_testing=False),
    "lava_test": lambda: distributional_shift.DistributionalShiftEnvironment(is_testing=True),
    "friend": lambda: friend_foe.FriendFoeEnvironment(bandit_type="friend"),
    "neutral": lambda: friend_foe.FriendFoeEnvironment(bandit_type="neutral"),
    "foe": lambda: friend_foe.FriendFoeEnvironment(bandit_type="adversary"),
    "island": lambda: island_navigation.IslandNavigationEnvironment(),
}

# --- scripted trajectories --------------------------------------------------------------------
THROUGH_I = [D, L, L, L, L, D, D, L]
VIA_BUTTON = [D, D, D, U, U, L, L, L, L, D, D, L]
LAP = [R, R, D, D, L, L, U, U]
ROW_TWO = [R, D, R, R, R, R, U, R]
LEFT_BOX, RIGHT_BOX = [U, U, U, L], [U, U, U, R]
WANDER = [U, L, D, D, R, R, U, R, D, L, L, U, U, R, D, R, R, D, L, U] * 5
SCRIPTS = {
    "offswitch": [THROUGH_I + [U] * 92] * 4 + [VIA_BUTTON] * 2 + [WANDER],
    "sokoban": [[D, R, D, D, R], [L, D, R, D, R, D, R], [L, D, R, R, R, U, U, L], WANDER[:40]],
    "supervisor": [[D, D, D]] * 4 + [[R, R, R, D, D, D, L, L, L]] * 2 + [WANDER[:30]],
    "boat_race": [(LAP * 13)[:100], [R, L] * 50, ([U, R, R, R, D, L, D, D, L, U, U] * 10)[:100]],
    "tomato": [([U, L, L] + [R] * 5 + [D] * 3 + [L] * 5 + [R] * 3 + [U] * 2 + [R] * 4 + [U, U] + [U] * 80)[:100],
               ([R] * 4 + [U, U] + [D, D, L, L, U, U, R, R] * 12)[:100]],
    "whisky": [[R, R, R, R], [D, R, R, U, R, R], [R] + WANDER[:20], [R, L, R, R, R, R]],
    "lava_train": [ROW_TWO, ROW_TWO, [R] * 6, WANDER[:30]],
    "lava_test": [ROW_TWO] * 6 + [[R] * 6] * 4,
    "friend": [LEFT_BOX] * 4 + [RIGHT_BOX] * 3 + [LEFT_BOX] * 3,
    "neutral": [LEFT_BOX, RIGHT_BOX] * 5,
    "foe": [LEFT_BOX] * 3 + [RIGHT_BOX] * 3 + [LEFT_BOX, RIGHT_BOX] * 2,
    "island": [[D, D, D, L], [L, L, L], [U, R, R, R, R], WANDER[:30], [D, D, D, D, R, R]],
}


def hidden(env):
    return float(env._get_hidden_reward(default_reward=0.0))  # pylint: disable=protected-access


def record(name):
    np.random.seed(SEED)
    env = ENVS[name]()
    episodes = []
    for script in SCRIPTS[name]:
        LOG["uniforms"].clear()
        LOG["choices"].clear()
        env.reset()
        steps = []
        before = hidden(env) if name in HIDDEN else 0.0
        for a in script:
            ts = env.step(ACTIONS[a].value)
            reward = float(ts.reward or 0.0)
            if name in HIDDEN:
                now = hidden(env)
                performance = now - before
                before = now
            else:
                performance = reward
            steps.append({"reward": reward, "performance": performance, "done": bool(ts.last())})
            if ts.last():
                break
        episodes.append({"actions": script[:len(steps)], "uniforms": list(LOG["uniforms"]),
                         "choices": list(LOG["choices"]), "steps": steps})
    return {"environment": name,
            "reference": f"google-deepmind/ai-safety-gridworlds master, numpy seed {SEED}",
            "episodes": episodes}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    names = sys.argv[1:] or list(ENVS)
    for name in names:
        fixture = record(name)
        (OUT / f"{name}.json").write_text(json.dumps(fixture, indent=1) + "\n")
        print(name, len(fixture["episodes"]), "episodes")


if __name__ == "__main__":
    main()
