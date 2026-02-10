# Codeview Edit Module — Plan

## Goal
Add symbol-aware editing to codeview. Agents think in **symbols**, not line numbers.

## CLI Interface

```
codeview edit <FILE> <SYMBOL> --replace <NEW_SOURCE>
codeview edit <FILE> <SYMBOL> --replace-body <NEW_BODY>
codeview edit <FILE> <SYMBOL> --delete
codeview edit <FILE> <SYMBOL> --dry-run   (combine with above; print result to stdout instead of writing)
```

## Architecture

New module: `src/editor/mod.rs`

### Core Flow
1. Parse file (`parser::parse`)
2. Find target symbol via expand query (`expand::extract`)
3. Resolve byte range from tree-sitter node
4. Apply edit operation (replace / replace-body / delete)
5. Validate result (re-parse, check `root.has_error()`)
6. Write back to file (or stdout with `--dry-run`)

### Operations

**`--replace`** — Replace entire item (including attributes) with new source text.

**`--replace-body`** — Replace only the `{ ... }` block, preserving signature/attributes/visibility. Reuse `collapse.rs` logic (in reverse) to find block boundaries. Re-indent new body to match context.

**`--delete`** — Remove the item entirely (including attributes and trailing newline).

### Key Implementation Details

- **Byte-range resolution:** Tree-sitter nodes give exact byte offsets. Use `find_attr_start` for attribute-inclusive ranges.
- **Multi-edit:** For batch edits, apply from bottom-to-top so byte offsets stay valid. Accept JSON input:
  ```json
  {"edits": [
    {"symbol": "process_file", "action": "replace-body", "content": "..."},
    {"symbol": "OutputFormat", "action": "replace", "content": "..."}
  ]}
  ```
- **Validation:** After edit, re-parse with tree-sitter → `root.has_error()` = reject before writing.
- **Body boundary detection:** Find `block` (Rust) / `statement_block` (TS) child node. `collapse.rs` already does this.
- **Re-indentation:** Detect indent level of the replaced block, apply to new content.

### Effort Estimate
~400 lines total. Weekend project.

| Component | ~Lines |
|---|---|
| Symbol → byte range resolution | 50 |
| Replace/delete operations | 80 |
| Body-only replacement | 60 |
| Re-indentation | 40 |
| Validation (re-parse check) | 20 |
| CLI flags + integration | 50 |
| Tests | 100 |

## V1 Scope (MVP)
- `--replace` full symbol replacement
- `--delete` symbol deletion
- `--dry-run` mode (stdout, don't write)
- Validation via re-parse
- Single symbol per invocation
- Rust + TypeScript support

## V2 (Later)
- `--replace-body` with auto re-indent
- Batch/multi-edit via JSON
- `--stdin` for reading replacement content
