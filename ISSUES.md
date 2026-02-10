# Issues

## #1 — Methods invisible in plain text with `--fns` / `--pub --fns` filters

**Status:** ✅ Resolved (2026-02-10)
**Severity:** Bug — core functionality broken for filtered method output

### Problem

When using `--fns` or `--pub --fns`, methods inside `impl` blocks produce empty output in plain text mode. JSON mode works correctly.

```bash
# Shows nothing:
codeview file.rs --fns
codeview file.rs --pub --fns

# JSON correctly shows methods with names, signatures, visibility:
codeview file.rs --fns --json
```

### Root Cause

The plain text formatter uses `generate_content_with_collapses()` to render each item. This function walks the source lines within an item's `[line_start, line_end]` range and replaces body ranges with ` { ... }`.

For **methods**, this breaks because:

1. `collect_impl_method_bodies()` registers body collapse ranges for method bodies (e.g. `{` on line 2 through `}` on line 4)
2. `extract_impl_methods()` creates method items with the same line range (2-4)
3. When `generate_content_with_collapses()` processes the method item, the method's **signature line** (e.g. `pub fn new(name: String) -> Self {`) is the **same line** as the body range start
4. So the entire method content becomes just ` { ... }` — the signature is swallowed

The `generate_content_with_collapses` model was designed for top-level items where the body `{` is a subset of the item's line range. For methods, the body IS most of the item.

### Why JSON works

JSON output uses `Item.signature`, `Item.name`, `Item.visibility` fields directly — it never goes through `generate_content_with_collapses`. The data is all there; only the plain renderer is broken.

### Possible fixes

#### Option A: Don't collapse method bodies in body_ranges
Stop `collect_impl_method_bodies` from adding body ranges. Instead, let `generate_content_with_collapses` handle methods by only collapsing the function body block node, not the whole line. Problem: the signature and `{` are on the same source line in Rust.

#### Option B: Render methods from structured data, not source lines
For `ItemKind::Method`, skip `generate_content_with_collapses` entirely. Instead, build content from `signature + " { ... }"`. The data is already extracted correctly. This is simpler but loses original formatting.

#### Option C: Split the collapse model
Use tree-sitter to identify the exact byte range of just the body block `{}`, and collapse at byte level rather than line level. This preserves the signature portion of the line. More accurate but requires rethinking the line-based collapse model.

#### Option D: Use tree-sitter queries instead of manual walking
Replace the manual `collect_body_ranges` / `collect_items_with_attributes` approach with tree-sitter's query language (S-expressions). Queries can pattern-match nested structures naturally:

```scheme
;; Match all function items, including inside impl blocks
(function_item
  name: (identifier) @fn_name
  parameters: (parameters) @params
  body: (block) @body) @fn

;; Match impl blocks
(impl_item
  type: (type_identifier) @impl_type
  body: (declaration_list) @impl_body) @impl
```

This would be a larger refactor but produces cleaner, more maintainable extraction logic. Tree-sitter queries handle nesting naturally — no need to manually recurse into impl blocks.

### Resolution

**Implemented Option D** — full tree-sitter query refactor:
- `src/languages/rust.rs`: S-expression queries for all Rust item types
- `src/extractor/interface.rs`: Query-based extraction with byte-level body collapse
- `src/extractor/expand.rs`: Query-based symbol matching
- `src/lib.rs`: Smart filtering — methods hidden in full mode (shown in impl blocks), shown individually with `--fns`
- Body replacement at byte level preserves signatures on same line as `{`

### Test case

```rust
impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }
    fn private_method(&self) -> bool {
        true
    }
}
```

Expected `--fns` output:
```
file.rs
2 | pub fn new (name: String) -> Self { ... }
6 | fn private_method (&self) -> bool { ... }
```

Expected `--pub --fns` output:
```
file.rs
2 | pub fn new (name: String) -> Self { ... }
```
