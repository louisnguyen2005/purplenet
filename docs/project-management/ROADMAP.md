# Delivery roadmap

This roadmap groups work into reviewable increments. The [Excel tracker](../PurpleNet_Implementation_Tracker.xlsx) owns task IDs, acceptance criteria, evidence, and completion status. The [implementation status](../IMPLEMENTATION_STATUS.md) describes runnable capabilities. Roadmap order expresses dependency and engineering intent, not a delivery date or owner approval.

| Increment | Tracker references | Deliverable | Exit evidence |
|---|---|---|---|
| Mock evidence baseline | P1-003, P2-004, P2-007 | Exact scenario approval and preserved-input reports | Policy regressions, mock report workflow, documented limitations |
| Interrupted-run recovery | P1-005, P2-008 | Guarded transitions, inventory command, startup recovery | Recovery idempotency, no terminal restart, malformed inventory rejection |
| Lab readiness | P0-002 through P0-010 | Named owner, actual VM inventory, topology, sensor and retention decisions | Reviewed phase-zero record; proposed XML alone does not pass |
| Durable controller | P1-007 through P1-012, P2-003 | Authenticated actors, transactional state/audit, leases and limits | Restart and transaction-failure evidence with no silent execution |
| Sensor evidence | P3-001 through P3-009 | One read-only sensor connector with explicit data-quality states | Fixture replay, source provenance, outage and parser-error reports |
| Governed operations | P4 and P5 | Human approval, stop, cleanup, health and recovery closure | Role boundaries, stop acknowledgement and independent health checks |
| Report and release maturity | P6 and P7 | Privacy controls, reproducible sensor reports, release provenance | Export audit, restore record and release gate review |
| Optional AI | P8 | Read-only evidence explanation | Demonstrated value, redaction, no authorization or dispatch permissions |

The next implementation priority after mock recovery is the durable controller design: choose a transactional storage contract and actor model before introducing asynchronous execution or live adapters. Lab ownership and actual inventory can proceed alongside this work, but software development cannot supply missing human approval.

The workbook's **Enhancement Backlog** expands this roadmap into 15 proposals. Start with ENH-001 (transactional migration/rollback) and ENH-002 (authenticated actors). The new `check-state` preflight supplies an integrity check for existing source records before migration design. Later proposals cover reviewed scenario promotion, sensor replay, scope drift, independent audit checkpoints, detection comparison, redaction, restore rehearsal, idempotency, fault injection, performance budgets, policy explanations, a local evidence viewer, and read-only AI evaluation. All remain Proposed until separately accepted into implementation work; their presence does not imply approval or delivery.

## Work-item procedure

1. Select an existing task ID and state the specific behavior to deliver. Break large tasks into a bounded increment without weakening the original acceptance criterion.
2. Record code boundaries, schema impact, failure behavior, and verification in an issue or local change note.
3. Implement a complete reviewable increment; add tests for the failure modes it introduces.
4. Run formatting, workspace tests and Clippy. Keep the result and remaining limitations in the validation record.
5. Update English documentation and the workbook. Use Finished only when the entire task acceptance criterion is met; partial implementations remain In Progress with evidence and remaining work.
6. Synchronize the workspace workbook and repository copy, review the staged files for private data, commit, push, and inspect CI.

## Completion and ownership

An implemented engineering artifact can be Finished while its independent review or phase gate remains open. CI success does not supply network ownership, privacy approval, production readiness, or authorization to operate a live adapter. Do not enter an invented owner, target address, approval, or review date to complete a task.
