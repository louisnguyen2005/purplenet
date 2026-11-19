# Contributing to PurpleNet

Start with the [roadmap](docs/project-management/ROADMAP.md), [folder guide](docs/project-management/FOLDER_GUIDE.md), and [tracker](docs/PurpleNet_Implementation_Tracker.xlsx). Reference a task ID in your issue or change description. Explain the behavior, failure cases, schema compatibility, and evidence for completion.

Run `cargo fmt --all -- --check`, `cargo test --workspace --locked`, and `cargo clippy --workspace --all-targets --locked -- -D warnings`. Changes to policy, run transitions, input parsing, or evidence handling need focused failure-path coverage. Update the relevant user guide and validation record.

Use the [automated testing guide](docs/TESTING.md) for suite coverage and release-mode checks. CI must pass on Linux and Windows; Unix-specific filesystem tests are explicitly gated.

Keep examples synthetic and local. Do not commit state directories, credentials, actual authorization records, or unrelated personal data. The current executable supports mock validation only. Live adapter work requires the architectural and operational gates recorded in the threat model and tracker.

For workbook edits, preserve task IDs and acceptance criteria, explain partial work in Notes, and mark Finished only when the complete criterion is evidenced. The repository workbook and original workspace workbook must contain the same saved bytes after a synchronized update.
