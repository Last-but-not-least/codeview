/// Tree-sitter query for JavaScript/JSX interface (top-level items only).
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
    name: (identifier) @name
    body: (class_body) @body) @item)

; Exported class declarations
(program
  (export_statement
    (class_declaration
      name: (identifier) @name
      body: (class_body) @body)) @item)

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

; Top-level variable declarations (var)
(program
  (variable_declaration
    (variable_declarator
      name: (identifier) @name)) @item)

; Exported variable declarations
(program
  (export_statement
    (variable_declaration
      (variable_declarator
        name: (identifier) @name))) @item)
"#;

/// Tree-sitter query for JavaScript/JSX expand (not restricted to top-level).
pub const EXPAND_QUERY: &str = r#"
(function_declaration
  name: (identifier) @name
  body: (statement_block) @body) @item

(class_declaration
  name: (identifier) @name
  body: (class_body) @body) @item

(lexical_declaration
  (variable_declarator
    name: (identifier) @name)) @item

(variable_declaration
  (variable_declarator
    name: (identifier) @name)) @item

(import_statement) @item

(export_statement
  (function_declaration
    name: (identifier) @name
    body: (statement_block) @body)) @item

(export_statement
  (class_declaration
    name: (identifier) @name
    body: (class_body) @body)) @item

(export_statement
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name))) @item

(export_statement
  (variable_declaration
    (variable_declarator
      name: (identifier) @name))) @item
"#;
