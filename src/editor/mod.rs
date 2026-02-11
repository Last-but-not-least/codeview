use crate::error::CodeviewError;
use crate::extractor::expand;
use crate::extractor::find_attr_start;
use crate::languages::{ts_language, Language};
use crate::parser;
use tree_sitter::{Node, Tree};
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

/// Replace only the body block (`{ ... }`) of a symbol, preserving signature/attributes.
/// `new_body` should be the inner content (without outer braces), e.g. `    println!("hi");\n`.
/// Indentation is auto-adjusted to match the original block's indent level.
pub fn replace_body(
    source: &str,
    symbol_name: &str,
    new_body: &str,
    language: Language,
) -> Result<String, CodeviewError> {
    let tree = parser::parse(source, language)?;
    let item_node = find_symbol_node(source, &tree, symbol_name, language)?;
    
    let body_node = find_body_node(item_node, language)?;
    let body_start = body_node.start_byte();
    let body_end = body_node.end_byte();
    
    // Detect indent level of the body's opening brace line
    let line_start = source[..body_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let original_indent = &source[line_start..body_start]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();
    
    // Build the new body block with proper indentation
    let reindented = reindent_body(new_body, original_indent);
    let new_block = format!("{{\n{}\n{}}}", reindented, original_indent);
    
    let mut result = String::new();
    result.push_str(&source[..body_start]);
    result.push_str(&new_block);
    result.push_str(&source[body_end..]);
    
    validate_result(&result, language)?;
    Ok(result)
}

/// Apply multiple edits to a file in one pass.
/// Edits are applied bottom-to-top so byte offsets remain valid.
pub fn batch(
    source: &str,
    edits: &[BatchEdit],
    language: Language,
) -> Result<String, CodeviewError> {
    // Resolve all byte ranges first, before any mutations
    let tree = parser::parse(source, language)?;
    let mut resolved: Vec<ResolvedEdit> = Vec::new();
    
    for edit in edits {
        match edit.action {
            BatchAction::Replace => {
                let content = edit.content.as_deref().ok_or_else(|| {
                    CodeviewError::ParseError(format!(
                        "Missing 'content' for replace action on '{}'", edit.symbol
                    ))
                })?;
                let (start, end) = find_symbol_range(source, &tree, &edit.symbol, language)?;
                resolved.push(ResolvedEdit { start, end, replacement: content.to_string() });
            }
            BatchAction::ReplaceBody => {
                let content = edit.content.as_deref().ok_or_else(|| {
                    CodeviewError::ParseError(format!(
                        "Missing 'content' for replace-body action on '{}'", edit.symbol
                    ))
                })?;
                let item_node = find_symbol_node(source, &tree, &edit.symbol, language)?;
                let body_node = find_body_node(item_node, language)?;
                let body_start = body_node.start_byte();
                let body_end = body_node.end_byte();
                
                let line_start = source[..body_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let original_indent = &source[line_start..body_start]
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>();
                let reindented = reindent_body(content, original_indent);
                let new_block = format!("{{\n{}\n{}}}", reindented, original_indent);
                
                resolved.push(ResolvedEdit { start: body_start, end: body_end, replacement: new_block });
            }
            BatchAction::Delete => {
                let (start, end) = find_symbol_range(source, &tree, &edit.symbol, language)?;
                let mut effective_end = end;
                if effective_end < source.len() && source.as_bytes()[effective_end] == b'\n' {
                    effective_end += 1;
                }
                resolved.push(ResolvedEdit { start, end: effective_end, replacement: String::new() });
            }
        }
    }
    
    // Sort by start byte descending (bottom-to-top) so earlier offsets stay valid
    resolved.sort_by(|a, b| b.start.cmp(&a.start));
    
    // Check for overlapping ranges
    for w in resolved.windows(2) {
        // w[0] has higher start than w[1] (sorted descending)
        if w[1].end > w[0].start {
            return Err(CodeviewError::ParseError(
                "Overlapping edit ranges detected".to_string()
            ));
        }
    }
    
    let mut result = source.to_string();
    for edit in &resolved {
        result = format!("{}{}{}", &result[..edit.start], edit.replacement, &result[edit.end..]);
    }
    
    validate_result(&result, language)?;
    Ok(result)
}

/// Result metadata for a single edit operation (used with --json output).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditResult {
    pub symbol: String,
    pub action: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// Get the 1-based line range of a symbol (including attributes).
pub fn symbol_line_range(
    source: &str,
    symbol_name: &str,
    language: Language,
) -> Result<(usize, usize), CodeviewError> {
    let tree = parser::parse(source, language)?;
    let (start_byte, end_byte) = find_symbol_range(source, &tree, symbol_name, language)?;
    let line_start = source[..start_byte].matches('\n').count() + 1;
    let line_end = source[..end_byte].matches('\n').count() + 1;
    Ok((line_start, line_end))
}

/// A single edit in a batch operation.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BatchEdit {
    pub symbol: String,
    pub action: BatchAction,
    pub content: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BatchAction {
    Replace,
    ReplaceBody,
    Delete,
}

struct ResolvedEdit {
    start: usize,
    end: usize,
    replacement: String,
}

/// Find the body block node of a symbol (Rust `block`, TS `statement_block`).
fn find_body_node<'a>(item_node: Node<'a>, language: Language) -> Result<Node<'a>, CodeviewError> {
    let body_kinds = match language {
        Language::Rust => &["block"][..],
        Language::TypeScript | Language::Tsx => &["statement_block"][..],
    };
    
    // First try the `body` field (works for functions)
    if let Some(body) = item_node.child_by_field_name("body") {
        if body_kinds.contains(&body.kind()) {
            return Ok(body);
        }
    }
    
    // Fallback: search children for a matching block kind
    let mut cursor = item_node.walk();
    for child in item_node.children(&mut cursor) {
        if body_kinds.contains(&child.kind()) {
            return Ok(child);
        }
    }
    
    Err(CodeviewError::ParseError(format!(
        "Symbol has no body block (kind: {})", item_node.kind()
    )))
}

