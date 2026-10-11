use super::{Episode, Error};
use serde::Serialize;
use serde_json::Value;

/// Compact, recursively sorted native JSON. Arrays and UTF-8 are preserved.
pub(crate) fn canonical_bytes(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    fn sorted(value: Value) -> Result<Value, Error> {
        Ok(match value {
            Value::Number(n) if !n.is_i64() && !n.is_u64() => {
                return Err(Error::InvalidReport(
                    "canonical native JSON requires integer numbers".into(),
                ))
            }
            Value::Array(a) => Value::Array(a.into_iter().map(sorted).collect::<Result<_, _>>()?),
            Value::Object(o) => {
                let ordered: std::collections::BTreeMap<_, _> = o.into_iter().collect();
                Value::Object(
                    ordered
                        .into_iter()
                        .map(|(key, value)| Ok((key, sorted(value)?)))
                        .collect::<Result<_, Error>>()?,
                )
            }
            other => other,
        })
    }
    let value = serde_json::to_value(value)
        .map_err(|e| Error::InvalidReport(format!("canonical serialization: {e}")))?;
    serde_json::to_vec(&sorted(value)?)
        .map_err(|e| Error::InvalidReport(format!("canonical bytes: {e}")))
}

pub fn trace_hash(episode: &Episode) -> Result<String, Error> {
    episode.validate()?;
    trace_identity(&canonical_bytes(episode)?)
}

// Only freshly generated native kernel output may bypass public replay validation.
pub(super) fn trusted_trace_hash(episode: &Episode) -> Result<String, Error> {
    trace_identity(&native_trace_bytes(episode)?)
}
fn trace_identity(body: &[u8]) -> Result<String, Error> {
    crate::surface_discovery::compiled_bytes_identity(&[("gate-assistance-trace-v1", body)])
        .map_err(|e| Error::InvalidReport(format!("trace identity: {e}")))
}

