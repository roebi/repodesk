use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum SearchError {
    RootNotFound,
    IoError(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::RootNotFound => write!(f, "Search root not found"),
            SearchError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl From<std::io::Error> for SearchError {
    fn from(e: std::io::Error) -> Self {
        SearchError::IoError(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Match types
// ---------------------------------------------------------------------------

/// Whether a match was exact (substring) or fuzzy (subsequence).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchKind {
    /// Query is a case-insensitive substring of the candidate.
    Exact,
    /// All query chars appear in candidate in order (subsequence).
    Fuzzy,
}

/// A matched file path.
#[derive(Debug, Clone, PartialEq)]
pub struct FileMatch {
    /// Path relative to the search root.
    pub path: String,
    /// Higher = better match.
    pub score: u32,
    pub kind: MatchKind,
}

/// A matched line inside a file.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentMatch {
    /// Path relative to the search root.
    pub path: String,
    /// 1-based line number.
    pub line_number: usize,
    /// Full line content (trimmed of newline).
    pub line: String,
    pub score: u32,
    pub kind: MatchKind,
}

// ---------------------------------------------------------------------------
// Scorer - pure functions, no IO
// ---------------------------------------------------------------------------

/// Try to match query against candidate.
/// Returns Some((MatchKind, score)) or None.
/// Exact match checked first; fuzzy used as fallback.
pub fn score_match(query: &str, candidate: &str) -> Option<(MatchKind, u32)> {
    let q_low = query.to_lowercase();
    let c_low = candidate.to_lowercase();

    // Exact: query is a substring of candidate.
    if c_low.contains(q_low.as_str()) {
        // Score: longer query relative to candidate = tighter match.
        let score = (q_low.len() as u32 * 100) / (c_low.len() as u32).max(1);
        return Some((MatchKind::Exact, score));
    }

    // Fuzzy: all query chars appear in candidate in order (subsequence).
    fuzzy_score(&q_low, &c_low)
        .map(|score| (MatchKind::Fuzzy, score))
}

/// Compute fuzzy subsequence score.
/// Returns None if query is not a subsequence of candidate.
/// Score = sum of consecutive run lengths (longer runs = higher score).
pub fn fuzzy_score(query: &str, candidate: &str) -> Option<u32> {
    if query.is_empty() {
        return Some(0);
    }

    let q_chars: Vec<char> = query.chars().collect();
    let c_chars: Vec<char> = candidate.chars().collect();

    let mut qi = 0;
    let mut score: u32 = 0;
    let mut consecutive: u32 = 0;
    let mut last_matched: Option<usize> = None;

    for (ci, &cc) in c_chars.iter().enumerate() {
        if qi >= q_chars.len() {
            break;
        }
        if cc == q_chars[qi] {
            // Bonus for consecutive matches.
            if last_matched == Some(ci.wrapping_sub(1)) {
                consecutive += 1;
                score += consecutive;
            } else {
                consecutive = 1;
                score += 1;
            }
            last_matched = Some(ci);
            qi += 1;
        }
    }

    if qi == q_chars.len() {
        Some(score)
    } else {
        None
    }
}

/// Sort results: Exact first (score desc), then Fuzzy (score desc).
fn sort_file_matches(matches: &mut Vec<FileMatch>) {
    matches.sort_by(|a, b| {
        a.kind.cmp(&b.kind).then(b.score.cmp(&a.score))
    });
}

fn sort_content_matches(matches: &mut Vec<ContentMatch>) {
    matches.sort_by(|a, b| {
        a.kind.cmp(&b.kind).then(b.score.cmp(&a.score))
    });
}

// ---------------------------------------------------------------------------
// Filesystem walk helper
// ---------------------------------------------------------------------------

/// Recursively collect all file paths under root, skipping hidden entries.
fn walk_files(root: &Path) -> Result<Vec<std::path::PathBuf>, SearchError> {
    let mut results = Vec::new();
    walk_recursive(root, &mut results)?;
    Ok(results)
}

fn walk_recursive(
    dir: &Path,
    results: &mut Vec<std::path::PathBuf>,
) -> Result<(), SearchError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden files and directories (leading dot).
        if name_str.starts_with('.') {
            continue;
        }

        let path = entry.path();
        if path.is_dir() {
            walk_recursive(&path, results)?;
        } else if path.is_file() {
            results.push(path);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Search for files by name under root.
/// Returns FileMatch for each file whose name matches query.
/// Results: Exact first, then Fuzzy, each sorted by score desc.
pub fn search_files(root: &Path, query: &str) -> Result<Vec<FileMatch>, SearchError> {
    if !root.exists() {
        return Err(SearchError::RootNotFound);
    }

    let all_files = walk_files(root)?;
    let mut matches: Vec<FileMatch> = Vec::new();

    for path in &all_files {
        let file_name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        if let Some((kind, score)) = score_match(query, &file_name) {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            matches.push(FileMatch { path: rel_path, score, kind });
        }
    }

    sort_file_matches(&mut matches);
    Ok(matches)
}

/// Search for query string inside file contents under root.
/// Returns ContentMatch for each matching line.
/// Binary files (invalid UTF-8) are skipped silently.
/// Results: Exact first, then Fuzzy, each sorted by score desc.
pub fn search_content(root: &Path, query: &str) -> Result<Vec<ContentMatch>, SearchError> {
    if !root.exists() {
        return Err(SearchError::RootNotFound);
    }

    let all_files = walk_files(root)?;
    let mut matches: Vec<ContentMatch> = Vec::new();

    for path in &all_files {
        // Skip binary files silently.
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        for (idx, line) in content.lines().enumerate() {
            if let Some((kind, score)) = score_match(query, line) {
                matches.push(ContentMatch {
                    path: rel_path.clone(),
                    line_number: idx + 1,
                    line: line.to_string(),
                    score,
                    kind,
                });
            }
        }
    }

    sort_content_matches(&mut matches);
    Ok(matches)
}

// ---------------------------------------------------------------------------
// Unit tests - pure scorer, no IO
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. test_exact_match_found
    #[test]
    fn test_exact_match_found() {
        let result = score_match("main", "main.rs");
        assert!(result.is_some());
        let (kind, _) = result.unwrap();
        assert_eq!(kind, MatchKind::Exact);
    }

    // 2. test_exact_match_case_insens
    #[test]
    fn test_exact_match_case_insens() {
        let result = score_match("MAIN", "main.rs");
        assert!(result.is_some());
        let (kind, _) = result.unwrap();
        assert_eq!(kind, MatchKind::Exact);
    }

    // 3. test_exact_no_match
    #[test]
    fn test_exact_no_match() {
        // "xyz" is not a substring of "main.rs" and not a subsequence.
        let result = score_match("xyz", "main.rs");
        assert!(result.is_none());
    }

    // 4. test_fuzzy_match_subsequence
    #[test]
    fn test_fuzzy_match_subsequence() {
        // "mn" -> m...n in "main.rs" = subsequence, not substring
        let result = score_match("mn", "main.rs");
        assert!(result.is_some());
        let (kind, _) = result.unwrap();
        assert_eq!(kind, MatchKind::Fuzzy);
    }

    // 5. test_fuzzy_no_match
    #[test]
    fn test_fuzzy_no_match() {
        let result = score_match("zz", "main.rs");
        assert!(result.is_none());
    }

    // 6. test_fuzzy_score_consecutive
    #[test]
    fn test_fuzzy_score_consecutive() {
        // "mai" has 3 consecutive chars in "main.rs" -> higher score
        // "m_i" = "mi" has chars spread apart -> lower score
        let score_consecutive = fuzzy_score("mai", "main.rs").unwrap();
        let score_spread = fuzzy_score("mi", "main.rs").unwrap();
        // consecutive run of 3 should outscore run of 1+1
        assert!(score_consecutive > score_spread,
            "consecutive={} spread={}", score_consecutive, score_spread);
    }

    // 7. test_exact_beats_fuzzy
    #[test]
    fn test_exact_beats_fuzzy() {
        let mut matches = vec![
            FileMatch { path: "fuzzy.rs".to_string(), score: 99, kind: MatchKind::Fuzzy },
            FileMatch { path: "exact.rs".to_string(), score: 1,  kind: MatchKind::Exact },
        ];
        sort_file_matches(&mut matches);
        assert_eq!(matches[0].kind, MatchKind::Exact);
        assert_eq!(matches[1].kind, MatchKind::Fuzzy);
    }

    // 8. test_match_kind_ordering
    #[test]
    fn test_match_kind_ordering() {
        let mut matches = vec![
            FileMatch { path: "c.rs".to_string(), score: 10, kind: MatchKind::Fuzzy },
            FileMatch { path: "a.rs".to_string(), score: 50, kind: MatchKind::Exact },
            FileMatch { path: "b.rs".to_string(), score: 30, kind: MatchKind::Exact },
            FileMatch { path: "d.rs".to_string(), score: 5,  kind: MatchKind::Fuzzy },
        ];
        sort_file_matches(&mut matches);
        // Exact first, sorted by score desc
        assert_eq!(matches[0].path, "a.rs");
        assert_eq!(matches[1].path, "b.rs");
        // Fuzzy after, sorted by score desc
        assert_eq!(matches[2].path, "c.rs");
        assert_eq!(matches[3].path, "d.rs");
    }

    // Extra: empty query matches everything as fuzzy score 0
    #[test]
    fn test_empty_query_matches() {
        let result = score_match("", "main.rs");
        // empty query: fuzzy_score returns Some(0), but exact check:
        // "".contains("") is true in Rust so it will be Exact
        assert!(result.is_some());
    }

    // Extra: full filename match = high exact score
    #[test]
    fn test_full_name_exact_score() {
        let (_, score_full) = score_match("main.rs", "main.rs").unwrap();
        let (_, score_part) = score_match("main", "main.rs").unwrap();
        assert!(score_full >= score_part);
    }
}
