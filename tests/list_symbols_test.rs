use codeview::{process_path, ProcessOptions, OutputFormat};

const FIXTURE_PATH: &str = "tests/fixtures/sample.rs";
const FIXTURE_DIR: &str = "tests/fixtures";

fn default_options() -> ProcessOptions {
    ProcessOptions {
        symbols: vec![],
        pub_only: false,
        fns_only: false,
        types_only: false,
        no_tests: false,
        depth: None,
        format: OutputFormat::Plain,
        stats: false,
        ext: vec![],
        signatures: false,
        max_lines: None,
        list_symbols: true,
    }
}

#[test]
fn test_list_symbols_single_file() {
    let output = process_path(FIXTURE_PATH, default_options()).unwrap();
    // Should contain file header
    assert!(output.contains("sample.rs"));
    // Should list symbols with kind and line number
    assert!(output.contains("struct User"));
    assert!(output.contains("enum Role"));
    assert!(output.contains("trait Authenticatable"));
    // Should NOT contain bodies or full signatures
    assert!(!output.contains("{ ... }"));
    assert!(!output.contains("->"));
}

#[test]
fn test_list_symbols_compact_one_line_per_symbol() {
    let output = process_path(FIXTURE_PATH, default_options()).unwrap();
    // Each non-header line should start with "  " (indented symbol)
    for line in output.lines().skip(1) {
        // Each symbol line is indented
        assert!(line.starts_with("  "), "Expected indented line: {}", line);
        // Each symbol line should contain L followed by a number
        assert!(line.contains(" L"), "Expected line number: {}", line);
    }
}

#[test]
fn test_list_symbols_smaller_than_interface() {
    let interface_opts = ProcessOptions {
        list_symbols: false,
        ..default_options()
    };
    let interface_output = process_path(FIXTURE_PATH, interface_opts).unwrap();
    let list_output = process_path(FIXTURE_PATH, default_options()).unwrap();
    assert!(
        list_output.len() < interface_output.len(),
        "list-symbols output ({}) should be smaller than interface output ({})",
        list_output.len(),
        interface_output.len()
    );
}

#[test]
fn test_list_symbols_with_pub_filter() {
    let options = ProcessOptions {
        pub_only: true,
        ..default_options()
    };
    let output = process_path(FIXTURE_PATH, options).unwrap();
    // Should only have public items — private items filtered out
    assert!(output.contains("User"));
    // The output should not contain private functions
    // (depends on fixture, but pub filter should reduce items)
    let line_count = output.lines().count();
    let all_output = process_path(FIXTURE_PATH, default_options()).unwrap();
    let all_line_count = all_output.lines().count();
    assert!(line_count <= all_line_count);
}

#[test]
fn test_list_symbols_with_fns_filter() {
    let options = ProcessOptions {
        fns_only: true,
        ..default_options()
    };
    let output = process_path(FIXTURE_PATH, options).unwrap();
    // Should only contain fn symbols
    for line in output.lines().skip(1) {
        if line.trim().is_empty() { continue; }
        assert!(line.contains("fn "), "Expected fn in: {}", line);
    }
}

#[test]
fn test_list_symbols_with_types_filter() {
    let options = ProcessOptions {
        types_only: true,
        ..default_options()
    };
    let output = process_path(FIXTURE_PATH, options).unwrap();
    // Should only contain type symbols (struct/enum/trait/type)
    for line in output.lines().skip(1) {
        if line.trim().is_empty() { continue; }
        let has_type = line.contains("struct ") || line.contains("enum ") 
            || line.contains("trait ") || line.contains("type ")
            || line.contains("class ");
        assert!(has_type, "Expected type symbol in: {}", line);
    }
}

#[test]
fn test_list_symbols_with_no_tests() {
    let options = ProcessOptions {
        no_tests: true,
        ..default_options()
    };
    let output = process_path(FIXTURE_PATH, options).unwrap();
    // Should not contain test module
    assert!(!output.contains("mod tests"));
}

#[test]
fn test_list_symbols_directory() {
    let options = ProcessOptions {
        ext: vec!["rs".to_string()],
        ..default_options()
    };
    let output = process_path(FIXTURE_DIR, options).unwrap();
    // Should contain multiple file headers
    assert!(output.contains("sample.rs"));
    // Directory mode should work
    assert!(!output.is_empty());
}
