mod error;
mod parser;
mod extractor;
mod languages;
mod output;
mod walk;
pub mod editor;

use std::fs;
use std::path::Path;

pub use error::CodeviewError;
pub use output::OutputFormat;
pub use languages::Language;
use extractor::{Item, ItemKind};

/// Options for processing paths
pub struct ProcessOptions {
    pub symbols: Vec<String>,
    pub pub_only: bool,
    pub fns_only: bool,
    pub types_only: bool,
    pub no_tests: bool,
    pub depth: Option<usize>,
    pub format: OutputFormat,
    pub stats: bool,
    pub ext: Vec<String>,
}

/// Process a file or directory and return formatted output
pub fn process_path(
    path: &str,
    options: ProcessOptions,
) -> Result<String, CodeviewError> {
    let path = Path::new(path);
    
    if !path.exists() {
        return Err(CodeviewError::PathNotFound(path.display().to_string()));
    }

    let expand_mode = !options.symbols.is_empty();
    
    let mut source_sizes: Vec<(usize, usize)> = Vec::new();
    let files_items: Vec<(String, Vec<Item>)> = if path.is_file() {
        let (items, lines, bytes) = process_file(path, &options.symbols, expand_mode)?;
        source_sizes.push((lines, bytes));
        vec![(path.to_string_lossy().to_string(), items)]
    } else if path.is_dir() {
        let files = walk::walk_directory(path, options.depth, &options.ext)?;
        let mut results = Vec::new();
        // Track which symbols still need to be found for early exit in expand mode
        let mut remaining_symbols: Vec<&str> = if expand_mode {
            options.symbols.iter().map(|s| s.as_str()).collect()
        } else {
            Vec::new()
        };
        
        for file_path in files {
            match process_file(&file_path, &options.symbols, expand_mode) {
                Ok((items, lines, bytes)) => {
                    if expand_mode && !items.is_empty() {
                        // Remove found symbols from remaining set
                        for item in &items {
                            if let Some(name) = &item.name {
                                remaining_symbols.retain(|s| *s != name.as_str());
                            }
                        }
                    }
                    source_sizes.push((lines, bytes));
                    results.push((file_path.to_string_lossy().to_string(), items));
                    // Early exit: all symbols found
                    if expand_mode && remaining_symbols.is_empty() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to process {}: {}", file_path.display(), e);
                }
            }
        }
        results
    } else {
        return Err(CodeviewError::InvalidPath(path.display().to_string()));
    };

    // Apply filters (union semantics: if multiple kind filters, match ANY)
    let has_kind_filter = options.fns_only || options.types_only;
    let filtered: Vec<(String, Vec<Item>)> = files_items
        .into_iter()
        .map(|(path, items)| {
            let filtered_items = items
                .into_iter()
                .filter(|item| {
                    if options.no_tests
                        && matches!(item.kind, ItemKind::Mod)
                        && item.name.as_deref() == Some("tests")
                    {
                        return false;
                    }
                    if options.pub_only && !item.is_public() {
                        return false;
                    }
                    if has_kind_filter {
                        let is_fn = matches!(item.kind, ItemKind::Function | ItemKind::Method);
                        let is_type = matches!(
                            item.kind,
                            ItemKind::Struct | ItemKind::Enum | ItemKind::Trait | ItemKind::TypeAlias | ItemKind::Class
                        );
                        let mut matched = false;
                        if options.fns_only && is_fn { matched = true; }
                        if options.types_only && is_type { matched = true; }
                        if !matched { return false; }
                        // When only --types (no --fns), still hide standalone methods
                        if matches!(item.kind, ItemKind::Method) && !options.fns_only {
                            return false;
                        }
                    } else {
                        // No kind filter: hide standalone Method items (shown inside impl blocks)
                        if matches!(item.kind, ItemKind::Method) {
                            return false;
                        }
                    }
                    true
                })
                .collect();
            (path, filtered_items)
        })
        .collect();

    // Format output
    if options.stats {
        output::stats::format_output(&filtered, &source_sizes, options.format)
    } else {
        match options.format {
            OutputFormat::Plain => output::plain::format_output(&filtered, expand_mode),
            OutputFormat::Json => output::json::format_output(&filtered),
        }
    }
}

/// Returns (items, lines, bytes)
fn process_file(
    path: &Path,
    symbols: &[String],
    expand_mode: bool,
) -> Result<(Vec<Item>, usize, usize), CodeviewError> {
    let source = fs::read_to_string(path)
        .map_err(|e| CodeviewError::ReadError {
            path: path.display().to_string(),
            source: e,
        })?;

    let lines = source.lines().count();
    let bytes = source.len();

    let language = languages::detect_language(path)?;
    let tree = parser::parse(&source, language)?;

    let items = if expand_mode {
        extractor::expand::extract(&source, &tree, symbols, language)
    } else {
        extractor::interface::extract(&source, &tree, language)
    };

    Ok((items, lines, bytes))
}