/// Byte length of compact native JSON without constructing a Value or buffer.
/// All caller schemas contain integers only; object ordering cannot affect length.
pub(crate) fn json_len(value: &impl Serialize) -> Result<u64, Error> {
    struct Counter(u64);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| std::io::Error::other("JSON length overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value)
        .map_err(|e| Error::InvalidReport(format!("JSON length: {e}")))?;
    Ok(counter.0)
}

// Serialize the closed native Episode schema directly in sorted-key order.
// Borrowing fields avoids a second whole-Episode Value tree while its canonical
// buffer is live. The independent witnesses/census authenticate exact bytes.
struct Sorted<'a, T: ?Sized>(&'a T);
impl<T: SerializeSorted> Serialize for Sorted<'_, T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize_sorted(s)
    }
}
trait SerializeSorted {
    fn serialize_sorted<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>;
}
impl<T: SerializeSorted> SerializeSorted for Vec<T> {
    fn serialize_sorted<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(self.len()))?;
        for v in self {
            seq.serialize_element(&Sorted(v))?;
        }
        seq.end()
    }
}
impl<T: SerializeSorted, const N: usize> SerializeSorted for [T; N] {
    fn serialize_sorted<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(N))?;
        for v in self {
            seq.serialize_element(&Sorted(v))?;
        }
        seq.end()
    }
}
macro_rules! sorted_object {
    ($ty:ty,$v:ident,{$($key:literal=>$value:expr),*$(,)?})=>{
        impl SerializeSorted for $ty {
            fn serialize_sorted<S:serde::Serializer>(&self,s:S)->Result<S::Ok,S::Error>{
                use serde::ser::SerializeMap;
                let $v=self;let mut map=s.serialize_map(None)?;
                $(map.serialize_entry($key,&$value)?;)*
                map.end()
            }
        }
    };
}
sorted_object!(super::Episode,v,{"config"=>Sorted(&v.config),"events"=>Sorted(&v.events),"histories"=>Sorted(&v.histories),"outcome"=>Sorted(&v.outcome),"profile"=>Sorted(&v.profile),"version"=>v.version});
sorted_object!(super::Config,v,{"ids"=>Sorted(&v.ids),"stratum"=>Sorted(&v.stratum)});
sorted_object!(super::Ids,v,{"holder"=>&v.holder,"retriever"=>&v.retriever});
sorted_object!(super::Stratum,v,{"distance"=>v.distance(),"holder_share"=>v.holder_share(),"holding_cost"=>v.holding_cost()});
sorted_object!(super::Profile,v,{"holder"=>v.holder,"retriever"=>v.retriever});
sorted_object!(super::Clock,v,{"phase"=>v.phase,"role"=>v.role,"round"=>v.round});
sorted_object!(super::PhysicalState,v,{"gate_open"=>v.gate_open,"positions"=>v.positions,"resource"=>v.resource,"retriever_cargo"=>v.retriever_cargo,"reward"=>v.reward,"spent"=>v.spent});
sorted_object!(super::Outcome,v,{"delivery_round"=>v.delivery_round,"group_net"=>v.group_net,"holder_attempts"=>v.holder_attempts,"holder_blocked"=>v.holder_blocked,"net"=>v.net,"retriever_attempts"=>v.retriever_attempts,"retriever_blocked"=>v.retriever_blocked,"retriever_waits"=>v.retriever_waits,"reward"=>v.reward,"spent"=>v.spent});
sorted_object!(super::Event,v,{"after"=>Sorted(&v.after),"before"=>Sorted(&v.before),"clock"=>Sorted(&v.clock),"kind"=>Sorted(&v.kind)});
sorted_object!(super::Observation,v,{"cells"=>Sorted(&v.cells),"clock"=>Sorted(&v.clock),"gate"=>v.gate});
sorted_object!(super::VisibleCell,v,{"cell"=>v.cell,"loose_token"=>v.loose_token,"occupants"=>&v.occupants});
impl SerializeSorted for super::EventKind {
    fn serialize_sorted<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Self::Reset => s.serialize_str("Reset"),
            Self::Terminal(value) => {
                let mut m = s.serialize_map(Some(1))?;
                m.serialize_entry("Terminal", value)?;
                m.end()
            }
            Self::Action {
                role,
                token,
                intent,
                result,
                paid,
            } => {
                #[derive(Serialize)]
                struct Action<'a> {
                    intent: &'a super::Primitive,
                    paid: &'a u8,
                    result: &'a super::ActionOutcome,
                    role: &'a super::Role,
                    token: &'a super::Token,
                }
                let mut m = s.serialize_map(Some(1))?;
                m.serialize_entry(
                    "Action",
                    &Action {
                        intent,
                        paid,
                        result,
                        role,
                        token,
                    },
                )?;
                m.end()
            }
        }
    }
}
impl SerializeSorted for super::LocalEntry {
    fn serialize_sorted<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut m = s.serialize_map(Some(1))?;
        match self {
            Self::Observed(o) => m.serialize_entry("Observed", &Sorted(o))?,
            Self::Public { clock, termination } => {
                #[derive(Serialize)]
                struct Public<'a> {
                    clock: Sorted<'a, super::Clock>,
                    termination: &'a Option<super::Termination>,
                }
                m.serialize_entry(
                    "Public",
                    &Public {
                        clock: Sorted(clock),
                        termination,
                    },
                )?;
            }
            Self::OwnAction {
                clock,
                token,
                intent,
                result,
                paid,
            } => {
                #[derive(Serialize)]
                struct Action<'a> {
                    clock: Sorted<'a, super::Clock>,
                    intent: &'a super::Primitive,
                    paid: &'a u8,
                    result: &'a super::ActionOutcome,
                    token: &'a super::Token,
                }
                m.serialize_entry(
                    "OwnAction",
                    &Action {
                        clock: Sorted(clock),
                        intent,
                        paid,
                        result,
                        token,
                    },
                )?;
            }
            Self::Settlement { clock, reward } => {
                #[derive(Serialize)]
                struct Settlement<'a> {
                    clock: Sorted<'a, super::Clock>,
                    reward: &'a u8,
                }
                m.serialize_entry(
                    "Settlement",
                    &Settlement {
                        clock: Sorted(clock),
                        reward,
                    },
                )?;
            }
        }
        m.end()
    }
}
pub(super) fn native_trace_bytes(episode: &Episode) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(&Sorted(episode))
        .map_err(|e| Error::InvalidReport(format!("native canonical bytes: {e}")))
}
