use purplenet_core::{audit, store::Store};
use std::{fs, path::PathBuf};

struct Fixture {
    root: PathBuf,
    store: Store,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("purplenet-storage-{}", uuid::Uuid::new_v4()));
        let store = Store::init(&root).unwrap();
        let root = store.root.clone();
        Self { root, store }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn append_rejects_corrupt_or_incomplete_history_without_changing_it() {
    let f = Fixture::new();
    audit::append(&f.store, 1, "actor", "test", "allowed", "fixture", None).unwrap();
    let path = f.root.join("audit.jsonl");
    let original = fs::read(&path).unwrap();
    let mut tampered = original.clone();
    let offset = tampered.windows(7).position(|w| w == b"fixture").unwrap();
    tampered[offset] = b'X';
    for bytes in [tampered, original[..original.len() - 1].to_vec()] {
        fs::write(&path, &bytes).unwrap();
        assert!(audit::verify(&f.store).is_err());
        assert!(audit::append(&f.store, 2, "actor", "test", "allowed", "next", None).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn audit_limits_reject_oversized_record_and_log() {
    let f = Fixture::new();
    let reason = "x".repeat(audit::MAX_RECORD_BYTES);
    assert!(audit::append(&f.store, 1, "actor", "test", "allowed", &reason, None).is_err());
    assert!(!f.root.join("audit.jsonl").exists());
    let file = fs::File::create(f.root.join("audit.jsonl")).unwrap();
    file.set_len(audit::MAX_LOG_BYTES + 1).unwrap();
    assert!(audit::verify(&f.store).is_err());
}

#[test]
fn bounded_json_reads_and_traversal_do_not_modify_existing_state() {
    let f = Fixture::new();
    let path = f.root.join("runs").join("example.json");
    f.store.write(&path, &vec![1, 2, 3]).unwrap();
    // PathBuf::join normalizes `..` for Windows verbatim roots before the
    // store sees it. Build a raw OsString to exercise the actual path guard.
    let mut traversal = f.root.as_os_str().to_os_string();
    traversal.push(std::path::MAIN_SEPARATOR_STR);
    traversal.push("runs");
    traversal.push(std::path::MAIN_SEPARATOR_STR);
    traversal.push("..");
    traversal.push(std::path::MAIN_SEPARATOR_STR);
    traversal.push("escaped.json");
    assert!(f
        .store
        .write_bytes(&PathBuf::from(traversal), b"unsafe")
        .is_err());
    assert!(!f.root.join("escaped.json").exists());
    assert_eq!(Store::read::<Vec<u8>>(&path).unwrap(), vec![1, 2, 3]);
    let oversized = f.root.join("examples/oversized.json");
    fs::write(&oversized, vec![b' '; 1_000_001]).unwrap();
    assert!(Store::read::<serde_json::Value>(&oversized).is_err());
    assert!(fs::read_dir(f.root.join("runs")).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));
}

#[cfg(unix)]
#[test]
fn symlinked_directory_lock_and_audit_are_rejected() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let outside = Fixture::new();
    symlink(&outside.root, f.root.join("redirect")).unwrap();
    assert!(f
        .store
        .write_bytes(&f.root.join("redirect/escaped.json"), b"unsafe")
        .is_err());
    assert!(!outside.root.join("escaped.json").exists());
    let target = outside.root.join("sentinel");
    fs::write(&target, b"unchanged").unwrap();
    symlink(&target, f.root.join(".lock")).unwrap();
    assert!(f.store.lock().is_err());
    symlink(&target, f.root.join("audit.jsonl")).unwrap();
    assert!(audit::append(&f.store, 1, "actor", "test", "allowed", "fixture", None).is_err());
    assert_eq!(fs::read(target).unwrap(), b"unchanged");
    fs::remove_dir(f.root.join("reports")).unwrap();
    symlink(&outside.root, f.root.join("reports")).unwrap();
    assert!(Store::init(&f.root).is_err());
}
