use purplenet_core::{
    audit,
    domain::*,
    report::{self, MockReport},
    runner,
    store::Store,
};

fn fixture() -> (Store, NetworkScope, Approval, Scenario) {
    let root = std::env::temp_dir().join(format!("purplenet-test-{}", uuid::Uuid::new_v4()));
    let store = Store::init(&root).unwrap();
    let now = runner::now_unix();
    let scope = NetworkScope {
        schema_version: 1,
        id: "scope".into(),
        owner: "owner".into(),
        environment: "lab".into(),
        allowed_cidrs: vec!["192.0.2.0/24".parse().unwrap()],
        assets: vec![Asset {
            id: "vm".into(),
            ip: "192.0.2.10".parse().unwrap(),
        }],
        excluded_ips: vec![],
        valid_from_unix: now - 1,
        valid_until_unix: now + 3600,
        max_mock_events: 2,
        allowed_scenarios: vec!["scenario".into()],
    };
    let scenario = Scenario {
        schema_version: 1,
        id: "scenario".into(),
        description: "fixture".into(),
        target: scope.assets[0].ip,
        target_asset_id: "vm".into(),
        mock_events: vec![MockEvent {
            id: "event".into(),
            label: "synthetic".into(),
        }],
    };
    let approval = Approval {
        schema_version: 2,
        id: "approval".into(),
        scope_id: scope.id.clone(),
        scope_sha256: scope.digest().unwrap(),
        scenario_id: scenario.id.clone(),
        scenario_sha256: scenario.digest().unwrap(),
        approved_by: "approver".into(),
        valid_from_unix: now - 1,
        valid_until_unix: now + 3600,
    };
    (store, scope, approval, scenario)
}

#[test]
fn exports_mock_provenance_and_rejects_modified_event() {
    let (store, scope, approval, scenario) = fixture();
    let mut run = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    let path = report::export(&store, &run.id, "reviewer").unwrap();
    let report: MockReport = Store::read(&path).unwrap();
    assert!(path.with_extension("md").is_file());
    assert_eq!(report.evidence_kind, "synthetic_only");
    assert_eq!(report.detection_result, "not_evaluated");
    assert_eq!(report.inputs.approval.scenario_sha256, run.scenario_sha256);
    assert!(audit::verify(&store).unwrap() >= 4);
    run.events[0].label = "modified".into();
    store
        .write(
            &store.root.join("runs").join(format!("{}.json", run.id)),
            &run,
        )
        .unwrap();
    assert!(report::export(&store, &run.id, "reviewer").is_err());
    std::fs::remove_dir_all(&store.root).unwrap();
}

#[test]
fn recovers_all_abandoned_states_once_and_preserves_evidence() {
    let (store, scope, approval, scenario) = fixture();
    let completed = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    let mut ids = Vec::new();
    for state in [RunState::Approved, RunState::Ready, RunState::Running] {
        let mut abandoned = completed.clone();
        abandoned.id = uuid::Uuid::new_v4().to_string();
        abandoned.state = state;
        store
            .write(
                &store
                    .root
                    .join("runs")
                    .join(format!("{}.json", abandoned.id)),
                &abandoned,
            )
            .unwrap();
        ids.push(abandoned.id);
    }
    let recovered =
        purplenet_core::lifecycle::recover_interrupted(&store, "recovery-owner").unwrap();
    assert_eq!(recovered.len(), 3);
    for id in ids {
        let saved: Run = Store::read(&store.root.join("runs").join(format!("{id}.json"))).unwrap();
        assert_eq!(saved.state, RunState::RecoveryRequired);
        assert_eq!(saved.events.len(), completed.events.len());
    }
    let records = audit::verify(&store).unwrap();
    assert!(
        purplenet_core::lifecycle::recover_interrupted(&store, "recovery-owner")
            .unwrap()
            .is_empty()
    );
    assert_eq!(audit::verify(&store).unwrap(), records);
    let saved: Run = Store::read(
        &store
            .root
            .join("runs")
            .join(format!("{}.json", completed.id)),
    )
    .unwrap();
    assert_eq!(saved.state, RunState::Completed);
    std::fs::remove_dir_all(&store.root).unwrap();
}

#[test]
fn new_run_recovers_abandoned_work_and_corrupt_inventory_blocks_dispatch() {
    let (store, scope, approval, scenario) = fixture();
    let mut old = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    old.state = RunState::Running;
    let old_path = store.root.join("runs").join(format!("{}.json", old.id));
    store.write(&old_path, &old).unwrap();
    let new = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    assert_ne!(old.id, new.id);
    let saved: Run = Store::read(&old_path).unwrap();
    assert_eq!(saved.state, RunState::RecoveryRequired);
    let recovered_report = report::export(&store, &old.id, "reviewer").unwrap();
    let recovered: MockReport = Store::read(&recovered_report).unwrap();
    assert_eq!(recovered.run.state, RunState::RecoveryRequired);
    // Preserve malformed inventory for investigation and refuse any new dispatch.
    std::fs::write(store.root.join("runs").join("broken.json"), b"not JSON").unwrap();
    let count = std::fs::read_dir(store.root.join("runs")).unwrap().count();
    assert!(runner::run_mock(&store, &scope, &approval, &scenario, "operator").is_err());
    assert_eq!(
        std::fs::read_dir(store.root.join("runs")).unwrap().count(),
        count
    );
    std::fs::remove_dir_all(&store.root).unwrap();
}

