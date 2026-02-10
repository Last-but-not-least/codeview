// Rust language tree-sitter queries

/// Query for extracting all top-level items in interface mode.
/// Methods inside impl blocks are extracted programmatically from the impl_item node,
/// NOT via a separate query pattern (avoids duplicates and depth issues).
pub const INTERFACE_QUERY: &str = r#"
;; Top-level function (source_file > function_item)
(source_file
  (function_item
    (visibility_modifier)? @vis
    name: (identifier) @name
    body: (block) @body) @item)

;; Struct
(source_file
  (struct_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Enum
(source_file
  (enum_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Trait
(source_file
  (trait_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Impl block (methods extracted from node, not query)
(source_file
  (impl_item) @item)

;; Module
(source_file
  (mod_item
    (visibility_modifier)? @vis
    name: (identifier) @name) @item)

;; Use declaration
(source_file
  (use_declaration
    (visibility_modifier)? @vis) @item)

;; Const
(source_file
  (const_item
    (visibility_modifier)? @vis
    name: (identifier) @name) @item)

;; Static
(source_file
  (static_item
    (visibility_modifier)? @vis
    name: (identifier) @name) @item)

;; Type alias
(source_file
  (type_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Macro definition
(source_file
  (macro_definition
    name: (identifier) @name) @item)

;; Attributed items: attribute_item followed by an item
;; (handled programmatically by looking at preceding siblings)
"#;

/// Query for extracting items by name in expand mode.
/// Matches at any depth to find named items.
pub const EXPAND_QUERY: &str = r#"
(function_item
  name: (identifier) @name) @item

(struct_item
  name: (type_identifier) @name) @item

(enum_item
  name: (type_identifier) @name) @item

(trait_item
  name: (type_identifier) @name) @item

(impl_item
  type: (_) @impl_type) @item

(mod_item
  name: (identifier) @name) @item

(const_item
  name: (identifier) @name) @item

(static_item
  name: (identifier) @name) @item

(type_item
  name: (type_identifier) @name) @item

(macro_definition
  name: (identifier) @name) @item
"#;
