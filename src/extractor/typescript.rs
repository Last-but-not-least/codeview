use super::collapse::{collapse_body, build_source_line_mappings};
use super::{find_attr_start, Item, ItemKind, Visibility};
use tree_sitter::Node;
use std::collections::BTreeMap;

pub struct TypeScriptExtractor;


fn build_method_signature(source: &str, node: Node) -> String {
    let mut parts = Vec::new();

    // Check for accessibility modifier (public/private/protected)
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "accessibility_modifier" | "readonly" => {
                parts.push(source[child.byte_range()].to_string());
            }
            "async" | "static" => {
                parts.push(source[child.byte_range()].to_string());
            }
            _ => {}
        }
    }

    if let Some(name) = node.child_by_field_name("name") {
        parts.push(source[name.byte_range()].to_string());
    }

    // type parameters
    let mut cursor2 = node.walk();
    for child in node.children(&mut cursor2) {
        if child.kind() == "type_parameters" {
            parts.push(source[child.byte_range()].to_string());
        }
    }

    if let Some(params) = node.child_by_field_name("parameters") {
        parts.push(source[params.byte_range()].to_string());
    }

    // return type
    let mut cursor3 = node.walk();
    for child in node.children(&mut cursor3) {
        if child.kind() == "type_annotation" {
            parts.push(source[child.byte_range()].to_string());
        }
    }

    parts.join(" ")
}

fn member_visibility(node: Node, source: &str) -> Visibility {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "accessibility_modifier" {
            let text = &source[child.byte_range()];
            return if text == "public" { Visibility::Public } else { Visibility::Private };
        }
    }
    // Default: public in TS classes (no modifier = public)
    Visibility::Public
}

impl super::LanguageExtractor for TypeScriptExtractor {
    fn interface_query(&self) -> &str {
        crate::languages::typescript::INTERFACE_QUERY
    }

    fn expand_query(&self) -> &str {
        crate::languages::typescript::EXPAND_QUERY
    }

    fn node_kind_to_item_kind(&self, kind: &str) -> Option<ItemKind> {
        match kind {
            "function_declaration" => Some(ItemKind::Function),
            "class_declaration" | "abstract_class_declaration" => Some(ItemKind::Class),
            "interface_declaration" => Some(ItemKind::Trait),
            "type_alias_declaration" => Some(ItemKind::TypeAlias),
            "enum_declaration" => Some(ItemKind::Enum),
            "import_statement" => Some(ItemKind::Use),
            "lexical_declaration" => Some(ItemKind::Const),
            "method_definition" => Some(ItemKind::Method),
            "export_statement" => {
                // Check inner declaration
                None
            }
            _ => None,
        }
    }

    fn extract_impl_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        if node.kind() == "class_declaration" || node.kind() == "abstract_class_declaration" || node.kind() == "interface_declaration" {
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

        let _is_abstract_class = block_node.kind() == "abstract_class_declaration";

        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            let is_abstract_method = child.kind() == "abstract_method_signature";
            if child.kind() != "method_definition" && !is_abstract_method {
                continue;
            }

            let visibility = member_visibility(child, source);
            let name = child
                .child_by_field_name("name")
                .map(|n| source[n.byte_range()].to_string());

            let (effective_start_byte, line_start) = find_attr_start(child);
            let line_end = child.end_position().row + 1;

            let (content, line_mappings, has_body) = if is_abstract_method {
                let text = &source[effective_start_byte..child.end_byte()];
                (text.to_string(), Vec::new(), false)
            } else if let Some(body) = child.child_by_field_name("body") {
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

            items.entry(line_start).or_insert(Item {
                kind: ItemKind::Method,
                name,
                visibility,
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
