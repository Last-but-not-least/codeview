use super::collapse::{collapse_block, build_source_line_mappings};
use super::{extractor_for, find_attr_start, Item, ItemKind, Visibility, LanguageExtractor};
use crate::languages::{ts_language, Language};
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator, Tree};

/// Extract full implementation for specified symbols using tree-sitter queries.
pub fn extract(source: &str, tree: &Tree, symbols: &[String], language: Language) -> Vec<Item> {
    let extractor = extractor_for(language);
    extract_with_extractor(source, tree, symbols, language, extractor.as_ref())
}

fn extract_with_extractor(source: &str, tree: &Tree, symbols: &[String], language: Language, extractor: &dyn LanguageExtractor) -> Vec<Item> {
    let ts_lang = ts_language(language);
    let query = Query::new(&ts_lang, extractor.expand_query())
        .expect("expand_query should compile");

    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let item_idx = query.capture_index_for_name("item").unwrap();
    let name_idx = query.capture_index_for_name("name");
    let impl_type_idx = query.capture_index_for_name("impl_type");

    let mut items = Vec::new();
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

        let name_str = match &name {
            Some(n) => n.as_str(),
            None => continue,
        };
        if !symbols.iter().any(|s| s == name_str) {
            continue;
        }

        let (effective_start_byte, line_start) = find_attr_start(item_node);
        let line_end = item_node.end_position().row + 1;

        let content = source[effective_start_byte..item_node.end_byte()].to_string();
        let visibility = Visibility::from_parent(item_node, source);

        let kind = match extractor.node_kind_to_item_kind(item_node.kind()) {
            Some(k) => k,
            None => continue,
        };

        items.push(Item {
            kind,
            name,
            visibility,
            line_start,
            line_end,
            signature: None,
            body: None,
            content,
            line_mappings: None,
        });
    }

    items.sort_by_key(|item| item.line_start);
    items
}

/// Extract a class with method signatures collapsed, optionally expanding specific methods.
pub fn extract_signatures(source: &str, tree: &Tree, class_name: &str, expand_methods: &[String], language: Language) -> Vec<Item> {
    let extractor = extractor_for(language);
    let ts_lang = ts_language(language);
    let query = Query::new(&ts_lang, extractor.expand_query())
        .expect("expand_query should compile");

    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let item_idx = query.capture_index_for_name("item").unwrap();
    let name_idx = query.capture_index_for_name("name");

    let mut matches_iter = cursor.matches(&query, tree.root_node(), source_bytes);

    while let Some(m) = matches_iter.next() {
        let item_node = match m.captures.iter().find(|c| c.index == item_idx) {
            Some(c) => c.node,
            None => continue,
        };

        let name = name_idx
            .and_then(|idx| m.captures.iter().find(|c| c.index == idx))
            .map(|c| source[c.node.byte_range()].to_string());

        let name_str = match &name {
            Some(n) => n.as_str(),
            None => continue,
        };
        if name_str != class_name {
            continue;
        }

        let kind = match extractor.node_kind_to_item_kind(item_node.kind()) {
            Some(k) => k,
            None => continue,
        };

        // Only apply signatures mode to class-like items
        if !matches!(kind, ItemKind::Class) {
            // Not a class — just return as full expand
            let (effective_start_byte, line_start) = find_attr_start(item_node);
            let line_end = item_node.end_position().row + 1;
            let content = source[effective_start_byte..item_node.end_byte()].to_string();
            let visibility = Visibility::from_parent(item_node, source);
            return vec![Item {
                kind,
                name,
                visibility,
                line_start,
                line_end,
                signature: None,
                body: None,
                content,
                line_mappings: None,
            }];
        }

        let (effective_start_byte, line_start) = find_attr_start(item_node);
        let line_end = item_node.end_position().row + 1;
        let visibility = Visibility::from_parent(item_node, source);

        if expand_methods.is_empty() {
            // Pure signatures mode: collapse all method bodies
            let (content, line_mappings) = collapse_block(source, effective_start_byte, item_node);
            let line_mappings = if line_mappings.is_empty() {
                Some(build_source_line_mappings(&content, line_start))
            } else {
                Some(line_mappings)
            };
            return vec![Item {
                kind,
                name,
                visibility,
                line_start,
                line_end,
                signature: None,
                body: None,
                content,
                line_mappings,
            }];
        } else {
            // Combined mode: collapse all method bodies except specified ones
            let (content, line_mappings) = collapse_block_except(source, effective_start_byte, item_node, expand_methods);
            let line_mappings = if line_mappings.is_empty() {
                Some(build_source_line_mappings(&content, line_start))
            } else {
                Some(line_mappings)
            };
            return vec![Item {
                kind,
                name,
                visibility,
                line_start,
                line_end,
                signature: None,
                body: None,
                content,
                line_mappings,
            }];
        }
    }

    Vec::new()
}

/// Like collapse_block but skips collapsing methods whose names are in `keep_expanded`.
fn collapse_block_except(source: &str, start_byte: usize, block_node: Node, keep_expanded: &[String]) -> (String, Vec<(usize, String)>) {
    let mut body_ranges: Vec<(usize, usize)> = Vec::new();
    collect_fn_bodies_except(block_node, source, keep_expanded, &mut body_ranges);
    body_ranges.sort_by_key(|&(s, _)| s);

    let end_byte = block_node.end_byte();
    let mut result = String::new();
    let mut pos = start_byte;

    for (body_start, body_end) in &body_ranges {
        result.push_str(&source[pos..*body_start]);
        result.push_str("{ ... }");
        pos = *body_end;
    }
    result.push_str(&source[pos..end_byte]);

    let start_line = source[..start_byte].matches('\n').count() + 1;
    let mappings = super::collapse::build_collapsed_block_mappings_pub(source, end_byte, &body_ranges, start_line, &result);

    (result, mappings)
}

fn collect_fn_bodies_except(node: Node, source: &str, keep_expanded: &[String], ranges: &mut Vec<(usize, usize)>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_item" || child.kind() == "method_definition" {
            let name = child.child_by_field_name("name")
                .map(|n| source[n.byte_range()].to_string());
            if let Some(ref n) = name {
                if keep_expanded.iter().any(|s| s == n) {
                    continue; // Don't collapse this method
                }
            }
            if let Some(body) = child.child_by_field_name("body") {
                ranges.push((body.start_byte(), body.end_byte()));
            }
        } else if child.kind() == "declaration_list" || child.kind() == "class_body" || child.kind() == "interface_body" || child.kind() == "class_declaration" || child.kind() == "abstract_class_declaration" || child.kind() == "interface_declaration" || child.kind() == "export_statement" {
            collect_fn_bodies_except(child, source, keep_expanded, ranges);
        }
    }
}
