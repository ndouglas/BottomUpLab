use super::*;
use rand::Rng;
#[derive(serde::Deserialize)]
struct BaselineRecord {
    episode_json: String,
    next_rng: u64,
}
#[test]
fn ordinary_episode_and_next_rng_match_pre_traffic_bytes() {
    let baseline: BaselineRecord =
        serde_json::from_str(include_str!("fixtures/local_yielding_baseline.json")).unwrap();
    let options = RunOptions {
        ticks: 40,
        sample_every: 7,
        snapshots: true,
    };
    assert_eq!(
        serde_json::to_string(&run(setup(), 7, options).unwrap()).unwrap(),
        baseline.episode_json
    );
    let mut world = World::new(setup(), 7).unwrap();
    for _ in 0..40 {
        world.step().unwrap();
    }
    assert_eq!(world.rng.gen::<u64>(), baseline.next_rng);
}
