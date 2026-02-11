---
name: codeview
description: Read and edit code files (.rs, .ts, .tsx) using codeview — a Tree-sitter-based code context extractor. Use INSTEAD of the read/edit tools for any Rust or TypeScript source file. Provides intelligent code viewing (collapsed function bodies, symbol expansion) and structured editing (replace symbols, replace bodies, delete symbols, batch edits). Use for all code reading, browsing, and editing tasks involving .rs, .ts, or .tsx files.
---

# codeview

Tree-sitter-powered code reader and editor. **Always use this instead of `read`/`edit` for `.rs`, `.ts`, `.tsx` files.**

## Setup

Install codeview and ensure it's on your PATH:

```sh
cargo install --path /path/to/codeview
```

Then invoke directly:

```sh
codeview <args>
```

If codeview runs on a different host (e.g. outside a container), adapt the invocation accordingly — the commands below remain the same.

## Reading Code

### Interface mode (no symbols) — see file shape

```
codeview path/to/file.rs
```

Shows signatures, types, struct fields — function bodies collapsed to `{ ... }`. Perfect for understanding structure without flooding context.

### Expand mode (with symbols) — see implementation

```
codeview path/to/file.rs my_function MyStruct
```

Expands named symbols fully. Use when you need to read specific function bodies or type definitions.

### Directory mode — browse a crate/package

```
codeview path/to/src/
codeview path/to/src/ --depth 2
codeview path/to/src/ --pub          # public items only
codeview path/to/src/ --fns          # functions/methods only
codeview path/to/src/ --types        # structs/enums/traits only
codeview path/to/src/ --no-tests     # skip test modules
codeview path/to/src/ --stats        # file/line/token counts
```

### Combining filters

```
codeview src/ --pub --fns            # public functions only
codeview src/lib.rs --pub --types    # public types in one file
```

## Editing Code

### Replace a symbol entirely

```
codeview edit path/to/file.rs my_function --replace 'fn my_function() -> i32 { 42 }'
```

### Replace only the body (preserves signature + attributes)

```
codeview edit path/to/file.rs my_function --replace-body '{ 42 }'
```

### Read replacement from stdin (for multi-line edits)

```bash
cat <<'EOF' | codeview edit path/to/file.rs my_function --replace --stdin
fn my_function(x: i32) -> i32 {
    x * 2
}
EOF
```

### Delete a symbol

```
codeview edit path/to/file.rs my_function --delete
```

### Batch edits (JSON file)

```
codeview edit path/to/file.rs --batch edits.json
```

Batch JSON format:
```json
[
  {"symbol": "foo", "action": "replace", "content": "fn foo() {}"},
  {"symbol": "bar", "action": "replace_body", "content": "{ 0 }"},
  {"symbol": "baz", "action": "delete"}
]
```

### Dry run (preview without writing)

Add `--dry-run` to any edit command to print the result to stdout instead of writing.

## Workflow

1. **Browse**: `codeview src/` to see project shape
2. **Focus**: `codeview src/lib.rs` to see a file's interface
3. **Read**: `codeview src/lib.rs specific_fn` to read implementation
4. **Edit**: `codeview edit src/lib.rs specific_fn --replace-body '{ new_impl }'`
5. **Verify**: `codeview src/lib.rs specific_fn` to confirm the edit

## When NOT to use codeview

- Non-code files (`.md`, `.toml`, `.json`, `.yaml`, etc.) → use regular `read`/`edit`
- Creating new files → use `write` tool, then browse with codeview
- Adding new symbols to existing files → use `edit` tool to insert, then verify with codeview
