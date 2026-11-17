# Implementation status

As of 17 November 2026, PurpleNet has a working local mock slice:

- Rust workspace with a core library and CLI.
- Versioned scope, approval, scenario, run, and synthetic event types.
- Exact asset ID/IP and CIDR scope checks, exclusions, window expiry, scope digest binding, separate approver/operator names, and event cap.
- Local atomic JSON run storage and a hash-linked audit log with verification command.
- Data-only mock event generation. No network traffic or endpoint actions.

The demo is intentionally not a release gate. It uses a generated example approval, a documentation IP range, and local files. It has no authenticated identity, external approval service, transactional database, independent telemetry connector, cleanup/recovery workflow, or signed scenario packs. The audit chain is not externally anchored, so a writer with full state access can forge a new chain.

## Next work in dependency order

1. Confirm network owner, isolated lab topology, exact asset list and first sensor. Record them in the tracker and an approved scope document.
2. Add authenticated actor identity and a time-bound human approval workflow.
3. Move durable state and audit to transactional storage with migrations and restart behavior.
4. Add one read-only sensor connector and data quality states.
5. Add stop, cleanup, recovery-required, service health, and evidence report gates.

The authoritative plan and acceptance criteria are in `../reports/PurpleNet project blueprint.md` relative to the workspace root, and progress is tracked in `../outputs/purplenet_tracker_20261117/PurpleNet_Implementation_Tracker.xlsx`.
