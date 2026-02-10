use super::collapse::{collapse_body, collapse_block, build_source_line_mappings};
use super::{extractor_for, find_attr_start, Item, Visibility, LanguageExtractor};
use crate::languages::{ts_language, Language};
use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};
use std::collections::BTreeMap;

/// Extract interface view (collapsed function bodies) using tree-sitter queries.
pub fn extract(source: &str, tree: &Tree, language: Language) -> Vec<Item> {
    let extractor = extractor_for(language);
    extract_with_extractor(source, tree, language, extractor.as_ref())
}

fn extract_with_extractor(source: &str, tree: &Tree, language: Language, extractor: &dyn LanguageExtractor) -> Vec<Item> {
    let ts_lang = ts_language(language);
    let query = Query::new(&ts_lang, extractor.interface_query())
        .expect("interface_query should compile");

    let mut cursor = QueryCursor::new();
    let source_bytes = source.as_bytes();

    let item_idx = query.capture_index_for_name("item").unwrap();
    let name_idx = query.capture_index_for_name("name");
    let vis_idx = query.capture_index_for_name("vis");
    let body_idx = query.capture_index_for_name("body");

    let mut items_map: BTreeMap<usize, Item> = BTreeMap::new();

    let root = tree.root_node();
    let mut matches_iter = cursor.matches(&query, root, source_bytes);

    while let Some(m) = matches_iter.next() {
        let item_node = match m.captures.iter().find(|c| c.index == item_idx) {
            Some(c) => c.node,
            None => continue,
        };

        let kind_str = item_node.kind();

        let visibility = vis_idx
            .and_then(|idx| m.captures.iter().find(|c| c.index == idx))
            .map(|c| Visibility::from_node(Some(c.node), source))
            .unwrap_or(Visibility::Private);

        let name = name_idx
            .and_then(|idx| m.captures.iter().find(|c| c.index == idx))
            .map(|c| source[c.node.byte_range()].to_string());

        let body_node = body_idx
            .and_then(|idx| m.captures.iter().find(|c| c.index == idx))
            .map(|c| c.node);

        let kind = match extractor.node_kind_to_item_kind(kind_str) {
            Some(k) => k,
            None => continue,
        };
        let (effective_start_byte, line_start) = find_attr_start(item_node);
        let line_end = item_node.end_position().row + 1;

        let (content, line_mappings, has_body) = match kind_str {
            "impl_item" | "trait_item" => {
                let (c, m) = collapse_block(source, effective_start_byte, item_node);
                (c, m, false)
            }
            _ if body_node.is_some() => {
                let body = body_node.unwrap();
                let (c, m) = collapse_body(
                    source,
                    effective_start_byte,
                    item_node.end_byte(),
                    body.start_byte(),
                    body.end_byte(),
                );
                (c, m, true)
            }
            _ => {
                let text = &source[effective_start_byte..item_node.end_byte()];
                (text.to_string(), Vec::new(), false)
            }
        };

        let name = if kind_str == "impl_item" {
            extractor.extract_impl_name(item_node, source)
        } else {
            name
        };

        let line_mappings = if line_mappings.is_empty() {
            Some(build_source_line_mappings(&content, line_start))
        } else {
            Some(line_mappings)
        };

        items_map.entry(line_start).or_insert(Item {
            kind: kind.clone(),
            name: name.clone(),
            visibility: visibility.clone(),
            line_start,
            line_end,
            signature: None,
            body: if has_body { Some("{ ... }".to_string()) } else { None },
            content: content.clone(),
            line_mappings: line_mappings.clone(),
        });

        if matches!(kind_str, "impl_item" | "trait_item") {
            extractor.extract_methods_from_block(source, item_node, &mut items_map);
        }
    }

    items_map.into_values().collect()
}
