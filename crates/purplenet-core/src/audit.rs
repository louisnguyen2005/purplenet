use crate::store::Store;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub sequence: u64,
    pub previous_hash: String,
    pub at_unix: u64,
    pub actor: String,
    pub action: String,
    pub decision: String,
    pub reason: String,
    pub run_id: Option<String>,
    pub record_hash: String,
}

impl AuditRecord {
    fn hash_material(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&(
            self.sequence,
            &self.previous_hash,
            self.at_unix,
            &self.actor,
            &self.action,
            &self.decision,
            &self.reason,
            &self.run_id,
        ))?)
    }
}

pub fn append(
    store: &Store,
    at_unix: u64,
    actor: &str,
    action: &str,
    decision: &str,
    reason: &str,
    run_id: Option<&str>,
) -> Result<AuditRecord> {
    let path = store.root.join("audit.jsonl");
    let contents = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).context("read audit log"),
    };
    let previous: Option<AuditRecord> = contents
        .lines()
        .last()
        .map(serde_json::from_str)
        .transpose()
        .context("parse last audit record")?;
    let mut record = AuditRecord {
        sequence: previous.as_ref().map_or(1, |v| v.sequence + 1),
        previous_hash: previous.map_or_else(|| "0".repeat(64), |v| v.record_hash),
        at_unix,
        actor: actor.into(),
        action: action.into(),
        decision: decision.into(),
        reason: reason.into(),
        run_id: run_id.map(str::to_owned),
        record_hash: String::new(),
    };
    record.record_hash = format!("{:x}", Sha256::digest(record.hash_material()?));
    let mut file = OpenOptions::new().append(true).create(true).open(&path)?;
    serde_json::to_writer(&mut file, &record)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(record)
}

pub fn verify(store: &Store) -> Result<usize> {
    let contents = fs::read_to_string(store.root.join("audit.jsonl")).context("read audit log")?;
    let mut expected_previous = "0".repeat(64);
    let mut expected_sequence = 1;
    let mut count = 0;
    for line in contents.lines() {
        let record: AuditRecord = serde_json::from_str(line).context("parse audit record")?;
        if record.sequence != expected_sequence || record.previous_hash != expected_previous {
            bail!("audit chain sequence or predecessor mismatch at record {expected_sequence}");
        }
        let hash = format!("{:x}", Sha256::digest(record.hash_material()?));
        if record.record_hash != hash {
            bail!("audit record hash mismatch at record {expected_sequence}");
        }
        expected_previous = record.record_hash;
        expected_sequence += 1;
        count += 1;
    }
    Ok(count)
}
