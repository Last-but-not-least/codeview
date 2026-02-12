# codeview

A code context extractor powered by [Tree-sitter](https://tree-sitter.github.io/). Shows the shape of a codebase — signatures, types, structure — without the noise. Supports symbol-aware editing.

**[Try the playground →](https://last-but-not-least.github.io/codeview/)**

## Install

### Quick install (Linux / macOS)

```sh
curl -fsSL https://raw.githubusercontent.com/Last-but-not-least/codeview/main/install.sh | sh
```

This auto-detects your OS and architecture, downloads the latest release binary, and installs to `/usr/local/bin`. Set `INSTALL_DIR` to change the location, or `VERSION` to pin a specific version:

```sh
INSTALL_DIR=~/.local/bin VERSION=v0.0.1 curl -fsSL https://raw.githubusercontent.com/Last-but-not-least/codeview/main/install.sh | sh
```

### Download from GitHub Releases

Prebuilt binaries for every release: [GitHub Releases](https://github.com/Last-but-not-least/codeview/releases)

| Target | Archive |
|--------|---------|
| Linux x86_64 (static/musl) | `codeview-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 | `codeview-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `codeview-<version>-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `codeview-<version>-aarch64-apple-darwin.tar.gz` |

Each release includes a `checksums.sha256` file for verification.

### Build from source

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

### Class signatures mode

Inspect a class with method bodies collapsed — see the shape without the noise:

```sh
$ codeview src/api.ts UserService --signatures
```

```
src/api.ts::UserService [11:30]
11 | export class UserService {
14 |     constructor(db: Database) { ... }
18 |     public getUser(id: UserId): User | undefined { ... }
22 |     public createUser(name: string, age: number): User { ... }
27 |     private validate(user: User): boolean { ... }
30 | }
```

Combine with specific method expansion — signatures for the class, full body for selected methods:

```sh
$ codeview src/api.ts UserService --signatures getUser
```

### Bounded expand

Peek at large symbols without dumping the full body:

```sh
$ codeview src/api.ts processData --max-lines 20
```

Truncates after N lines with a `... [truncated: X more lines]` indicator. Works with `--signatures` too.

### Structural search

Grep with AST context — matches are annotated with their enclosing class/method:

```sh
$ codeview src/api.ts --search "validate"
```

```
src/api.ts
  UserService > createUser()
    L24:     if (!this.validate(user)) {
  UserService > validate()
    L27:     private validate(user: User): boolean {
```

Supports regex, case-insensitive (`-i`), and directory search:

```sh
$ codeview src/ --search "TODO|FIXME" -i
```

### Directory mode

Point at a directory to walk all supported files:

```sh
$ codeview src/
$ codeview src/ --depth 0    # target dir only, no subdirs
$ codeview src/ --depth 1    # one level deep
$ codeview src/ --ext rs,ts  # only .rs and .ts files
```

Use `--ext` to filter by file extension (comma-separated, without the dot).

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

### Python support

Works with `.py` files. The `_private` naming convention maps to private visibility:

```sh
$ codeview app.py
```

```
app.py
 1 | import os
 2 | from dataclasses import dataclass

 4 | @dataclass
 5 | class Config:
 6 |     host: str
 7 |     port: int
 8 |     _secret: str

11 | class App:
12 |     def __init__(self, config: Config): ...
15 |     def run(self): ...
18 |     def handle_request(self, path: str) -> dict: ...
22 |     def _validate(self, data: dict) -> bool: ...

25 | def create_app(env: str = "dev") -> App: ...

28 | def _load_defaults() -> dict: ...
```

Names starting with `_` are treated as private — `--pub` will hide `_validate`, `_load_defaults`, and `_secret`.

### JavaScript support

Works with `.js` and `.jsx` files:

```sh
$ codeview api.js
```

```
api.js
 1 | import express from "express";

 3 | export class Router {
 4 |     constructor(prefix) { ... }
 7 |     get(path, handler) { ... }
10 |     post(path, handler) { ... }
13 | }

15 | export function createApp(config) { ... }

19 | function loadMiddleware(name) { ... }

22 | export default Router;
```

## Filters

| Flag         | Effect                                       |
|--------------|----------------------------------------------|
| `--pub`      | Only public/exported items                   |
| `--fns`      | Only functions and methods                   |
| `--types`    | Only types (struct/class, enum, trait/interface, type alias) |
| `--no-tests` | Exclude test blocks (`#[cfg(test)]` in Rust)  |
| `--depth N`  | Limit directory recursion (0 = target dir only) |
| `--ext rs,ts` | Filter directory walk by file extension (comma-separated) |
| `--signatures` | Class signatures mode (collapsed method bodies) |
| `--max-lines N` | Truncate expanded output after N lines      |
| `--search "pat"` | Structural grep (matches with AST context) |
| `-i`         | Case-insensitive search (with `--search`)    |
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

### JSON output

Add `--json` to any edit command to get structured JSON metadata about what changed:

```sh
$ codeview edit src/lib.rs helper --replace 'fn helper() {}' --json
```

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
│   ├── typescript.rs    # TypeScript/TSX tree-sitter queries
│   ├── python.rs        # Python tree-sitter queries
│   └── javascript.rs    # JavaScript/JSX tree-sitter queries
├── extractor/           # Item extraction from AST
│   ├── mod.rs           # Item/ItemKind/Visibility types, LanguageExtractor trait
│   ├── interface.rs     # Interface mode (collapsed bodies)
│   ├── expand.rs        # Expand mode (full source for named symbols)
│   ├── collapse.rs      # Body collapsing logic
│   ├── rust.rs          # Rust-specific extraction (impl blocks, fn signatures)
│   ├── typescript.rs    # TypeScript/TSX-specific extraction
│   ├── python.rs        # Python-specific extraction (classes, decorators)
│   └── javascript.rs    # JavaScript/JSX-specific extraction
├── search.rs            # Structural search (--search, AST-aware grep)
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
- TypeScript (`.ts`, `.tsx`)
- Python (`.py`)
- JavaScript (`.js`, `.jsx`)

## OpenClaw Skill

codeview ships as an [OpenClaw](https://openclaw.ai) agent skill, letting AI agents read and edit code with full structural awareness.

### Setup

1. Install the `codeview` binary (see [Install](#install))
2. Copy the `skill/` directory into your OpenClaw workspace skills folder:

```sh
cp -r skill/ ~/.openclaw/workspace/skills/codeview/
```

3. The agent will automatically use codeview instead of raw file reads for `.rs`, `.ts`, `.tsx`, `.py`, `.js`, and `.jsx` files.

### What the skill provides

- **Structural reading** — agents see file shapes (signatures, types, fields) without drowning in implementation details
- **Targeted expansion** — agents can expand specific symbols to read only the code they need
- **Symbol-aware editing** — replace, replace-body, delete, and batch edits with automatic syntax validation
- **Directory browsing** — walk entire crates/packages with filters (`--pub`, `--fns`, `--types`, `--no-tests`)

### Example agent workflow

```
1. Search:  codeview src/ --search "keyword"                     → find where things live (with structural context)
2. Browse:  codeview src/                                        → see project shape
3. Focus:   codeview src/lib.rs                                  → see file interface
4. Inspect: codeview src/lib.rs MyClass --signatures             → see class methods without bodies
5. Read:    codeview src/lib.rs MyClass --signatures target_fn   → expand only the method you need
6. Edit:    codeview edit src/lib.rs target_fn --replace-body '{ new_impl }'
7. Verify:  codeview src/lib.rs target_fn                        → confirm the edit
```

The skill reduces token usage by giving agents structural awareness — collapsed views, class signatures, bounded expansion, and structural search replace brute-force file reading. The syntax validation on edits prevents agents from introducing parse errors.

## License

MIT
