use super::collapse::{collapse_body, build_source_line_mappings};
use super::{find_attr_start, Item, ItemKind, Visibility};
use tree_sitter::Node;
use std::collections::BTreeMap;

pub struct JavaScriptExtractor;

fn build_method_signature(source: &str, node: Node) -> String {
    let mut parts = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "async" | "static" => {
                parts.push(source[child.byte_range()].to_string());
            }
            _ => {}
        }
    }

    if let Some(name) = node.child_by_field_name("name") {
        parts.push(source[name.byte_range()].to_string());
    }

    if let Some(params) = node.child_by_field_name("parameters") {
        parts.push(source[params.byte_range()].to_string());
    }

    parts.join(" ")
}

impl super::LanguageExtractor for JavaScriptExtractor {
    fn interface_query(&self) -> &str {
        crate::languages::javascript::INTERFACE_QUERY
    }

    fn expand_query(&self) -> &str {
        crate::languages::javascript::EXPAND_QUERY
    }

    fn node_kind_to_item_kind(&self, kind: &str) -> Option<ItemKind> {
        match kind {
            "function_declaration" => Some(ItemKind::Function),
            "class_declaration" => Some(ItemKind::Class),
            "import_statement" => Some(ItemKind::Use),
            "lexical_declaration" | "variable_declaration" => Some(ItemKind::Const),
            "method_definition" => Some(ItemKind::Method),
            "export_statement" => None,
            _ => None,
        }
    }

    fn extract_impl_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        if node.kind() == "class_declaration" {
            node.child_by_field_name("name")
                .map(|n| source[n.byte_range()].to_string())
        } else {
            None
        }
    }

    fn extract_methods_from_block(&self, source: &str, block_node: tree_sitter::Node, items: &mut BTreeMap<usize, Item>) {
        let body = match block_node.child_by_field_name("body") {
            Some(b) if b.kind() == "class_body" => b,
            _ => return,
        };

        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() != "method_definition" {
                continue;
            }

            let name = child
                .child_by_field_name("name")
                .map(|n| source[n.byte_range()].to_string());

            let (effective_start_byte, line_start) = find_attr_start(child);
            let line_end = child.end_position().row + 1;

            let (content, line_mappings, has_body) = if let Some(body) = child.child_by_field_name("body") {
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

            let signature = build_method_signature(source, child);

            // All JS methods are public (no accessibility modifiers)
            items.entry(line_start).or_insert(Item {
                kind: ItemKind::Method,
                name,
                visibility: Visibility::Public,
                line_start,
                line_end,
                signature: Some(signature),
                body: if has_body { Some("{ ... }".to_string()) } else { None },
                content,
                line_mappings,
            });
        }
    }
}
