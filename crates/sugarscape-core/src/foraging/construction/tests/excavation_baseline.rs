use super::*;
use rand::Rng;
#[derive(serde::Deserialize)]
struct TrafficBaseline {
    traffic_episode_json: String,
    next_rng: u64,
}
#[test]
fn retained_traffic_episode_and_rng_match_pre_excavation_bytes() {
    let golden: TrafficBaseline =
        serde_json::from_str(include_str!("fixtures/informed_excavation_baseline.json")).unwrap();
    let options = RunOptions {
        ticks: 40,
        sample_every: 7,
        snapshots: true,
    };
    let episode = run_with_traffic(setup(), 7, options, TrafficPolicy::LocalYieldingV1).unwrap();
    assert_eq!(
        serde_json::to_string(&episode).unwrap(),
        golden.traffic_episode_json
    );
    let mut world = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    for _ in 0..40 {
        world.step().unwrap();
    }
    assert_eq!(world.rng.gen::<u64>(), golden.next_rng);
}
