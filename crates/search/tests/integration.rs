/// Integration tests for repodesk-search.
/// All tests use real temp directories and files.
use repodesk_search::{search_content, search_files, MatchKind, SearchError};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn temp_dir(label: &str) -> PathBuf {
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = std::env::temp_dir().join(format!("repodesk-search-{}-{}", label, ns));
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------------------
// search_files integration tests
// ---------------------------------------------------------------------------

// 9. test_search_files_exact
#[test]
fn test_search_files_exact() {
    let dir = temp_dir("files-exact");
    fs::write(dir.join("main.rs"), "").unwrap();
    fs::write(dir.join("lib.rs"), "").unwrap();

    let results = search_files(&dir, "main").unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|m| m.path.contains("main.rs")));
    assert_eq!(results[0].kind, MatchKind::Exact);

    fs::remove_dir_all(&dir).ok();
}

// 10. test_search_files_fuzzy
#[test]
fn test_search_files_fuzzy() {
    let dir = temp_dir("files-fuzzy");
    // "mlb.rs" - "ml" is not a substring but is a subsequence
    fs::write(dir.join("mlb.rs"), "").unwrap();
    fs::write(dir.join("other.rs"), "").unwrap();

    let results = search_files(&dir, "ml").unwrap();
    assert!(!results.is_empty());
    let first = &results[0];
    assert!(first.path.contains("mlb.rs"));
    // "ml" IS a substring of "mlb.rs" -> Exact
    assert_eq!(first.kind, MatchKind::Exact);

    // Use a query that is truly only fuzzy: "mb" (m..b in mlb.rs)
    let results2 = search_files(&dir, "mb").unwrap();
    assert!(!results2.is_empty());
    assert!(results2.iter().any(|m| m.kind == MatchKind::Fuzzy));

    fs::remove_dir_all(&dir).ok();
}

// 11. test_search_files_no_results
#[test]
fn test_search_files_no_results() {
    let dir = temp_dir("files-no-results");
    fs::write(dir.join("main.rs"), "").unwrap();

    let results = search_files(&dir, "zzz").unwrap();
    assert!(results.is_empty());

    fs::remove_dir_all(&dir).ok();
}

// 12. test_search_files_hidden_skip
#[test]
fn test_search_files_hidden_skip() {
    let dir = temp_dir("files-hidden");
    fs::write(dir.join(".hidden"), "").unwrap();
    fs::write(dir.join(".gitignore"), "").unwrap();
    fs::write(dir.join("visible.rs"), "").unwrap();

    // Query "hidden" should NOT find .hidden
    let results = search_files(&dir, "hidden").unwrap();
    assert!(
        results.is_empty(),
        "hidden files should be skipped, got: {:?}",
        results
    );

    // Visible file is found
    let results2 = search_files(&dir, "visible").unwrap();
    assert!(!results2.is_empty());

    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// search_content integration tests
// ---------------------------------------------------------------------------

// 13. test_search_content_exact
#[test]
fn test_search_content_exact() {
    let dir = temp_dir("content-exact");
    fs::write(dir.join("main.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();

    let results = search_content(&dir, "main").unwrap();
    assert!(!results.is_empty());
    assert!(results[0].line.contains("main"));
    assert_eq!(results[0].kind, MatchKind::Exact);

    fs::remove_dir_all(&dir).ok();
}

// 14. test_search_content_fuzzy
#[test]
fn test_search_content_fuzzy() {
    let dir = temp_dir("content-fuzzy");
    // "fn" is exact, but "fmn" would be fuzzy in "fn main()"
    fs::write(dir.join("main.rs"), "fn main() {}\n").unwrap();

    // "fm" is not a substring of "fn main() {}" but f..m is a subsequence
    let results = search_content(&dir, "fm").unwrap();
    assert!(!results.is_empty(), "expected fuzzy match for 'fm'");
    assert_eq!(results[0].kind, MatchKind::Fuzzy);

    fs::remove_dir_all(&dir).ok();
}

// 15. test_search_content_line_num
#[test]
fn test_search_content_line_num() {
    let dir = temp_dir("content-linenum");
    fs::write(
        dir.join("lib.rs"),
        "use std::fs;\n\npub fn hello() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    let results = search_content(&dir, "hello").unwrap();
    assert!(!results.is_empty());
    // "pub fn hello()" is on line 3, "println!" on line 4
    let line_nums: Vec<usize> = results.iter().map(|m| m.line_number).collect();
    assert!(line_nums.contains(&3) || line_nums.contains(&4),
        "expected line 3 or 4, got: {:?}", line_nums);

    fs::remove_dir_all(&dir).ok();
}

// 16. test_search_content_multi_file
#[test]
fn test_search_content_multi_file() {
    let dir = temp_dir("content-multifile");
    fs::write(dir.join("a.rs"), "fn hello_a() {}\n").unwrap();
    fs::write(dir.join("b.rs"), "fn hello_b() {}\n").unwrap();
    fs::write(dir.join("c.rs"), "fn unrelated() {}\n").unwrap();

    let results = search_content(&dir, "hello").unwrap();
    let paths: Vec<&str> = results.iter().map(|m| m.path.as_str()).collect();

    assert!(paths.iter().any(|p| p.contains("a.rs")), "a.rs missing: {:?}", paths);
    assert!(paths.iter().any(|p| p.contains("b.rs")), "b.rs missing: {:?}", paths);
    assert!(!paths.iter().any(|p| p.contains("c.rs")), "c.rs should not match: {:?}", paths);

    fs::remove_dir_all(&dir).ok();
}

// 17. test_search_content_binary_skip
#[test]
fn test_search_content_binary_skip() {
    let dir = temp_dir("content-binary");
    // Write a file with invalid UTF-8 bytes (binary).
    let binary_content: Vec<u8> = vec![0xFF, 0xFE, 0x00, 0x01, 0x80, 0x90];
    fs::write(dir.join("binary.bin"), &binary_content).unwrap();
    fs::write(dir.join("text.rs"), "fn main() {}\n").unwrap();

    // Should not error - binary file is skipped silently.
    let results = search_content(&dir, "main");
    assert!(results.is_ok(), "binary file caused error: {:?}", results);
    let matches = results.unwrap();
    assert!(!matches.is_empty(), "text.rs match should still be found");

    fs::remove_dir_all(&dir).ok();
}

// 18. test_search_root_not_found
#[test]
fn test_search_root_not_found() {
    let missing = std::env::temp_dir().join("repodesk-search-no-such-root-11111");

    let r1 = search_files(&missing, "main");
    assert_eq!(r1, Err(SearchError::RootNotFound));

    let r2 = search_content(&missing, "main");
    assert_eq!(r2, Err(SearchError::RootNotFound));
}
