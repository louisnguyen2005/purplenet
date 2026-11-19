//! Offline integrity checks for the local mock store; no repair or dispatch.
use crate::{
    audit,
    domain::RunState,
    report::{self, RunInputs},
    store::Store,
};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Finding {
    pub code: String,
    pub run_id: Option<String>,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct StateCheck {
    pub schema_version: u32,
    pub consistent: bool,
    pub runs_checked: usize,
    pub active_runs: usize,
    pub recovery_required_runs: usize,
    pub audit_records: Option<usize>,
    pub findings: Vec<Finding>,
    pub limitations: Vec<String>,
}

impl StateCheck {
    fn finding(&mut self, code: &str, run_id: Option<&str>, detail: String) {
        self.consistent = false;
        self.findings.push(Finding {
            code: code.into(),
            run_id: run_id.map(str::to_owned),
            detail,
        });
    }
}

/// Lock-protected, read-only preflight for migrations and operator inspection.
/// Findings are data; only failure to acquire the store lock is a tool error.
pub fn check(store: &Store) -> Result<StateCheck> {
    let _lock = store.lock()?;
    let mut result = StateCheck {
        schema_version: 1, consistent: true, runs_checked: 0, active_runs: 0,
        recovery_required_runs: 0, audit_records: None, findings: Vec::new(),
        limitations: vec![
            "Internal consistency does not prove authenticity against a local writer.".into(),
            "No network isolation, independent sensor, approval identity, or service health is checked.".into(),
            "Active or recovery-required runs require operator review even if their stored data is consistent.".into(),
        ],
    };
    let runs = match store.list_runs() {
        Ok(runs) => runs,
        Err(error) => {
            result.finding("invalid_run_inventory", None, format!("{error:#}"));
            Vec::new()
        }
    };
    let audit_path = store.root.join("audit.jsonl");
    // A fresh empty store legitimately has no audit history. Missing history
    // is not accepted when runs exist or inventory could not be inspected.
    if !runs.is_empty() || !result.consistent || std::fs::symlink_metadata(&audit_path).is_ok() {
        let ids = runs.iter().map(|run| run.id.clone()).collect::<Vec<_>>();
        match audit::verify_runs(store, &ids) {
            Ok((count, missing)) => {
                result.audit_records = Some(count);
                for id in missing {
                    result.finding(
                        "missing_run_authorization",
                        Some(&id),
                        "no allowed run_mock decision exists for this run ID".into(),
                    );
                }
            }
            Err(error) => result.finding("invalid_audit_history", None, format!("{error:#}")),
        }
    } else {
        result.audit_records = Some(0);
    }
    for run in runs {
        result.runs_checked += 1;
        result.active_runs += usize::from(run.state.is_active());
        result.recovery_required_runs += usize::from(run.state == RunState::RecoveryRequired);
        let path = store.root.join("inputs").join(format!("{}.json", run.id));
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {}
            _ => {
                result.finding(
                    "missing_or_unsafe_inputs",
                    Some(&run.id),
                    "preserved input snapshot must be a regular file".into(),
                );
                continue;
            }
        }
        let checked = Store::read::<RunInputs>(&path)
            .and_then(|inputs| report::validate_provenance(&run, &inputs));
        if let Err(error) = checked {
            result.finding(
                "invalid_run_provenance",
                Some(&run.id),
                format!("{error:#}"),
            );
        }
    }
    Ok(result)
}
