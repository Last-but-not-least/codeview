use super::collapse::{build_source_line_mappings, collapse_body};
use super::{Item, ItemKind, Visibility};
use std::collections::BTreeMap;
use tree_sitter::Node;

pub struct PythonExtractor;

fn python_visibility(name: &str) -> Visibility {
    if name.starts_with('_') {
        Visibility::Private
    } else {
        Visibility::Public
    }
}

fn build_method_signature(source: &str, node: Node) -> String {
    let mut parts = Vec::new();

    parts.push("def".to_string());

    if let Some(name) = node.child_by_field_name("name") {
        parts.push(source[name.byte_range()].to_string());
    }

    if let Some(params) = node.child_by_field_name("parameters") {
        // No space before params
        let last = parts.pop().unwrap_or_default();
        parts.push(format!("{}{}", last, &source[params.byte_range()]));
    }

    // Return type annotation
    if let Some(ret) = node.child_by_field_name("return_type") {
        parts.push("->".to_string());
        parts.push(source[ret.byte_range()].to_string());
    }

    parts.join(" ")
}

/// Find the start of decorator chain preceding a node (for decorated_definition).
fn find_decorator_start(node: Node) -> (usize, usize) {
    // For decorated_definition, the node itself includes decorators
    // For plain function/class, walk back through sibling decorated_definitions
    (node.start_byte(), node.start_position().row + 1)
}

impl super::LanguageExtractor for PythonExtractor {
    fn interface_query(&self) -> &str {
        crate::languages::python::INTERFACE_QUERY
    }

    fn expand_query(&self) -> &str {
        crate::languages::python::EXPAND_QUERY
    }

    fn node_kind_to_item_kind(&self, kind: &str) -> Option<ItemKind> {
        match kind {
            "function_definition" => Some(ItemKind::Function),
            "class_definition" => Some(ItemKind::Class),
            "decorated_definition" => {
                // Will be resolved based on inner node
                None
            }
            "import_statement" | "import_from_statement" => Some(ItemKind::Use),
            "expression_statement" => Some(ItemKind::Const),
            _ => None,
        }
    }

    fn extract_impl_name(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "class_definition" => node
                .child_by_field_name("name")
                .map(|n| source[n.byte_range()].to_string()),
            "decorated_definition" => {
                // Look for class_definition child
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "class_definition" {
                        return child
                            .child_by_field_name("name")
                            .map(|n| source[n.byte_range()].to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn extract_methods_from_block(
        &self,
        source: &str,
        block_node: Node,
        items: &mut BTreeMap<usize, Item>,
    ) {
        // block_node is the class_definition or decorated_definition
        // Find the body (block) inside the class
        let class_node = if block_node.kind() == "decorated_definition" {
            let mut cursor = block_node.walk();
            let x = block_node
                .children(&mut cursor)
                .find(|c| c.kind() == "class_definition");
            x
        } else if block_node.kind() == "class_definition" {
            Some(block_node)
        } else {
            None
        };

        let class_node = match class_node {
            Some(n) => n,
            None => return,
        };

        let body = match class_node.child_by_field_name("body") {
            Some(b) if b.kind() == "block" => b,
            _ => return,
        };

        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            let (func_node, effective_start_byte, line_start) =
                if child.kind() == "decorated_definition" {
                    let (start, line) = find_decorator_start(child);
                    let mut inner_cursor = child.walk();
                    let func = child
                        .children(&mut inner_cursor)
                        .find(|c| c.kind() == "function_definition");
                    match func {
                        Some(f) => (f, start, line),
                        None => continue,
                    }
                } else if child.kind() == "function_definition" {
                    let (start, line) = find_decorator_start(child);
                    (child, start, line)
                } else {
                    continue;
                };

            let name = func_node
                .child_by_field_name("name")
                .map(|n| source[n.byte_range()].to_string());

            let visibility = name
                .as_deref()
                .map(python_visibility)
                .unwrap_or(Visibility::Public);

            let line_end = child.end_position().row + 1;

            let (content, line_mappings, has_body) =
                if let Some(body) = func_node.child_by_field_name("body") {
                    let (c, m) = collapse_body(
                        source,
                        effective_start_byte,
                        child.end_byte(),
                        body.start_byte(),
                        body.end_byte(),
                    );
                    (c, m, true)
                } else {
                    let text = &source[effective_start_byte..child.end_byte()];
                    (text.to_string(), Vec::new(), false)
                };

            let line_mappings = if line_mappings.is_empty() {
                Some(build_source_line_mappings(&content, line_start))
            } else {
                Some(line_mappings)
            };

            let signature = build_method_signature(source, func_node);

            items.entry(line_start).or_insert(Item {
                kind: ItemKind::Method,
                name,
                visibility,
                line_start,
                line_end,
                signature: Some(signature),
                body: if has_body {
                    Some("{ ... }".to_string())
                } else {
                    None
                },
                content,
                line_mappings,
            });
        }
    }
}
