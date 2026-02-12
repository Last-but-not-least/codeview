use crate::error::CodeviewError;
use crate::languages::{self, Language};
use crate::parser;
use crate::walk;
use regex::{Regex, RegexBuilder};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use tree_sitter::{Node, Tree};

/// A single search match with its line number, content, and enclosing symbol path.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line_content: String,
    pub symbol_path: Vec<String>,
}

/// Options for structural search.
pub struct SearchOptions {
    pub pattern: String,
    pub case_insensitive: bool,
    pub depth: Option<usize>,
    pub ext: Vec<String>,
    pub max_results: Option<usize>,
}

/// Perform structural search on a path (file or directory).
pub fn search_path(
    path: &str,
    options: &SearchOptions,
) -> Result<String, CodeviewError> {
    let regex = RegexBuilder::new(&options.pattern)
        .case_insensitive(options.case_insensitive)
        .build()
        .map_err(|e| CodeviewError::ParseError(format!("Invalid regex pattern: {}", e)))?;

    let path = Path::new(path);
    if !path.exists() {
        return Err(CodeviewError::PathNotFound(path.display().to_string()));
    }

    let file_results: Vec<(String, Vec<SearchMatch>)> = if path.is_file() {
        let lang = languages::detect_language(path)?;
        let matches = search_file(path, &regex, lang)?;
        if matches.is_empty() {
            vec![]
        } else {
            vec![(path.to_string_lossy().to_string(), matches)]
        }
    } else if path.is_dir() {
        let files = walk::walk_directory(path, options.depth, &options.ext)?;
        let mut results = Vec::new();
        for file_path in files {
            let lang = match languages::detect_language(&file_path) {
                Ok(l) => l,
                Err(_) => continue,
            };
            match search_file(&file_path, &regex, lang) {
                Ok(matches) if !matches.is_empty() => {
                    results.push((file_path.to_string_lossy().to_string(), matches));
                }
                _ => {}
            }
        }
        results
    } else {
        return Err(CodeviewError::InvalidPath(path.display().to_string()));
    };

    // Apply max_results cap
    if let Some(max) = options.max_results {
        let total_matches: usize = file_results.iter().map(|(_, m)| m.len()).sum();
        if total_matches > max {
            let overflow = total_matches - max;
            let mut kept = 0;
            let mut capped_results: Vec<(String, Vec<SearchMatch>)> = Vec::new();
            let mut overflow_files = 0usize;

            for (file_path, matches) in file_results {
                if kept >= max {
                    overflow_files += 1;
                    continue;
                }
                let remaining = max - kept;
                if matches.len() <= remaining {
                    kept += matches.len();
                    capped_results.push((file_path, matches));
                } else {
                    let taken: Vec<SearchMatch> = matches.into_iter().take(remaining).collect();
                    kept += taken.len();
                    capped_results.push((file_path, taken));
                }
            }

            // Count how many files had matches that were completely excluded
            let total_files_with_matches = capped_results.len() + overflow_files;
            let shown_files = capped_results.len();
            let extra_files = total_files_with_matches - shown_files;

            let mut output = format_search_results(&capped_results);
            writeln!(output, "\n... and {} more matches across {} files", overflow, extra_files).unwrap();
            return Ok(output);
        }
    }

    Ok(format_search_results(&file_results))
}

/// Search a single file and return matches with structural context.
fn search_file(
    path: &Path,
    regex: &Regex,
    language: Language,
) -> Result<Vec<SearchMatch>, CodeviewError> {
    let source = fs::read_to_string(path).map_err(|e| CodeviewError::ReadError {
        path: path.display().to_string(),
        source: e,
    })?;

    let tree = parser::parse(&source, language)?;
    let lines: Vec<&str> = source.lines().collect();

    let mut matches = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if regex.is_match(line) {
            let line_number = idx + 1; // 1-indexed
            let symbol_path = find_enclosing_symbols(&tree, &source, idx, language);
            matches.push(SearchMatch {
                line_number,
                line_content: line.to_string(),
                symbol_path,
            });
        }
    }

    Ok(matches)
}

/// Find the enclosing symbol hierarchy for a given line (0-indexed).
fn find_enclosing_symbols(
    tree: &Tree,
    source: &str,
    line_idx: usize,
    language: Language,
) -> Vec<String> {
    let root = tree.root_node();
    let mut symbols = Vec::new();
    find_symbols_at_line(root, source, line_idx, language, &mut symbols);
    symbols
}

