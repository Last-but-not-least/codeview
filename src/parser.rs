use crate::error::CodeviewError;
use crate::languages::Language;
use tree_sitter::{Parser, Tree};

/// Parse source code into a Tree-sitter AST
pub fn parse(source: &str, language: Language) -> Result<Tree, CodeviewError> {
    let mut parser = Parser::new();
    
    let ts_language = match language {
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        Language::Python => tree_sitter_python::LANGUAGE.into(),
        Language::JavaScript | Language::Jsx => tree_sitter_javascript::LANGUAGE.into(),
    };

    parser
        .set_language(&ts_language)
        .map_err(|e| CodeviewError::ParseError(format!("Failed to set language: {}", e)))?;

    parser
        .parse(source, None)
        .ok_or_else(|| CodeviewError::ParseError("Failed to parse source code".to_string()))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;

    #[test]
    fn parse_valid_rust() {
        let tree = parse("fn main() {}", Language::Rust).unwrap();
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        assert!(root.child_count() > 0);
    }

    #[test]
    fn parse_complex_rust() {
        let source = r#"
pub struct Foo {
    x: i32,
}

impl Foo {
    pub fn new(x: i32) -> Self {
        Foo { x }
    }
}
"#;
        let tree = parse(source, Language::Rust).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn parse_empty_source() {
        let tree = parse("", Language::Rust).unwrap();
        assert_eq!(tree.root_node().child_count(), 0);
    }

    #[test]
    fn parse_returns_tree_even_for_partial_errors() {
        // tree-sitter is error-tolerant, so garbage still parses (with error nodes)
        let tree = parse("fn {{{{{", Language::Rust).unwrap();
        assert!(tree.root_node().has_error());
    }
}
