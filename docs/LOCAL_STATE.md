# Local state constraints

PurpleNet's current store is a single-operator mock-stage file store. Cooperative writers share an exclusive lock. CLI audit verification and run inspection also acquire the lock, so they do not read a cooperative write in progress. It is not a transactional database or a boundary against an attacker with write access to the state directory.

## Bounds and failure behavior

| Input | Limit or rule |
|---|---|
| JSON input file | 1,000,000 bytes, read with a bound before parsing |
| Audit JSONL record | 65,536 encoded bytes including the terminating newline |
| Audit log | 32 MiB encoded bytes |
| Run inventory | At most 10,000 JSON files; malformed records block new dispatch |
| Output path | Normal path components under the canonical state root; traversal and existing symlink components rejected |

Audit verification streams one bounded record at a time. Every append checks the full existing chain and refuses unknown record fields, wrong hashes or sequences, oversized records/logs, and incomplete trailing records. This removes whole-log buffering and catches corruption before extending history. Verification remains linear in log length, and each append currently repeats that scan; high-volume ingestion will require transactional storage or a reviewed checkpoint design. There is no benchmark claim of higher throughput.

An audit-limit or integrity failure blocks the attempted operation; logs are never truncated automatically. Preserve the state directory and investigate before migrating or archiving. Automatic rotation, external anchoring, and a supported archive-verification protocol remain planned. An existing valid final JSON record without a newline is now rejected as incomplete.

## Writes and filesystem assumptions

State writes use a new temporary file in the destination directory, flush its contents, and rename it into place. Unix builds also sync the parent directory after rename. Temporary files are removed after a failed write when possible. Each file update is separate from audit writes; there is no atomic transaction spanning both files, and Windows does not use the Unix directory-sync path.

Initialization rejects symlinked state subdirectories. Writes reject parent traversal and existing symlink components, including lock and audit output files. These are preflight checks: they do not prevent a hostile local writer from replacing filesystem objects between a check and an open. Keep the state directory and its parents restricted to the trusted operator. Hard links and malicious filesystem races require a stronger storage/OS design before multi-user deployment.

Input-file hashes and local audit links provide consistency checks. A local writer can replace the records and generate a new history. Reports expose this limitation and label mock observations as synthetic.
