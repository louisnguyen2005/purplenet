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

The sample scope uses `192.0.2.0/24`, a documentation network. The generated approval is only a demo fixture and expires after one day. It does not authorize activity on a real network. Replace it with a reviewed scope and approval record only after the owner, environment, and lab boundary are decided.

The proposed first lab uses libvirt and `10.77.0.0/24`. Its isolated-network XML and deliberately expired scope template are in [deploy/libvirt](deploy/libvirt/README.md). They are not activated by the CLI.

## Policy checks

Before a mock run, PurpleNet checks schema versions, IDs, scope and approval windows, the exact scope digest, distinct approver and operator, allowed scenario, event cap, excluded target IPs, and CIDR membership. It rechecks before each synthetic event. Denials and completions are written to a hash-linked JSONL audit log.

The audit chain is locally tamper-evident but not tamper-proof: an attacker with write access to both state and audit files can rewrite history. A future release must anchor digests outside the controller host and add authenticated multi-user approval.

## Layout

```text
apps/purplenet-cli/         operator CLI
crates/purplenet-core/      domain, policy, audit, store, mock runner
state/                     local generated examples, runs and audit log (gitignored)
```

## Development status

This is the first implementation increment, not a production deployment. It lacks a live sensor connector, external identity provider, transactional database, service recovery, dashboard, and signed releases. The project tracker records those as open work. No production or non-mock execution path exists.

## Safety and operation

The first release operates entirely on local JSON fixtures. Do not interpret the generated approval as organizational permission. Keep real scope manifests, credentials, and evidence outside the example state directory and follow the network owner's authorization process.
