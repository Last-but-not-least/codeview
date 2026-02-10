# codeview

A code context extractor powered by [Tree-sitter](https://tree-sitter.github.io/). Shows the shape of a codebase — signatures, types, structure — without the noise.

## Install

```sh
cargo install --path .
```

## Usage

```
codeview [OPTIONS] <PATH> [SYMBOLS]...
```

### Interface mode (default)

Shows file structure with function bodies collapsed:

```sh
$ codeview src/lib.rs
```

```
src/lib.rs
 1 | use std::collections::HashMap;

 4 | #[derive(Debug, Clone)]
 5 | pub struct User {
 6 |     pub name: String,
 7 |     pub age: u32,
 8 |     email: String,
 9 | }

11 | impl User {
12 |     pub fn new(name: String, age: u32, email: String) -> Self { ... }
16 |     pub fn greeting(&self) -> String { ... }
20 |     fn validate_email(&self) -> bool { ... }
23 | }
```

### Expand mode

Pass symbol names to see their full implementation:

```sh
$ codeview src/lib.rs User new
```

```
src/lib.rs::User [4:9]
 4 | #[derive(Debug, Clone)]
 5 | pub struct User {
 6 |     pub name: String,
 7 |     pub age: u32,
 8 |     email: String,
 9 | }

src/lib.rs::new [12:14]
12 | pub fn new(name: String, age: u32, email: String) -> Self {
13 |         Self { name, age, email }
14 |     }
```

### Directory mode

Point at a directory to walk all supported files. Respects `.gitignore`, `.ignore`, and global gitignore — directories like `target/` and `node_modules/` are skipped automatically:

```sh
$ codeview src/
$ codeview src/ --depth 0    # target dir only, no subdirs
$ codeview src/ --depth 1    # target dir + one level of subdirs
```

In expand mode, directory traversal stops early once all requested symbols have been found — no need to parse remaining files.

## Stats mode

Show metadata instead of content — useful for context budgeting:

```sh
$ codeview src/ --stats
```

```
files: 16  lines: 1785  bytes: 56493  items: 111
  const: 2  enum: 5  function: 27  impl: 4  mod: 20  struct: 8  trait: 1  use: 44

  src/lib.rs — 166 lines, 5935 bytes, 14 items (2 function, 6 mod, 1 struct, 5 use)
  ...
```

Also works with `--json` for structured output. Composes with all filters.

## Filters

| Flag         | Effect                                       |
|--------------|----------------------------------------------|
| `--pub`      | Only public items                            |
| `--fns`      | Only functions and methods                   |
| `--types`    | Only types (struct, enum, trait, type alias)  |
| `--no-tests` | Exclude `#[cfg(test)] mod tests` blocks      |
| `--depth N`  | Limit directory recursion (0 = target dir only) |
| `--json`     | JSON output                                  |
| `--stats`    | Show file/item counts instead of content     |

Filters compose with union semantics — `--pub --fns` shows only public functions, `--fns --types` shows both functions and types.

## Supported languages

- Rust

## Architecture

```
src/
├── main.rs              # CLI entry (clap)
├── lib.rs               # Core orchestration
├── parser.rs            # Tree-sitter parsing
├── languages/           # Language detection + grammars
├── extractor/           # Item extraction
│   ├── interface.rs     # Interface mode (collapsed bodies)
│   ├── expand.rs        # Expand mode (full source for symbols)
│   ├── collapse.rs      # Body collapsing logic
│   ├── rust.rs          # Rust-specific extractor
│   └── mod.rs           # Item/Visibility types, LanguageExtractor trait
├── output/              # Formatters (plain text, JSON, stats)
└── walk.rs              # Directory traversal (ignore crate, respects .gitignore)
```

## License

MIT
