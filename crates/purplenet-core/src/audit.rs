use crate::store::Store;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
};

// Limits apply to encoded JSONL bytes, including newlines.
pub const MAX_RECORD_BYTES: usize = 65_536;
pub const MAX_LOG_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    store.prepare_file(&path)?;
    let (count, previous_hash, existing_bytes) = match File::open(&path) {
        Ok(file) => scan(file)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (0, "0".repeat(64), 0),
        Err(e) => return Err(e).context("read audit log"),
    };
    let mut record = AuditRecord {
        sequence: count as u64 + 1,
        previous_hash,
        at_unix,
        actor: actor.into(),
        action: action.into(),
        decision: decision.into(),
        reason: reason.into(),
        run_id: run_id.map(str::to_owned),
        record_hash: String::new(),
    };
    record.record_hash = format!("{:x}", Sha256::digest(record.hash_material()?));
    let mut encoded = serde_json::to_vec(&record)?;
    encoded.push(b'\n');
    if encoded.len() > MAX_RECORD_BYTES || existing_bytes + encoded.len() as u64 > MAX_LOG_BYTES {
        bail!("audit record or log byte limit exceeded");
    }
    let mut file = OpenOptions::new().append(true).create(true).open(&path)?;
    file.write_all(&encoded)?;
    file.sync_all()?;
    Ok(record)
}

pub fn verify(store: &Store) -> Result<usize> {
    let path = store.root.join("audit.jsonl");
    store.prepare_file(&path)?;
    let (count, _, _) = scan(File::open(path).context("read audit log")?)?;
    Ok(count)
}

fn scan(file: File) -> Result<(usize, String, u64)> {
    if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_LOG_BYTES {
        bail!("audit must be a regular file within the log byte limit");
    }
    let mut reader = BufReader::new(file);
    let mut expected_previous = "0".repeat(64);
    let mut count = 0;
    let mut total = 0_u64;
    loop {
        let mut line = Vec::new();
        let bytes = reader
            .by_ref()
            .take((MAX_RECORD_BYTES + 1) as u64)
            .read_until(b'\n', &mut line)?;
        if bytes == 0 {
            break;
        }
        total += bytes as u64;
        if bytes > MAX_RECORD_BYTES || total > MAX_LOG_BYTES || line.last() != Some(&b'\n') {
            bail!("audit byte limit exceeded or incomplete trailing record");
        }
        let expected_sequence = count as u64 + 1;
        let record: AuditRecord = serde_json::from_slice(&line).context("parse audit record")?;
        if record.sequence != expected_sequence || record.previous_hash != expected_previous {
            bail!("audit chain sequence or predecessor mismatch at record {expected_sequence}");
        }
        let hash = format!("{:x}", Sha256::digest(record.hash_material()?));
        if record.record_hash != hash {
            bail!("audit record hash mismatch at record {expected_sequence}");
        }
        expected_previous = record.record_hash;
        count += 1;
    }
    Ok((count, expected_previous, total))
}
