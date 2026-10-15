//! Canonical record encoding. Only one independently bounded record is materialized.
use super::{budget::encoded_len, Error};
use serde::Serialize;
use serde_json::Value;
use std::io::Write;
fn io(error: std::io::Error) -> Error {
    Error::Io(error.to_string())
}
fn value<W: Write>(writer: &mut W, v: &Value) -> Result<(), Error> {
    match v {
        Value::Object(object) => {
            writer.write_all(b"{").map_err(io)?;
            let mut keys: Vec<_> = object.keys().collect();
            keys.sort_unstable();
            for (i, key) in keys.into_iter().enumerate() {
                if i != 0 {
                    writer.write_all(b",").map_err(io)?;
                }
                serde_json::to_writer(&mut *writer, key).map_err(|e| Error::Io(e.to_string()))?;
                writer.write_all(b":").map_err(io)?;
                value(writer, &object[key])?;
            }
            writer.write_all(b"}").map_err(io)
        }
        Value::Array(array) => {
            writer.write_all(b"[").map_err(io)?;
            for (i, item) in array.iter().enumerate() {
                if i != 0 {
                    writer.write_all(b",").map_err(io)?;
                }
                value(writer, item)?;
            }
            writer.write_all(b"]").map_err(io)
        }
        Value::Number(n) if !n.is_i64() && !n.is_u64() => {
            Err(Error::InvalidReport("integer required".into()))
        }
        _ => serde_json::to_writer(writer, v).map_err(|e| Error::Io(e.to_string())),
    }
}
pub(crate) fn write_record<W: Write, T: Serialize>(
    writer: &mut W,
    record: &T,
    limit: u64,
) -> Result<(), Error> {
    encoded_len(record, limit)?;
    let record = serde_json::to_value(record).map_err(|e| Error::InvalidReport(e.to_string()))?;
    struct Chunks<'a, W>(&'a mut W);
    impl<W: Write> Write for Chunks<'_, W> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            for chunk in bytes.chunks(65536) {
                self.0.write_all(chunk)?;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.flush()
        }
    }
    value(&mut Chunks(writer), &record)
}
pub(crate) fn write_record_flush<W: Write, T: Serialize>(
    writer: &mut W,
    record: &T,
    limit: u64,
) -> Result<(), Error> {
    write_record(writer, record, limit)?;
    writer.flush().map_err(io)
}