/// Re-indent body content to match the target indent level.
/// Each non-empty line gets `base_indent + one level (4 spaces)`.
fn reindent_body(body: &str, base_indent: &str) -> String {
    let inner_indent = format!("{}    ", base_indent);
    
    // Detect the minimum indent of the input to strip it
    let min_indent = body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    
    body.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                let stripped = if line.len() >= min_indent { &line[min_indent..] } else { line.trim_start() };
                format!("{}{}", inner_indent, stripped)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find the tree-sitter Node for a named symbol.
fn find_symbol_node<'a>(
    source: &str,
    tree: &'a Tree,
    symbol_name: &str,
    language: Language,
) -> Result<Node<'a>, CodeviewError> {
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
                return Ok(item_node);
            }
        }
    }
    
    Err(CodeviewError::ParseError(format!("Symbol not found: {}", symbol_name)))
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
    fn test_replace_body_function() {
        let source = r#"fn foo(x: i32) -> i32 {
    x + 1
}

fn bar() {}
"#;
        let result = replace_body(source, "foo", "x * 2", Language::Rust).unwrap();
        assert!(result.contains("fn foo(x: i32) -> i32"));
        assert!(result.contains("x * 2"));
        assert!(!result.contains("x + 1"));
        assert!(result.contains("fn bar()"));
    }
    
    #[test]
    fn test_replace_body_preserves_attributes() {
        let source = r#"#[inline]
pub fn foo() -> i32 {
    42
}
"#;
        let result = replace_body(source, "foo", "99", Language::Rust).unwrap();
        assert!(result.contains("#[inline]"));
        assert!(result.contains("pub fn foo() -> i32"));
        assert!(result.contains("99"));
        assert!(!result.contains("42"));
    }
    
    #[test]
    fn test_replace_body_reindents() {
        let source = "    fn foo() {\n        old_code();\n    }\n";
        // Providing body with no indent — should get auto-indented
        let result = replace_body(source, "foo", "new_code();\nmore_code();", Language::Rust).unwrap();
        assert!(result.contains("        new_code();"));
        assert!(result.contains("        more_code();"));
    }
    
    #[test]
    fn test_replace_body_no_body_errors() {
        let source = "struct Foo { x: i32 }\n";
        // struct doesn't have a "block" body in the function sense
        // This should still work since struct has a field_declaration_list, not a block
        let result = replace_body(source, "Foo", "y: i32", Language::Rust);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_batch_multiple_edits() {
        let source = r#"fn foo() {
    old_foo();
}

fn bar() {
    old_bar();
}

fn baz() {
    old_baz();
}
"#;
        let edits = vec![
            BatchEdit { symbol: "foo".to_string(), action: BatchAction::ReplaceBody, content: Some("new_foo();".to_string()) },
            BatchEdit { symbol: "baz".to_string(), action: BatchAction::Delete, content: None },
        ];
        let result = batch(source, &edits, Language::Rust).unwrap();
        assert!(result.contains("new_foo()"));
        assert!(!result.contains("old_foo"));
        assert!(result.contains("old_bar")); // bar untouched
        assert!(!result.contains("baz"));
    }
    
    #[test]
    fn test_batch_replace_and_replace_body() {
        let source = r#"fn alpha() {
    1
}

fn beta() {
    2
}
"#;
        let edits = vec![
            BatchEdit { symbol: "alpha".to_string(), action: BatchAction::Replace, content: Some("fn alpha() {\n    100\n}".to_string()) },
            BatchEdit { symbol: "beta".to_string(), action: BatchAction::ReplaceBody, content: Some("200".to_string()) },
        ];
        let result = batch(source, &edits, Language::Rust).unwrap();
        assert!(result.contains("100"));
        assert!(result.contains("200"));
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
