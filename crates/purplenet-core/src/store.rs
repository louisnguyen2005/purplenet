use anyhow::{bail, Context, Result};
use fs2::FileExt;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    path::{Component, Path, PathBuf},
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
    /// Reject traversal and existing symlink components before a local write.
    /// The state directory must remain writable only by the trusted operator;
    /// these checks do not defend against concurrent hostile filesystem races.
    pub(crate) fn prepare_file(&self, path: &Path) -> Result<()> {
        let relative = path
            .strip_prefix(&self.root)
            .context("output is outside state directory")?;
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            bail!("output must use normal path components inside state");
        }
        let parent = relative.parent().context("output has no parent")?;
        let mut current = self.root.clone();
        for part in parent.components() {
            current.push(part);
            match fs::symlink_metadata(&current) {
                Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
                Ok(_) => bail!("unsafe output directory component"),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&current)?,
                Err(e) => return Err(e.into()),
            }
        }
        match fs::symlink_metadata(path) {
            Ok(meta) if !meta.is_file() || meta.file_type().is_symlink() => {
                bail!("unsafe output file")
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    /// Caller holds the store lock for a consistent snapshot.
    pub fn list_runs(&self) -> Result<Vec<crate::domain::Run>> {
        let mut runs = Vec::new();
        for entry in fs::read_dir(self.root.join("runs"))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|v| v.to_str()) != Some("json") {
                continue;
            }
            if !entry.file_type()?.is_file() || runs.len() >= 10_000 {
                bail!("unsafe run entry or run inventory exceeds 10000 files");
            }
            let run: crate::domain::Run = Self::read(&path)?;
            if run.schema_version != 1
                || !crate::domain::valid_id(&run.id)
                || path.file_stem().and_then(|v| v.to_str()) != Some(&run.id)
                || run.updated_unix < run.created_unix
            {
                bail!("invalid stored run: {}", path.display());
            }
            runs.push(run);
        }
        runs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(runs)
    }

    pub fn init(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        let root = fs::canonicalize(path)?;
        let store = Self { root };
        for dir in ["examples", "runs", "reports", "inputs"] {
            let path = store.root.join(dir);
            if let Ok(meta) = fs::symlink_metadata(&path) {
                if !meta.is_dir() || meta.file_type().is_symlink() {
                    bail!("unsafe state subdirectory: {dir}");
                }
            }
            fs::create_dir_all(path)?;
        }
        let marker = store.root.join(".purplenet-state");
        store.prepare_file(&marker)?;
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
        self.prepare_file(&self.root.join(".lock"))?;
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
        self.prepare_file(path)?;
        let parent = path.parent().context("output has no parent")?;
        let temp = parent.join(format!(".{}.tmp", Uuid::new_v4()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        use std::io::Write;
        let result = (|| -> Result<()> {
            file.write_all(data)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temp, path)?;
            #[cfg(unix)]
            File::open(parent)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result?;
        Ok(())
    }
}
