use super::{extractor_for, find_attr_start, Item, Visibility, LanguageExtractor};
use crate::languages::{ts_language, Language};
use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};

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
