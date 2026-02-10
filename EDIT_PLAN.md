# Codeview Edit Module — Plan

## V1 — DONE ✅ (2026-02-10)
- `codeview edit <FILE> <SYMBOL> --replace <SOURCE>` — full symbol replacement
- `codeview edit <FILE> <SYMBOL> --delete` — symbol deletion
- `--dry-run` — print to stdout instead of writing
- `--stdin` — read replacement from stdin
- Validation via tree-sitter re-parse (rejects bad syntax)
- Attribute-aware (deletes/replaces include `#[derive(...)]` etc.)
- Rust + TypeScript support

## V2 — DONE ✅ (2026-02-10)
### `--replace-body` (the killer feature)
Replace only the `{ ... }` block, preserving signature/attributes/visibility.
- Agent writes just the new function body, codeview handles the rest
- Reuse `collapse.rs` logic (in reverse) to find block boundaries
- Find `block` (Rust) / `statement_block` (TS) child node
- Auto re-indent new body to match surrounding context
- Detect indent level of the original block, apply to replacement

### Batch/multi-edit via JSON
Apply multiple edits in one pass (bottom-to-top so byte offsets stay valid):
```json
{"edits": [
  {"symbol": "process_file", "action": "replace-body", "content": "..."},
  {"symbol": "OutputFormat", "action": "replace", "content": "..."}
]}
```
CLI: `codeview edit <FILE> --batch edits.json [--dry-run]`

## Future Ideas
- **Append/insert** — add a new method to an impl block, add a field to a struct
- **Rename** — rename a symbol (local to file)
- **Move** — extract a symbol to a different file
- **Persistent service/API** — cached ASTs, incremental re-parse
- **Symbol index** — query by type, visibility, return type, trait impl
- **Diff-aware mode** — structural diff from git changes
- **More languages** — Python, Go, Java
