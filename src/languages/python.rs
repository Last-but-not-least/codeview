/// Tree-sitter query for Python interface (top-level items only).
pub const INTERFACE_QUERY: &str = r#"
; Top-level function definitions
(module
  (function_definition
    name: (identifier) @name
    body: (block) @body) @item)

; Top-level decorated definitions (functions/classes with decorators)
(module
  (decorated_definition
    (function_definition
      name: (identifier) @name
      body: (block) @body)) @item)

; Top-level class definitions
(module
  (class_definition
    name: (identifier) @name
    body: (block) @body) @item)

; Top-level decorated class definitions
(module
  (decorated_definition
    (class_definition
      name: (identifier) @name
      body: (block) @body)) @item)

; Top-level import statements
(module
  (import_statement) @item)

; Top-level import-from statements
(module
  (import_from_statement) @item)

; Top-level assignments (constants)
(module
  (expression_statement
    (assignment
      left: (identifier) @name)) @item)
"#;

/// Tree-sitter query for Python expand (not restricted to top-level).
pub const EXPAND_QUERY: &str = r#"
(function_definition
  name: (identifier) @name
  body: (block) @body) @item

(decorated_definition
  (function_definition
    name: (identifier) @name
    body: (block) @body)) @item

(class_definition
  name: (identifier) @name
  body: (block) @body) @item

(decorated_definition
  (class_definition
    name: (identifier) @name
    body: (block) @body)) @item

(import_statement) @item

(import_from_statement) @item

(expression_statement
  (assignment
    left: (identifier) @name)) @item
"#;
