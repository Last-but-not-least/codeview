# Issue #001: Clean up dead code in extractor

## Summary

Two `#[allow(dead_code)]` annotations in `src/extractor/mod.rs` suppress warnings for code that's genuinely unused. These should be resolved rather than silenced.

## Details

### 1. `ItemKind::Attribute` variant (line 45-46)

```rust
#[allow(dead_code)]
Attribute,
```

**Problem:** The `Attribute` variant is never constructed anywhere in the codebase. Attributes are handled programmatically by `find_attr_start()` which looks at preceding siblings — they're never extracted as standalone `Item`s.

**Fix:** Remove the variant entirely. If attribute-as-item support is planned for the future, add it when needed.

### 2. `LanguageExtractor::build_fn_signature` trait method (line 124-125)

```rust
#[allow(dead_code)]
fn build_fn_signature(&self, source: &str, node: tree_sitter::Node) -> String;
```

**Problem:** This trait method is never called through the trait interface. The only usage is the free function `rust::build_fn_signature()` called directly in `rust::extract_methods_from_block()`. The trait impl on `RustExtractor` just delegates to that free function, but nobody calls `extractor.build_fn_signature(...)`.

**Fix:** Remove `build_fn_signature` from the `LanguageExtractor` trait. Keep the free function in `rust.rs` — that's what's actually used. When a second language is added, decide then whether signature building belongs in the trait contract.

## Impact

No behavior change. Purely cleanup — removes ~10 lines and two `#[allow(dead_code)]` suppressions.

## Files affected

- `src/extractor/mod.rs` — remove `Attribute` variant + trait method
- `src/extractor/rust.rs` — remove trait impl for `build_fn_signature`
