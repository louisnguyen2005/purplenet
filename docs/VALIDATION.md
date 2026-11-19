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

## Interrupted-run recovery increment

The subsequent mock lifecycle increment was checked with formatting, workspace tests, and Clippy. Five policy tests and five workflow tests passed. Recovery cases cover all three abandoned active states, repeated recovery with no duplicate audit mutation, preserved observations, unchanged completed records, automatic recovery before a fresh run, report export for a recovered run, malformed inventory blocking dispatch, invalid audit blocking recovery, invalid terminal transitions, and rejection of backwards transition timestamps.

Recovery tests simulate interrupted state by writing active run records into disposable stores. They do not establish transactional crash durability or exercise a live endpoint. Manual recovery and automatic startup recovery use the same exclusive-lock path; emergency stop and human recovery closure remain open.

A CLI smoke check simulated an interrupted saved run, recovered it, confirmed repeated recovery returned no IDs, listed the recovery-required state, exported its report, and verified twelve audit records. The workspace and repository workbook copies were compared byte for byte after synchronization.

## Automated verification and local-state hardening increment

Local Linux checks passed: formatting, Clippy with warnings denied, and all twenty tests in both debug and release mode. The suite comprises eight policy tests, five report/recovery tests, four storage/audit tests, and three actual-binary CLI tests. New cases include window boundaries, operator-name padding, corrupt and incomplete audit append refusal, byte limits, traversal and Unix symlink rejection, and concurrent CLI processes sharing the store lock.

Audit validation now streams bounded records instead of buffering the entire log; each append verifies history and rejects an incomplete final record. The path checks and Unix directory sync improve local file handling but do not create a transaction or defend against concurrent hostile filesystem mutation. See the local-state guide for limits and compatibility.

CI is configured for Linux and Windows, with Linux release tests, manual dispatch, and weekly verification. Platform results are available on the repository Actions page. Independent operator review, production-scale performance, adversarial filesystem races, and transactional crash durability remain outside these results.

The initial Windows run exposed a test-construction difference: joining onto a verbatim Windows root normalized `..` before the store received the path. The traversal regression now constructs a raw OS string to preserve those components and exercise the rejection directly. This follows the documented [Rust PathBuf normalization behavior](https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push).

## State preflight and enhancement backlog increment

Formatting, workspace tests, and Clippy passed locally for the new state preflight. Twenty-four tests cover Linux; the four added regressions check fresh and valid stores without evidence mutation, missing inputs and audit authorization links, combined audit/provenance failures and malformed inventory, and structured CLI failure output without repair. The checker reports active/recovery-required counts without treating stored-data consistency as permission to resume.

The workbook now has 15 Proposed enhancements with acceptance evidence and dependencies. Original implementation totals remain 81 tasks, 5 Finished and 18 In Progress. Dashboard text was refreshed to reflect implemented work and the separate proposal backlog. The new backlog and dashboard were rendered, and formula inspection found no errors.