#[test]
fn terminal_runs_cannot_restart_or_complete_from_ready() {
    assert!(!RunState::Completed.can_transition_to(RunState::Running));
    assert!(!RunState::Failed.can_transition_to(RunState::Running));
    assert!(!RunState::RecoveryRequired.can_transition_to(RunState::Running));
    assert!(!RunState::Ready.can_transition_to(RunState::Completed));
    let (store, scope, approval, scenario) = fixture();
    let mut run = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    run.state = RunState::Ready;
    let updated = run.updated_unix;
    assert!(run.transition(RunState::Running, updated - 1).is_err());
    assert_eq!(run.state, RunState::Ready);
    assert_eq!(run.updated_unix, updated);
    std::fs::remove_dir_all(&store.root).unwrap();
}

#[test]
fn invalid_audit_blocks_recovery_before_state_changes() {
    let (store, scope, approval, scenario) = fixture();
    let mut run = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    run.state = RunState::Running;
    let path = store.root.join("runs").join(format!("{}.json", run.id));
    store.write(&path, &run).unwrap();
    std::fs::write(store.root.join("audit.jsonl"), b"corrupted audit\n").unwrap();
    assert!(purplenet_core::lifecycle::recover_interrupted(&store, "recovery-owner").is_err());
    let saved: Run = Store::read(&path).unwrap();
    assert_eq!(saved.state, RunState::Running);
    assert!(runner::run_mock(&store, &scope, &approval, &scenario, "operator").is_err());
    std::fs::remove_dir_all(&store.root).unwrap();
}

#[test]
fn state_check_accepts_fresh_and_valid_stores_without_changing_evidence() {
    let (store, scope, approval, scenario) = fixture();
    let fresh = purplenet_core::health::check(&store).unwrap();
    assert!(fresh.consistent);
    assert_eq!(fresh.audit_records, Some(0));
    assert!(!store.root.join("audit.jsonl").exists());
    let mut run = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    let path = store.root.join("runs").join(format!("{}.json", run.id));
    run.state = RunState::Running;
    store.write(&path, &run).unwrap();
    let before = std::fs::read(&path).unwrap();
    let audit = std::fs::read(store.root.join("audit.jsonl")).unwrap();
    let checked = purplenet_core::health::check(&store).unwrap();
    assert!(checked.consistent);
    assert_eq!(checked.active_runs, 1);
    assert_eq!(checked.runs_checked, 1);
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert_eq!(
        std::fs::read(store.root.join("audit.jsonl")).unwrap(),
        audit
    );
    std::fs::remove_dir_all(&store.root).unwrap();
}

#[test]
fn state_check_reports_missing_inputs_and_unlinked_run_ids() {
    let (store, scope, approval, scenario) = fixture();
    let run = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    let mut orphan = run.clone();
    orphan.id = uuid::Uuid::new_v4().to_string();
    store
        .write(
            &store.root.join("runs").join(format!("{}.json", orphan.id)),
            &orphan,
        )
        .unwrap();
    let checked = purplenet_core::health::check(&store).unwrap();
    assert!(!checked.consistent);
    assert!(checked
        .findings
        .iter()
        .any(|f| f.code == "missing_run_authorization" && f.run_id.as_deref() == Some(&orphan.id)));
    assert!(checked
        .findings
        .iter()
        .any(|f| f.code == "missing_or_unsafe_inputs"));
    std::fs::remove_dir_all(&store.root).unwrap();
}

#[test]
fn state_check_collects_provenance_and_audit_failures_then_handles_invalid_inventory() {
    let (store, scope, approval, scenario) = fixture();
    let mut run = runner::run_mock(&store, &scope, &approval, &scenario, "operator").unwrap();
    run.events[0].label = "tampered".into();
    store
        .write(
            &store.root.join("runs").join(format!("{}.json", run.id)),
            &run,
        )
        .unwrap();
    std::fs::write(store.root.join("audit.jsonl"), b"invalid\n").unwrap();
    let checked = purplenet_core::health::check(&store).unwrap();
    assert!(!checked.consistent);
    assert_eq!(checked.audit_records, None);
    assert!(checked
        .findings
        .iter()
        .any(|f| f.code == "invalid_audit_history"));
    assert!(checked
        .findings
        .iter()
        .any(|f| f.code == "invalid_run_provenance"));
    std::fs::write(store.root.join("runs/broken.json"), b"invalid").unwrap();
    let checked = purplenet_core::health::check(&store).unwrap();
    assert!(checked
        .findings
        .iter()
        .any(|f| f.code == "invalid_run_inventory"));
    assert_eq!(checked.runs_checked, 0);
    std::fs::remove_dir_all(&store.root).unwrap();
}
