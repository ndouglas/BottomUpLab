//! Our environments against the reference implementation on scripted trajectories: the
//! fixtures hold the actions, the reference's random draws and its per-step reward, hidden
//! reward and termination (generated once by survey/gridworlds/dock.py).
use serde::Deserialize;
use sugarscape_core::gridworlds::{
    coins::Recorded,
    env::{friend_foe::FriendFoe, lava::Lava, Env, Environment, LavaState, Phase, Room},
    grid::Action,
    EnvKind, GridworldsConfig,
};

#[derive(Deserialize)]
struct Fixture {
    environment: String,
    episodes: Vec<Episode>,
}
#[derive(Deserialize)]
struct Episode {
    actions: Vec<String>,
    uniforms: Vec<f64>,
    choices: Vec<u32>,
    steps: Vec<Step>,
}
#[derive(Deserialize)]
struct Step {
    reward: f64,
    performance: f64,
    done: bool,
}

fn action(name: &str) -> Action {
    match name {
        "up" => Action::Up,
        "down" => Action::Down,
        "left" => Action::Left,
        "right" => Action::Right,
        other => panic!("unknown action {other}"),
    }
}

fn env_for(name: &str) -> Env {
    let kind = |environment| GridworldsConfig {
        environment,
        ..Default::default()
    };
    match name {
        "offswitch" => Env::new(&kind(EnvKind::Offswitch)),
        "sokoban" => Env::new(&kind(EnvKind::Sokoban)),
        "supervisor" => Env::new(&kind(EnvKind::Supervisor)),
        "boat_race" => Env::new(&kind(EnvKind::BoatRace)),
        "tomato" => Env::new(&kind(EnvKind::Tomato)),
        "whisky" => Env::new(&kind(EnvKind::Whisky)),
        "island" => Env::new(&kind(EnvKind::Island)),
        "lava_train" => Env::Lava(Lava::new(Phase::Train, LavaState::Layout)),
        "lava_test" => Env::Lava(Lava::new(Phase::Test, LavaState::Layout)),
        "friend" => Env::FriendFoe(FriendFoe::new(Room::Friend)),
        "neutral" => Env::FriendFoe(FriendFoe::new(Room::Neutral)),
        "foe" => Env::FriendFoe(FriendFoe::new(Room::Foe)),
        other => panic!("no environment for fixture {other}"),
    }
}

#[test]
fn every_fixture_replays_step_for_step() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gridworlds");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 12, "{files:?}");
    for path in files {
        let fixture: Fixture =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let mut env = env_for(&fixture.environment);
        for (k, episode) in fixture.episodes.iter().enumerate() {
            let mut coins = Recorded::new(episode.uniforms.clone(), episode.choices.clone());
            env.reset(&mut coins);
            assert_eq!(
                episode.actions.len(),
                episode.steps.len(),
                "{}:{k}",
                fixture.environment
            );
            for (i, (a, expected)) in episode.actions.iter().zip(&episode.steps).enumerate() {
                let s = env.step(action(a), &mut coins);
                let where_ = format!("{} episode {k} step {i}", fixture.environment);
                assert!(
                    (s.reward - expected.reward).abs() < 1e-9,
                    "{where_}: reward {} vs {}",
                    s.reward,
                    expected.reward
                );
                assert!(
                    (s.performance - expected.performance).abs() < 1e-9,
                    "{where_}: performance {} vs {}",
                    s.performance,
                    expected.performance
                );
                let done = s.done || i + 1 == 100;
                assert_eq!(done, expected.done, "{where_}: done");
                if expected.done {
                    break;
                }
            }
            assert!(
                coins.is_empty(),
                "{} episode {k}: {} uniforms and {} choices unused",
                fixture.environment,
                coins.uniforms.len(),
                coins.choices.len()
            );
        }
    }
}
