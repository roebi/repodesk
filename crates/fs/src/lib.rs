use repodesk_editor::Buffer;
use std::fs;
use std::io;
use std::path::Path;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All errors that can occur during file system operations.
/// No panics - every failure produces an FsError.
#[derive(Debug, Clone, PartialEq)]
pub enum FsError {
    /// The path does not exist.
    NotFound,
    /// The operation was denied by the OS.
    PermissionDenied,
    /// A file operation was attempted on a directory, or vice versa.
    IsDirectory,
    /// Any other IO error.
    IoError(String),
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::NotFound => write!(f, "File not found"),
            FsError::PermissionDenied => write!(f, "Permission denied"),
            FsError::IsDirectory => write!(f, "Path is a directory"),
            FsError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl From<io::Error> for FsError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => FsError::NotFound,
            io::ErrorKind::PermissionDenied => FsError::PermissionDenied,
            _ => FsError::IoError(e.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Entry types
// ---------------------------------------------------------------------------

/// Whether a directory entry is a file or a directory.
#[derive(Debug, Clone, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
}

/// A single entry returned by list_dir.
#[derive(Debug, Clone, PartialEq)]
pub struct DirEntry {
    /// File or directory name (not full path).
    pub name: String,
    pub kind: EntryKind,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Read a file from disk into a Buffer.
/// Returns FsError::IsDirectory if path is a directory.
pub fn read_file(path: impl AsRef<Path>) -> Result<Buffer, FsError> {
    let path = path.as_ref();

    if path.is_dir() {
        return Err(FsError::IsDirectory);
    }

    let content = fs::read_to_string(path)?;
    Ok(Buffer::from_str(&content))
}

/// Write a Buffer's contents to disk.
/// Creates the file if it does not exist; overwrites if it does.
pub fn write_file(path: impl AsRef<Path>, buffer: &Buffer) -> Result<(), FsError> {
    let path = path.as_ref();

    if path.is_dir() {
        return Err(FsError::IsDirectory);
    }

    let content = buffer.lines().join("\n");
    fs::write(path, content)?;
    Ok(())
}

/// List the entries of a directory, sorted by name.
/// Returns FsError::NotFound if the path does not exist.
/// Returns FsError::IsDirectory (reused as "wrong kind") if path is a file.
pub fn list_dir(path: impl AsRef<Path>) -> Result<Vec<DirEntry>, FsError> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(FsError::NotFound);
    }

    if path.is_file() {
        return Err(FsError::IsDirectory);
    }

    let mut entries: Vec<DirEntry> = fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let kind = if entry.path().is_dir() {
                EntryKind::Directory
            } else {
                EntryKind::File
            };
            Some(DirEntry { name, kind })
        })
        .collect();

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Detect whether a path is a File or Directory.
/// Returns FsError::NotFound if the path does not exist.
pub fn entry_kind(path: impl AsRef<Path>) -> Result<EntryKind, FsError> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(FsError::NotFound);
    }

    if path.is_dir() {
        Ok(EntryKind::Directory)
    } else {
        Ok(EntryKind::File)
    }
}
