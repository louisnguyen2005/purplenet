# Validation record: 19 November 2026

Validated the approval-v2 and mock-report implementation locally before its first GitHub publication:

- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --locked`: five policy tests and one report workflow integration test passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- CLI smoke workflow in a disposable state directory: init, validate, run-mock, report-run, and verify-audit passed. Two synthetic events and JSON/Markdown reports were produced; four audit records verified.
- The policy regression cases reject scenario content changed after approval, legacy approval schema, an unknown named asset, expiry, self-approval, and stale scope digests.
- The report integration case confirms synthetic-only output and rejects a modified stored observation.
- Workbook formulas recalculated without reported formula errors; the affected work-item view was rendered and reviewed. P2-007 is Finished; partially implemented provenance and CLI items remain In Progress.

These checks cover the local mock slice. They do not validate independent detection, authenticated authorization, VM isolation, stop/recovery behavior, transactional crash recovery, or production readiness. Those capabilities remain open in the tracker.
