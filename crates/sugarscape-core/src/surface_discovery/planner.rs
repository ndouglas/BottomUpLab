use super::budget::Budget;
use super::*;
use crate::shared_surface::{Outcome, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceValue {
    pub choice: Choice,
    pub value: ScoreFraction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub choice: Choice,
    pub alternatives: Vec<ChoiceValue>,
    pub continuation_policy: PolicyKind,
}

const ZERO: ScoreFraction = ScoreFraction {
    numerator: 0,
    denominator: 1,
};
const RAW_ACTIONS: [Action; 8] = [
    Action::Wait,
    Action::Read,
    Action::Write(Symbol::Probe0),
    Action::Write(Symbol::Ack0),
    Action::Write(Symbol::Probe1),
    Action::Write(Symbol::Ack1),
    Action::Write(Symbol::Data0),
    Action::Write(Symbol::Data1),
];

#[derive(Clone, Copy)]
struct Node {
    alternatives: [ChoiceValue; 8],
    len: u8,
    selected: u8,
}
impl Node {
    fn new() -> Self {
        Self {
            alternatives: [ChoiceValue {
                choice: Choice::Stop,
                value: ZERO,
            }; 8],
            len: 0,
            selected: 0,
        }
    }
    fn push(&mut self, choice: Choice, value: ScoreFraction) -> Result<(), Error> {
        let i = usize::from(self.len);
        if i >= self.alternatives.len() {
            return Err(Error::InvalidHistory("choice set exceeds raw menu".into()));
        }
        self.alternatives[i] = ChoiceValue { choice, value };
        if i != 0 && value.checked_cmp(self.value())? == std::cmp::Ordering::Greater {
            self.selected = self.len;
        }
        self.len += 1;
        Ok(())
    }
    fn value(self) -> ScoreFraction {
        self.alternatives[usize::from(self.selected)].value
    }
    fn decision(self, policy: PolicyKind) -> Decision {
        Decision {
            choice: self.alternatives[usize::from(self.selected)].choice,
            alternatives: self.alternatives[..usize::from(self.len)].to_vec(),
            continuation_policy: policy,
        }
    }
}

/// The reviewed calibration bisimulation retains BOTH visible fields and both
/// helper recipe predicates for every original model. The array index is the
/// original model identity; None means zero original mass, never renumbering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CalAtom {
    fields: [Symbol; 2],
    b_probe0: bool,
    a_probe1: bool,
}
type CalStates = [Option<CalAtom>; 4];
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CalClock {
    Boundary(u8),
    Slot { round: u8, slot: u8 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CalKey {
    states: CalStates,
    clock: CalClock,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LiveKey {
    weights: [u64; 4],
    remaining: u8,
}

pub struct CompiledPolicy {
    protocol: Protocol,
    prior: OwnPrior,
    calibration: BTreeMap<CalKey, Node>,
    live: BTreeMap<LiveKey, Node>,
    initial: ScoreFraction,
    pub(crate) budget: Budget,
}
impl CompiledPolicy {
    pub fn build(protocol: &Protocol, prior: OwnPrior, limits: &Limits) -> Result<Self, Error> {
        protocol.validate()?;
        if matches!(prior, OwnPrior::PointMass(_)) != (protocol.policy == PolicyKind::KnownRaw) {
            return Err(Error::InvalidProtocol(
                "investigator prior differs from declared policy".into(),
            ));
        }
        let mut budget = Budget::new(limits)?;
        // Reserve BEFORE allocating protocol clones, cache nodes, branches or
        // inference candidates. Maximum 3 calibration rounds + 4 live trials;
        // recursion depth < 32; each local history < 64 entries. String storage
        // is charged separately because valid identifiers have no size ceiling.
        let id_bytes = protocol
            .ids
            .surface
            .len()
            .checked_add(protocol.ids.agents[0].len())
            .and_then(|n| n.checked_add(protocol.ids.agents[1].len()))
            .ok_or(Error::ArithmeticOverflow)?;
        let workspace = u64::try_from(id_bytes)
            .map_err(|_| Error::ArithmeticOverflow)?
            .checked_mul(256)
            .and_then(|n| n.checked_add(1_048_576))
            .ok_or(Error::ArithmeticOverflow)?;
        budget.allocate(workspace)?;
        budget.stats.candidate_worlds = if prior == OwnPrior::Uniform {
            1024
        } else {
            256
        };
        let mut compiled = Self {
            protocol: protocol.clone(),
            prior,
            calibration: BTreeMap::new(),
            live: BTreeMap::new(),
            initial: ZERO,
            budget,
        };
        compiled.initial = compiled
            .solve_cal(CalKey {
                states: initial_cal(prior),
                clock: CalClock::Boundary(0),
            })?
            .value();
        Ok(compiled)
    }
    pub fn initial_value(&self) -> ScoreFraction {
        self.initial
    }
    pub fn stats(&self) -> &SearchStats {
        &self.budget.stats
    }
    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }
    pub fn own_prior(&self) -> OwnPrior {
        self.prior
    }
    pub fn decide(&self, view: &View) -> Result<Decision, Error> {
        let support = self.candidate_support(&view.prefix)?;
        let belief = super::belief::belief_from_support(&view.prefix, &support)?;
        let canonical = View::from_prefix(view.prefix.clone(), belief)?;
        if canonical != *view {
            return Err(Error::InvalidHistory(
                "view belief, credits or private bit differs from own prefix".into(),
            ));
        }
        let Checkpoint::Boundary(boundary) = view.prefix.checkpoint else {
            return Err(Error::InvalidHistory(
                "decision requires an unselected boundary".into(),
            ));
        };
        self.protocol.validate_boundary(boundary)?;
        if view
            .prefix
            .entries
            .iter()
            .any(|e| matches!(e, Entry::OwnChoice {boundary:b,..} if *b==boundary))
        {
            return Err(Error::InvalidHistory(
                "boundary already has a queued own choice".into(),
            ));
        }
        let node = match boundary {
            Boundary::Trial { trial } => self.live.get(&LiveKey {
                weights: normalized(super::belief::model_weights(&support))?,
                remaining: 4 - trial,
            }),
            _ => self.calibration.get(&self.cal_key(&view.prefix, boundary)?),
        }
        .ok_or_else(|| {
            Error::InvalidHistory("supported boundary absent from complete compiled policy".into())
        })?;
        Ok(node.decision(self.protocol.policy))
    }
    fn cal_key(&self, prefix: &Prefix, boundary: Boundary) -> Result<CalKey, Error> {
        let wanted = match boundary {
            Boundary::Probe { completed } => CalClock::Boundary(completed),
            Boundary::Calibration(p) => CalClock::Slot {
                round: p.round,
                slot: p.slot,
            },
            Boundary::Trial { .. } => {
                return Err(Error::InvalidHistory(
                    "live boundary has no calibration key".into(),
                ))
            }
        };
        let mut key = CalKey {
            states: initial_cal(self.prior),
            clock: CalClock::Boundary(0),
        };
        while key.clock != wanted {
            match key.clock {
                CalClock::Boundary(completed) => {
                    if !prefix.entries.contains(&Entry::OwnChoice {
                        boundary: Boundary::Probe { completed },
                        choice: Choice::Continue,
                    }) {
                        return Err(Error::InvalidHistory(
                            "calibration continuation lacks own purchase".into(),
                        ));
                    }
                    key.clock = CalClock::Slot {
                        round: completed + 1,
                        slot: 1,
                    };
                }
                CalClock::Slot { round, slot } => {
                    let own = slot_role(slot) == self.protocol.investigator;
                    let observed = prefix.entries.iter().find_map(|e| match e {
                        Entry::Physical(LocalEntry::Action(event))
                            if event.position == position(round, slot) =>
                        {
                            Some(event)
                        }
                        _ => None,
                    });
                    if own && observed.is_none() {
                        return Err(Error::InvalidHistory(
                            "missing own calibration event".into(),
                        ));
                    }
                    for (model, atom) in key.states.iter_mut().enumerate() {
                        if let Some(a) = *atom {
                            let action =
                                observed.map_or_else(|| recipe(a, round, slot), |e| e.action);
                            let (outcome, moved) =
                                cal_action(&self.protocol, model, a, round, slot, action)?;
                            *atom = if own && observed.is_some_and(|e| e.outcome != outcome) {
                                None
                            } else {
                                Some(moved)
                            };
                        }
                    }
                    key.clock = next_cal(round, slot);
                }
            }
        }
        Ok(key)
    }

    fn solve_live(&mut self, weights: [u64; 4], remaining: u8) -> Result<Node, Error> {
        if remaining == 0 {
            return Ok(Node::new());
        }
        let key = LiveKey {
            weights: normalized(weights)?,
            remaining,
        };
        if let Some(node) = self.live.get(&key) {
            return Ok(*node);
        }
        self.budget.state(cache_bytes::<LiveKey>())?;
        let mut node = Node::new();
        self.budget.branch()?;
        let inspect = ScoreFraction::new(3, 1)?
            .checked_add(self.solve_live(key.weights, remaining - 1)?.value())?;
        node.push(Choice::Inspect, inspect)?;
        self.budget.branch()?;
        // At A's unselected boundary the field is blank. At B's boundary
        // hidden A has ALREADY written target under SP. Its exact correlation
        // is integrated here. PP at A reads its own bit, fixed to zero by the
        // reviewed simultaneous Data0/Data1 symmetry. No partial-live cache uses
        // this reduction. Every target/model branch retains its joint mass.
        let mut branches = [([0u64; 4], 0u64); 3]; // Blank, Data0, Data1
        for (model, mass) in key.weights.iter().copied().enumerate() {
            for target in 0..=1usize {
                let observed = if model == 0 {
                    1 + target
                } else if model == 2 && self.protocol.investigator == Role::A {
                    1
                } else {
                    0
                };
                branches[observed].0[model] = branches[observed].0[model]
                    .checked_add(mass)
                    .ok_or(Error::ArithmeticOverflow)?;
                if target == 1 {
                    branches[observed].1 = branches[observed]
                        .1
                        .checked_add(mass)
                        .ok_or(Error::ArithmeticOverflow)?;
                }
            }
        }
        let total = sum(key.weights)?
            .checked_mul(2)
            .ok_or(Error::ArithmeticOverflow)?;
        let mut attempt = ScoreFraction::new(-6, 1)?;
        for (weights, ones) in branches {
            let mass = sum(weights)?;
            if mass == 0 {
                continue;
            }
            let reward = ScoreFraction::new(
                i64::try_from(
                    ones.max(mass - ones)
                        .checked_mul(12)
                        .ok_or(Error::ArithmeticOverflow)?,
                )
                .map_err(|_| Error::ArithmeticOverflow)?,
                mass,
            )?;
            let next = self.solve_live(weights, remaining - 1)?.value();
            attempt = attempt.checked_add(weighted(reward.checked_add(next)?, mass, total)?)?;
        }
        node.push(Choice::Attempt, attempt)?;
        if self.protocol.policy == PolicyKind::InspectOnly {
            node.selected = 0;
        }
        self.live.insert(key, node);
        Ok(node)
    }
    fn solve_cal(&mut self, key: CalKey) -> Result<Node, Error> {
        if let Some(node) = self.calibration.get(&key) {
            return Ok(*node);
        }
        self.budget.state(cache_bytes::<CalKey>())?;
        let node = match key.clock {
            CalClock::Boundary(completed) => {
                let mut node = Node::new();
                self.budget.branch()?;
                let stop = self
                    .solve_live(key.states.map(|a| u64::from(a.is_some())), 4)?
                    .value();
                node.push(Choice::Stop, stop)?;
                if completed < 3 {
                    self.budget.branch()?;
                    let next = self.solve_cal(CalKey {
                        states: key.states,
                        clock: CalClock::Slot {
                            round: completed + 1,
                            slot: 1,
                        },
                    })?;
                    node.push(Choice::Continue, next.value())?;
                }
                if matches!(
                    self.protocol.policy,
                    PolicyKind::NoProbe | PolicyKind::InspectOnly
                ) {
                    node.selected = 0;
                }
                node
            }
            CalClock::Slot { round, slot } => {
                let owner = slot_role(slot);
                if owner != self.protocol.investigator {
                    let mut moved = key.states;
                    for (model, atom) in moved.iter_mut().enumerate() {
                        if let Some(a) = atom {
                            *a = cal_action(
                                &self.protocol,
                                model,
                                *a,
                                round,
                                slot,
                                recipe(*a, round, slot),
                            )?
                            .1;
                        }
                    }
                    self.solve_cal(CalKey {
                        states: moved,
                        clock: next_cal(round, slot),
                    })?
                } else {
                    let mut node = Node::new();
                    let supplied = key
                        .states
                        .iter()
                        .flatten()
                        .next()
                        .map(|a| recipe(*a, round, slot))
                        .ok_or_else(|| Error::InvalidHistory("empty calibration support".into()))?;
                    let menu: &[Action] = if self.protocol.policy.raw() {
                        &RAW_ACTIONS
                    } else {
                        std::slice::from_ref(&supplied)
                    };
                    let total = key.states.iter().flatten().count() as u64;
                    for action in menu {
                        self.budget.branch()?;
                        let mut groups: BTreeMap<Outcome, CalStates> = BTreeMap::new();
                        for (model, atom) in key.states.iter().copied().enumerate() {
                            if let Some(atom) = atom {
                                if !self.protocol.policy.raw()
                                    && recipe(atom, round, slot) != supplied
                                {
                                    return Err(Error::InvalidHistory(
                                        "supplied action differs within local support".into(),
                                    ));
                                }
                                let (outcome, moved) =
                                    cal_action(&self.protocol, model, atom, round, slot, *action)?;
                                groups.entry(outcome).or_insert([None; 4])[model] = Some(moved);
                            }
                        }
                        let mut value = ScoreFraction::new(-1, 1)?;
                        for states in groups.into_values() {
                            let mass = states.iter().flatten().count() as u64;
                            let next = self
                                .solve_cal(CalKey {
                                    states,
                                    clock: next_cal(round, slot),
                                })?
                                .value();
                            value = value.checked_add(weighted(next, mass, total)?)?;
                        }
                        node.push(Choice::Calibration(*action), value)?;
                    }
                    node
                }
            }
        };
        self.calibration.insert(key, node);
        Ok(node)
    }
}
fn cache_bytes<K>() -> u64 {
    // BTreeMap nodes have up to 11 entries; charging 3x each entry plus 256
    // covers minimum occupancy, parent pointers and allocator bookkeeping.
    (3 * (std::mem::size_of::<K>() + std::mem::size_of::<Node>()) + 256) as u64
}
fn weighted(value: ScoreFraction, mass: u64, total: u64) -> Result<ScoreFraction, Error> {
    Ok(value.checked_mul(ScoreFraction::new(
        i64::try_from(mass).map_err(|_| Error::ArithmeticOverflow)?,
        total,
    )?)?)
}
fn sum(weights: [u64; 4]) -> Result<u64, Error> {
    weights.into_iter().try_fold(0u64, |s, w| {
        s.checked_add(w).ok_or(Error::ArithmeticOverflow)
    })
}
fn normalized(mut weights: [u64; 4]) -> Result<[u64; 4], Error> {
    let mut divisor = 0;
    for w in weights {
        let (mut a, mut b) = (divisor, w);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        divisor = a;
    }
    if divisor == 0 {
        return Err(Error::InvalidHistory("empty model support".into()));
    }
    for w in &mut weights {
        *w /= divisor;
    }
    Ok(weights)
}
fn initial_cal(prior: OwnPrior) -> CalStates {
    Mechanism::ALL.map(|m| {
        if prior == OwnPrior::Uniform || prior == OwnPrior::PointMass(m) {
            Some(CalAtom {
                fields: [Symbol::Blank; 2],
                b_probe0: false,
                a_probe1: false,
            })
        } else {
            None
        }
    })
}
fn slot_role(slot: u8) -> Role {
    if matches!(slot, 1 | 4) {
        Role::A
    } else {
        Role::B
    }
}
fn next_cal(round: u8, slot: u8) -> CalClock {
    if slot == 4 {
        CalClock::Boundary(round)
    } else {
        CalClock::Slot {
            round,
            slot: slot + 1,
        }
    }
}
fn recipe(atom: CalAtom, round: u8, slot: u8) -> Action {
    match (round, slot) {
        (1, 1) => Action::Write(Symbol::Probe0),
        (1, 2 | 4) | (2, 1 | 2 | 4) | (3, 2) => Action::Read,
        (1, 3) if atom.b_probe0 => Action::Write(Symbol::Ack0),
        (2, 3) => Action::Write(Symbol::Probe1),
        (3, 1) if atom.a_probe1 => Action::Write(Symbol::Ack1),
        _ => Action::Wait,
    }
}
fn position(round: u8, slot: u8) -> Position {
    Position {
        phase: Phase::Calibration,
        round,
        slot,
    }
}
fn cal_action(
    protocol: &Protocol,
    model: usize,
    mut atom: CalAtom,
    round: u8,
    slot: u8,
    action: Action,
) -> Result<(Outcome, CalAtom), Error> {
    // Reconstruct only the sufficient fields; all physics still comes from the
    // frozen World. These projection writes/reads are hypothetical, never paid
    // episode events and never appended to a controller prefix.
    let mechanism = Mechanism::ALL[model];
    let mut world = World::new(Environment::InFamily(mechanism), protocol.ids.clone());
    for role in [Role::A, Role::B] {
        if role == Role::B && mechanism != Mechanism::PrivatePersistent {
            continue;
        }
        let symbol = atom.fields[role.index()];
        if symbol != Symbol::Blank {
            world.apply(
                role,
                position(1, role.index() as u8 + 1),
                &Action::Write(symbol),
                None,
                48,
            )?;
        }
    }
    let outcome = world
        .apply(slot_role(slot), position(round, slot), &action, None, 48)?
        .outcome;
    if (round, slot) == (1, 2) {
        atom.b_probe0 = outcome == Outcome::Read(Symbol::Probe0);
    }
    if (round, slot) == (2, 4) {
        atom.a_probe1 = outcome == Outcome::Read(Symbol::Probe1);
    }
    if slot == 4 {
        world.finish_round();
    }
    for role in [Role::A, Role::B] {
        if let Outcome::Read(symbol) = world
            .apply(
                role,
                position(1, role.index() as u8 + 1),
                &Action::Read,
                None,
                48,
            )?
            .outcome
        {
            atom.fields[role.index()] = symbol;
        }
    }
    Ok((outcome, atom))
}
