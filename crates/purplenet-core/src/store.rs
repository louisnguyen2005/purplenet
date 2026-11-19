use anyhow::{bail, Context, Result};
use fs2::FileExt;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct Store {
    pub root: PathBuf,
}
pub struct StoreLock(File);
impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

impl Store {
    pub fn init(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        let root = fs::canonicalize(path)?;
        let store = Self { root };
        for dir in ["examples", "runs", "reports", "inputs"] {
            fs::create_dir_all(store.root.join(dir))?;
        }
        let marker = store.root.join(".purplenet-state");
        if !marker.exists() {
            fs::write(marker, b"purplenet mock state v1\n")?;
        }
        Ok(store)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let root =
            fs::canonicalize(path).with_context(|| format!("open state {}", path.display()))?;
        if !root.join(".purplenet-state").is_file() {
            bail!("state directory is not initialized");
        }
        for dir in ["examples", "runs", "reports", "inputs"] {
            let meta = fs::symlink_metadata(root.join(dir))?;
            if !meta.is_dir() || meta.file_type().is_symlink() {
                bail!("unsafe state subdirectory: {dir}");
            }
        }
        Ok(Self { root })
    }

    pub fn lock(&self) -> Result<StoreLock> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".lock"))?;
        file.lock_exclusive()?;
        Ok(StoreLock(file))
    }

    pub fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
        use std::io::Read;
        let file = File::open(path).with_context(|| format!("read {}", path.display()))?;
        let mut data = Vec::new();
        file.take(1_000_001).read_to_end(&mut data)?;
        if data.len() > 1_000_000 {
            bail!("input exceeds 1 MiB: {}", path.display());
        }
        serde_json::from_slice(&data).with_context(|| format!("parse {}", path.display()))
    }

    pub fn write<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        self.write_bytes(path, &serde_json::to_vec_pretty(value)?)
    }

    pub fn write_bytes(&self, path: &Path, data: &[u8]) -> Result<()> {
        let parent = path.parent().context("output has no parent")?;
        if !parent.starts_with(&self.root) {
            bail!("output path is outside state directory");
        }
        fs::create_dir_all(parent)?;
        let temp = parent.join(format!(".{}.tmp", Uuid::new_v4()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        use std::io::Write;
        file.write_all(data)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        Ok(())
    }
}
