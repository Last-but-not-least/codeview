# Issue 001: Refactor interface.rs for multi-language support

## Summary
Split `src/extractor/interface.rs` (~400 lines) into focused modules and introduce a `LanguageExtractor` trait so adding new languages (TypeScript, Python, etc.) doesn't require touching core extraction logic.

## Current State
- `interface.rs` mixes three concerns: query-based extraction, body collapsing (language-agnostic text surgery), and Rust-specific signature/impl logic
- `ItemKind` node-kind mapping is duplicated between `interface.rs` and `expand.rs`
- Adding a new language would require forking the entire extraction loop

## Steps (execute in order)

### Step 1: `ItemKind::from_node_kind`
- Add `pub fn from_node_kind(kind: &str) -> Option<ItemKind>` to `ItemKind` in `src/extractor/mod.rs`
- Replace the match blocks in both `interface.rs` and `expand.rs` with calls to it
- **Test:** `cargo test` passes, `codeview` output unchanged

### Step 2: Extract `collapse` module
- Create `src/extractor/collapse.rs`
- Move from `interface.rs`: `collapse_body`, `collapse_inner_bodies`, `collect_fn_bodies`, `build_collapsed_block_mappings`, `build_source_line_mappings`
- These are language-agnostic (operate on byte ranges + source text)
- Public API: `pub fn collapse_body(...)`, `pub fn collapse_block(...)`, `pub fn build_source_line_mappings(...)`
- **Test:** `cargo test` passes, output unchanged

### Step 3: Extract Rust-specific helpers
- Create `src/extractor/rust.rs`
- Move from `interface.rs`: `build_fn_signature`, `extract_impl_name`, `extract_methods_from_block`
- These are Rust-specific tree-sitter node operations
- **Test:** `cargo test` passes, output unchanged

### Step 4: `LanguageExtractor` trait
- Create trait in `src/extractor/mod.rs` (or `src/extractor/lang.rs`):
```rust
pub trait LanguageExtractor {
    fn interface_query(&self) -> &str;
    fn expand_query(&self) -> &str;
    fn node_kind_to_item_kind(&self, kind: &str) -> Option<ItemKind>;
    fn extract_impl_name(&self, node: Node, source: &str) -> Option<String>;
    fn build_fn_signature(&self, source: &str, node: Node) -> String;
    fn extract_methods_from_block(&self, source: &str, block_node: Node, items: &mut BTreeMap<usize, Item>);
}
```
- Implement `RustExtractor` using the helpers from step 3
- Refactor `interface::extract` and `expand::extract` to take `&dyn LanguageExtractor`
- Wire up in `lib.rs` via `Language` → extractor lookup
- **Test:** `cargo test` passes, output unchanged

### Step 5: Clean up
- `interface.rs` should be ~80-100 lines (orchestration only)
- Remove any dead code, update `mod` declarations
- Run `cargo clippy` clean
- **Test:** full `cargo test`, compare output of `codeview src/` before/after

## Constraints
- Pure refactor — zero behavior changes. Output must be identical before/after each step.
- Do NOT modify test fixtures or test expectations (unless fixing a bug found during refactor)
- Each step must compile and pass tests before moving to the next
- Run `cargo test` after every step

## Files Involved
- `src/extractor/mod.rs` — Item types, new trait
- `src/extractor/interface.rs` — main refactor target
- `src/extractor/expand.rs` — dedup node-kind mapping
- `src/extractor/collapse.rs` — new module
- `src/extractor/rust.rs` — new module
- `src/languages/rust.rs` — query constants (no changes expected)
- `src/lib.rs` — wiring if needed

## Verification
```sh
# Before refactor, capture baseline:
codeview src/ > /tmp/before.txt
codeview src/ --json > /tmp/before.json
codeview src/extractor/interface.rs collapse_body extract_rust > /tmp/before_expand.txt

# After each step:
cargo test
codeview src/ | diff /tmp/before.txt -
codeview src/ --json | diff /tmp/before.json -
```
