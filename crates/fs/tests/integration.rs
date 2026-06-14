/// Integration tests for repodesk-fs.
/// All tests use real temp directories and files.
/// No mocks, no trait abstractions.
use repodesk_fs::{entry_kind, list_dir, read_file, write_file, EntryKind, FsError};
use repodesk_editor::Buffer;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Helper: isolated temp directory per test
// ---------------------------------------------------------------------------

fn temp_dir(label: &str) -> PathBuf {
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = std::env::temp_dir().join(format!("repodesk-fs-{}-{}", label, ns));
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------------------
// read_file tests
// ---------------------------------------------------------------------------

// 1. test_read_file_content
#[test]
fn test_read_file_content() {
    let dir = temp_dir("read-content");
    let path = dir.join("hello.rs");
    fs::write(&path, "fn main() {}\nprintln!(\"hi\");").unwrap();

    let buf = read_file(&path).unwrap();
    assert_eq!(buf.lines().len(), 2);
    assert_eq!(buf.lines()[0], "fn main() {}");
    assert_eq!(buf.lines()[1], "println!(\"hi\");");

    fs::remove_dir_all(&dir).ok();
}

// 2. test_read_file_empty
#[test]
fn test_read_file_empty() {
    let dir = temp_dir("read-empty");
    let path = dir.join("empty.rs");
    fs::write(&path, "").unwrap();

    let buf = read_file(&path).unwrap();
    // Buffer::from_str("") returns one empty line
    assert_eq!(buf.lines(), &[""]);

    fs::remove_dir_all(&dir).ok();
}

// 3. test_read_file_not_found
#[test]
fn test_read_file_not_found() {
    let path = std::env::temp_dir().join("repodesk-fs-does-not-exist-99999.rs");
    let result = read_file(&path);
    assert_eq!(result, Err(FsError::NotFound));
}

// 4. test_read_file_is_directory
#[test]
fn test_read_file_is_directory() {
    let dir = temp_dir("read-isdir");
    let result = read_file(&dir);
    assert_eq!(result, Err(FsError::IsDirectory));
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// write_file tests
// ---------------------------------------------------------------------------

// 5. test_write_file_creates
#[test]
fn test_write_file_creates() {
    let dir = temp_dir("write-creates");
    let path = dir.join("new.rs");

    assert!(!path.exists());
    let buf = Buffer::from_str("fn foo() {}");
    write_file(&path, &buf).unwrap();
    assert!(path.exists());

    fs::remove_dir_all(&dir).ok();
}

// 6. test_write_file_content
#[test]
fn test_write_file_content() {
    let dir = temp_dir("write-content");
    let path = dir.join("lib.rs");

    let buf = Buffer::from_str("line one\nline two\nline three");
    write_file(&path, &buf).unwrap();

    // Read back and verify
    let read_back = read_file(&path).unwrap();
    assert_eq!(read_back.lines(), buf.lines());

    fs::remove_dir_all(&dir).ok();
}

// 7. test_write_file_overwrites
#[test]
fn test_write_file_overwrites() {
    let dir = temp_dir("write-overwrite");
    let path = dir.join("over.rs");

    let first = Buffer::from_str("first content");
    write_file(&path, &first).unwrap();

    let second = Buffer::from_str("second content");
    write_file(&path, &second).unwrap();

    let read_back = read_file(&path).unwrap();
    assert_eq!(read_back.lines()[0], "second content");

    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// list_dir tests
// ---------------------------------------------------------------------------

// 8. test_list_dir_entries
#[test]
fn test_list_dir_entries() {
    let dir = temp_dir("list-entries");
    fs::write(dir.join("alpha.rs"), "").unwrap();
    fs::write(dir.join("beta.rs"), "").unwrap();

    let entries = list_dir(&dir).unwrap();
    assert_eq!(entries.len(), 2);

    fs::remove_dir_all(&dir).ok();
}

// 9. test_list_dir_sorted
#[test]
fn test_list_dir_sorted() {
    let dir = temp_dir("list-sorted");
    fs::write(dir.join("zz.rs"), "").unwrap();
    fs::write(dir.join("aa.rs"), "").unwrap();
    fs::write(dir.join("mm.rs"), "").unwrap();

    let entries = list_dir(&dir).unwrap();
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["aa.rs", "mm.rs", "zz.rs"]);

    fs::remove_dir_all(&dir).ok();
}

// 10. test_list_dir_not_found
#[test]
fn test_list_dir_not_found() {
    let path = std::env::temp_dir().join("repodesk-fs-no-such-dir-88888");
    let result = list_dir(&path);
    assert_eq!(result, Err(FsError::NotFound));
}

// 11. test_list_dir_is_file
#[test]
fn test_list_dir_is_file() {
    let dir = temp_dir("list-isfile");
    let path = dir.join("afile.rs");
    fs::write(&path, "").unwrap();

    let result = list_dir(&path);
    assert_eq!(result, Err(FsError::IsDirectory));

    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// entry_kind tests
// ---------------------------------------------------------------------------

// 12. test_entry_kind_file
#[test]
fn test_entry_kind_file() {
    let dir = temp_dir("kind-file");
    let path = dir.join("main.rs");
    fs::write(&path, "").unwrap();

    assert_eq!(entry_kind(&path).unwrap(), EntryKind::File);

    fs::remove_dir_all(&dir).ok();
}

// 13. test_entry_kind_directory
#[test]
fn test_entry_kind_directory() {
    let dir = temp_dir("kind-dir");
    assert_eq!(entry_kind(&dir).unwrap(), EntryKind::Directory);
    fs::remove_dir_all(&dir).ok();
}

// 14. test_entry_kind_not_found
#[test]
fn test_entry_kind_not_found() {
    let path = std::env::temp_dir().join("repodesk-fs-no-such-entry-77777");
    assert_eq!(entry_kind(&path), Err(FsError::NotFound));
}

// 15. test_list_dir_includes_subdirs
#[test]
fn test_list_dir_includes_subdirs() {
    let dir = temp_dir("list-subdirs");
    fs::write(dir.join("file.rs"), "").unwrap();
    fs::create_dir(dir.join("subdir")).unwrap();

    let entries = list_dir(&dir).unwrap();
    let file_entry = entries.iter().find(|e| e.name == "file.rs").unwrap();
    let dir_entry  = entries.iter().find(|e| e.name == "subdir").unwrap();

    assert_eq!(file_entry.kind, EntryKind::File);
    assert_eq!(dir_entry.kind,  EntryKind::Directory);

    fs::remove_dir_all(&dir).ok();
}
