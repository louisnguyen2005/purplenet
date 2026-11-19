# Automated verification

Run the same gates used in CI:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --release
```

The workspace currently has twenty tests on Linux. Windows omits the Unix symlink regression and runs nineteen. Tests use disposable directories and synthetic documentation addresses; they create no lab network traffic.

| Suite | Behavior exercised |
|---|---|
| Core policy | Exact scope and scenario binding; window start/end boundaries; unknown and excluded assets; duplicate inventory; stale approval; distinct operator names including padding and input length |
| Core report/recovery | Provenance, modified observations, interrupted states, recovery idempotency, no terminal restart, invalid timestamps, corrupt audit and inventory blocking dispatch |
| Core storage/audit | Corrupt and incomplete JSONL, append refusal without mutation, byte limits, bounded input reads, parent traversal, symlinked outputs and lock/audit paths on Unix |
| CLI process tests | Init, run, list, report, recover, verify; rejected changed scenario with nonzero exit and no run; concurrent processes serialized by the store lock |

The CLI tests launch the actual Cargo-built binary rather than calling only library methods. Recovery cases simulate interrupted saved state. They do not prove power-loss durability, real adapter stop behavior, or independent sensor detection.

## Continuous checks

GitHub Actions runs formatting, tests, and Clippy on Linux and Windows for pushes and pull requests. Linux also runs release-mode regression tests. The workflow supports manual dispatch and a weekly Monday check at 02:00 UTC against the current stable Rust toolchain. Each job has a fifteen-minute timeout; superseded runs for the same ref are cancelled. Checkout is pinned to a verified v6 commit.

The schedule runs verification only. It does not edit source, deploy a lab, publish a release, or approve a scope. Scheduled GitHub runs depend on repository Actions settings and platform scheduling; inspect the Actions page for actual execution results.

## Maintaining coverage

For a new behavior, add a focused regression that would fail without the change. Keep malformed-state checks separate from happy paths. Assert denied actions do not create a run or change protected evidence, and use explicit boundary timestamps rather than sleeps. Test new execution or persistence paths in both debug and release builds where appropriate.

When changing filesystem behavior, preserve platform-specific expectations with `cfg` attributes. A green Linux job does not substitute for the Windows job. The current tests assume a trusted operator controls the state directory; they do not simulate a concurrent hostile process replacing filesystem components.
