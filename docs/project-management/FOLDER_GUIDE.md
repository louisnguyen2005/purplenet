# Repository folder responsibilities

| Location | Responsibility | Change guidance |
|---|---|---|
| `apps/purplenet-cli/src` | Parse commands and present results | Keep policy and lifecycle behavior in the core library. |
| `crates/purplenet-core/src/domain.rs` | Versioned types, validation, legal transitions | Document compatibility for serialized changes. |
| `crates/purplenet-core/src/policy.rs` | Deterministic authorization | Add denial cases for new approval or scope behavior. |
| `crates/purplenet-core/src/runner.rs` | Synthetic execution | Preserve exact approved inputs and recheck before dispatch. |
| `crates/purplenet-core/src/lifecycle.rs` | Audited transition persistence and recovery | Require the store lock and explain interruption behavior. |
| `crates/purplenet-core/src/store.rs` | Local persistence, inventory and locking | Keep bounds and malformed-state behavior explicit. |
| `crates/purplenet-core/src/audit.rs` | Local hash-linked decisions | Do not describe local hashes as external tamper protection. |
| `crates/purplenet-core/src/report.rs` | Evidence consistency checks and mock export | Keep synthetic results distinct from independent detection evidence. |
| `crates/purplenet-core/tests` | Workflow and fault regressions | Exercise observable behavior and concrete failure risks. |
| `deploy/libvirt` | Proposed isolated lab configuration | Keep owner, inventory and deployment state explicit. |
| `docs` | User guides, design, validation and workbook | Update the relevant guide with behavior and schema changes. |
| `docs/project-management` | Delivery ordering and repository responsibilities | Reference tracker IDs; avoid a second editable task-status database. |
| `.github` | CI and work-item templates | Keep workflow permissions minimal. |
| `state` (ignored) | Local examples, runs, preserved inputs, audit and reports | Keep credentials, real scope records and private evidence out of Git. |

Add a crate when a real dependency or process boundary requires it. The modular monolith can retain clear module responsibilities while the product is small. A future connector process should have its own permission boundary and protocol; a folder split alone does not isolate its privileges.
