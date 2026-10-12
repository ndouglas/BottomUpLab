use super::validation::{decode, strict_value};
use serde::{Deserialize, Deserializer, Serialize};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput(String),
    InvalidHistory(String),
    InvalidReport(String),
    ResourceLimit { resource: String, limit: u64 },
    ArithmeticOverflow,
    Io(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "key delegation: {self:?}")
    }
}
impl std::error::Error for Error {}
pub(super) fn invalid(s: impl Into<String>) -> Error {
    Error::InvalidInput(s.into())
}
pub(super) fn history_error(s: impl Into<String>) -> Error {
    Error::InvalidHistory(s.into())
}
// Each native object rejects lossy input before decoding its exact typed shape.
macro_rules! object {
    ($name:ident { $($vis:vis $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
        pub struct $name { $($vis $field: $ty),* }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                #[derive(Serialize, Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Wire { $($field: $ty),* }
                let wire: Wire = decode(strict_value(d)?)?;
                let result = Self { $($field: wire.$field),* };
                result.validate().map_err(serde::de::Error::custom)?;
                Ok(result)
            }
        }
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Role {
    Giver,
    Receiver,
}
impl<'de> Deserialize<'de> for Role {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "Giver" => Ok(Self::Giver),
            "Receiver" => Ok(Self::Receiver),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum InitialOwner {
    Giver,
    Receiver,
}
impl<'de> Deserialize<'de> for InitialOwner {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "Giver" => Ok(Self::Giver),
            "Receiver" => Ok(Self::Receiver),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Cell {
    P,
    O,
    C,
}
impl<'de> Deserialize<'de> for Cell {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "P" => Ok(Self::P),
            "O" => Ok(Self::O),
            "C" => Ok(Self::C),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum GiverToken {
    Wait,
    Approach,
    Pass,
}
impl<'de> Deserialize<'de> for GiverToken {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "Wait" => Ok(Self::Wait),
            "Approach" => Ok(Self::Approach),
            "Pass" => Ok(Self::Pass),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ReceiverToken {
    Wait,
    Advance,
}
impl<'de> Deserialize<'de> for ReceiverToken {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "Wait" => Ok(Self::Wait),
            "Advance" => Ok(Self::Advance),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Termination {
    Delivered,
    Deadline,
}
impl<'de> Deserialize<'de> for Termination {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "Delivered" => Ok(Self::Delivered),
            "Deadline" => Ok(Self::Deadline),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Phase {
    Initial,
    BeforeAction,
    AfterAction,
    Finished,
}
impl<'de> Deserialize<'de> for Phase {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "Initial" => Ok(Self::Initial),
            "BeforeAction" => Ok(Self::BeforeAction),
            "AfterAction" => Ok(Self::AfterAction),
            "Finished" => Ok(Self::Finished),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ResourceLocation {
    Cache,
    Carried,
    Delivered,
}
impl<'de> Deserialize<'de> for ResourceLocation {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "Cache" => Ok(Self::Cache),
            "Carried" => Ok(Self::Carried),
            "Delivered" => Ok(Self::Delivered),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum BlockReason {
    AlreadyAtOutside,
    TransferDisabled,
    NotKeyOwner,
    NotColocated,
    NoToken,
    WrongPosition,
    EmptyHands,
    FullHands,
}
impl<'de> Deserialize<'de> for BlockReason {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "AlreadyAtOutside" => Ok(Self::AlreadyAtOutside),
            "TransferDisabled" => Ok(Self::TransferDisabled),
            "NotKeyOwner" => Ok(Self::NotKeyOwner),
            "NotColocated" => Ok(Self::NotColocated),
            "NoToken" => Ok(Self::NoToken),
            "WrongPosition" => Ok(Self::WrongPosition),
            "EmptyHands" => Ok(Self::EmptyHands),
            "FullHands" => Ok(Self::FullHands),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
impl Role {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Giver => 0,
            Self::Receiver => 1,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Token {
    Giver(GiverToken),
    Receiver(ReceiverToken),
}
impl<'de> Deserialize<'de> for Token {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Giver(GiverToken),
            Receiver(ReceiverToken),
        }
        let wire: Wire = decode(strict_value(d)?)?;
        Ok(match wire {
            Wire::Giver(v) => Self::Giver(v),
            Wire::Receiver(v) => Self::Receiver(v),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum OwnWord {
    Giver([GiverToken; 6]),
    Receiver([ReceiverToken; 6]),
}
impl<'de> Deserialize<'de> for OwnWord {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Giver([GiverToken; 6]),
            Receiver([ReceiverToken; 6]),
        }
        let wire: Wire = decode(strict_value(d)?)?;
        Ok(match wire {
            Wire::Giver(v) => Self::Giver(v),
            Wire::Receiver(v) => Self::Receiver(v),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Intent {
    Wait,
    Move(Cell),
    Pass,
    Pickup,
    Deliver,
}
impl<'de> Deserialize<'de> for Intent {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Wait,
            Move(Cell),
            Pass,
            Pickup,
            Deliver,
        }
        let wire: Wire = decode(strict_value(d)?)?;
        Ok(match wire {
            Wire::Wait => Self::Wait,
            Wire::Move(v) => Self::Move(v),
            Wire::Pass => Self::Pass,
            Wire::Pickup => Self::Pickup,
            Wire::Deliver => Self::Deliver,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ActionOutcome {
    Success,
    Blocked(BlockReason),
}
impl<'de> Deserialize<'de> for ActionOutcome {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Success,
            Blocked(BlockReason),
        }
        let wire: Wire = decode(strict_value(d)?)?;
        Ok(match wire {
            Wire::Success => Self::Success,
            Wire::Blocked(v) => Self::Blocked(v),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum EventKind {
    Action {
        role: Role,
        token: Token,
        intent: Intent,
        result: ActionOutcome,
        paid: u8,
    },
    Terminal(Termination),
}
impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Action {
                role: Role,
                token: Token,
                intent: Intent,
                result: ActionOutcome,
                paid: u8,
            },
            Terminal(Termination),
        }
        let wire: Wire = decode(strict_value(d)?)?;
        Ok(match wire {
            Wire::Action {
                role,
                token,
                intent,
                result,
                paid,
            } => Self::Action {
                role,
                token,
                intent,
                result,
                paid,
            },
            Wire::Terminal(v) => Self::Terminal(v),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum LocalEntry {
    Public {
        clock: Clock,
        termination: Option<Termination>,
    },
    Observed(Observation),
    OwnAction {
        clock: Clock,
        token: Token,
        intent: Intent,
        result: ActionOutcome,
        paid: u8,
    },
    Ownership(OwnershipReceipt),
    Settlement {
        clock: Clock,
        reward: u8,
        net: i16,
    },
}
impl<'de> Deserialize<'de> for LocalEntry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Public {
                clock: Clock,
                termination: Option<Termination>,
            },
            Observed(Observation),
            OwnAction {
                clock: Clock,
                token: Token,
                intent: Intent,
                result: ActionOutcome,
                paid: u8,
            },
            Ownership(OwnershipReceipt),
            Settlement {
                clock: Clock,
                reward: u8,
                net: i16,
            },
        }
        let wire: Wire = decode(strict_value(d)?)?;
        Ok(match wire {
            Wire::Public { clock, termination } => Self::Public { clock, termination },
            Wire::Observed(v) => Self::Observed(v),
            Wire::OwnAction {
                clock,
                token,
                intent,
                result,
                paid,
            } => Self::OwnAction {
                clock,
                token,
                intent,
                result,
                paid,
            },
            Wire::Ownership(v) => Self::Ownership(v),
            Wire::Settlement { clock, reward, net } => Self::Settlement { clock, reward, net },
        })
    }
}
object!(Stratum {
    transfer_enabled: bool,
    transfer_cost: u8,
    giver_share: u8,
    distance: u8
});
impl Stratum {
    pub fn transfer_enabled(&self) -> bool {
        self.transfer_enabled
    }
    pub fn transfer_cost(&self) -> u8 {
        self.transfer_cost
    }
    pub fn giver_share(&self) -> u8 {
        self.giver_share
    }
    pub fn distance(&self) -> u8 {
        self.distance
    }
}
object!(Ids {
    giver: String,
    receiver: String
});
impl Ids {
    pub fn giver(&self) -> &str {
        &self.giver
    }
    pub fn receiver(&self) -> &str {
        &self.receiver
    }
}
object!(Config {
    stratum: Stratum,
    ids: Ids,
    initial_owner: InitialOwner
});
impl Config {
    pub fn stratum(&self) -> &Stratum {
        &self.stratum
    }
    pub fn ids(&self) -> &Ids {
        &self.ids
    }
    pub fn initial_owner(&self) -> InitialOwner {
        self.initial_owner
    }
}
object!(Profile {
    giver: u16,
    receiver: u8
});
impl Profile {
    pub fn giver(&self) -> u16 {
        self.giver
    }
    pub fn receiver(&self) -> u8 {
        self.receiver
    }
}
object!(Clock {
    pub round: u8,
    pub phase: Phase,
    pub role: Option<Role>,
});
object!(PhysicalState {
    pub positions: [Cell; 2],
    pub receiver_cargo: bool,
    pub resource: ResourceLocation,
    pub key_owner: Role,
    pub spent: [u8; 2],
    pub reward: [u8; 2],
});
object!(Event {
    pub clock: Clock,
    pub kind: EventKind,
    pub before: PhysicalState,
    pub after: PhysicalState,
});
object!(VisibleOccupant {
    pub role: Role,
    pub id: String,
    pub cargo: bool,
    pub key: bool,
});
object!(VisibleCell {
    pub cell: Cell,
    pub occupants: Vec<VisibleOccupant>,
    pub loose_token: bool,
});
object!(Observation {
    pub clock: Clock,
    pub cells: Vec<VisibleCell>,
});
object!(OwnershipReceipt {
    pub clock: Clock,
    pub counterpart: String,
    pub before: bool,
    pub after: bool,
});
object!(View {
    pub role: Role,
    pub word: OwnWord,
    pub own_cell: Cell,
    pub own_cargo: bool,
    pub own_key: bool,
    pub credits: u8,
    pub own_reward: u8,
    pub own_net: i16,
    pub clock: Clock,
    pub history: Vec<LocalEntry>,
    pub last_observation: Observation,
});
object!(Outcome {
    pub spent: [u8; 2],
    pub reward: [u8; 2],
    pub net: [i16; 2],
    pub group_net: i16,
    pub delivery_round: Option<u8>,
    pub giver_attempts: [u8; 3],
    pub giver_blocked: [u8; 3],
    pub receiver_attempts: [u8; 4],
    pub receiver_blocked: [u8; 4],
    pub receiver_waits: u8,
    pub transfer_count: u8,
    pub approach_spent: u8,
    pub pass_spent: u8,
    pub unused_credits: [u8; 2],
});
object!(Episode {
    pub version: u16,
    pub config: Config,
    pub profile: Profile,
    pub words: [OwnWord; 2],
    pub events: Vec<Event>,
    pub histories: [Vec<LocalEntry>; 2],
    pub outcome: Outcome,
});
object!(CapturePoint {
    pub clock: Clock,
    pub views: [View; 2],
    pub physical: PhysicalState,
});
impl Stratum {
    pub fn new(
        transfer_enabled: bool,
        transfer_cost: u8,
        giver_share: u8,
        distance: u8,
    ) -> Result<Self, Error> {
        let s = Self {
            transfer_enabled,
            transfer_cost,
            giver_share,
            distance,
        };
        s.validate()?;
        Ok(s)
    }
}
impl Ids {
    pub fn new(giver: String, receiver: String) -> Result<Self, Error> {
        let s = Self { giver, receiver };
        s.validate()?;
        Ok(s)
    }
    pub fn get(&self, role: Role) -> &str {
        match role {
            Role::Giver => &self.giver,
            Role::Receiver => &self.receiver,
        }
    }
}
impl Default for Ids {
    fn default() -> Self {
        Self {
            giver: "Agent-A".into(),
            receiver: "Agent-B".into(),
        }
    }
}
impl Config {
    pub fn new(stratum: Stratum, ids: Ids, initial_owner: InitialOwner) -> Result<Self, Error> {
        let s = Self {
            stratum,
            ids,
            initial_owner,
        };
        s.validate()?;
        Ok(s)
    }
}
impl Copy for Profile {}
impl Profile {
    pub fn new(giver: u16, receiver: u8) -> Result<Self, Error> {
        let s = Self { giver, receiver };
        s.validate()?;
        Ok(s)
    }
    pub fn words(self) -> [OwnWord; 2] {
        let mut g = self.giver;
        let mut r = self.receiver;
        let mut gw = [GiverToken::Wait; 6];
        let mut rw = [ReceiverToken::Wait; 6];
        for i in (0..6).rev() {
            gw[i] = match g % 3 {
                0 => GiverToken::Wait,
                1 => GiverToken::Approach,
                _ => GiverToken::Pass,
            };
            rw[i] = if r.is_multiple_of(2) {
                ReceiverToken::Wait
            } else {
                ReceiverToken::Advance
            };
            g /= 3;
            r /= 2;
        }
        [OwnWord::Giver(gw), OwnWord::Receiver(rw)]
    }
}
impl Copy for Clock {}
impl OwnWord {
    pub fn token(&self, round: u8) -> Result<Token, Error> {
        if !(1..=6).contains(&round) {
            return Err(history_error("word round"));
        }
        let i = usize::from(round - 1);
        Ok(match self {
            Self::Giver(w) => Token::Giver(w[i]),
            Self::Receiver(w) => Token::Receiver(w[i]),
        })
    }
}
