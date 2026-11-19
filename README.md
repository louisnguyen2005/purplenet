# PurpleNet

PurpleNet is a governed, single-network validation platform under construction. This first increment is a Rust workspace with versioned scope, approval, scenario, run, and audit models; a fail-closed policy engine; and a synthetic mock runner. It creates no network traffic, shell commands, or endpoint changes.

## Current commands

```sh
cargo run -p purplenet-cli -- init
cargo run -p purplenet-cli -- validate --scope state/examples/scope.json --approval state/examples/approval.json --scenario state/examples/scenario.json --actor example-operator
cargo run -p purplenet-cli -- run-mock --scope state/examples/scope.json --approval state/examples/approval.json --scenario state/examples/scenario.json --actor example-operator
```

The mock run prints a run ID. Inspect it with `cargo run -p purplenet-cli -- show-run <ID>`. Use `--state DIR` to select a different local state directory.
Check the local audit chain with `cargo run -p purplenet-cli -- verify-audit`.

Export a provenance-checked JSON and Markdown evidence report with `cargo run -p purplenet-cli -- report-run <ID> --actor example-reviewer`. Reports preserve approved inputs and label all observations as synthetic. See [report format and migration](docs/MOCK_REPORTS.md).

The sample scope uses `192.0.2.0/24`, a documentation network. The generated approval is only a demo fixture and expires after one day. It does not authorize activity on a real network. Replace it with a reviewed scope and approval record only after the owner, environment, and lab boundary are decided.

The proposed first lab uses libvirt and `10.77.0.0/24`. Its isolated-network XML and deliberately expired scope template are in [deploy/libvirt](deploy/libvirt/README.md). They are not activated by the CLI.

The [threat model](docs/THREAT_MODEL.md) records the current mock-stage trust boundaries, misuse cases, and gates for planned connectors, evidence, and AI features.

## Policy checks

Before a mock run, PurpleNet checks schema versions, IDs, scope and approval windows, the exact scope digest, distinct approver and operator, allowed scenario, event cap, excluded target IPs, and CIDR membership. It rechecks before each synthetic event. Denials and completions are written to a hash-linked JSONL audit log.

Approval schema 2 also binds the exact scenario digest. Legacy approvals must be regenerated; changes to scenario contents invalidate approval even when the scenario ID stays the same.

The audit chain is locally tamper-evident but not tamper-proof: an attacker with write access to both state and audit files can rewrite history. A future release must anchor digests outside the controller host and add authenticated multi-user approval.

## Layout

```text
apps/purplenet-cli/         operator CLI
crates/purplenet-core/      domain, policy, audit, store, mock runner
docs/                      blueprint, threat model, report guide, Excel tracker
.github/workflows/         format, test, and Clippy checks
state/                     local generated examples, runs and audit log (gitignored)
```

## Development status

This is the first implementation increment, not a production deployment. It lacks a live sensor connector, external identity provider, transactional database, service recovery, dashboard, and signed releases. The project tracker records those as open work. No production or non-mock execution path exists.

## Safety and operation

The first release operates entirely on local JSON fixtures. Do not interpret the generated approval as organizational permission. Keep real scope manifests, credentials, and evidence outside the example state directory and follow the network owner's authorization process.

## Development

Install Rust with Cargo, rustfmt, and Clippy. Run `cargo fmt --all -- --check`, `cargo test --workspace --locked`, and `cargo clippy --workspace --all-targets --locked -- -D warnings` before contributing. CI runs the same checks on pushes and pull requests.

See the [project blueprint](docs/PROJECT_BLUEPRINT.md), [implementation status](docs/IMPLEMENTATION_STATUS.md), and [Excel tracker](docs/PurpleNet_Implementation_Tracker.xlsx) for the roadmap. Only tasks whose full acceptance criteria are met are marked Finished.
