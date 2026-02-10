use crate::error::CodeviewError;
use crate::extractor::expand;
use crate::extractor::find_attr_start;
use crate::languages::{ts_language, Language};
use crate::parser;
use tree_sitter::Tree;
use tree_sitter::StreamingIterator;

/// Replace an entire symbol (including attributes) with new content.
/// Returns the modified source code.
pub fn replace(
    source: &str,
    symbol_name: &str,
    new_content: &str,
    language: Language,
) -> Result<String, CodeviewError> {
    let tree = parser::parse(source, language)?;
    let (start_byte, end_byte) = find_symbol_range(source, &tree, symbol_name, language)?;
    
    // Build the new source
    let mut result = String::new();
    result.push_str(&source[..start_byte]);
    result.push_str(new_content);
    result.push_str(&source[end_byte..]);
    
    // Validate by re-parsing
    validate_result(&result, language)?;
    
    Ok(result)
}

/// Delete a symbol (including attributes).
/// Returns the modified source code.
pub fn delete(
    source: &str,
    symbol_name: &str,
    language: Language,
) -> Result<String, CodeviewError> {
    let tree = parser::parse(source, language)?;
    let (start_byte, end_byte) = find_symbol_range(source, &tree, symbol_name, language)?;
    
    // Find if there's a trailing newline to remove
    let mut effective_end = end_byte;
    if end_byte < source.len() && source.as_bytes()[end_byte] == b'\n' {
        effective_end = end_byte + 1;
    }
    
    // Build the new source
    let mut result = String::new();
    result.push_str(&source[..start_byte]);
    result.push_str(&source[effective_end..]);
    
    // Validate by re-parsing
    validate_result(&result, language)?;
    
    Ok(result)
}

/// Find the byte range of a symbol using expand extraction.
/// Returns (start_byte, end_byte) tuple.
fn find_symbol_range(
    source: &str,
    tree: &Tree,
    symbol_name: &str,
    language: Language,
) -> Result<(usize, usize), CodeviewError> {
    let items = expand::extract(source, tree, &[symbol_name.to_string()], language);
    
    if items.is_empty() {
        return Err(CodeviewError::ParseError(format!(
            "Symbol not found: {}",
            symbol_name
        )));
    }
    
    // We expect exactly one match
    // The expand extraction gives us exactly one match
    
    // The expand extraction already uses find_attr_start, so we need to
    // reconstruct the byte range from the content and line information.
    // Actually, we need to re-run the query to get the actual node.
    
    // Re-run the query to get node byte ranges
    let extractor = crate::extractor::extractor_for(language);
    let ts_lang = ts_language(language);
    let query = tree_sitter::Query::new(&ts_lang, extractor.expand_query())
        .map_err(|e| CodeviewError::ParseError(format!("Query compilation failed: {}", e)))?;
    
    let mut cursor = tree_sitter::QueryCursor::new();
    let source_bytes = source.as_bytes();
    
    let item_idx = query.capture_index_for_name("item")
        .ok_or_else(|| CodeviewError::ParseError("Query missing 'item' capture".to_string()))?;
    let name_idx = query.capture_index_for_name("name");
    let impl_type_idx = query.capture_index_for_name("impl_type");
    
    let mut matches_iter = cursor.matches(&query, tree.root_node(), source_bytes);
    
    while let Some(m) = matches_iter.next() {
        let item_node = match m.captures.iter().find(|c| c.index == item_idx) {
            Some(c) => c.node,
            None => continue,
        };
        
        let name = name_idx
            .and_then(|idx| m.captures.iter().find(|c| c.index == idx))
            .map(|c| source[c.node.byte_range()].to_string())
            .or_else(|| {
                impl_type_idx
                    .and_then(|idx| m.captures.iter().find(|c| c.index == idx))
                    .map(|c| source[c.node.byte_range()].to_string())
            });
        
        if let Some(ref n) = name {
            if n == symbol_name {
                let (start_byte, _line_start) = find_attr_start(item_node);
                let end_byte = item_node.end_byte();
                return Ok((start_byte, end_byte));
            }
        }
    }
    
    Err(CodeviewError::ParseError(format!(
        "Symbol not found: {}",
        symbol_name
    )))
}

/// Validate the result by re-parsing and checking for errors.
fn validate_result(source: &str, language: Language) -> Result<(), CodeviewError> {
    let tree = parser::parse(source, language)?;
    if tree.root_node().has_error() {
        return Err(CodeviewError::ParseError(
            "Edit resulted in invalid syntax".to_string()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_replace_function() {
        let source = r#"fn foo() {
    println!("old");
}

fn bar() {
    println!("keep");
}
"#;
        let new_fn = r#"fn foo() {
    println!("new");
}"#;
        
        let result = replace(source, "foo", new_fn, Language::Rust).unwrap();
        assert!(result.contains(r#"println!("new")"#));
        assert!(result.contains(r#"println!("keep")"#));
        assert!(!result.contains(r#"println!("old")"#));
    }
    
    #[test]
    fn test_delete_function() {
        let source = r#"fn foo() {
    println!("delete me");
}

fn bar() {
    println!("keep me");
}
"#;
        
        let result = delete(source, "foo", Language::Rust).unwrap();
        assert!(!result.contains("delete me"));
        assert!(result.contains("keep me"));
        assert!(!result.contains("fn foo()"));
    }
    
    #[test]
    fn test_delete_struct() {
        let source = r#"struct Foo {
    x: i32,
}

struct Bar {
    y: i32,
}
"#;
        
        let result = delete(source, "Foo", Language::Rust).unwrap();
        assert!(!result.contains("struct Foo"));
        assert!(result.contains("struct Bar"));
    }
    
    #[test]
    fn test_validation_catches_bad_replacement() {
        let source = "fn foo() {}\n";
        let bad_replacement = "fn foo() { {{{{{ }";
        
        let result = replace(source, "foo", bad_replacement, Language::Rust);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid syntax"));
    }
    
    #[test]
    fn test_symbol_not_found() {
        let source = "fn foo() {}\n";
        
        let result = replace(source, "nonexistent", "fn bar() {}", Language::Rust);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Symbol not found"));
        
        let result = delete(source, "nonexistent", Language::Rust);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Symbol not found"));
    }
    
    #[test]
    fn test_replace_with_attributes() {
        let source = r#"#[inline]
#[must_use]
fn foo() -> i32 {
    42
}
"#;
        let new_fn = r#"#[inline]
fn foo() -> i32 {
    43
}"#;
        
        let result = replace(source, "foo", new_fn, Language::Rust).unwrap();
        assert!(result.contains("43"));
        assert!(!result.contains("42"));
        // Should replace the entire thing including old attributes
        assert_eq!(result.lines().filter(|l| l.contains("#[must_use]")).count(), 0);
    }
    
    #[test]
    fn test_delete_with_attributes() {
        let source = r#"#[test]
fn test_foo() {
    assert!(true);
}

fn bar() {}
"#;
        
        let result = delete(source, "test_foo", Language::Rust).unwrap();
        assert!(!result.contains("test_foo"));
        assert!(!result.contains("#[test]"));
        assert!(result.contains("fn bar()"));
    }
}
