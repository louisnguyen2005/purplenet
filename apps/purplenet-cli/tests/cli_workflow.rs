use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Lab {
    root: PathBuf,
}
impl Lab {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("purplenet-cli-{}-{stamp}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let lab = Self { root };
        lab.ok(&["init"]);
        lab
    }
    fn command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_purplenet-cli"));
        c.arg("--state")
            .arg(&self.root)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c
    }
    fn ok(&self, args: &[&str]) -> Output {
        let out = self.command(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }
    fn mock(&self) -> Command {
        let mut c = self.command(&["run-mock"]);
        for (flag, name) in [
            ("--scope", "scope"),
            ("--approval", "approval"),
            ("--scenario", "scenario"),
        ] {
            c.arg(flag)
                .arg(self.root.join("examples").join(format!("{name}.json")));
        }
        c.args(["--actor", "example-operator"]);
        c
    }
}
impl Drop for Lab {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn cli_reports_and_recovers_interrupted_mock_without_resuming() {
    let lab = Lab::new();
    assert!(lab.mock().output().unwrap().status.success());
    let runs: Value = serde_json::from_slice(&lab.ok(&["list-runs"]).stdout).unwrap();
    let id = runs[0]["id"].as_str().unwrap();
    lab.ok(&["report-run", id, "--actor", "reviewer"]);
    let report: Value = serde_json::from_slice(
        &fs::read(lab.root.join("reports").join(format!("{id}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(report["detection_result"], "not_evaluated");
    assert!(lab.root.join("reports").join(format!("{id}.md")).is_file());
    let mut run = runs[0].clone();
    run["state"] = json!("running");
    fs::write(
        lab.root.join("runs").join(format!("{id}.json")),
        serde_json::to_vec(&run).unwrap(),
    )
    .unwrap();
    let first: Value = serde_json::from_slice(
        &lab.ok(&["recover-interrupted", "--actor", "recovery-owner"])
            .stdout,
    )
    .unwrap();
    assert_eq!(first, json!([id]));
    let second: Value = serde_json::from_slice(
        &lab.ok(&["recover-interrupted", "--actor", "recovery-owner"])
            .stdout,
    )
    .unwrap();
    assert_eq!(second, json!([]));
    lab.ok(&["verify-audit"]);
    assert!(!lab
        .command(&["show-run", "../outside"])
        .output()
        .unwrap()
        .status
        .success());
}

#[test]
fn concurrent_cli_runs_serialize_and_preserve_audit_chain() {
    let lab = Lab::new();
    let first = lab.mock().spawn().unwrap();
    let second = lab.mock().spawn().unwrap();
    assert!(first.wait_with_output().unwrap().status.success());
    assert!(second.wait_with_output().unwrap().status.success());
    let runs: Value = serde_json::from_slice(&lab.ok(&["list-runs"]).stdout).unwrap();
    assert_eq!(runs.as_array().unwrap().len(), 2);
    assert!(runs
        .as_array()
        .unwrap()
        .iter()
        .all(|run| run["state"] == "completed"));
    lab.ok(&["verify-audit"]);
}

#[test]
fn changed_scenario_returns_failure_and_creates_no_run() {
    let lab = Lab::new();
    let path = lab.root.join("examples/scenario.json");
    let mut scenario: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    scenario["mock_events"][0]["label"] = json!("changed after approval");
    fs::write(path, serde_json::to_vec(&scenario).unwrap()).unwrap();
    let result = lab.mock().output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("scenario not approved"));
    assert_eq!(fs::read_dir(lab.root.join("runs")).unwrap().count(), 0);
    lab.ok(&["verify-audit"]);
}
