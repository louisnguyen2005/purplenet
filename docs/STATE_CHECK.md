# State consistency preflight

Run `cargo run -p purplenet-cli -- check-state` to inspect the current mock store. Use the global `--state DIR` option for a different initialized directory. The command emits a JSON result on stdout, exits successfully when its checked records are consistent, and exits nonzero with a summary on stderr when findings exist.

The checker acquires the exclusive store lock and checks:

- Run inventory schema, filenames and timestamp ordering through the existing inventory reader.
- The complete bounded audit hash chain, including the terminating newline and record/log limits.
- An `allowed` `run_mock` audit decision referencing each stored run ID.
- A regular, non-symlink preserved input snapshot for every run.
- Exact scope/scenario digests, approval relationships, historical authorization windows and synthetic observation provenance through the report validator.

The output contains `consistent`, `runs_checked`, `active_runs`, `recovery_required_runs`, `audit_records`, structured `findings`, and explicit `limitations`. Findings have a stable code, optional run ID, and detail. Codes are `invalid_run_inventory`, `invalid_audit_history`, `missing_run_authorization`, `missing_or_unsafe_inputs`, and `invalid_run_provenance`.

A fresh empty store may have no audit file and returns zero audit records. A run store without history fails. A malformed inventory prevents reliable enumeration, so `runs_checked` is zero and the inventory finding remains visible. Audit failure does not prevent separately checking preserved run inputs where inventory can be enumerated. The checker does not examine orphan input snapshots or historical exported report files.

## Consistency and recovery

An active or recovery-required run can have consistent stored data. Their counts remain visible and require operator review; a successful result is not permission to resume or a readiness gate. Use `recover-interrupted` separately when appropriate. The checker writes no run, approval, report or audit record and does not repair, delete, renew or resume anything. Acquiring the store lock can create its `.lock` coordination file in a fresh store.

Use the command before preparing a storage migration or sharing a report bundle. Preserve findings and original files when a check fails; resolve the cause with a reviewed recovery plan. Old run records without preserved inputs will be reported instead of being supplied with invented provenance.

Local hashes and matching run IDs cannot establish authenticity against a user who can rewrite the whole store. The command does not authenticate actors or check network isolation, sensor availability, actual service health, or current authorization for a new run. Historical windows are checked at recorded run/event times; expired historical approval can remain valid evidence for a completed exercise.
