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
        match self {
            Self::InvalidInput(s) => write!(f, "invalid gate input: {s}"),
            Self::InvalidHistory(s) => write!(f, "invalid gate history: {s}"),
            Self::InvalidReport(s) => write!(f, "invalid gate report: {s}"),
            Self::ResourceLimit { resource, limit } => {
                write!(f, "gate resource {resource} exceeds {limit}")
            }
            Self::ArithmeticOverflow => f.write_str("gate arithmetic overflow"),
            Self::Io(s) => write!(f, "gate I/O: {s}"),
        }
    }
}
impl std::error::Error for Error {}
pub(super) fn invalid(s: impl Into<String>) -> Error {
    Error::InvalidInput(s.into())
}
pub(super) fn history_error(s: impl Into<String>) -> Error {
    Error::InvalidHistory(s.into())
}

// deserialize_with makes nullable fields required, rather than serde's implicit None.
pub(super) fn required_option<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
macro_rules! unit_enum {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
        pub enum $name { $($variant),* }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D:Deserializer<'de>>(d:D)->Result<Self,D::Error> {
                match String::deserialize(d)?.as_str() {
                    $(stringify!($variant)=>Ok(Self::$variant),)*
                    other=>Err(serde::de::Error::unknown_variant(other,&[$(stringify!($variant)),*])),
                }
            }
        }
    };
}
unit_enum!(Role { Holder, Retriever });
impl Role {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Holder => 0,
            Self::Retriever => 1,
        }
    }
}
unit_enum!(Cell { P, O, C });
unit_enum!(HolderToken { Wait, Travel, Hold });
unit_enum!(RetrieverToken { Wait, Advance });
unit_enum!(Termination {
    Delivered,
    Deadline
});
unit_enum!(Phase {
    Initial,
    Reset,
    BeforeAction,
    AfterAction,
    Finished
});
unit_enum!(ResourceLocation {
    Cache,
    Carried,
    Delivered
});
unit_enum!(BlockReason {
    AlreadyAtPlate,
    AwayFromPlate,
    GateClosed,
    NoToken,
    WrongPosition,
    EmptyHands,
    FullHands
});

// Default serde also accepts {"Unit":null}; protocol unit variants are strings only.
macro_rules! mixed_enum_deserialize {
    ($name:ident, [$($unit:ident),*], $wire:item, $convert:expr) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D:Deserializer<'de>>(d:D)->Result<Self,D::Error> {
                struct EnumVisitor;
                impl<'de> serde::de::Visitor<'de> for EnumVisitor {
                    type Value=$name;
                    fn expecting(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {f.write_str("a unit variant string or one-key payload variant object")}
                    fn visit_str<E:serde::de::Error>(self,value:&str)->Result<Self::Value,E> {
                        match value {$(stringify!($unit)=>Ok($name::$unit),)*other=>Err(E::unknown_variant(other,&[$(stringify!($unit)),*]))}
                    }
                    fn visit_map<M:serde::de::MapAccess<'de>>(self,map:M)->Result<Self::Value,M::Error> {
                        $wire
                        let wire=Wire::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                        Ok(($convert)(wire))
                    }
                }
                d.deserialize_any(EnumVisitor)
            }
        }
    };
}

// Native field objects never use serde's positional struct representation.
// Delegating the original MapAccess preserves duplicate-field detection.
pub(super) fn deserialize_object<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ObjectVisitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for ObjectVisitor<T> {
        type Value = T;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a native field object")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }
    d.deserialize_map(ObjectVisitor(std::marker::PhantomData))
}

