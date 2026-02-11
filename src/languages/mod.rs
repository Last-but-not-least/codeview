pub mod rust;
pub mod typescript;
pub mod python;
pub mod javascript;

use crate::error::CodeviewError;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
    Python,
    JavaScript,
    Jsx,
}

/// Detect language from file extension
pub fn detect_language(path: &Path) -> Result<Language, CodeviewError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| CodeviewError::NoExtension(path.display().to_string()))?;

    match extension {
        "rs" => Ok(Language::Rust),
        "ts" => Ok(Language::TypeScript),
        "tsx" => Ok(Language::Tsx),
        "js" => Ok(Language::JavaScript),
        "jsx" => Ok(Language::Jsx),
        "py" => Ok(Language::Python),
        _ => Err(CodeviewError::UnsupportedExtension(extension.to_string())),
    }
}

/// Check if a file should be processed based on its extension
pub fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| matches!(ext, "rs" | "ts" | "tsx" | "js" | "jsx" | "py"))
        .unwrap_or(false)
}

/// Get tree-sitter Language for a given language enum
pub fn ts_language(lang: Language) -> tree_sitter::Language {
    match lang {
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        Language::JavaScript | Language::Jsx => tree_sitter_javascript::LANGUAGE.into(),
        Language::Python => tree_sitter_python::LANGUAGE.into(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detect_language_rust() {
        let lang = detect_language(Path::new("foo.rs")).unwrap();
        assert_eq!(lang, Language::Rust);
    }

    #[test]
    fn detect_language_unsupported() {
        let err = detect_language(Path::new("foo.rb")).unwrap_err();
        assert!(err.to_string().contains("Unsupported"));
    }

    #[test]
    fn detect_language_no_extension() {
        let err = detect_language(Path::new("Makefile")).unwrap_err();
        assert!(err.to_string().contains("No file extension"));
    }

    #[test]
    fn detect_language_nested_path() {
        let lang = detect_language(Path::new("/a/b/c/main.rs")).unwrap();
        assert_eq!(lang, Language::Rust);
    }

    #[test]
    fn is_supported_file_rs() {
        assert!(is_supported_file(Path::new("lib.rs")));
        assert!(is_supported_file(Path::new("/deep/path/mod.rs")));
    }

    #[test]
    fn is_supported_file_not_rs() {
        assert!(!is_supported_file(Path::new("main.rb")));
        assert!(!is_supported_file(Path::new("README.md")));
        assert!(!is_supported_file(Path::new("Makefile")));
        assert!(!is_supported_file(Path::new(".hidden")));
        assert!(is_supported_file(Path::new("app.ts")));
        assert!(is_supported_file(Path::new("component.tsx")));
    }

    #[test]
    fn is_supported_file_no_extension() {
        assert!(!is_supported_file(Path::new("noext")));
    }
}
