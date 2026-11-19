use purplenet_core::{
    audit,
    domain::*,
    report::{self, MockReport},
    runner,
    store::Store,
};

#[test]
fn exports_mock_provenance_and_rejects_modified_event() {
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
    std::fs::remove_dir_all(root).unwrap();
}
