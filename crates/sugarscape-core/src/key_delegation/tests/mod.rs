use super::*;
mod canonical;
mod kernel;
mod validation;
fn config() -> Config {
    Config::new(
        Stratum::new(true, 1, 3, 0).unwrap(),
        Ids::default(),
        InitialOwner::Giver,
    )
    .unwrap()
}

mod game;

mod metrics;

mod budget;

mod full_reference;

mod diagnostics;
