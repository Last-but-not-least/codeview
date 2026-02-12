use crate::CodeviewError;
use crate::extractor::Item;

/// Format items as plain text with line numbers
pub fn format_output(files: &[(String, Vec<Item>)], expand_mode: bool, max_lines: Option<usize>) -> Result<String, CodeviewError> {
    let mut output = String::new();

    for (file_path, items) in files {
        if items.is_empty() {
            continue;
        }

        if expand_mode {
            // Expand mode: each item gets a header with file::symbol [start:end]
            for item in items {
                if let Some(ref name) = item.name {
                    output.push_str(&format!(
                        "{}::{} [{}:{}]\n",
                        file_path, name, item.line_start, item.line_end
                    ));
                } else {
                    output.push_str(&format!(
                        "{} [{}:{}]\n",
                        file_path, item.line_start, item.line_end
                    ));
                }
                let formatted = format_item(item);
                if let Some(max) = max_lines {
                    let lines: Vec<&str> = formatted.lines().collect();
                    if lines.len() > max {
                        for line in &lines[..max] {
                            output.push_str(line);
                            output.push('\n');
                        }
                        let remaining = lines.len() - max;
                        output.push_str(&format!("  ... [truncated: {} more lines]\n", remaining));
                    } else {
                        output.push_str(&formatted);
                    }
                } else {
                    output.push_str(&formatted);
                }
                output.push('\n');
            }
        } else {
            // Interface mode: file header once, then all items
            output.push_str(file_path);
            output.push('\n');

            for item in items {
                output.push_str(&format_item(item));
                output.push('\n');
            }
        }
    }

    Ok(output)
}

fn format_item(item: &Item) -> String {
    let mut result = String::new();

    // Calculate max line number width for alignment
    let max_line_num = item.line_end;
    let width = max_line_num.to_string().len();

    // Use explicit line mappings if available (for interface mode with collapsed bodies)
    if let Some(ref mappings) = item.line_mappings {
        for (line_num, line_text) in mappings {
            result.push_str(&format!("{:>width$} | {}\n", line_num, line_text, width = width));
        }
    } else {
        // Default: sequential line numbers (for expand mode)
        let lines: Vec<&str> = item.content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let line_num = item.line_start + i;
            result.push_str(&format!("{:>width$} | {}\n", line_num, line, width = width));
        }
    }

    result
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{Item, ItemKind, Visibility};

    fn make_item(name: &str, content: &str, line_start: usize, line_end: usize) -> Item {
        Item {
            kind: ItemKind::Function,
            name: Some(name.to_string()),
            visibility: Visibility::Public,
            line_start,
            line_end,
            signature: None,
            body: None,
            content: content.to_string(),
            line_mappings: None,
        }
    }

    #[test]
    fn format_item_sequential_lines() {
        let item = make_item("foo", "fn foo() {\n    42\n}", 10, 12);
        let result = format_item(&item);
        assert!(result.contains("10 | fn foo() {"));
        assert!(result.contains("11 |     42"));
        assert!(result.contains("12 | }"));
    }

    #[test]
    fn format_item_with_line_mappings() {
        let mut item = make_item("foo", "", 1, 5);
        item.line_mappings = Some(vec![
            (1, "fn foo() { ... }".to_string()),
        ]);
        let result = format_item(&item);
        assert!(result.contains("1 | fn foo() { ... }"));
    }

    #[test]
    fn format_output_interface_mode() {
        let item = make_item("bar", "fn bar() {}", 1, 1);
        let files = vec![("src/lib.rs".to_string(), vec![item])];
        let result = format_output(&files, false, None).unwrap();
        assert!(result.starts_with("src/lib.rs\n"));
        assert!(result.contains("fn bar() {}"));
    }

    #[test]
    fn format_output_expand_mode() {
        let item = make_item("bar", "fn bar() {}", 1, 1);
        let files = vec![("src/lib.rs".to_string(), vec![item])];
        let result = format_output(&files, true, None).unwrap();
        assert!(result.contains("src/lib.rs::bar [1:1]"));
    }

    #[test]
    fn format_output_expand_mode_no_name() {
        let mut item = make_item("bar", "use std::io;", 1, 1);
        item.name = None;
        let files = vec![("src/lib.rs".to_string(), vec![item])];
        let result = format_output(&files, true, None).unwrap();
        assert!(result.contains("src/lib.rs [1:1]"));
    }

    #[test]
    fn format_output_skips_empty_files() {
        let files = vec![("empty.rs".to_string(), vec![])];
        let result = format_output(&files, false, None).unwrap();
        assert!(result.is_empty());
    }
}
