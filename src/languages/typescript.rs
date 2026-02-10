/// Tree-sitter query for TypeScript/TSX interface (top-level items only).
pub const INTERFACE_QUERY: &str = r#"
; Top-level function declarations
(program
  (function_declaration
    name: (identifier) @name
    body: (statement_block) @body) @item)

; Exported function declarations
(program
  (export_statement
    (function_declaration
      name: (identifier) @name
      body: (statement_block) @body)) @item)

; Top-level class declarations
(program
  (class_declaration
    name: (type_identifier) @name
    body: (class_body) @body) @item)

; Exported class declarations
(program
  (export_statement
    (class_declaration
      name: (type_identifier) @name
      body: (class_body) @body)) @item)
; Top-level abstract class declarations(program  (abstract_class_declaration    name: (type_identifier) @name    body: (class_body) @body) @item); Exported abstract class declarations(program  (export_statement    (abstract_class_declaration      name: (type_identifier) @name      body: (class_body) @body)) @item)

; Top-level abstract class declarations
(program
  (abstract_class_declaration
    name: (type_identifier) @name
    body: (class_body) @body) @item)

; Exported abstract class declarations
(program
  (export_statement
    (abstract_class_declaration
      name: (type_identifier) @name
      body: (class_body) @body)) @item)

; Top-level interface declarations
(program
  (interface_declaration
    name: (type_identifier) @name
    body: (interface_body) @body) @item)

; Exported interface declarations
(program
  (export_statement
    (interface_declaration
      name: (type_identifier) @name
      body: (interface_body) @body)) @item)

; Top-level type alias declarations
(program
  (type_alias_declaration
    name: (type_identifier) @name) @item)

; Exported type alias declarations
(program
  (export_statement
    (type_alias_declaration
      name: (type_identifier) @name)) @item)

; Top-level enum declarations
(program
  (enum_declaration
    name: (identifier) @name
    body: (enum_body) @body) @item)

; Exported enum declarations
(program
  (export_statement
    (enum_declaration
      name: (identifier) @name
      body: (enum_body) @body)) @item)

; Top-level import statements
(program
  (import_statement) @item)

; Top-level lexical declarations (const/let)
(program
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name)) @item)

; Exported lexical declarations
(program
  (export_statement
    (lexical_declaration
      (variable_declarator
        name: (identifier) @name))) @item)
"#;

/// Tree-sitter query for TypeScript/TSX expand (not restricted to top-level).
pub const EXPAND_QUERY: &str = r#"
(function_declaration
  name: (identifier) @name
  body: (statement_block) @body) @item

(class_declaration
  name: (type_identifier) @name
  body: (class_body) @body) @item

(abstract_class_declaration
  name: (type_identifier) @name
  body: (class_body) @body) @item

(interface_declaration
  name: (type_identifier) @name
  body: (interface_body) @body) @item

(type_alias_declaration
  name: (type_identifier) @name) @item

(enum_declaration
  name: (identifier) @name
  body: (enum_body) @body) @item

(lexical_declaration
  (variable_declarator
    name: (identifier) @name)) @item

(import_statement) @item

(export_statement
  (function_declaration
    name: (identifier) @name
    body: (statement_block) @body)) @item

(export_statement
  (class_declaration
    name: (type_identifier) @name
    body: (class_body) @body)) @item

(export_statement
  (abstract_class_declaration
    name: (type_identifier) @name
    body: (class_body) @body)) @item

(export_statement
  (interface_declaration
    name: (type_identifier) @name
    body: (interface_body) @body)) @item

(export_statement
  (type_alias_declaration
    name: (type_identifier) @name)) @item

(export_statement
  (enum_declaration
    name: (identifier) @name
    body: (enum_body) @body)) @item

(export_statement
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name))) @item
"#;
