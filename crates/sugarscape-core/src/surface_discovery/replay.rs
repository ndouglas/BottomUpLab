//! Candidate-local likelihoods for the declared investigator policy.
use super::budget::Budget;
use super::*;
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Default)]
struct Support([u64; 16]);
impl Support {
    fn insert(&mut self, id: u16) {
        self.0[usize::from(id / 64)] |= 1 << (id % 64);
    }
    fn ids(&self) -> Vec<u16> {
        (0..1024)
            .filter(|id| self.0[usize::from(*id / 64)] & (1 << (*id % 64)) != 0)
            .collect()
    }
}

/// Original-world replay. Private selection can repeat a peer prefix; bitsets
/// ensure each original model*256+sequence contributes exactly once.
/// Accounting belongs to this one compilation and survives every host call.
pub struct Replay {
    protocol: Protocol,
    prior: OwnPrior,
    supports: BTreeMap<Prefix, Support>,
    beliefs: BTreeMap<Prefix, Belief>,
    decisions: BTreeMap<Prefix, Decision>,
    budget: RefCell<Budget>,
}
impl Replay {
    pub fn build(
        protocol: &Protocol,
        policy: &CompiledPolicy,
        limits: &Limits,
    ) -> Result<Self, Error> {
        protocol.validate()?;
        limits.validate()?;
        if protocol != policy.protocol() || limits != &policy.budget.limits {
            return Err(Error::InvalidProtocol(
                "replay declaration or limits differ from compilation".into(),
            ));
        }
        let mut budget = policy.budget.clone();
        // One bounded workspace is shared by synchronous candidate/host calls.
        budget.allocate(1 << 20)?;
        let mut replay = Self {
            protocol: protocol.clone(),
            prior: policy.own_prior(),
            supports: BTreeMap::new(),
            beliefs: BTreeMap::new(),
            decisions: BTreeMap::new(),
            budget: RefCell::new(budget),
        };
        for model in Mechanism::ALL {
            if replay.prior != OwnPrior::Uniform && replay.prior != OwnPrior::PointMass(model) {
                continue;
            }
            for sequence in 0..256 {
                let id = model.index() as u16 * 256 + sequence;
                let mut priors = [OwnPrior::PointMass(model); 2];
                priors[protocol.investigator.index()] = replay.prior;
                let mut state = EpisodeState::new(
                    protocol,
                    Environment::InFamily(model),
                    EpisodeBits::from_index(sequence)?,
                    priors,
                )?;
                loop {
                    replay.checkpoint()?;
                    for role in [Role::A, Role::B] {
                        let prefix = state.prefix(role);
                        if !replay.supports.contains_key(prefix) {
                            let bytes = prefix_bytes(prefix)? + 512;
                            if role == protocol.investigator {
                                replay.budget.borrow_mut().allocate(bytes)?;
                            } else {
                                replay.budget.borrow_mut().state(bytes)?;
                            }
                            replay.supports.insert(prefix.clone(), Support::default());
                        }
                        replay
                            .supports
                            .get_mut(prefix)
                            .expect("inserted support")
                            .insert(id);
                    }
                    if state.is_finished() {
                        break;
                    }
                    replay.transition()?;
                    if state.next_boundary().is_some() {
                        let prefix = state.prefix(protocol.investigator);
                        let choice = if let Some(decision) = replay.decisions.get(prefix) {
                            decision.choice
                        } else {
                            let decision = policy.decide(&View::from_prefix(
                                prefix.clone(),
                                policy.infer(prefix)?,
                            )?)?;
                            replay
                                .budget
                                .borrow_mut()
                                .allocate(prefix_bytes(prefix)? + 1024)?;
                            let choice = decision.choice;
                            replay.decisions.insert(prefix.clone(), decision);
                            choice
                        };
                        select_choice(&mut state, choice)?;
                    } else {
                        advance_one(&mut state)?;
                    }
                }
            }
        }
        // During conversion support, belief index and temporary IDs coexist.
        replay.budget.borrow_mut().allocate(2048)?;
        for prefix in replay.supports.keys() {
            replay
                .budget
                .borrow_mut()
                .allocate(prefix_bytes(prefix)? + 512)?;
            replay.beliefs.insert(
                prefix.clone(),
                super::belief::belief_from_support(prefix, &replay.candidate_support(prefix)?)?,
            );
        }
        replay.budget.borrow_mut().release(2048)?;
        Ok(replay)
    }
    pub fn infer(&self, prefix: &Prefix) -> Result<Belief, Error> {
        self.validate_prefix(prefix)?;
        self.beliefs
            .get(prefix)
            .cloned()
            .ok_or_else(|| Error::UnsupportedHistory {
                prefix: prefix.clone(),
            })
    }
    pub fn stats(&self) -> SearchStats {
        self.budget.borrow().stats.clone()
    }
    /// Sorted original model*256+sequence IDs for a declared-policy prefix.
    pub(crate) fn candidate_support(&self, prefix: &Prefix) -> Result<Vec<u16>, Error> {
        self.validate_prefix(prefix)?;
        self.supports
            .get(prefix)
            .map(Support::ids)
            .ok_or_else(|| Error::UnsupportedHistory {
                prefix: prefix.clone(),
            })
    }
    fn validate_prefix(&self, prefix: &Prefix) -> Result<(), Error> {
        self.protocol.validate_prefix(prefix)?;
        if prefix.role == self.protocol.investigator && prefix.own_prior != self.prior {
            return Err(Error::InvalidHistory(
                "prefix prior differs from compilation".into(),
            ));
        }
        Ok(())
    }
    pub(super) fn infer_for_host(
        &self,
        prefix: &Prefix,
        policy: &CompiledPolicy,
    ) -> Result<Belief, Error> {
        // Deterministic on-policy own choices carry no additional likelihood.
        // The cached original-ID mass is therefore the intervention posterior
        // on these prefixes. Unindexed investigator interventions use full inference.
        match self.infer(prefix) {
            Err(Error::UnsupportedHistory { .. }) if prefix.role == self.protocol.investigator => {
                policy.infer(prefix)
            }
            result => result,
        }
    }
    pub(super) fn decide(&self, policy: &CompiledPolicy, view: &View) -> Result<Decision, Error> {
        if let Some(decision) = self.decisions.get(&view.prefix) {
            Ok(decision.clone())
        } else {
            policy.decide(view)
        }
    }
    pub(super) fn validate_policy(
        &self,
        protocol: &Protocol,
        policy: &CompiledPolicy,
    ) -> Result<(), Error> {
        if protocol != &self.protocol
            || protocol != policy.protocol()
            || self.prior != policy.own_prior()
        {
            return Err(Error::InvalidProtocol(
                "host, replay, and compiled policy must agree".into(),
            ));
        }
        Ok(())
    }
    pub(super) fn checkpoint(&self) -> Result<(), Error> {
        self.budget.borrow_mut().state(0)
    }
    pub(super) fn transition(&self) -> Result<(), Error> {
        self.budget.borrow_mut().branch()
    }
    pub(super) fn reserve_episode(&self) -> Result<EpisodeReservation<'_>, Error> {
        let bytes = self.episode_bytes()?;
        self.budget.borrow_mut().allocate(bytes)?;
        Ok(EpisodeReservation {
            budget: &self.budget,
            bytes,
            retained: false,
        })
    }
    pub(super) fn release_episodes(&self, count: usize) -> Result<(), Error> {
        let bytes = self
            .episode_bytes()?
            .checked_mul(count as u64)
            .ok_or(Error::ArithmeticOverflow)?;
        self.budget.borrow_mut().release(bytes)
    }
    fn episode_bytes(&self) -> Result<u64, Error> {
        // Returned values can coexist outside this owner. Charge every return
        // conservatively; never pretend the caller has dropped a prior panel.
        // <=64 records/history, <=16 decisions, <=60 events; capacity slack x2.
        let id_bytes = identifier_bytes(&self.protocol.ids)?;
        let bytes = 131072u64
            .checked_add(id_bytes.checked_mul(80).ok_or(Error::ArithmeticOverflow)?)
            .ok_or(Error::ArithmeticOverflow)?;
        Ok(bytes)
    }
}
fn identifier_bytes(ids: &Ids) -> Result<u64, Error> {
    [&ids.surface, &ids.agents[0], &ids.agents[1]]
        .into_iter()
        .try_fold(0u64, |sum, id| {
            sum.checked_add(u64::try_from(id.len()).map_err(|_| Error::ArithmeticOverflow)?)
                .ok_or(Error::ArithmeticOverflow)
        })
}
fn prefix_bytes(prefix: &Prefix) -> Result<u64, Error> {
    let entries = (prefix.entries.len() as u64)
        .checked_mul(2 * std::mem::size_of::<Entry>() as u64)
        .ok_or(Error::ArithmeticOverflow)?;
    (std::mem::size_of::<Prefix>() as u64)
        .checked_add(identifier_bytes(&prefix.ids)?)
        .and_then(|n| n.checked_add(entries))
        .and_then(|n| n.checked_mul(3))
        .ok_or(Error::ArithmeticOverflow)
}

/// Failed host calls return no retained result; release their reservation while
/// keeping the peak and cumulative computation counters intact.
pub(super) struct EpisodeReservation<'a> {
    budget: &'a RefCell<Budget>,
    bytes: u64,
    retained: bool,
}
impl EpisodeReservation<'_> {
    pub(super) fn retain(mut self) {
        self.retained = true;
    }
}
impl Drop for EpisodeReservation<'_> {
    fn drop(&mut self) {
        if !self.retained {
            self.budget
                .borrow_mut()
                .release(self.bytes)
                .expect("balanced episode reservation");
        }
    }
}