// Each wire object retains duplicate detection before any Value/map conversion.
macro_rules! validated_struct {
    ($name:ident { $($(#[$attr:meta])* $vis:vis $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
        pub struct $name { $($(#[$attr])* $vis $field: $ty),* }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Wire { $($(#[$attr])* $field: $ty),* }
                let wire: Wire = deserialize_object(d)?;
                let value = Self { $($field: wire.$field),* };
                value.validate().map_err(serde::de::Error::custom)?;
                Ok(value)
            }
        }
    };
}
validated_struct!(Ids { pub holder: String, pub retriever: String });
impl Default for Ids {
    fn default() -> Self {
        Self {
            holder: "Agent-A".into(),
            retriever: "Agent-B".into(),
        }
    }
}
impl Ids {
    pub fn validate(&self) -> Result<(), Error> {
        if self.holder.is_empty() || self.retriever.is_empty() || self.holder == self.retriever {
            return Err(invalid("IDs must be distinct nonempty Unicode strings"));
        }
        Ok(())
    }
}
validated_struct!(Stratum {
    holding_cost: u8,
    holder_share: u8,
    distance: u8
});
impl Stratum {
    pub fn new(k: u8, b: u8, d: u8) -> Result<Self, Error> {
        let s = Self {
            holding_cost: k,
            holder_share: b,
            distance: d,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if ![1, 2, 4].contains(&self.holding_cost)
            || ![0, 3, 6].contains(&self.holder_share)
            || self.distance > 1
        {
            return Err(invalid(
                "stratum requires k in 1/2/4, b in 0/3/6 and d in 0/1",
            ));
        }
        Ok(())
    }
    pub fn holding_cost(&self) -> u8 {
        self.holding_cost
    }
    pub fn holder_share(&self) -> u8 {
        self.holder_share
    }
    pub fn distance(&self) -> u8 {
        self.distance
    }
}
validated_struct!(Config { pub stratum: Stratum, pub ids: Ids });
impl Config {
    pub fn validate(&self) -> Result<(), Error> {
        self.stratum.validate()?;
        self.ids.validate()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Profile {
    pub holder: u16,
    pub retriever: u8,
}
impl Profile {
    pub fn new(h: u16, r: u8) -> Result<Self, Error> {
        let p = Self {
            holder: h,
            retriever: r,
        };
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.holder >= 729 || self.retriever >= 64 {
            return Err(invalid("word IDs require Holder <729 and Retriever <64"));
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for Profile {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            holder: u16,
            retriever: u8,
        }
        let w: Wire = deserialize_object(d)?;
        Self::new(w.holder, w.retriever).map_err(serde::de::Error::custom)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolderWord(pub(super) [HolderToken; 6]);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrieverWord(pub(super) [RetrieverToken; 6]);
impl HolderWord {
    pub fn from_id(id: u16) -> Result<Self, Error> {
        Profile::new(id, 0)?;
        let mut n = id;
        let mut tokens = [HolderToken::Wait; 6];
        for token in tokens.iter_mut().rev() {
            *token = match n % 3 {
                0 => HolderToken::Wait,
                1 => HolderToken::Travel,
                _ => HolderToken::Hold,
            };
            n /= 3;
        }
        Ok(Self(tokens))
    }
    pub fn id(&self) -> u16 {
        self.0.iter().fold(0, |id, t| {
            id * 3
                + match t {
                    HolderToken::Wait => 0,
                    HolderToken::Travel => 1,
                    HolderToken::Hold => 2,
                }
        })
    }
}
impl RetrieverWord {
    pub fn from_id(id: u8) -> Result<Self, Error> {
        Profile::new(0, id)?;
        let mut n = id;
        let mut tokens = [RetrieverToken::Wait; 6];
        for token in tokens.iter_mut().rev() {
            *token = if n.is_multiple_of(2) {
                RetrieverToken::Wait
            } else {
                RetrieverToken::Advance
            };
            n /= 2;
        }
        Ok(Self(tokens))
    }
    pub fn id(&self) -> u8 {
        self.0
            .iter()
            .fold(0, |id, t| id * 2 + u8::from(*t == RetrieverToken::Advance))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Primitive {
    Wait,
    Move(Cell),
    OperatePlate,
    Pickup,
    Deliver,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Token {
    Holder(HolderToken),
    Retriever(RetrieverToken),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ActionOutcome {
    Success,
    Blocked(BlockReason),
}
mixed_enum_deserialize!(
    Primitive,
    [Wait, OperatePlate, Pickup, Deliver],
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    enum Wire {
        Move(Cell),
    },
    |wire| match wire {
        Wire::Move(cell) => Primitive::Move(cell),
    }
);
mixed_enum_deserialize!(
    ActionOutcome,
    [Success],
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    enum Wire {
        Blocked(BlockReason),
    },
    |wire| match wire {
        Wire::Blocked(reason) => ActionOutcome::Blocked(reason),
    }
);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Clock {
    pub round: u8,
    pub phase: Phase,
    pub role: Option<Role>,
}
impl Clock {
    pub fn validate(&self) -> Result<(), Error> {
        let valid = match self.phase {
            Phase::Initial => self.round == 0 && self.role.is_none(),
            Phase::Reset | Phase::Finished => (1..=6).contains(&self.round) && self.role.is_none(),
            Phase::BeforeAction | Phase::AfterAction => {
                (1..=6).contains(&self.round) && self.role.is_some()
            }
        };
        if !valid {
            return Err(history_error("invalid public clock"));
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for Clock {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            round: u8,
            phase: Phase,
            #[serde(deserialize_with = "required_option")]
            role: Option<Role>,
        }
        let w: Wire = deserialize_object(d)?;
        let c = Self {
            round: w.round,
            phase: w.phase,
            role: w.role,
        };
        c.validate().map_err(serde::de::Error::custom)?;
        Ok(c)
    }
}
validated_struct!(PhysicalState { pub positions:[Cell;2], pub retriever_cargo:bool, pub resource:ResourceLocation, pub gate_open:bool, pub spent:[u8;2], pub reward:[u8;2] });
impl PhysicalState {
    pub fn validate(&self) -> Result<(), Error> {
        if self.positions[0] == Cell::C
            || self.positions[1] == Cell::P
            || self.spent.iter().any(|v| *v > 32)
        {
            return Err(history_error(
                "positions or credits violate role capabilities",
            ));
        }
        if self.retriever_cargo != (self.resource == ResourceLocation::Carried) {
            return Err(history_error("cargo/resource invariant"));
        }
        if self.resource == ResourceLocation::Delivered && self.positions[1] != Cell::O {
            return Err(history_error("delivered resource must be at O"));
        }
        if self.reward != [0, 0]
            && !(self.resource == ResourceLocation::Delivered
                && [0, 3, 6].contains(&self.reward[0])
                && u16::from(self.reward[0]) + u16::from(self.reward[1]) == 12)
        {
            return Err(history_error("invalid settlement"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum EventKind {
    Reset,
    Action {
        role: Role,
        token: Token,
        intent: Primitive,
        result: ActionOutcome,
        paid: u8,
    },
    Terminal(Termination),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionPayload {
    role: Role,
    token: Token,
    intent: Primitive,
    result: ActionOutcome,
    paid: u8,
}
mixed_enum_deserialize!(
    EventKind,
    [Reset],
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    enum Wire {
        Action(#[serde(deserialize_with = "deserialize_object")] ActionPayload),
        Terminal(Termination),
    },
    |wire| match wire {
        Wire::Action(action) => EventKind::Action {
            role: action.role,
            token: action.token,
            intent: action.intent,
            result: action.result,
            paid: action.paid,
        },
        Wire::Terminal(t) => EventKind::Terminal(t),
    }
);
validated_struct!(Event { pub clock:Clock, pub kind:EventKind, pub before:PhysicalState, pub after:PhysicalState });
impl Event {
    pub fn validate(&self) -> Result<(), Error> {
        self.clock.validate()?;
        self.before.validate()?;
        self.after.validate()?;
        match &self.kind {
            EventKind::Reset if self.clock.phase == Phase::Reset => {
                let mut expected = self.before.clone();
                expected.gate_open = false;
                if self.before.resource == ResourceLocation::Delivered
                    || self.before.reward != [0, 0]
                    || expected != self.after
                {
                    return Err(history_error("invalid free reset transition"));
                }
                Ok(())
            }
            EventKind::Terminal(termination) if self.clock.phase == Phase::Finished => {
                let mut expected = self.before.clone();
                expected.gate_open = false;
                let valid = match termination {
                    Termination::Delivered => {
                        expected.reward = self.after.reward;
                        self.clock.round >= 4
                            && self.before.resource == ResourceLocation::Delivered
                            && u16::from(self.after.reward[0]) + u16::from(self.after.reward[1])
                                == 12
                    }
                    Termination::Deadline => {
                        self.clock.round == 6
                            && self.before.resource != ResourceLocation::Delivered
                            && self.after.reward == [0, 0]
                    }
                };
                if !valid || self.before.reward != [0, 0] || expected != self.after {
                    return Err(history_error("invalid terminal transition"));
                }
                Ok(())
            }
            EventKind::Action {
                role,
                token,
                intent,
                result,
                paid,
            } if self.clock.phase == Phase::AfterAction && self.clock.role == Some(*role) => {
                validate_feedback(*role, *token, *intent, *result, *paid)?;
                if self.before.resource == ResourceLocation::Delivered
                    || self.before.reward != [0, 0]
                {
                    return Err(history_error("action after delivery or settlement"));
                }
                let (expected, expected_result) =
                    super::kernel::transaction(&self.before, *role, *intent, *paid)?;
                if expected != self.after || expected_result != *result {
                    return Err(history_error(
                        "action snapshots/result disagree with primitive",
                    ));
                }
                Ok(())
            }
            _ => Err(history_error("event kind/clock mismatch")),
        }
    }
}
validated_struct!(VisibleCell { pub cell:Cell, pub occupants:Vec<(Role,bool)>, pub loose_token:bool });
impl VisibleCell {
    pub fn validate(&self) -> Result<(), Error> {
        let mut previous = None;
        for (role, cargo) in &self.occupants {
            if previous.is_some_and(|p| p >= *role)
                || (*role == Role::Holder && *cargo)
                || (*role == Role::Holder && self.cell == Cell::C)
                || (*role == Role::Retriever && self.cell == Cell::P)
            {
                return Err(history_error("invalid visible occupants"));
            }
            previous = Some(*role);
        }
        if self.loose_token && self.cell != Cell::C {
            return Err(history_error("loose token outside C"));
        }
        Ok(())
    }
}
validated_struct!(Observation { pub clock:Clock, pub cells:Vec<VisibleCell>, #[serde(deserialize_with="required_option")] pub gate:Option<bool> });
impl Observation {
    pub fn validate(&self) -> Result<(), Error> {
        self.clock.validate()?;
        if !matches!(
            self.clock.phase,
            Phase::Initial | Phase::BeforeAction | Phase::AfterAction
        ) {
            return Err(history_error("observation at non-observing phase"));
        }
        let cells: Vec<_> = self.cells.iter().map(|c| c.cell).collect();
        if !matches!(
            cells.as_slice(),
            [Cell::P, Cell::O] | [Cell::P, Cell::O, Cell::C] | [Cell::O, Cell::C]
        ) || self.gate.is_none() != (cells == [Cell::P, Cell::O])
        {
            return Err(history_error("invalid observation visibility"));
        }
        let mut seen = [false; 2];
        let mut cargo = false;
        let mut loose = false;
        for cell in &self.cells {
            cell.validate()?;
            loose |= cell.loose_token;
            for (role, has_cargo) in &cell.occupants {
                if seen[role.index()] {
                    return Err(history_error("duplicate role across visible cells"));
                }
                seen[role.index()] = true;
                cargo |= *has_cargo;
            }
        }
        if cargo && loose {
            return Err(history_error("carried resource also visible as loose"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum LocalEntry {
    Public {
        clock: Clock,
        #[serde(deserialize_with = "required_option")]
        termination: Option<Termination>,
    },
    Observed(Observation),
    OwnAction {
        clock: Clock,
        token: Token,
        intent: Primitive,
        result: ActionOutcome,
        paid: u8,
    },
    Settlement {
        clock: Clock,
        reward: u8,
    },
}
impl LocalEntry {
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Public { clock, termination } => {
                clock.validate()?;
                if (clock.phase == Phase::Finished) != termination.is_some() {
                    return Err(history_error("public termination at wrong phase"));
                }
                Ok(())
            }
            Self::Observed(o) => o.validate(),
            Self::OwnAction {
                clock,
                token,
                intent,
                result,
                paid,
            } => {
                clock.validate()?;
                if clock.phase != Phase::AfterAction {
                    return Err(history_error("own feedback outside AfterAction"));
                }
                validate_feedback(
                    clock
                        .role
                        .ok_or_else(|| history_error("own feedback lacks role"))?,
                    *token,
                    *intent,
                    *result,
                    *paid,
                )
            }
            Self::Settlement { clock, reward } => {
                clock.validate()?;
                if clock.phase != Phase::Finished || ![0, 3, 6, 9, 12].contains(reward) {
                    return Err(history_error("invalid settlement clock or share"));
                }
                Ok(())
            }
        }
    }
}
impl<'de> Deserialize<'de> for LocalEntry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PublicPayload {
            clock: Clock,
            #[serde(deserialize_with = "required_option")]
            termination: Option<Termination>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct OwnActionPayload {
            clock: Clock,
            token: Token,
            intent: Primitive,
            result: ActionOutcome,
            paid: u8,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct SettlementPayload {
            clock: Clock,
            reward: u8,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Wire {
            Public(#[serde(deserialize_with = "deserialize_object")] PublicPayload),
            Observed(Observation),
            OwnAction(#[serde(deserialize_with = "deserialize_object")] OwnActionPayload),
            Settlement(#[serde(deserialize_with = "deserialize_object")] SettlementPayload),
        }
        let value = match Wire::deserialize(d)? {
            Wire::Public(public) => Self::Public {
                clock: public.clock,
                termination: public.termination,
            },
            Wire::Observed(o) => Self::Observed(o),
            Wire::OwnAction(action) => Self::OwnAction {
                clock: action.clock,
                token: action.token,
                intent: action.intent,
                result: action.result,
                paid: action.paid,
            },
            Wire::Settlement(settlement) => Self::Settlement {
                clock: settlement.clock,
                reward: settlement.reward,
            },
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OwnWord {
    Holder(HolderWord),
    Retriever(RetrieverWord),
}
impl OwnWord {
    pub(super) fn token(&self, round: u8) -> Result<Token, Error> {
        let index = usize::from(
            round
                .checked_sub(1)
                .ok_or_else(|| history_error("no token at round zero"))?,
        );
        match self {
            Self::Holder(w) => w.0.get(index).copied().map(Token::Holder),
            Self::Retriever(w) => w.0.get(index).copied().map(Token::Retriever),
        }
        .ok_or_else(|| history_error("word round outside six slots"))
    }
}
validated_struct!(View { pub role:Role,pub word:OwnWord,pub own_cell:Cell,pub own_cargo:bool,pub credits:u8,pub own_reward:u8,pub clock:Clock,pub history:Vec<LocalEntry>,pub last_observation:Observation });
impl View {
    pub fn validate(&self) -> Result<(), Error> {
        super::observation::validate_view(self)
    }
}
validated_struct!(Outcome { pub spent:[u8;2],pub reward:[u8;2],pub net:[i16;2],pub group_net:i16,#[serde(deserialize_with="required_option")] pub delivery_round:Option<u8>,pub holder_attempts:[u8;3],pub holder_blocked:[u8;3],pub retriever_attempts:[u8;4],pub retriever_blocked:[u8;4],pub retriever_waits:u8 });
impl Outcome {
    pub fn validate(&self) -> Result<(), Error> {
        let ha: u16 = self.holder_attempts.iter().map(|n| u16::from(*n)).sum();
        let ra: u16 = self.retriever_attempts.iter().map(|n| u16::from(*n)).sum();
        if ha > 6
            || ra + u16::from(self.retriever_waits) > 6
            || ha != ra + u16::from(self.retriever_waits)
            || self.spent.iter().any(|n| *n > 32)
            || ra != u16::from(self.spent[1])
            || self.holder_blocked[0] != 0
            || self
                .holder_attempts
                .iter()
                .zip(self.holder_blocked)
                .any(|(a, b)| b > *a)
            || self
                .retriever_attempts
                .iter()
                .zip(self.retriever_blocked)
                .any(|(a, b)| b > *a)
        {
            return Err(history_error("invalid action counts or spending"));
        }
        if ![1, 2, 4].iter().any(|k| {
            u16::from(self.spent[0])
                == u16::from(self.holder_attempts[1]) + k * u16::from(self.holder_attempts[2])
        }) {
            return Err(history_error("Holder spending does not match charges"));
        }
        let net = [
            i16::from(self.reward[0]) - i16::from(self.spent[0]),
            i16::from(self.reward[1]) - i16::from(self.spent[1]),
        ];
        if self.net != net || i32::from(self.group_net) != i32::from(net[0]) + i32::from(net[1]) {
            return Err(history_error("net accounting mismatch"));
        }
        match self.delivery_round {
            Some(q)
                if (4..=6).contains(&q)
                    && ha == u16::from(q)
                    && [0, 3, 6].contains(&self.reward[0])
                    && u16::from(self.reward[0]) + u16::from(self.reward[1]) == 12 => {}
            None if ha == 6 && self.reward == [0, 0] => {}
            _ => return Err(history_error("invalid delivery/settlement")),
        }
        Ok(())
    }
}
validated_struct!(Episode { pub version:u16,pub config:Config,pub profile:Profile,pub events:Vec<Event>,pub histories:[Vec<LocalEntry>;2],pub outcome:Outcome });
impl Episode {
    pub fn validate(&self) -> Result<(), Error> {
        self.config.validate()?;
        self.profile.validate()?;
        if self.version != 1 {
            return Err(history_error("unsupported Episode version"));
        }
        // Reconstruct with the native kernel; cached outcomes cannot authenticate histories.
        let reconstructed = super::run_episode(&self.config, self.profile)?;
        if &reconstructed != self {
            return Err(history_error(
                "Episode differs from registered reconstruction",
            ));
        }
        Ok(())
    }
}
validated_struct!(CapturePoint { pub clock:Clock,pub views:[View;2],pub physical:PhysicalState });
impl CapturePoint {
    pub fn validate(&self) -> Result<(), Error> {
        self.clock.validate()?;
        self.physical.validate()?;
        for (i, view) in self.views.iter().enumerate() {
            view.validate()?;
            if view.role.index() != i
                || view.clock != self.clock
                || view.own_cell != self.physical.positions[i]
                || view.own_cargo != (i == 1 && self.physical.retriever_cargo)
                || view.credits != 32 - self.physical.spent[i]
                || view.own_reward != self.physical.reward[i]
            {
                return Err(history_error("capture/own state mismatch"));
            }
        }
        Ok(())
    }
}
pub(super) fn validate_feedback(
    role: Role,
    token: Token,
    intent: Primitive,
    result: ActionOutcome,
    paid: u8,
) -> Result<(), Error> {
    let valid = match (role, token, intent) {
        (Role::Holder, Token::Holder(HolderToken::Wait), Primitive::Wait)
        | (Role::Retriever, Token::Retriever(RetrieverToken::Wait), Primitive::Wait) => {
            paid == 0 && result == ActionOutcome::Success
        }
        (Role::Holder, Token::Holder(HolderToken::Travel), Primitive::Move(Cell::P)) => {
            paid == 1
                && matches!(
                    result,
                    ActionOutcome::Success | ActionOutcome::Blocked(BlockReason::AlreadyAtPlate)
                )
        }
        (Role::Holder, Token::Holder(HolderToken::Hold), Primitive::OperatePlate) => {
            [1, 2, 4].contains(&paid)
                && matches!(
                    result,
                    ActionOutcome::Success | ActionOutcome::Blocked(BlockReason::AwayFromPlate)
                )
        }
        (
            Role::Retriever,
            Token::Retriever(RetrieverToken::Advance),
            Primitive::Move(Cell::O | Cell::C),
        ) => {
            paid == 1
                && matches!(
                    result,
                    ActionOutcome::Success
                        | ActionOutcome::Blocked(
                            BlockReason::WrongPosition
                                | BlockReason::EmptyHands
                                | BlockReason::FullHands
                                | BlockReason::GateClosed
                        )
                )
        }
        (Role::Retriever, Token::Retriever(RetrieverToken::Advance), Primitive::Pickup) => {
            paid == 1
                && matches!(
                    result,
                    ActionOutcome::Success
                        | ActionOutcome::Blocked(
                            BlockReason::WrongPosition
                                | BlockReason::FullHands
                                | BlockReason::NoToken
                        )
                )
        }
        (Role::Retriever, Token::Retriever(RetrieverToken::Advance), Primitive::Deliver) => {
            paid == 1
                && matches!(
                    result,
                    ActionOutcome::Success
                        | ActionOutcome::Blocked(
                            BlockReason::WrongPosition | BlockReason::EmptyHands
                        )
                )
        }
        _ => false,
    };
    if !valid {
        return Err(history_error("role/token/intent/result/charge mismatch"));
    }
    Ok(())
}

// Object-only wire structs for solver/metrics envelopes; semantic game validation
// is performed by the solver and report integrity layer, not by generic maps.
macro_rules! wire_struct {
    ($name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
        pub struct $name { $($(#[$attr])* pub $field: $ty),* }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d:D)->Result<Self,D::Error> {
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Wire { $($(#[$attr])* $field: $ty),* }
                let w:Wire=super::types::deserialize_object(d)?;
                Ok(Self { $($field:w.$field),* })
            }
        }
    };
}
pub(super) use unit_enum;
pub(super) use wire_struct;
