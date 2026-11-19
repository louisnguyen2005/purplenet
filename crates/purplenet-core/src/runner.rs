use crate::{
    audit,
    domain::{Approval, NetworkScope, ObservedEvent, Run, RunState, Scenario},
    lifecycle, policy,
    store::Store,
};
use anyhow::{bail, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn validate(
    scope: &NetworkScope,
    approval: &Approval,
    scenario: &Scenario,
    actor: &str,
) -> Result<()> {
    policy::authorize(scope, approval, scenario, actor, now_unix())
}

pub fn run_mock(
    store: &Store,
    scope: &NetworkScope,
    approval: &Approval,
    scenario: &Scenario,
    actor: &str,
) -> Result<Run> {
    let _lock = store.lock()?;
    let now = now_unix();
    if let Err(error) = policy::authorize(scope, approval, scenario, actor, now) {
        audit::append(
            store,
            now,
            actor,
            "run_mock",
            "denied",
            &error.to_string(),
            None,
        )?;
        return Err(error);
    }
    lifecycle::recover_locked(store, actor)?;
    let mut run = Run {
        schema_version: 1,
        id: Uuid::new_v4().to_string(),
        scope_id: scope.id.clone(),
        scope_sha256: scope.digest()?,
        approval_id: approval.id.clone(),
        scenario_id: scenario.id.clone(),
        scenario_sha256: scenario.digest()?,
        actor: actor.into(),
        target: scenario.target,
        state: RunState::Approved,
        created_unix: now,
        updated_unix: now,
        events: Vec::new(),
    };
    let path = store.root.join("runs").join(format!("{}.json", run.id));
    audit::append(
        store,
        now,
        actor,
        "run_mock",
        "allowed",
        "policy checks passed",
        Some(&run.id),
    )?;
    // Preserve the approved input objects for later report reproduction.
    store.write(
        &store.root.join("inputs").join(format!("{}.json", run.id)),
        &crate::report::RunInputs {
            scope: scope.clone(),
            approval: approval.clone(),
            scenario: scenario.clone(),
        },
    )?;
    store.write(&path, &run)?;
    lifecycle::transition_locked(
        store,
        &mut run,
        RunState::Ready,
        actor,
        "mock inputs preserved",
    )?;
    lifecycle::transition_locked(
        store,
        &mut run,
        RunState::Running,
        actor,
        "begin synthetic observations",
    )?;
    for event in &scenario.mock_events {
        // Revalidate before each mock dispatch. No packet or process is created.
        if let Err(error) = policy::authorize(scope, approval, scenario, actor, now_unix()) {
            lifecycle::transition_locked(
                store,
                &mut run,
                RunState::Failed,
                actor,
                &error.to_string(),
            )?;
            audit::append(
                store,
                now_unix(),
                actor,
                "run_mock",
                "stopped",
                &error.to_string(),
                Some(&run.id),
            )?;
            return Err(error);
        }
        if run.events.len() >= scope.max_mock_events as usize {
            bail!("mock event limit reached");
        }
        run.events.push(ObservedEvent {
            id: event.id.clone(),
            label: event.label.clone(),
            source: "purplenet/mock".into(),
            observed_unix: now_unix().max(run.updated_unix),
        });
        run.updated_unix = now_unix().max(run.updated_unix);
        store.write(&path, &run)?;
    }
    lifecycle::transition_locked(
        store,
        &mut run,
        RunState::Completed,
        actor,
        "all synthetic observations stored",
    )?;
    audit::append(
        store,
        now_unix(),
        actor,
        "run_mock",
        "completed",
        "all mock events recorded",
        Some(&run.id),
    )?;
    Ok(run)
}
