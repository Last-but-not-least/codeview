pub mod rust;
pub mod collapse;
pub mod interface;
pub mod expand;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub kind: ItemKind,
    pub name: Option<String>,
    pub visibility: Visibility,
    pub line_start: usize,
    pub line_end: usize,
    pub signature: Option<String>,
    pub body: Option<String>,
    pub content: String,
    /// Explicit line mappings for content lines (line_num, text)
    /// Used when content has been modified (e.g., collapsed bodies)
    #[serde(skip)]
    pub line_mappings: Option<Vec<(usize, String)>>,
}

impl Item {
    pub fn is_public(&self) -> bool {
        matches!(self.visibility, Visibility::Public)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Mod,
    Use,
    Const,
    Static,
    TypeAlias,
    MacroDef,
}


impl ItemKind {
    pub fn from_node_kind(kind: &str) -> Option<ItemKind> {
        match kind {
            "function_item" => Some(ItemKind::Function),
            "struct_item" => Some(ItemKind::Struct),
            "enum_item" => Some(ItemKind::Enum),
            "trait_item" => Some(ItemKind::Trait),
            "impl_item" => Some(ItemKind::Impl),
            "mod_item" => Some(ItemKind::Mod),
            "use_declaration" => Some(ItemKind::Use),
            "const_item" => Some(ItemKind::Const),
            "static_item" => Some(ItemKind::Static),
            "type_item" => Some(ItemKind::TypeAlias),
            "macro_definition" => Some(ItemKind::MacroDef),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
}

/// Walk backwards through preceding `attribute_item` siblings to find the true start
/// of an attributed item (byte offset, 1-based line number).
pub fn find_attr_start(node: tree_sitter::Node) -> (usize, usize) {
    let mut start = node;
    loop {
        match start.prev_sibling() {
            Some(prev) if prev.kind() == "attribute_item" => start = prev,
            _ => break,
        }
    }
    (start.start_byte(), start.start_position().row + 1)
}

impl Visibility {
    pub fn from_node(node: Option<tree_sitter::Node>, source: &str) -> Self {
        if let Some(vis_node) = node {
            let vis_text = &source[vis_node.byte_range()];
            if vis_text.starts_with("pub") {
                if vis_text.contains("crate") {
                    return Visibility::Crate;
                } else if vis_text.contains("super") {
                    return Visibility::Super;
                }
                return Visibility::Public;
            }
        }
        Visibility::Private
    }

    /// Find visibility by searching children for `visibility_modifier` node kind
    pub fn from_parent(parent: tree_sitter::Node, source: &str) -> Self {
        let mut cursor = parent.walk();
        for child in parent.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                return Self::from_node(Some(child), source);
            }
        }
        Visibility::Private
    }
}

/// Resolve a `Language` to its concrete `LanguageExtractor`.
pub fn extractor_for(language: crate::languages::Language) -> Box<dyn LanguageExtractor> {
    match language {
        crate::languages::Language::Rust => Box::new(rust::RustExtractor),
    }
}

/// Language-specific extraction behavior.
pub trait LanguageExtractor {
    fn interface_query(&self) -> &str;
    fn expand_query(&self) -> &str;
    fn node_kind_to_item_kind(&self, kind: &str) -> Option<ItemKind>;
    fn extract_impl_name(&self, node: tree_sitter::Node, source: &str) -> Option<String>;
    fn extract_methods_from_block(&self, source: &str, block_node: tree_sitter::Node, items: &mut std::collections::BTreeMap<usize, Item>);
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::languages::Language;

    #[test]
    fn item_kind_from_node_kind_known() {
        assert_eq!(ItemKind::from_node_kind("function_item"), Some(ItemKind::Function));
        assert_eq!(ItemKind::from_node_kind("struct_item"), Some(ItemKind::Struct));
        assert_eq!(ItemKind::from_node_kind("enum_item"), Some(ItemKind::Enum));
        assert_eq!(ItemKind::from_node_kind("trait_item"), Some(ItemKind::Trait));
        assert_eq!(ItemKind::from_node_kind("impl_item"), Some(ItemKind::Impl));
        assert_eq!(ItemKind::from_node_kind("mod_item"), Some(ItemKind::Mod));
        assert_eq!(ItemKind::from_node_kind("use_declaration"), Some(ItemKind::Use));
        assert_eq!(ItemKind::from_node_kind("const_item"), Some(ItemKind::Const));
        assert_eq!(ItemKind::from_node_kind("static_item"), Some(ItemKind::Static));
        assert_eq!(ItemKind::from_node_kind("type_item"), Some(ItemKind::TypeAlias));
        assert_eq!(ItemKind::from_node_kind("macro_definition"), Some(ItemKind::MacroDef));
    }

    #[test]
    fn item_kind_from_node_kind_unknown() {
        assert_eq!(ItemKind::from_node_kind("if_expression"), None);
        assert_eq!(ItemKind::from_node_kind(""), None);
        assert_eq!(ItemKind::from_node_kind("random_garbage"), None);
    }

    #[test]
    fn visibility_from_node_none_is_private() {
        let vis = Visibility::from_node(None, "");
        assert_eq!(vis, Visibility::Private);
    }

    #[test]
    fn visibility_from_parent_pub() {
        let source = "pub fn foo() {}";
        let tree = parse(source, Language::Rust).unwrap();
        let root = tree.root_node();
        let fn_node = root.child(0).unwrap();
        let vis = Visibility::from_parent(fn_node, source);
        assert_eq!(vis, Visibility::Public);
    }

    #[test]
    fn visibility_from_parent_private() {
        let source = "fn foo() {}";
        let tree = parse(source, Language::Rust).unwrap();
        let root = tree.root_node();
        let fn_node = root.child(0).unwrap();
        let vis = Visibility::from_parent(fn_node, source);
        assert_eq!(vis, Visibility::Private);
    }

    #[test]
    fn visibility_from_parent_pub_crate() {
        let source = "pub(crate) fn foo() {}";
        let tree = parse(source, Language::Rust).unwrap();
        let root = tree.root_node();
        let fn_node = root.child(0).unwrap();
        let vis = Visibility::from_parent(fn_node, source);
        assert_eq!(vis, Visibility::Crate);
    }

    #[test]
    fn visibility_from_parent_pub_super() {
        let source = "pub(super) fn foo() {}";
        let tree = parse(source, Language::Rust).unwrap();
        let root = tree.root_node();
        let fn_node = root.child(0).unwrap();
        let vis = Visibility::from_parent(fn_node, source);
        assert_eq!(vis, Visibility::Super);
    }

    #[test]
    fn find_attr_start_no_attrs() {
        let source = "fn foo() {}";
        let tree = parse(source, Language::Rust).unwrap();
        let root = tree.root_node();
        let fn_node = root.child(0).unwrap();
        let (byte, line) = find_attr_start(fn_node);
        assert_eq!(byte, 0);
        assert_eq!(line, 1);
    }

    #[test]
    fn find_attr_start_with_attr() {
        let source = "#[inline]\nfn foo() {}";
        let tree = parse(source, Language::Rust).unwrap();
        let root = tree.root_node();
        // The fn_node should be the function_item
        let fn_node = root.child(1).unwrap();
        assert_eq!(fn_node.kind(), "function_item");
        let (byte, line) = find_attr_start(fn_node);
        assert_eq!(byte, 0); // attribute starts at byte 0
        assert_eq!(line, 1);
    }
}
