use crate::{
    audit,
    domain::{Run, RunState},
    runner,
    store::Store,
};
use anyhow::{bail, Result};

pub(crate) fn transition_locked(
    store: &Store,
    run: &mut Run,
    next: RunState,
    actor: &str,
    reason: &str,
) -> Result<()> {
    let mut changed = run.clone();
    changed.transition(next, runner::now_unix().max(run.updated_unix))?;
    let detail = format!("{:?} -> {:?}: {reason}", run.state, next);
    audit::append(
        store,
        changed.updated_unix,
        actor,
        "run_transition",
        "requested",
        &detail,
        Some(&run.id),
    )?;
    store.write(
        &store.root.join("runs").join(format!("{}.json", run.id)),
        &changed,
    )?;
    *run = changed;
    audit::append(
        store,
        run.updated_unix,
        actor,
        "run_transition",
        "recorded",
        &detail,
        Some(&run.id),
    )?;
    Ok(())
}

/// The exclusive store lock proves that no cooperative mock runner is active.
/// Persisted active states are abandoned; they must never silently resume.
pub(crate) fn recover_locked(store: &Store, actor: &str) -> Result<Vec<String>> {
    if actor.trim().is_empty() || actor.len() > 120 {
        bail!("recovery actor must be named and at most 120 bytes");
    }
    let runs = store.list_runs()?;
    if !runs.is_empty() || store.root.join("audit.jsonl").exists() {
        audit::verify(store)?;
    }
    let mut recovered = Vec::new();
    for mut run in runs.into_iter().filter(|run| run.state.is_active()) {
        transition_locked(
            store,
            &mut run,
            RunState::RecoveryRequired,
            actor,
            "abandoned mock run found under exclusive lock; no resumption attempted",
        )?;
        recovered.push(run.id);
    }
    Ok(recovered)
}

pub fn recover_interrupted(store: &Store, actor: &str) -> Result<Vec<String>> {
    let _lock = store.lock()?;
    recover_locked(store, actor)
}
