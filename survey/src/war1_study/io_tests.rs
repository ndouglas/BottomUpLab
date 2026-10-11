use super::io::{self, DurableWrite, Journal};
use std::io::Write;
#[derive(Default)]
struct Fault {
    bytes: Vec<u8>,
    fail: &'static str,
}
impl Write for Fault {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if self.fail == "write" {
            self.bytes.extend_from_slice(&b[..b.len().min(3)]);
            return Err(std::io::Error::other("partial write"));
        }
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.fail == "flush" {
            Err(std::io::Error::other("flush"))
        } else {
            Ok(())
        }
    }
}
impl DurableWrite for Fault {
    fn sync_all(&mut self) -> std::io::Result<()> {
        if self.fail == "sync" {
            Err(std::io::Error::other("sync"))
        } else {
            Ok(())
        }
    }
}
#[test]
fn failed_write_flush_or_sync_never_acknowledges_and_poison_prevents_retry() {
    for fail in ["write", "flush", "sync"] {
        let mut j = Journal::with_writer(
            Fault {
                fail,
                ..Default::default()
            },
            1024,
        );
        assert!(j.append(&vec![1, 2, 3], 1024).is_err());
        assert_eq!(j.receipt().bytes, 0);
        assert!(j.append(&vec![4], 1024).is_err());
    }
}
#[test]
fn serialization_and_campaign_budget_are_enforced_before_write() {
    assert!(io::encode(&"x".repeat(64), 8, false).is_err());
    let mut j = Journal::with_writer(Fault::default(), 4);
    assert!(j.append(&vec![1, 2, 3], 128).is_err());
    assert_eq!(j.receipt().bytes, 0);
}
#[derive(Clone)]
pub(super) enum Effect {
    Fail,
    Partial,
    Short,
}
struct Rule {
    suffix: String,
    event: &'static str,
    nth: usize,
    seen: usize,
    effect: Effect,
}
thread_local! {static RULES:std::cell::RefCell<Vec<Rule>>=const {std::cell::RefCell::new(Vec::new())};}
pub(super) fn set_faults(items: Vec<(&str, &'static str, usize, Effect)>) {
    RULES.with(|rules| {
        *rules.borrow_mut() = items
            .into_iter()
            .map(|(suffix, event, nth, effect)| Rule {
                suffix: suffix.into(),
                event,
                nth,
                seen: 0,
                effect,
            })
            .collect()
    });
}
pub(super) fn injected(path: &std::path::Path, event: &str) -> Option<Effect> {
    RULES.with(|rules| {
        for r in rules.borrow_mut().iter_mut() {
            if path.to_string_lossy().ends_with(&r.suffix) && r.event == event {
                r.seen += 1;
                if r.seen == r.nth {
                    return Some(r.effect.clone());
                }
            }
        }
        None
    })
}
#[test]
fn native_short_write_is_completed_before_acknowledgment() {
    let path = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-short-{}.jsonl", std::process::id()));
    if path.exists() {
        std::fs::remove_file(&path).unwrap();
    }
    set_faults(vec![(
        path.file_name().unwrap().to_str().unwrap(),
        "write",
        1,
        Effect::Short,
    )]);
    let mut journal = Journal::new(&path, 1024).unwrap();
    journal.append(&vec![1, 2, 3], 1024).unwrap();
    set_faults(vec![]);
    assert_eq!(std::fs::read(&path).unwrap(), b"[1,2,3]\n");
    assert_eq!(journal.receipt().bytes, 8);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn oversized_symlink_and_fifo_inputs_are_refused_without_opening() {
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-inputs-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("large");
    std::fs::File::create(&file)
        .unwrap()
        .set_len(16 * 1024 * 1024 + 1)
        .unwrap();
    assert!(io::read_input(&file).is_err());
    #[cfg(unix)]
    {
        let link = root.join("link");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        assert!(io::read_input(&link).is_err());
        let fifo = root.join("fifo");
        assert!(std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success());
        assert!(io::read_input(&fifo).is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}
