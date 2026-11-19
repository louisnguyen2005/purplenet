# Implementation status

As of 19 November 2026, PurpleNet has a working local mock slice:

- Rust workspace with a core library and CLI.
- Versioned scope, approval, scenario, run, and synthetic event types.
- Exact asset ID/IP and CIDR scope checks, exclusions, window expiry, scope digest binding, separate approver/operator names, and event cap.
- Local atomic JSON run storage and a hash-linked audit log with verification command.
- Data-only mock event generation. No network traffic or endpoint actions.
- A proposed libvirt isolated-network configuration for `10.77.0.0/24`, not yet deployed.
- A versioned [threat model](THREAT_MODEL.md) covering implemented and planned trust boundaries, misuse cases, and live-use gates.
- Approval schema 2 binding exact scenario and scope digests, with legacy approvals rejected.
- Preserved per-run inputs and provenance-checked JSON/Markdown mock reports with audited exports.
- A bounded JSON input reader and GitHub CI for formatting, tests, and Clippy.
- Guarded mock state transitions and abandoned-run recovery before new dispatch, plus explicit `list-runs` and `recover-interrupted` commands.
- Repository work-item templates, contribution guidance, and a [delivery roadmap](project-management/ROADMAP.md) tied to the workbook task IDs.

The demo is intentionally not a release gate. It uses a generated example approval, a documentation IP range, and local files. It has no authenticated identity, external approval service, transactional database, independent telemetry connector, cleanup/recovery workflow, or signed scenario packs. The audit chain is not externally anchored, so a writer with full state access can forge a new chain.

## Next work in dependency order

1. Confirm network owner, host route overlap, exact guest asset list, snapshot owner and first sensor. Review and, when libvirt is installed, deploy the proposed isolated-network configuration.
2. Add authenticated actor identity and a time-bound human approval workflow.
3. Move durable state and audit to transactional storage with migrations and restart behavior.
4. Add one read-only sensor connector and data quality states.
5. Add stop, cleanup, recovery-required, service health, and evidence report gates.

The authoritative plan and acceptance criteria are in [PROJECT_BLUEPRINT.md](PROJECT_BLUEPRINT.md), and progress is tracked in [the Excel workbook](PurpleNet_Implementation_Tracker.xlsx). The original workspace copy of the workbook is kept synchronized with this repository copy when implementation status is updated.
