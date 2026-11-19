# Mock evidence reports

The `report-run` command creates a JSON report and a Markdown companion under `state/reports/`. Each report embeds the preserved scope, approval, scenario, run record, and synthetic observations. It includes SHA-256 digests of the serialized run and input objects and the report generator version.

```sh
cargo run -p purplenet-cli -- init
cargo run -p purplenet-cli -- run-mock --scope state/examples/scope.json --approval state/examples/approval.json --scenario state/examples/scenario.json --actor example-operator
# Replace RUN_ID with the ID printed by run-mock.
cargo run -p purplenet-cli -- report-run RUN_ID --actor example-reviewer
cargo run -p purplenet-cli -- verify-audit
```

The exporter verifies the local audit chain, checks the preserved input digests against the run, re-evaluates authorization at recorded dispatch times, and compares observations with the approved synthetic event sequence. A completed run must contain every expected synthetic event. Export requests and completions are audited. A crash can leave a request without completion, or only one output file; re-export under the same run ID to regenerate both files. The pair and the audit are not a single transaction.

Reports say `evidence_kind: synthetic_only` and `detection_result: not_evaluated`. A completed mock run is not evidence that an independent sensor detected an exercise. The hashes cover normalized Rust serialization, not the original input-file bytes. Local users with write access can forge the store and rebuild the audit chain; these reports do not provide signatures or an externally anchored chain of custody. Reports include their input records, so review scope metadata before sharing them.

## Approval schema migration

Approval schema **2** adds a required `scenario_sha256` computed by `Scenario::digest()`. Policy checks both the scenario ID and digest. Version 1 approval files are rejected. Re-run `init` in a new state directory to create fresh demonstration fixtures. For separately reviewed inputs, regenerate the approval through the approving person's review process after any scenario or scope change. Editing JSON names is not identity authentication.

Older state directories lack the `inputs` directory. Running `init` adds it but also replaces demonstration files. Back up old state first, or use a fresh `--state` directory. Old run records without preserved inputs cannot be exported by this command; the exporter does not invent historical provenance.

## Remaining work

Authenticated approval, transactional persistence, restricted exports, independent sensor evidence, framework mapping versions, recovery states, and external audit anchoring are still planned. See the implementation tracker for acceptance criteria.
