use crate::{
    audit,
    domain::{valid_id, Approval, NetworkScope, Run, RunState, Scenario},
    policy, runner,
    store::Store,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunInputs {
    pub scope: NetworkScope,
    pub approval: Approval,
    pub scenario: Scenario,
}

#[derive(Serialize, Deserialize)]
pub struct MockReport {
    pub schema_version: u32,
    pub generator_version: String,
    pub generated_unix: u64,
    pub evidence_kind: String,
    pub detection_result: String,
    pub run_sha256: String,
    pub inputs_sha256: String,
    pub verified_audit_records: usize,
    pub limitations: Vec<String>,
    pub run: Run,
    pub inputs: RunInputs,
}

pub fn validate_provenance(run: &Run, inputs: &RunInputs) -> Result<()> {
    let RunInputs {
        scope,
        approval,
        scenario,
    } = inputs;
    if run.schema_version != 1
        || !valid_id(&run.id)
        || run.scope_id != scope.id
        || run.scope_sha256 != scope.digest()?
        || run.approval_id != approval.id
        || run.scenario_id != scenario.id
        || run.scenario_sha256 != scenario.digest()?
        || run.target != scenario.target
        || run.updated_unix < run.created_unix
        || run.events.len() > scenario.mock_events.len()
    {
        bail!("stored run does not match preserved input provenance");
    }
    policy::authorize(scope, approval, scenario, &run.actor, run.created_unix)?;
    for (observed, expected) in run.events.iter().zip(&scenario.mock_events) {
        if observed.id != expected.id
            || observed.label != expected.label
            || observed.source != "purplenet/mock"
            || observed.observed_unix < run.created_unix
            || observed.observed_unix > run.updated_unix
        {
            bail!("stored mock event differs from approved scenario");
        }
        policy::authorize(
            scope,
            approval,
            scenario,
            &run.actor,
            observed.observed_unix,
        )?;
    }
    if matches!(run.state, RunState::Completed) && run.events.len() != scenario.mock_events.len() {
        bail!("completed run is missing expected synthetic events");
    }
    Ok(())
}

/// Export into the state directory under its exclusive writer lock.
/// This checks internal consistency; local writers can still forge all inputs.
pub fn export(store: &Store, id: &str, actor: &str) -> Result<PathBuf> {
    if !valid_id(id) || actor.trim().is_empty() || actor.len() > 120 {
        bail!("invalid run ID or report actor");
    }
    let _lock = store.lock()?;
    let count = audit::verify(store)?;
    let run: Run = Store::read(&store.root.join("runs").join(format!("{id}.json")))?;
    let inputs: RunInputs = Store::read(&store.root.join("inputs").join(format!("{id}.json")))?;
    if run.id != id {
        bail!("run ID mismatch");
    }
    validate_provenance(&run, &inputs)?;
    let report = MockReport {
        schema_version: 1,
        generator_version: env!("CARGO_PKG_VERSION").into(),
        generated_unix: runner::now_unix(),
        evidence_kind: "synthetic_only".into(),
        detection_result: "not_evaluated".into(),
        run_sha256: format!("{:x}", Sha256::digest(serde_json::to_vec(&run)?)),
        inputs_sha256: format!("{:x}", Sha256::digest(serde_json::to_vec(&inputs)?)),
        verified_audit_records: count,
        limitations: vec![
            "Synthetic events do not demonstrate independent sensor detection.".into(),
            "Local hashes provide consistency checks, not signatures or external anchoring.".into(),
            "Actor and approver names are not authenticated identities.".into(),
        ],
        run,
        inputs,
    };
    let path = store.root.join("reports").join(format!("{id}.json"));
    // Log intent before writing, then completion; a write failure leaves intent only.
    audit::append(
        store,
        runner::now_unix(),
        actor,
        "report_export",
        "requested",
        "mock JSON report requested",
        Some(id),
    )?;
    store.write(&path, &report)?;
    // Indented JSON keeps all untrusted labels inside a Markdown code block.
    let detail = serde_json::to_string_pretty(&report)?
        .lines()
        .map(|line| format!("    {line}\n"))
        .collect::<String>();
    let markdown = format!("# PurpleNet mock evidence report\n\nRun: `{id}`\n\nEvidence: **synthetic only**. Detection: **not evaluated**.\n\nThe embedded JSON records scope, approval, scenario, observations, and hashes. Local writers can rewrite the whole store; these hashes are not signatures.\n\n{detail}");
    store.write_bytes(
        &store.root.join("reports").join(format!("{id}.md")),
        markdown.as_bytes(),
    )?;
    audit::append(
        store,
        runner::now_unix(),
        actor,
        "report_export",
        "completed",
        "mock JSON report saved",
        Some(id),
    )?;
    Ok(path)
}
