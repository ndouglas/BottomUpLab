//! Private local control channel; collection requires a completed bounded handshake.
use super::{
    gate::{CampaignBinding, Limits},
    io, wire,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::{Duration, Instant},
};
const MESSAGE_LIMIT: usize = 16384;
#[derive(Serialize)]
struct Hello<'a> {
    schema: &'static str,
    binding: &'a CampaignBinding,
    child_pid: u32,
    limits: Limits,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Control {
    schema: String,
    action: String,
}
pub(super) struct Monitor {
    channel: UnixStream,
    input: Vec<u8>,
    start: Instant,
    heartbeat: Instant,
    last_progress: Instant,
}
impl Monitor {
    pub(super) fn connect(path: &Path, binding: &CampaignBinding) -> Result<Self, String> {
        let canonical_tmp = Path::new("/tmp")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if path.as_os_str().len() > 99 || !path.starts_with(canonical_tmp) {
            return Err("monitor socket must be private canonical /tmp path <=99 bytes".into());
        }
        let mut channel = UnixStream::connect(path).map_err(|e| format!("monitor connect: {e}"))?;
        channel
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        channel
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        let hello = Hello {
            schema: "war1-study-monitor-v1",
            binding,
            child_pid: std::process::id(),
            limits: Limits::required(),
        };
        channel
            .write_all(&io::encode(&hello, MESSAGE_LIMIT, true)?)
            .map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        loop {
            let mut byte = [0];
            channel
                .read_exact(&mut byte)
                .map_err(|e| format!("monitor handshake: {e}"))?;
            if byte[0] == b'\n' {
                break;
            }
            if bytes.len() == MESSAGE_LIMIT {
                return Err("monitor handshake exceeds bound".into());
            }
            bytes.push(byte[0]);
        }
        let control: Control = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if control.schema != "war1-study-monitor-v1" || control.action != "ready" {
            return Err("monitor ready handshake required".into());
        }
        channel.set_nonblocking(true).map_err(|e| e.to_string())?;
        let now = Instant::now();
        Ok(Self {
            channel,
            input: Vec::new(),
            start: now,
            heartbeat: now,
            last_progress: now,
        })
    }
    pub(super) fn check(&mut self) -> Result<(), String> {
        if self.start.elapsed() > Duration::from_secs(43200) {
            return Err("native wall limit".into());
        }
        let mut bytes = [0; 1024];
        let mut drained = 0;
        loop {
            match self.channel.read(&mut bytes) {
                Ok(0) => return Err("monitor disconnected".into()),
                Ok(n) => {
                    drained += n;
                    if drained > MESSAGE_LIMIT || self.input.len() + n > MESSAGE_LIMIT {
                        return Err("monitor message bound".into());
                    }
                    self.input.extend_from_slice(&bytes[..n]);
                    while let Some(end) = self.input.iter().position(|b| *b == b'\n') {
                        let control: Control = serde_json::from_slice(&self.input[..end])
                            .map_err(|e| e.to_string())?;
                        self.input.drain(..=end);
                        if control.schema != "war1-study-monitor-v1"
                            || control.action != "heartbeat"
                        {
                            return Err("monitor stop or invalid control".into());
                        }
                        self.heartbeat = Instant::now();
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(format!("monitor read: {e}")),
            }
        }
        if self.heartbeat.elapsed() > Duration::from_secs(2) {
            return Err("monitor heartbeat stale".into());
        }
        Ok(())
    }
    pub(super) fn progress(
        &mut self,
        key: &wire::Key,
        semantic: u64,
        observed: u64,
        durable: u64,
    ) -> Result<(), String> {
        self.check()?;
        if self.last_progress.elapsed() < Duration::from_millis(100) {
            return Ok(());
        }
        #[derive(Serialize)]
        struct Progress<'a> {
            schema: &'static str,
            key: &'a wire::Key,
            semantic_steps: u64,
            observed_steps: u64,
            durable_audit_steps: u64,
        }
        let bytes = io::encode(
            &Progress {
                schema: "war1-study-progress-v1",
                key,
                semantic_steps: semantic,
                observed_steps: observed,
                durable_audit_steps: durable,
            },
            MESSAGE_LIMIT,
            true,
        )?;
        // A partial/blocked write is a monitor failure, never an unbounded queue.
        match self.channel.write(&bytes) {
            Ok(n) if n == bytes.len() => {
                self.last_progress = Instant::now();
                Ok(())
            }
            Ok(_) => Err("partial monitor progress write".into()),
            Err(e) => Err(format!("monitor progress: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pair() -> (Monitor, UnixStream) {
        let (reader, writer) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let now = Instant::now();
        (
            Monitor {
                channel: reader,
                input: vec![],
                start: now,
                heartbeat: now,
                last_progress: now,
            },
            writer,
        )
    }
    #[test]
    fn stale_heartbeat_and_native_wall_are_independently_enforced() {
        let (mut monitor, _writer) = pair();
        monitor.heartbeat = Instant::now() - Duration::from_secs(3);
        assert!(monitor.check().unwrap_err().contains("stale"));
        monitor.heartbeat = Instant::now();
        monitor.start = Instant::now() - Duration::from_secs(43201);
        assert!(monitor.check().unwrap_err().contains("wall"));
    }
    #[test]
    fn stop_and_duplicate_keys_fail_closed() {
        for bytes in [
            b"{\"schema\":\"war1-study-monitor-v1\",\"action\":\"stop\"}\n".to_vec(),
            b"{\"schema\":\"war1-study-monitor-v1\",\"action\":\"heartbeat\",\"action\":\"heartbeat\"}\n".to_vec(),
        ] {
            let (mut monitor, mut writer) = pair();
            writer.write_all(&bytes).unwrap();
            assert!(monitor.check().is_err());
        }
    }
    #[test]
    fn buffered_message_bound_is_checked_before_extending() {
        let (mut monitor, mut writer) = pair();
        monitor.input = vec![b'x'; MESSAGE_LIMIT];
        writer.write_all(b"x").unwrap();
        assert!(monitor.check().unwrap_err().contains("message bound"));
        assert_eq!(monitor.input.len(), MESSAGE_LIMIT);
    }
    #[test]
    fn disconnect_does_not_look_like_a_healthy_empty_channel() {
        let (mut monitor, writer) = pair();
        drop(writer);
        assert!(monitor.check().unwrap_err().contains("disconnected"));
    }
}
