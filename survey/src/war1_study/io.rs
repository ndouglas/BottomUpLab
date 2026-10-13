//! Bounded, create-only writes. Only synchronized complete prefixes are acknowledged.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};
pub const METADATA_LIMIT: usize = 16 * 1024 * 1024;
pub const ROW_LIMIT: usize = 32 * 1024;
pub const CAMPAIGN_LIMIT: u64 = 4 * 1024 * 1024 * 1024;
pub const FAILURE_RESERVE: u64 = 1024 * 1024;
pub fn check_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut prefix = PathBuf::new();
    for component in absolute.components() {
        if component == Component::ParentDir {
            return Err("parent traversal refused".into());
        }
        prefix.push(component);
        let m = fs::symlink_metadata(&prefix).map_err(|e| format!("{}: {e}", prefix.display()))?;
        if m.file_type().is_symlink() {
            return Err("symlink path refused".into());
        }
    }
    Ok(absolute)
}
pub fn read_input(path: &Path) -> Result<Vec<u8>, String> {
    read_bounded(path, METADATA_LIMIT)
}
pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let path = check_path(path)?;
    let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > limit as u64 {
        return Err("input must be a bounded regular file".into());
    }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err("input limit exceeded".into());
    }
    Ok(bytes)
}
pub fn hash_file(path: &Path, limit: u64) -> Result<(String, u64), String> {
    let path = check_path(path)?;
    let m = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if !m.is_file() || m.len() > limit {
        return Err("nonregular/oversized hashed input".into());
    }
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut count = 0;
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > limit {
            return Err("hashed input grew beyond bound".into());
        }
        h.update(&buffer[..n]);
    }
    Ok((format!("{:x}", h.finalize()), count))
}
pub fn create_directory(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    check_path(parent)?;
    let name = path.file_name().ok_or("new output name required")?;
    fs::create_dir(parent.join(name)).map_err(|e| e.to_string())?;
    sync_directory(parent)
}
pub fn sync_directory(path: &Path) -> Result<(), String> {
    #[cfg(test)]
    fault(path, "directory_sync")?;
    File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|e| format!("parent/directory sync: {e}"))
}
struct Bounded {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for Bounded {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if b.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("encoded record limit exceeded"));
        }
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn encode(value: &impl Serialize, limit: usize, newline: bool) -> Result<Vec<u8>, String> {
    let mut out = Bounded {
        bytes: Vec::with_capacity(limit.min(1024)),
        limit,
    };
    serde_json::to_writer(&mut out, value).map_err(|e| e.to_string())?;
    if newline {
        out.write_all(b"\n").map_err(|e| e.to_string())?
    }
    Ok(out.bytes)
}
pub trait DurableWrite: Write {
    fn sync_all(&mut self) -> io::Result<()>;
}
impl DurableWrite for File {
    fn sync_all(&mut self) -> io::Result<()> {
        File::sync_all(self)
    }
}
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub sha256: String,
    pub bytes: u64,
    pub records: u64,
}
pub struct Journal<W: DurableWrite = NativeFile> {
    writer: W,
    hash: Sha256,
    bytes: u64,
    records: u64,
    limit: u64,
    poisoned: bool,
}
impl Journal<NativeFile> {
    pub fn new(path: &Path, limit: u64) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        sync_directory(path.parent().ok_or("file parent")?)?;
        Ok(Self::with_writer(
            NativeFile {
                file,
                #[cfg(test)]
                path: path.to_path_buf(),
            },
            limit,
        ))
    }
}
impl<W: DurableWrite> Journal<W> {
    pub fn with_writer(writer: W, limit: u64) -> Self {
        Self {
            writer,
            hash: Sha256::new(),
            bytes: 0,
            records: 0,
            limit,
            poisoned: false,
        }
    }
    pub fn append(&mut self, value: &impl Serialize, limit: usize) -> Result<(), String> {
        if self.poisoned {
            return Err("writer poisoned; retry unavailable".into());
        }
        let result = (|| {
            let bytes = encode(value, limit, true)?;
            if bytes.len() as u64 > self.limit.saturating_sub(self.bytes) {
                return Err("reserved campaign byte limit exceeded".into());
            }
            self.writer
                .write_all(&bytes)
                .and_then(|()| self.writer.flush())
                .and_then(|()| self.writer.sync_all())
                .map_err(|e| format!("write/flush/sync: {e}"))?;
            self.hash.update(&bytes);
            self.bytes += bytes.len() as u64;
            self.records += 1;
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true
        }
        result
    }
    pub fn receipt(&self) -> Prefix {
        Prefix {
            sha256: format!("{:x}", self.hash.clone().finalize()),
            bytes: self.bytes,
            records: self.records,
        }
    }
    pub fn finish(&mut self) -> Result<Prefix, String> {
        if self.poisoned {
            return Err("writer poisoned; cannot close descriptor".into());
        }
        if let Err(e) = self.writer.flush().and_then(|()| self.writer.sync_all()) {
            self.poisoned = true;
            return Err(format!("finish flush/sync: {e}"));
        }
        Ok(self.receipt())
    }
}
pub fn save_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > METADATA_LIMIT {
        return Err("metadata limit exceeded".into());
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let mut file = NativeFile {
        file,
        #[cfg(test)]
        path: path.to_path_buf(),
    };
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())?;
    #[cfg(test)]
    fault(path, "parent_sync")?;
    sync_directory(path.parent().ok_or("file parent")?)
}

// The immutable path exists only in test builds, for injecting failures beneath
// real filesystem writes without changing archive/observer behavior.
pub struct NativeFile {
    file: File,
    #[cfg(test)]
    path: PathBuf,
}
#[cfg(test)]
fn fault(path: &Path, event: &str) -> Result<(), String> {
    if super::io_tests::injected(path, event).is_some() {
        Err(format!("injected {event} failure"))
    } else {
        Ok(())
    }
}
impl Write for NativeFile {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        #[cfg(test)]
        if let Some(effect) = super::io_tests::injected(&self.path, "write") {
            match effect {
                super::io_tests::Effect::Fail => {
                    return Err(io::Error::other("injected write failure"))
                }
                super::io_tests::Effect::Partial => {
                    self.file.write_all(&bytes[..bytes.len().min(3)])?;
                    return Err(io::Error::other("injected partial write failure"));
                }
                super::io_tests::Effect::Short => {
                    return self.file.write(&bytes[..bytes.len().min(3)])
                }
            }
        }
        self.file.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        #[cfg(test)]
        fault(&self.path, "flush").map_err(io::Error::other)?;
        self.file.flush()
    }
}
impl DurableWrite for NativeFile {
    fn sync_all(&mut self) -> io::Result<()> {
        #[cfg(test)]
        fault(&self.path, "sync").map_err(io::Error::other)?;
        self.file.sync_all()
    }
}
