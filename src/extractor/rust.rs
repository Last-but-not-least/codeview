//! Rust-specific tree-sitter node operations.
//!
//! Helpers for extracting signatures, impl names, and methods from Rust AST nodes.

use super::collapse::{collapse_body, build_source_line_mappings};
use super::{find_attr_start, Item, ItemKind, Visibility};
use tree_sitter::Node;
use std::collections::BTreeMap;

pub fn extract_methods_from_block(
    source: &str,
    block_node: Node,
    items: &mut BTreeMap<usize, Item>,
) {
    let decl_list = match block_node.child_by_field_name("body") {
        Some(body) if body.kind() == "declaration_list" => body,
        _ => return,
    };

    let mut cursor = decl_list.walk();
    for child in decl_list.children(&mut cursor) {
        if child.kind() != "function_item" {
            continue;
        }

        let visibility = Visibility::from_parent(child, source);
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

        let signature = build_fn_signature(source, child);

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

/// Build a function signature string from a function_item node.
pub fn build_fn_signature(source: &str, node: Node) -> String {
    let mut parts = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "visibility_modifier" => parts.push(source[child.byte_range()].to_string()),
            "async" | "const" | "unsafe" | "extern" => {
                parts.push(source[child.byte_range()].to_string())
            }
            _ => {}
        }
    }

    parts.push("fn".to_string());

    if let Some(name) = node.child_by_field_name("name") {
        parts.push(source[name.byte_range()].to_string());
    }
    if let Some(tp) = node.child_by_field_name("type_parameters") {
        parts.push(source[tp.byte_range()].to_string());
    }
    if let Some(params) = node.child_by_field_name("parameters") {
        parts.push(source[params.byte_range()].to_string());
    }
    if let Some(ret) = node.child_by_field_name("return_type") {
        parts.push("->".to_string());
        parts.push(source[ret.byte_range()].to_string());
    }

    let mut cursor2 = node.walk();
    for child in node.children(&mut cursor2) {
        if child.kind() == "where_clause" {
            parts.push(source[child.byte_range()].to_string());
        }
    }

    parts.join(" ")
}

/// Extract impl name (trait name or type name).
pub fn extract_impl_name(node: Node, source: &str) -> Option<String> {
    if let Some(trait_node) = node.child_by_field_name("trait") {
        return Some(source[trait_node.byte_range()].to_string());
    }
    if let Some(type_node) = node.child_by_field_name("type") {
        return Some(source[type_node.byte_range()].to_string());
    }
    None
}

/// Rust language extractor.
pub struct RustExtractor;

impl super::LanguageExtractor for RustExtractor {
    fn interface_query(&self) -> &str {
        crate::languages::rust::INTERFACE_QUERY
    }

    fn expand_query(&self) -> &str {
        crate::languages::rust::EXPAND_QUERY
    }

    fn node_kind_to_item_kind(&self, kind: &str) -> Option<ItemKind> {
        ItemKind::from_node_kind(kind)
    }

    fn extract_impl_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        extract_impl_name(node, source)
    }


    fn extract_methods_from_block(&self, source: &str, block_node: tree_sitter::Node, items: &mut std::collections::BTreeMap<usize, Item>) {
        extract_methods_from_block(source, block_node, items)
    }
}
