use crate::shared_surface::{action_cost, role_at, validate_action, Outcome};
pub use crate::shared_surface::{
    Action, Belief, Environment, EpisodeBits, Event, Ids, LocalEntry, Mechanism, OwnPrior, Phase,
    Position, Probability, Role, ScoreFraction, Symbol,
};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PolicyKind {
    RawAdaptive,
    SuppliedAdaptive,
    NoProbe,
    KnownRaw,
    InspectOnly,
}
impl PolicyKind {
    pub fn raw(self) -> bool {
        matches!(self, Self::RawAdaptive | Self::KnownRaw)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Choice {
    Stop,
    Continue,
    Calibration(Action),
    Inspect,
    Attempt,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Boundary {
    Probe { completed: u8 },
    Calibration(Position),
    Trial { trial: u8 },
}
impl Boundary {
    pub fn validate(self) -> Result<(), Error> {
        match self {
            Self::Probe { completed } if completed > 3 => {
                invalid("completed calibration rounds must be in 0..=3")
            }
            Self::Trial { trial } if trial > 3 => invalid("trial must be in 0..=3"),
            Self::Calibration(p) => {
                p.validate()?;
                if p.phase != Phase::Calibration {
                    return invalid("calibration boundary requires calibration phase");
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn validate_choice(self, choice: Choice) -> Result<(), Error> {
        self.validate()?;
        match (self, choice) {
            (Self::Probe { .. }, Choice::Stop)
            | (Self::Probe { completed: 0..=2 }, Choice::Continue)
            | (Self::Trial { .. }, Choice::Inspect | Choice::Attempt) => Ok(()),
            (Self::Calibration(p), Choice::Calibration(action)) => {
                validate_action(role_at(p)?, p, &action)?;
                Ok(())
            }
            _ => invalid("choice is illegal at this boundary"),
        }
    }
    pub fn trial_position(self, role: Role) -> Result<Position, Error> {
        self.validate()?;
        let Self::Trial { trial } = self else {
            return invalid("only a trial boundary has a live position");
        };
        Ok(Position {
            phase: Phase::Live { trial },
            round: 1,
            slot: if role == Role::A { 1 } else { 2 },
        })
    }
}
impl<'de> Deserialize<'de> for Boundary {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Probe { completed: u8 },
            Calibration(Position),
            Trial { trial: u8 },
        }
        let b = match Wire::deserialize(d)? {
            Wire::Probe { completed } => Self::Probe { completed },
            Wire::Calibration(p) => Self::Calibration(p),
            Wire::Trial { trial } => Self::Trial { trial },
        };
        b.validate().map_err(serde::de::Error::custom)?;
        Ok(b)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Checkpoint {
    Boundary(Boundary),
    BeforeSlot(Position),
    AfterSlot(Position),
    Prediction { trial: u8 },
    Finished,
}
impl Checkpoint {
    pub fn validate(self) -> Result<(), Error> {
        match self {
            Self::Boundary(b) => b.validate(),
            Self::BeforeSlot(p) | Self::AfterSlot(p) => Ok(p.validate()?),
            Self::Prediction { trial } => Boundary::Trial { trial }.validate(),
            Self::Finished => Ok(()),
        }
    }
}
impl<'de> Deserialize<'de> for Checkpoint {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Boundary(Boundary),
            BeforeSlot(Position),
            AfterSlot(Position),
            Prediction { trial: u8 },
            Finished,
        }
        let c = match Wire::deserialize(d)? {
            Wire::Boundary(b) => Self::Boundary(b),
            Wire::BeforeSlot(p) => Self::BeforeSlot(p),
            Wire::AfterSlot(p) => Self::AfterSlot(p),
            Wire::Prediction { trial } => Self::Prediction { trial },
            Wire::Finished => Self::Finished,
        };
        c.validate().map_err(serde::de::Error::custom)?;
        Ok(c)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Entry {
    Physical(LocalEntry),
    OwnChoice { boundary: Boundary, choice: Choice },
    PublicProbeStop { completed: u8 },
}
impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Physical(LocalEntry),
            OwnChoice { boundary: Boundary, choice: Choice },
            PublicProbeStop { completed: u8 },
        }
        match Wire::deserialize(d)? {
            Wire::Physical(e) => Ok(Self::Physical(e)),
            Wire::OwnChoice { boundary, choice } => {
                boundary
                    .validate_choice(choice)
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::OwnChoice { boundary, choice })
            }
            Wire::PublicProbeStop { completed } => {
                Boundary::Probe { completed }
                    .validate()
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::PublicProbeStop { completed })
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Protocol {
    pub investigator: Role,
    pub policy: PolicyKind,
    pub ids: Ids,
}
impl Protocol {
    pub fn new(investigator: Role, policy: PolicyKind) -> Self {
        Self {
            investigator,
            policy,
            ids: Ids {
                surface: "status-field".into(),
                agents: ["Agent-A".into(), "Agent-B".into()],
            },
        }
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.ids
            .validate()
            .map_err(|e| Error::InvalidProtocol(e.to_string()))
    }
    /// Check declaration-specific ownership, intervention menu and prior kind.
    pub fn validate_prefix(&self, prefix: &Prefix) -> Result<(), Error> {
        self.validate()?;
        prefix.validate()?;
        if prefix.ids != self.ids {
            return invalid("prefix identifiers differ from protocol");
        }
        let investigator = prefix.role == self.investigator;
        let prior_ok = if investigator && self.policy != PolicyKind::KnownRaw {
            prefix.own_prior == OwnPrior::Uniform
        } else {
            matches!(prefix.own_prior, OwnPrior::PointMass(_))
        };
        if !prior_ok {
            return invalid("own prior disagrees with declared policy role");
        }
        self.validate_supplied_actions(prefix)?;
        if !investigator {
            if prefix
                .entries
                .iter()
                .any(|e| matches!(e, Entry::OwnChoice { .. }))
                || matches!(
                    prefix.checkpoint,
                    Checkpoint::Boundary(Boundary::Calibration(_) | Boundary::Trial { .. })
                )
            {
                return invalid("helper cannot receive private choices or boundaries");
            }
            return Ok(());
        }
        for (index, entry) in prefix.entries.iter().enumerate() {
            let preceding = &prefix.entries[..index];
            match entry {
                Entry::OwnChoice { boundary, choice } => {
                    self.validate_choice(*boundary, *choice)?
                }
                Entry::PublicProbeStop { completed } => {
                    if !preceding.contains(&Entry::OwnChoice {
                        boundary: Boundary::Probe {
                            completed: *completed,
                        },
                        choice: Choice::Stop,
                    }) {
                        return invalid("public stop lacks own Stop intervention");
                    }
                }
                Entry::Physical(LocalEntry::Reset {
                    phase: Phase::Calibration,
                }) => {
                    if !preceding.contains(&Entry::OwnChoice {
                        boundary: Boundary::Probe { completed: 0 },
                        choice: Choice::Continue,
                    }) {
                        return invalid("calibration reset lacks first Continue");
                    }
                }
                Entry::Physical(LocalEntry::Action(event)) => match event.position.phase {
                    Phase::Calibration => {
                        if !preceding.contains(&Entry::OwnChoice {
                            boundary: Boundary::Probe {
                                completed: event.position.round - 1,
                            },
                            choice: Choice::Continue,
                        }) {
                            return invalid("purchased round lacks Continue");
                        }
                        if self.policy.raw()
                            && !preceding.contains(&Entry::OwnChoice {
                                boundary: Boundary::Calibration(event.position),
                                choice: Choice::Calibration(event.action),
                            })
                        {
                            return invalid("raw paid slot lacks selected own action");
                        }
                    }
                    Phase::Live { trial } => {
                        let routine = preceding
                            .iter()
                            .find_map(|entry| match entry {
                                Entry::OwnChoice {
                                    boundary: Boundary::Trial { trial: t },
                                    choice,
                                } if *t == trial => Some(*choice),
                                _ => None,
                            })
                            .ok_or_else(|| {
                                Error::InvalidHistory("live paid slot lacks own routine".into())
                            })?;
                        if event.action
                            != live_action(
                                prefix.role,
                                event.position,
                                routine,
                                current_bit(prefix, trial)?,
                            )?
                        {
                            return invalid("live action disagrees with selected routine");
                        }
                    }
                },
                _ => {}
            }
        }
        if let Checkpoint::Boundary(boundary) = prefix.checkpoint {
            self.validate_boundary(boundary)?;
        }
        if let Checkpoint::BeforeSlot(p)
        | Checkpoint::AfterSlot(p)
        | Checkpoint::Boundary(Boundary::Calibration(p)) = prefix.checkpoint
        {
            if p.phase == Phase::Calibration
                && !prefix.entries.contains(&Entry::OwnChoice {
                    boundary: Boundary::Probe {
                        completed: p.round - 1,
                    },
                    choice: Choice::Continue,
                })
            {
                return invalid("entered purchased round lacks Continue");
            }
        }
        Ok(())
    }
    fn validate_supplied_actions(&self, prefix: &Prefix) -> Result<(), Error> {
        use crate::shared_surface::{self as physical, Knowledge, LocalPrefix, LocalView};
        let models = match prefix.own_prior {
            OwnPrior::Uniform => [Probability::new(1, 4)?; 4],
            OwnPrior::PointMass(model) => Mechanism::ALL
                .map(|m| Probability::new(u64::from(m == model), 1).expect("bounded point prior")),
        };
        let mut entries = Vec::new();
        let mut credits = 48;
        let mut private_bit = None;
        for entry in &prefix.entries {
            let Entry::Physical(entry) = entry else {
                continue;
            };
            match entry {
                LocalEntry::Reset { .. } => private_bit = None,
                LocalEntry::PrivateBit { bit, .. } => private_bit = Some(*bit),
                LocalEntry::Action(event) => {
                    let supplied = prefix.role != self.investigator
                        || (event.position.phase == Phase::Calibration && !self.policy.raw());
                    if supplied {
                        // Reconstruct only the acting Agent's preceding physical records.
                        // Neither peer history nor actual environment enters this view.
                        let view = LocalView {
                            prefix: LocalPrefix {
                                role: prefix.role,
                                ids: prefix.ids.clone(),
                                own_prior: prefix.own_prior,
                                checkpoint: physical::Checkpoint::Action(event.position),
                                entries: entries.clone(),
                            },
                            credits,
                            private_bit,
                            belief: Belief {
                                models,
                                target: None,
                            },
                        };
                        let expected = physical::decide(&view, Knowledge::Known)?.action;
                        if expected != event.action {
                            return Err(Error::InvalidHistory(format!(
                                "Agent {:?} supplied action at {:?} differs from own-prefix routine",
                                prefix.role, event.position)));
                        }
                    }
                    credits = event.credits_after;
                }
            }
            entries.push(entry.clone());
        }
        Ok(())
    }
    pub fn validate_boundary(&self, boundary: Boundary) -> Result<(), Error> {
        boundary.validate()?;
        if let Boundary::Calibration(p) = boundary {
            if !self.policy.raw() || role_at(p)? != self.investigator {
                return invalid("raw calibration boundary is not owned by investigator");
            }
        }
        Ok(())
    }
    /// Validate a legal own intervention; CompiledPolicy selects on-policy choices.
    pub fn validate_choice(&self, boundary: Boundary, choice: Choice) -> Result<(), Error> {
        self.validate_boundary(boundary)?;
        boundary.validate_choice(choice)?;
        Ok(())
    }
}
impl<'de> Deserialize<'de> for Protocol {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            investigator: Role,
            policy: PolicyKind,
            ids: Ids,
        }
        let w = Wire::deserialize(d)?;
        let p = Self {
            investigator: w.investigator,
            policy: w.policy,
            ids: w.ids,
        };
        p.validate().map_err(serde::de::Error::custom)?;
        Ok(p)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Prefix {
    pub role: Role,
    pub ids: Ids,
    pub own_prior: OwnPrior,
    pub checkpoint: Checkpoint,
    pub entries: Vec<Entry>,
}
impl Prefix {
    pub fn validate(&self) -> Result<(), Error> {
        self.local_state().map(|_| ())
    }
    fn local_state(&self) -> Result<(u8, Option<bool>), Error> {
        self.ids.validate()?;
        self.checkpoint.validate()?;
        let (mut phase, mut actions, mut credits, mut bit) = (None, 0u8, 48u8, None);
        let (mut needs_bit, mut stop, mut probe, mut calibration, mut routine) =
            (false, None, None, None, None);
        for entry in &self.entries {
            if needs_bit && !matches!(entry, Entry::Physical(LocalEntry::PrivateBit { .. })) {
                return invalid("live reset must immediately issue own bit");
            }
            if matches!(probe, Some((_, Choice::Stop)))
                && stop.is_none()
                && !matches!(entry, Entry::PublicProbeStop { .. })
            {
                return invalid("own Stop must immediately publish stop");
            }
            match entry {
                Entry::OwnChoice { boundary, choice } => {
                    boundary.validate_choice(*choice)?;
                    if public_clock(Checkpoint::Boundary(*boundary), self.role)?
                        > public_clock(self.checkpoint, self.role)?
                    {
                        return invalid("own choice precedes its public opportunity");
                    }
                    match *boundary {
                        Boundary::Probe { completed } => {
                            if stop.is_some()
                                || matches!(phase, Some(Phase::Live { .. }))
                                || actions != 2 * completed
                                || (completed == 0 && phase.is_some())
                                || probe.is_some_and(|(c, _)| c >= completed)
                                || calibration.is_some()
                            {
                                return invalid("duplicate or mistimed probe choice");
                            }
                            probe = Some((completed, *choice));
                        }
                        Boundary::Calibration(p) => {
                            if phase != Some(Phase::Calibration)
                                || calibration.is_some()
                                || p != own_position(self.role, Phase::Calibration, actions)?
                                || stop.is_some()
                            {
                                return invalid(
                                    "duplicate, wrong-owner or mistimed calibration choice",
                                );
                            }
                            calibration = Some((p, *choice));
                        }
                        Boundary::Trial { trial } => {
                            if phase != Some(Phase::Live { trial })
                                || bit.is_none()
                                || actions != 0
                                || routine.is_some()
                            {
                                return invalid("duplicate or mistimed live choice");
                            }
                            routine = Some(*choice);
                        }
                    }
                }
                Entry::PublicProbeStop { completed } => {
                    Boundary::Probe {
                        completed: *completed,
                    }
                    .validate()?;
                    if stop.is_some()
                        || matches!(phase, Some(Phase::Live { .. }))
                        || actions != 2 * completed
                        || (*completed == 0 && phase.is_some())
                        || probe.is_some_and(|p| p != (*completed, Choice::Stop))
                        || calibration.is_some()
                    {
                        return invalid("duplicate or mistimed public stop");
                    }
                    stop = Some(*completed);
                }
                Entry::Physical(LocalEntry::Reset { phase: next }) => {
                    next.validate()?;
                    match *next {
                        Phase::Calibration => {
                            if phase.is_some()
                                || stop.is_some()
                                || probe.is_some_and(|p| p != (0, Choice::Continue))
                            {
                                return invalid("calibration reset requires first purchased round");
                            }
                        }
                        Phase::Live { trial: 0 } => {
                            if stop.is_none() || matches!(phase, Some(Phase::Live { .. })) {
                                return invalid("first live reset requires public stop");
                            }
                        }
                        Phase::Live { trial } => {
                            if phase != Some(Phase::Live { trial: trial - 1 }) || actions != 6 {
                                return invalid("live reset skips unfinished trial");
                            }
                        }
                    }
                    if calibration.is_some() {
                        return invalid("reset precedes selected paid action");
                    }
                    phase = Some(*next);
                    actions = 0;
                    bit = None;
                    needs_bit = matches!(next, Phase::Live { .. });
                    routine = None;
                }
                Entry::Physical(LocalEntry::PrivateBit { trial, bit: issued }) => {
                    if !needs_bit || phase != Some(Phase::Live { trial: *trial }) || bit.is_some() {
                        return invalid("duplicate, missing or future own bit");
                    }
                    bit = Some(*issued);
                    needs_bit = false;
                }
                Entry::Physical(LocalEntry::Action(event)) => {
                    if phase != Some(event.position.phase)
                        || event.position != own_position(self.role, event.position.phase, actions)?
                        || (event.position.phase == Phase::Calibration && stop.is_some())
                    {
                        return invalid("missing, repeated or out-of-order own paid opportunity");
                    }
                    validate_action(self.role, event.position, &event.action)?;
                    if event.position.phase == Phase::Calibration {
                        if probe.is_some_and(|p| p != (event.position.round - 1, Choice::Continue))
                        {
                            return invalid("calibration action lacks current Continue");
                        }
                        if let Some((position, choice)) = calibration.take() {
                            if position != event.position
                                || choice != Choice::Calibration(event.action)
                            {
                                return invalid(
                                    "paid calibration action differs from queued selection",
                                );
                            }
                        }
                    } else {
                        let issued = bit.ok_or_else(|| {
                            Error::InvalidHistory("live action lacks issued bit".into())
                        })?;
                        if let Some(choice) = routine {
                            if event.action
                                != live_action(self.role, event.position, choice, issued)?
                            {
                                return invalid("paid live action differs from routine");
                            }
                        }
                    }
                    credits = credits
                        .checked_sub(action_cost(&event.action))
                        .ok_or_else(|| Error::InvalidHistory("insufficient own credits".into()))?;
                    if credits != event.credits_after {
                        return invalid("own credits disagree with paid action cost");
                    }
                    if !matches!(
                        (event.action, event.outcome),
                        (Action::Read, Outcome::Read(_))
                            | (Action::Write(_), Outcome::Accepted)
                            | (Action::Wait, Outcome::Waited)
                            | (Action::InspectOwnTarget, Outcome::Inspected(_))
                    ) {
                        return invalid("outcome differs from paid action kind");
                    }
                    actions += 1;
                }
            }
        }
        if needs_bit {
            return invalid("live reset omitted issued own bit");
        }
        if matches!(probe, Some((_, Choice::Stop))) && stop.is_none() {
            return invalid("own Stop omitted atomic public stop");
        }
        if routine.is_some()
            && actions == 0
            && !matches!(
                self.checkpoint,
                Checkpoint::Boundary(Boundary::Trial { .. })
            )
        {
            return invalid("queued live selection must retain its Trial boundary");
        }
        match self.checkpoint {
            Checkpoint::Boundary(Boundary::Probe { completed }) => {
                if matches!(phase, Some(Phase::Live { .. }))
                    || actions != 2 * completed
                    || (completed == 0 && phase.is_some())
                    || calibration.is_some()
                {
                    return invalid("probe checkpoint differs from completed rounds");
                }
            }
            Checkpoint::Boundary(Boundary::Calibration(p)) => {
                if phase != Some(Phase::Calibration)
                    || stop.is_some()
                    || p != own_position(self.role, Phase::Calibration, actions)?
                {
                    return invalid("calibration boundary is not next owned opportunity");
                }
                if calibration.is_some_and(|(selected, _)| selected != p) {
                    return invalid("queued calibration position differs from checkpoint");
                }
            }
            Checkpoint::Boundary(Boundary::Trial { trial }) => {
                if phase != Some(Phase::Live { trial }) || actions != 0 {
                    return invalid("trial boundary is not first owned opportunity");
                }
            }
            Checkpoint::BeforeSlot(p) | Checkpoint::AfterSlot(p) => {
                let inclusive = matches!(self.checkpoint, Checkpoint::AfterSlot(_));
                let slots = if self.role == Role::A { [1, 4] } else { [2, 3] };
                let expected = (1..=3)
                    .flat_map(|round| slots.into_iter().map(move |slot| (round, slot)))
                    .filter(|&(round, slot)| {
                        (round, slot) < (p.round, p.slot)
                            || (inclusive && (round, slot) == (p.round, p.slot))
                    })
                    .count();
                if phase != Some(p.phase)
                    || usize::from(actions) != expected
                    || (p.phase == Phase::Calibration && stop.is_some())
                    || calibration.is_some()
                {
                    return invalid("paid checkpoint differs from own chronology");
                }
            }
            Checkpoint::Prediction { trial } => {
                if phase != Some(Phase::Live { trial }) || actions != 6 {
                    return invalid("prediction precedes complete live trial");
                }
            }
            Checkpoint::Finished => {
                if phase != Some(Phase::Live { trial: 3 }) || actions != 6 {
                    return invalid("Finished precedes four complete trials");
                }
            }
        }
        Ok((credits, bit))
    }
}
// Order public opportunities without inserting an unobserved peer Event.
// The final component distinguishes a paid AfterSlot from the later free finish.
fn public_clock(checkpoint: Checkpoint, role: Role) -> Result<[u8; 4], Error> {
    let position_clock = |position: Position, after| {
        let phase = match position.phase {
            Phase::Calibration => 0,
            Phase::Live { trial } => trial + 1,
        };
        [phase, position.round, position.slot, after]
    };
    Ok(match checkpoint {
        Checkpoint::Boundary(Boundary::Probe { completed }) => [0, completed, 4, 2],
        Checkpoint::Boundary(Boundary::Calibration(p)) | Checkpoint::BeforeSlot(p) => {
            position_clock(p, 0)
        }
        Checkpoint::Boundary(boundary @ Boundary::Trial { .. }) => {
            position_clock(boundary.trial_position(role)?, 0)
        }
        Checkpoint::AfterSlot(p) => position_clock(p, 1),
        Checkpoint::Prediction { trial } => [trial + 1, 3, 4, 2],
        Checkpoint::Finished => [5, 0, 0, 0],
    })
}
fn own_position(role: Role, phase: Phase, actions: u8) -> Result<Position, Error> {
    if actions >= 6 {
        return invalid("phase has no further owned opportunities");
    }
    Ok(Position {
        phase,
        round: actions / 2 + 1,
        slot: if role == Role::A { [1, 4] } else { [2, 3] }[usize::from(actions % 2)],
    })
}
fn current_bit(prefix: &Prefix, trial: u8) -> Result<bool, Error> {
    prefix
        .entries
        .iter()
        .rev()
        .find_map(|e| match e {
            Entry::Physical(LocalEntry::PrivateBit { trial: t, bit }) if *t == trial => Some(*bit),
            _ => None,
        })
        .ok_or_else(|| Error::InvalidHistory("routine lacks current own bit".into()))
}
pub(super) fn live_action(
    role: Role,
    p: Position,
    routine: Choice,
    bit: bool,
) -> Result<Action, Error> {
    if !matches!(p.phase, Phase::Live { .. }) {
        return invalid("live routine requires live phase");
    }
    let send = matches!((role, p.round, p.slot), (Role::A, 1, 1) | (Role::B, 2, 3));
    match routine {
        Choice::Inspect => Ok(if send {
            Action::InspectOwnTarget
        } else {
            Action::Wait
        }),
        Choice::Attempt => Ok(if send {
            Action::Write(if bit { Symbol::Data1 } else { Symbol::Data0 })
        } else if matches!((role, p.round, p.slot), (Role::A, 3, 1) | (Role::B, 2, 2)) {
            Action::Read
        } else {
            Action::Wait
        }),
        _ => invalid("invalid live routine"),
    }
}
impl<'de> Deserialize<'de> for Prefix {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            role: Role,
            ids: Ids,
            own_prior: OwnPrior,
            checkpoint: Checkpoint,
            entries: Vec<Entry>,
        }
        let w = Wire::deserialize(d)?;
        let p = Self {
            role: w.role,
            ids: w.ids,
            own_prior: w.own_prior,
            checkpoint: w.checkpoint,
            entries: w.entries,
        };
        p.validate().map_err(serde::de::Error::custom)?;
        Ok(p)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct View {
    pub prefix: Prefix,
    pub credits: u8,
    pub private_bit: Option<bool>,
    pub belief: Belief,
}
impl View {
    pub fn from_prefix(prefix: Prefix, belief: Belief) -> Result<Self, Error> {
        let (credits, private_bit) = prefix.local_state()?;
        let total = belief
            .models
            .iter()
            .try_fold(Probability::new(0, 1)?, |sum, p| sum.checked_add(*p))?;
        if total != Probability::new(1, 1)? {
            return invalid("model probabilities must sum to one");
        }
        if matches!(
            prefix.checkpoint,
            Checkpoint::Boundary(Boundary::Trial { .. }) | Checkpoint::Prediction { .. }
        ) && belief.target.is_none()
        {
            return invalid("live decision and prediction require a target posterior");
        }
        if let Some(target) = belief.target {
            Probability::new(target.numerator, target.denominator)?;
            if private_bit.is_none() {
                return invalid("target posterior requires issued live bit");
            }
        }
        if let OwnPrior::PointMass(model) = prefix.own_prior {
            if belief.models[model.index()] != Probability::new(1, 1)? {
                return invalid("belief contradicts own point prior");
            }
        }
        Ok(Self {
            prefix,
            credits,
            private_bit,
            belief,
        })
    }
}
fn required_option<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::deserialize(d)
}
fn required_belief<'de, D: Deserializer<'de>>(d: D) -> Result<Belief, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        models: [Probability; 4],
        #[serde(deserialize_with = "required_option")]
        target: Option<Probability>,
    }
    let w = Wire::deserialize(d)?;
    Ok(Belief {
        models: w.models,
        target: w.target,
    })
}
impl<'de> Deserialize<'de> for View {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            prefix: Prefix,
            credits: u8,
            #[serde(deserialize_with = "required_option")]
            private_bit: Option<bool>,
            #[serde(deserialize_with = "required_belief")]
            belief: Belief,
        }
        let w = Wire::deserialize(d)?;
        let view = Self::from_prefix(w.prefix, w.belief).map_err(serde::de::Error::custom)?;
        if view.credits != w.credits || view.private_bit != w.private_bit {
            return Err(serde::de::Error::custom(
                "view credits/private bit differ from own prefix",
            ));
        }
        Ok(view)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub states: u64,
    pub branches: u64,
    pub allocation_bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Error {
    InvalidProtocol(String),
    InvalidHistory(String),
    UnsupportedHistory { prefix: Prefix },
    ResourceLimit { resource: String, limit: u64 },
    InvalidReport(String),
    ArithmeticOverflow,
    Physical(crate::shared_surface::Error),
}
impl From<crate::shared_surface::Error> for Error {
    fn from(e: crate::shared_surface::Error) -> Self {
        match e {
            crate::shared_surface::Error::ArithmeticOverflow => Self::ArithmeticOverflow,
            other => Self::Physical(other),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProtocol(s) => write!(f, "invalid surface-discovery protocol: {s}"),
            Self::InvalidHistory(s) => write!(f, "invalid surface-discovery history: {s}"),
            Self::InvalidReport(s) => write!(f, "invalid surface-discovery report: {s}"),
            Self::UnsupportedHistory { prefix } => write!(
                f,
                "unsupported surface-discovery history for Agent {:?} at {:?}",
                prefix.role, prefix.checkpoint
            ),
            Self::ResourceLimit { resource, limit } => {
                write!(f, "surface-discovery {resource} limit {limit} exhausted")
            }
            Self::ArithmeticOverflow => f.write_str("surface-discovery exact arithmetic overflow"),
            Self::Physical(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {}
fn invalid<T>(message: &str) -> Result<T, Error> {
    Err(Error::InvalidHistory(message.into()))
}
