# codeview

A code context extractor powered by [Tree-sitter](https://tree-sitter.github.io/). Shows the shape of a codebase — signatures, types, structure — without the noise. Supports symbol-aware editing.

## Install

```sh
cargo install --path .
```

## Reading Code

### Interface mode (default)

Shows file structure with function bodies collapsed to `{ ... }`:

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

Line numbers match the original file — collapsed bodies don't shift numbering.

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

Point at a directory to walk all supported files:

```sh
$ codeview src/
$ codeview src/ --depth 0    # target dir only, no subdirs
$ codeview src/ --depth 1    # one level deep
```

Respects `.gitignore`, `.ignore`, and global gitignore — `target/`, `node_modules/`, etc. are skipped automatically.

In expand mode, directory traversal stops early once all requested symbols have been found.

### Stats mode

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

Also works with `--json` for structured output.

### TypeScript support

Works identically with `.ts` and `.tsx` files:

```sh
$ codeview src/api.ts
```

```
src/api.ts
 1 | import { Database } from "./db";
 3 | export interface User {
 4 |     name: string;
 5 |     age: number;
 6 |     email?: string;
 7 | }
 9 | export type UserId = string | number;
11 | export class UserService {
14 |     constructor(db: Database) { ... }
18 |     public getUser(id: UserId): User | undefined { ... }
22 |     public createUser(name: string, age: number): User { ... }
27 |     private validate(user: User): boolean { ... }
30 | }
32 | export function parseUserId(raw: string): UserId { ... }
```

## Filters

| Flag         | Effect                                       |
|--------------|----------------------------------------------|
| `--pub`      | Only public/exported items                   |
| `--fns`      | Only functions and methods                   |
| `--types`    | Only types (struct/class, enum, trait/interface, type alias) |
| `--no-tests` | Exclude `#[cfg(test)] mod tests` blocks      |
| `--depth N`  | Limit directory recursion (0 = target dir only) |
| `--json`     | JSON output                                  |
| `--stats`    | Show file/item counts instead of content     |

Filters compose: `--pub --fns` shows only public functions.

## Editing Code

codeview can edit files by targeting symbols by name. All edits are **validated** — if the result produces invalid syntax (tree-sitter re-parse), the operation is rejected and the file is left untouched.

Attributes are handled correctly: deleting or replacing a symbol includes its attributes (e.g. `#[derive(...)]`) in the affected range.

### Replace a symbol

Replace the entire symbol (signature + body + attributes):

```sh
$ codeview edit src/lib.rs helper --replace 'fn helper() -> i32 { 42 }'

# Read replacement from stdin (for multi-line edits)
$ cat <<'EOF' | codeview edit src/lib.rs helper --replace --stdin
fn helper(x: i32) -> i32 {
    x * 2
}
EOF
```

### Replace only the body

Keep the existing signature and attributes, replace just the body:

```sh
$ codeview edit src/lib.rs helper --replace-body '{ 42 }'

# From stdin
$ echo '{ x * 2 }' | codeview edit src/lib.rs helper --replace-body --stdin
```

### Delete a symbol

```sh
$ codeview edit src/lib.rs helper --delete
```

### Batch edits

Apply multiple edits to one file atomically via a JSON file:

```sh
$ codeview edit src/lib.rs --batch edits.json
```

```json
[
  { "symbol": "foo", "action": "replace", "content": "fn foo() {}" },
  { "symbol": "bar", "action": "replace-body", "content": "{ 0 }" },
  { "symbol": "baz", "action": "delete" }
]
```

Actions: `replace`, `replace-body`, `delete`. The `content` field is required for replace/replace-body, ignored for delete.

### Dry run

Add `--dry-run` to any edit command to print the result to stdout without writing the file:

```sh
$ codeview edit src/lib.rs helper --replace 'fn helper() {}' --dry-run
```

## Architecture

```
src/
├── main.rs              # CLI entry (clap)
├── lib.rs               # Core orchestration (process_path)
├── parser.rs            # Tree-sitter parsing
├── error.rs             # Error types (thiserror)
├── languages/           # Language detection + grammar queries
│   ├── mod.rs           # Language enum, detection, TS language loader
│   ├── rust.rs          # Rust tree-sitter queries
│   └── typescript.rs    # TypeScript/TSX tree-sitter queries
├── extractor/           # Item extraction from AST
│   ├── mod.rs           # Item/ItemKind/Visibility types, LanguageExtractor trait
│   ├── interface.rs     # Interface mode (collapsed bodies)
│   ├── expand.rs        # Expand mode (full source for named symbols)
│   ├── collapse.rs      # Body collapsing logic
│   ├── rust.rs          # Rust-specific extraction (impl blocks, fn signatures)
│   └── typescript.rs    # TypeScript/TSX-specific extraction
├── editor/              # Symbol-aware editing
│   └── mod.rs           # replace, replace_body, delete, batch — with validation
├── output/              # Formatters
│   ├── mod.rs           # OutputFormat enum
│   ├── plain.rs         # Plain text formatter (with line numbers)
│   ├── json.rs          # JSON formatter
│   └── stats.rs         # Stats formatter (file/line/item counts)
└── walk.rs              # Directory traversal (ignore crate, respects .gitignore)
```

## Supported Languages

- Rust (`.rs`)
- TypeScript (`.ts`)
- TSX (`.tsx`)

## License

MIT