/// Recursively find named symbols that contain the given line.
fn find_symbols_at_line(
    node: Node,
    source: &str,
    line_idx: usize,
    language: Language,
    symbols: &mut Vec<String>,
) {
    let start_line = node.start_position().row;
    let end_line = node.end_position().row;

    if line_idx < start_line || line_idx > end_line {
        return;
    }

    // Check if this node is a named symbol
    if let Some(name) = extract_symbol_name(node, source, language) {
        symbols.push(name);
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_symbols_at_line(child, source, line_idx, language, symbols);
    }
}

/// Extract a symbol name from a node if it represents a named symbol.
fn extract_symbol_name(node: Node, source: &str, language: Language) -> Option<String> {
    let kind = node.kind();

    match language {
        Language::Rust => match kind {
            "function_item" | "const_item" | "static_item" | "mod_item" | "macro_definition" => {
                get_child_by_field(node, "name", source)
            }
            "struct_item" | "enum_item" | "trait_item" | "type_item" => {
                get_child_by_field(node, "name", source)
            }
            "impl_item" => {
                // Get "impl Type" or "impl Trait for Type"
                let mut name_parts = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "type_identifier" | "generic_type" | "scoped_type_identifier" => {
                            name_parts.push(child.utf8_text(source.as_bytes()).ok()?.to_string());
                        }
                        _ => {}
                    }
                }
                if name_parts.is_empty() {
                    None
                } else {
                    Some(format!("impl {}", name_parts.last().unwrap()))
                }
            }
            _ => None,
        },
        Language::TypeScript | Language::Tsx => match kind {
            "function_declaration" | "method_definition" | "public_field_definition" => {
                get_child_by_field(node, "name", source)
                    .map(|n| if kind == "method_definition" || kind == "function_declaration" {
                        format!("{}()", n)
                    } else {
                        n
                    })
            }
            "class_declaration" | "abstract_class_declaration" => {
                get_child_by_field(node, "name", source)
            }
            "interface_declaration" => {
                get_child_by_field(node, "name", source)
            }
            "lexical_declaration" => {
                // const/let declarations
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "variable_declarator" {
                        return get_child_by_field(child, "name", source);
                    }
                }
                None
            }
            _ => None,
        },
        Language::JavaScript | Language::Jsx => match kind {
            "function_declaration" | "method_definition" => {
                get_child_by_field(node, "name", source)
                    .map(|n| format!("{}()", n))
            }
            "class_declaration" => {
                get_child_by_field(node, "name", source)
            }
            "lexical_declaration" | "variable_declaration" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "variable_declarator" {
                        return get_child_by_field(child, "name", source);
                    }
                }
                None
            }
            _ => None,
        },
        Language::Python => match kind {
            "function_definition" => {
                get_child_by_field(node, "name", source)
                    .map(|n| format!("{}()", n))
            }
            "class_definition" => {
                get_child_by_field(node, "name", source)
            }
            _ => None,
        },
    }
}

fn get_child_by_field(node: Node, field: &str, source: &str) -> Option<String> {
    node.child_by_field_name(field)
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(|s| s.to_string())
}

