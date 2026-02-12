use crate::error::CodeviewError;
use crate::languages;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Walk a directory and collect all supported source files.
/// Respects .gitignore, .ignore, and global gitignore rules.
pub fn walk_directory(path: &Path, max_depth: Option<usize>, ext_filter: &[String]) -> Result<Vec<PathBuf>, CodeviewError> {
    // Verify path exists and is readable before walking
    if !path.is_dir() {
        return Err(CodeviewError::ReadError {
            path: path.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not a directory"),
        });
    }

    let mut builder = WalkBuilder::new(path);
    builder
        .hidden(true)          // skip hidden files/dirs
        .git_ignore(true)      // respect .gitignore
        .git_global(true)      // respect global gitignore
        .git_exclude(true)     // respect .git/info/exclude
        .sort_by_file_path(|a, b| a.cmp(b));

    // The `ignore` crate's max_depth includes the root directory itself,
    // so depth=1 means root + one level. Our API defines depth as levels
    // *below* root (depth=0 → root only, depth=1 → root + one sub-level),
    // which maps to ignore's max_depth = depth + 1.
    if let Some(d) = max_depth {
        builder.max_depth(Some(d + 1));
    }

    let mut files = Vec::new();
    for entry in builder.build() {
        let entry = entry.map_err(|e| CodeviewError::ReadError {
            path: path.display().to_string(),
            source: std::io::Error::other(e.to_string()),
        })?;

        let entry_path = entry.path();
        if entry_path.is_file() && languages::is_supported_file(entry_path) {
            if !ext_filter.is_empty() {
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    if !ext_filter.iter().any(|f| f == ext) {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            files.push(entry_path.to_path_buf());
        }
    }

    Ok(files)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn walk_empty_directory() {
        let dir = TempDir::new().unwrap();
        let files = walk_directory(dir.path(), None, &[]).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn walk_finds_rs_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("readme.md"), "# hi").unwrap();
        let files = walk_directory(dir.path(), None, &[]).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[test]
    fn walk_recurses_into_subdirs() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/lib.rs"), "").unwrap();
        let files = walk_directory(dir.path(), None, &[]).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn walk_depth_limit_zero() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/lib.rs"), "").unwrap();
        let files = walk_directory(dir.path(), Some(0), &[]).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[test]
    fn walk_depth_limit_one() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("top.rs"), "").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/nested.rs"), "").unwrap();
        let files = walk_directory(dir.path(), Some(1), &[]).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn walk_sorted_output() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("z.rs"), "").unwrap();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        let files = walk_directory(dir.path(), None, &[]).unwrap();
        assert!(files[0] < files[1]);
    }

    #[test]
    fn walk_nonexistent_dir() {
        let result = walk_directory(Path::new("/nonexistent_dir_xyz"), None, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn walk_respects_gitignore() {
        let dir = TempDir::new().unwrap();
        // Init a git repo so .gitignore is respected
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();
        fs::write(dir.path().join("keep.rs"), "").unwrap();
        fs::create_dir(dir.path().join("ignored")).unwrap();
        fs::write(dir.path().join("ignored/skip.rs"), "").unwrap();
        let files = walk_directory(dir.path(), None, &[]).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("keep.rs"));
    }

    #[test]
    fn walk_skips_hidden_dirs() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("visible.rs"), "").unwrap();
        fs::create_dir(dir.path().join(".hidden")).unwrap();
        fs::write(dir.path().join(".hidden/secret.rs"), "").unwrap();
        let files = walk_directory(dir.path(), None, &[]).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("visible.rs"));
    }

    #[test]
    fn walk_ext_filter_rs_only() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("lib.ts"), "export {}").unwrap();
        let exts = vec!["rs".to_string()];
        let files = walk_directory(dir.path(), None, &exts).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[test]
    fn walk_ext_filter_multiple() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("app.ts"), "export {}").unwrap();
        fs::write(dir.path().join("comp.tsx"), "export {}").unwrap();
        let exts = vec!["rs".to_string(), "tsx".to_string()];
        let files = walk_directory(dir.path(), None, &exts).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn walk_ext_filter_empty_means_all() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("app.ts"), "export {}").unwrap();
        let files = walk_directory(dir.path(), None, &[]).unwrap();
        assert_eq!(files.len(), 2);
    }
}
