//! Investigator likelihoods under private own interventions. This replay never
//! consults the actual episode, peer transcript, or unissued actual task bits.
use super::*;

impl CompiledPolicy {
    pub fn infer(&self, prefix: &Prefix) -> Result<Belief, Error> {
        let support = self.candidate_support(prefix)?;
        belief_from_support(prefix, &support)
    }
    pub(crate) fn candidate_support(&self, prefix: &Prefix) -> Result<Vec<u16>, Error> {
        self.protocol().validate_prefix(prefix)?;
        if prefix.role != self.protocol().investigator || prefix.own_prior != self.own_prior() {
            return Err(Error::InvalidHistory(
                "prefix owner/prior differs from compiled investigator".into(),
            ));
        }
        let latest = issued_trial(prefix);
        // Unissued bits have no physical/controller influence before this clock.
        // Replay each issued-bit equivalence class once, then lift EVERY original
        // ID in the class. No marginal renumbering or duplicated mass is allowed.
        let period = latest.map_or(1u16, |trial| 1 << (2 * (trial + 1)));
        let mut support = Vec::with_capacity(1024);
        for model in Mechanism::ALL {
            if matches!(self.own_prior(), OwnPrior::PointMass(m) if m != model) {
                continue;
            }
            for low in 0..period {
                let bits = EpisodeBits::from_index(low)?;
                if prefix.entries.iter().any(|e| match e {
                    Entry::Physical(LocalEntry::PrivateBit { trial, bit }) => {
                        let pair = bits.0[usize::from(*trial)];
                        (if prefix.role == Role::A {
                            pair.0
                        } else {
                            pair.1
                        }) != *bit
                    }
                    _ => false,
                }) {
                    continue;
                }
                if matches_candidate(self.protocol(), self.own_prior(), model, bits, prefix)? {
                    for sequence in (low..256).step_by(usize::from(period)) {
                        support.push((model.index() as u16) * 256 + sequence);
                    }
                }
            }
        }
        if support.is_empty() {
            return Err(Error::UnsupportedHistory {
                prefix: prefix.clone(),
            });
        }
        support.sort_unstable();
        Ok(support)
    }
}
fn matches_candidate(
    protocol: &Protocol,
    prior: OwnPrior,
    model: Mechanism,
    bits: EpisodeBits,
    prefix: &Prefix,
) -> Result<bool, Error> {
    let mut priors = [OwnPrior::PointMass(model); 2];
    priors[protocol.investigator.index()] = prior;
    let mut state = EpisodeState::new(protocol, Environment::InFamily(model), bits, priors)?;
    loop {
        let current = state.prefix(prefix.role);
        if current == prefix {
            return Ok(true);
        }
        if !prefix.entries.starts_with(&current.entries) || state.is_finished() {
            return Ok(false);
        }
        if let Some(boundary) = state.next_boundary() {
            let Some(Entry::OwnChoice {
                boundary: declared,
                choice,
            }) = prefix.entries.get(current.entries.len())
            else {
                return Ok(false);
            };
            if *declared != boundary {
                return Ok(false);
            }
            // The given own action is imposed. Its policy selection probability
            // is NEVER another likelihood factor (including forced controls).
            select_choice(&mut state, *choice)?;
        } else {
            advance_one(&mut state)?;
        }
    }
}
pub(super) fn issued_trial(prefix: &Prefix) -> Option<u8> {
    prefix.entries.iter().rev().find_map(|e| match e {
        Entry::Physical(LocalEntry::PrivateBit { trial, .. }) => Some(*trial),
        _ => None,
    })
}
pub(super) fn model_weights(support: &[u16]) -> [u64; 4] {
    let mut weights = [0; 4];
    for id in support {
        weights[usize::from(*id / 256)] += 1;
    }
    weights
}
pub(super) fn belief_from_support(prefix: &Prefix, support: &[u16]) -> Result<Belief, Error> {
    let total = support.len() as u64;
    let weights = model_weights(support);
    let mut models = [Probability::new(0, 1)?; 4];
    for (index, weight) in weights.into_iter().enumerate() {
        models[index] = Probability::new(weight, total)?;
    }
    let target = issued_trial(prefix)
        .map(|trial| {
            let peer = 1 - prefix.role.index();
            let ones = support
                .iter()
                .filter(|id| ((*id % 256) >> (2 * usize::from(trial) + peer)) & 1 == 1)
                .count() as u64;
            Probability::new(ones, total).map_err(Error::from)
        })
        .transpose()?;
    Ok(Belief { models, target })
}