/// Format search results grouped by file and enclosing symbol.
fn format_search_results(file_results: &[(String, Vec<SearchMatch>)]) -> String {
    let mut output = String::new();

    for (i, (file_path, matches)) in file_results.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        writeln!(output, "{}", file_path).unwrap();

        // Group matches by symbol path
        let mut groups: BTreeMap<String, Vec<&SearchMatch>> = BTreeMap::new();
        let mut order: Vec<String> = Vec::new();

        for m in matches {
            let key = if m.symbol_path.is_empty() {
                "(top-level)".to_string()
            } else {
                m.symbol_path.join(" > ")
            };
            if !groups.contains_key(&key) {
                order.push(key.clone());
            }
            groups.entry(key).or_default().push(m);
        }

        for key in &order {
            let group = &groups[key];
            writeln!(output).unwrap();
            writeln!(output, "  {}", key).unwrap();
            for m in group {
                writeln!(output, "    L{}:{}", m.line_number, m.line_content).unwrap();
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_ts_file(dir: &TempDir, name: &str, content: &str) -> String {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    fn write_rs_file(dir: &TempDir, name: &str, content: &str) -> String {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_basic_search_rust() {
        let dir = TempDir::new().unwrap();
        let path = write_rs_file(&dir, "test.rs", r#"fn hello() {
    println!("world");
}

fn goodbye() {
    println!("farewell");
}
"#);
        let opts = SearchOptions {
            pattern: "println".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains("goodbye"));
        assert!(result.contains("L2:"));
        assert!(result.contains("L6:"));
    }

    #[test]
    fn test_case_insensitive() {
        let dir = TempDir::new().unwrap();
        let path = write_rs_file(&dir, "test.rs", r#"fn hello() {
    let Message = "hi";
}
"#);
        // Case-sensitive: should not match "Message" with pattern "message"
        let opts = SearchOptions {
            pattern: "message".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(!result.contains("Message"));

        // Case-insensitive: should match
        let opts = SearchOptions {
            pattern: "message".to_string(),
            case_insensitive: true,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(result.contains("Message"));
    }

    #[test]
    fn test_regex_pattern() {
        let dir = TempDir::new().unwrap();
        let path = write_rs_file(&dir, "test.rs", r#"fn process() {
    let x = 42;
    let y = 100;
    let z = "hello";
}
"#);
        let opts = SearchOptions {
            pattern: r"let \w+ = \d+".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(result.contains("L2:"));
        assert!(result.contains("L3:"));
        assert!(!result.contains("L4:")); // "hello" is not digits
    }

    #[test]
    fn test_directory_search() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        write_rs_file(&dir, "a.rs", "fn foo() {\n    target_word();\n}\n");
        write_rs_file(&dir, "b.rs", "fn bar() {\n    other();\n}\n");
        let opts = SearchOptions {
            pattern: "target_word".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&dir.path().to_string_lossy().as_ref(), &opts).unwrap();
        assert!(result.contains("a.rs"));
        assert!(!result.contains("b.rs"));
    }

    #[test]
    fn test_no_matches() {
        let dir = TempDir::new().unwrap();
        let path = write_rs_file(&dir, "test.rs", "fn hello() {}\n");
        let opts = SearchOptions {
            pattern: "nonexistent".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_top_level_matches() {
        let dir = TempDir::new().unwrap();
        let path = write_rs_file(&dir, "test.rs", "use std::io;\nfn hello() {}\n");
        let opts = SearchOptions {
            pattern: "std::io".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(result.contains("(top-level)"));
    }

    #[test]
    fn test_typescript_class_method() {
        let dir = TempDir::new().unwrap();
        let path = write_ts_file(&dir, "test.ts", r#"class MyClass {
    run() {
        this.enqueue("data");
    }

    enqueue(data: string) {
        console.log(data);
    }
}
"#);
        let opts = SearchOptions {
            pattern: "enqueue".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(result.contains("MyClass"));
        assert!(result.contains("run()"));
        assert!(result.contains("enqueue()"));
    }

    #[test]
    fn test_nested_rust_impl() {
        let dir = TempDir::new().unwrap();
        let path = write_rs_file(&dir, "test.rs", r#"struct Foo;

impl Foo {
    fn bar(&self) {
        self.do_thing();
    }
}
"#);
        let opts = SearchOptions {
            pattern: "do_thing".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None,
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(result.contains("impl Foo"));
        assert!(result.contains("bar"));
    }

    #[test]
    fn test_max_results_caps_directory_search() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        // Create files with many matches
        write_rs_file(&dir, "a.rs", "fn f1() { target(); }\nfn f2() { target(); }\nfn f3() { target(); }\n");
        write_rs_file(&dir, "b.rs", "fn g1() { target(); }\nfn g2() { target(); }\nfn g3() { target(); }\n");
        let opts = SearchOptions {
            pattern: "target".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: Some(3),
        };
        let result = search_path(&dir.path().to_string_lossy().as_ref(), &opts).unwrap();
        // Should contain the summary line
        assert!(result.contains("... and 3 more matches across"));
    }

    #[test]
    fn test_max_results_no_cap_when_under_limit() {
        let dir = TempDir::new().unwrap();
        let path = write_rs_file(&dir, "test.rs", "fn foo() { target(); }\n");
        let opts = SearchOptions {
            pattern: "target".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: Some(10),
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(!result.contains("... and"));
    }

    #[test]
    fn test_single_file_no_default_cap() {
        let dir = TempDir::new().unwrap();
        // 25 matches in a single file - should all show (no default cap for single file)
        let mut content = String::new();
        for i in 0..25 {
            content.push_str(&format!("fn f{}() {{ target(); }}\n", i));
        }
        let path = write_rs_file(&dir, "test.rs", &content);
        let opts = SearchOptions {
            pattern: "target".to_string(),
            case_insensitive: false,
            depth: None,
            ext: vec![],
            max_results: None, // single-file default: no cap
        };
        let result = search_path(&path, &opts).unwrap();
        assert!(!result.contains("... and"));
        // All 25 matches should be present
        assert!(result.contains("f24"));
    }
}
