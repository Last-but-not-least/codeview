mod error;
mod parser;
mod extractor;
mod languages;
mod output;
mod walk;
pub mod editor;
pub mod search;

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
    pub signatures: bool,
    pub max_lines: Option<usize>,
    pub list_symbols: bool,
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
    
    // In signatures mode, first symbol is the class, rest are methods to expand
    let (symbols, expand_methods) = if options.signatures && options.symbols.len() > 1 {
        (vec![options.symbols[0].clone()], options.symbols[1..].to_vec())
    } else {
        (options.symbols.clone(), Vec::new())
    };
    
    let mut source_sizes: Vec<(usize, usize)> = Vec::new();
    let files_items: Vec<(String, Vec<Item>)> = if path.is_file() {
        let (items, lines, bytes) = process_file(path, &symbols, expand_mode, options.signatures, &expand_methods)?;
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
            match process_file(&file_path, &symbols, expand_mode, options.signatures, &expand_methods) {
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
    } else if options.list_symbols {
        output::plain::format_list_symbols(&filtered)
    } else {
        match options.format {
            OutputFormat::Plain => output::plain::format_output(&filtered, expand_mode, options.max_lines),
            OutputFormat::Json => output::json::format_output(&filtered),
        }
    }
}

/// Returns (items, lines, bytes)
/// Extract a line range from a file with structural context.
///
/// `lines_arg` should be in the format "N-M" (1-indexed, inclusive).
/// Returns formatted output with an enclosing-symbol context header and line numbers.
pub fn extract_lines(path_str: &str, lines_arg: &str) -> Result<String, CodeviewError> {
    use std::fmt::Write;

    let path = Path::new(path_str);
    if !path.exists() {
        return Err(CodeviewError::PathNotFound(path.display().to_string()));
    }
    if path.is_dir() {
        return Err(CodeviewError::InvalidPath(
            "--lines only works on single files, not directories".to_string(),
        ));
    }

    // Parse the range
    let (start, end) = parse_line_range(lines_arg)?;

    let source = fs::read_to_string(path).map_err(|e| CodeviewError::ReadError {
        path: path.display().to_string(),
        source: e,
    })?;

    let total_lines = source.lines().count();
    if start > total_lines {
        return Err(CodeviewError::ParseError(format!(
            "Start line {} is beyond end of file ({} lines)",
            start, total_lines
        )));
    }
    let end = end.min(total_lines);

    let language = languages::detect_language(path)?;
    let tree = parser::parse(&source, language)?;

    // Find enclosing symbols for the start line (0-indexed for tree-sitter)
    let symbols = search::find_enclosing_symbols(&tree, &source, start - 1, language);

    let mut output = String::new();

    // Context header
    if !symbols.is_empty() {
        writeln!(output, "// Inside: {}", symbols.join(" > ")).unwrap();
    }

    // Extract and format lines
    let lines: Vec<&str> = source.lines().collect();
    let width = end.to_string().len().max(start.to_string().len());
    for i in (start - 1)..end {
        writeln!(output, "L{:<width$}: {}", i + 1, lines[i], width = width).unwrap();
    }

    Ok(output)
}

fn parse_line_range(arg: &str) -> Result<(usize, usize), CodeviewError> {
    let parts: Vec<&str> = arg.split('-').collect();
    if parts.len() != 2 {
        return Err(CodeviewError::ParseError(format!(
            "Invalid line range '{}': expected format N-M (e.g. 50-75)",
            arg
        )));
    }
    let start: usize = parts[0].parse().map_err(|_| {
        CodeviewError::ParseError(format!("Invalid start line '{}' in range", parts[0]))
    })?;
    let end: usize = parts[1].parse().map_err(|_| {
        CodeviewError::ParseError(format!("Invalid end line '{}' in range", parts[1]))
    })?;
    if start == 0 {
        return Err(CodeviewError::ParseError(
            "Line numbers are 1-indexed; start line cannot be 0".to_string(),
        ));
    }
    if start > end {
        return Err(CodeviewError::ParseError(format!(
            "Inverted range: start line {} is after end line {}",
            start, end
        )));
    }
    Ok((start, end))
}

fn process_file(
    path: &Path,
    symbols: &[String],
    expand_mode: bool,
    signatures: bool,
    expand_methods: &[String],
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

    let items = if signatures && !symbols.is_empty() {
        extractor::expand::extract_signatures(&source, &tree, &symbols[0], expand_methods, language)
    } else if expand_mode {
        extractor::expand::extract(&source, &tree, symbols, language)
    } else {
        extractor::interface::extract(&source, &tree, language)
    };

    Ok((items, lines, bytes))
}
