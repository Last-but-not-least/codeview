use clap::{Parser, Subcommand};
use codeview::{editor, process_path, search, ProcessOptions, OutputFormat, Language, CodeviewError};
use codeview::editor::{BatchEdit, EditResult};
use std::{fs, io::{self, Read}, path::Path, process};

#[derive(Parser)]
#[command(name = "codeview")]
#[command(about = "Code context extractor using Tree-sitter", long_about = None, version)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// File or directory to analyze
    #[arg(value_name = "PATH")]
    path: Option<String>,
    
    /// Symbol names to expand (triggers expand mode)
    #[arg(value_name = "SYMBOLS")]
    symbols: Vec<String>,
    
    /// Only public items
    #[arg(long = "pub")]
    pub_only: bool,
    
    /// Only show functions/methods
    #[arg(long)]
    fns: bool,
    
    /// Only show types (struct/enum/trait/type alias)
    #[arg(long)]
    types: bool,
    
    /// Directory recursion depth (default: unlimited)
    #[arg(long)]
    depth: Option<usize>,
    
    /// JSON output instead of plain text
    #[arg(long)]
    json: bool,
    
    /// Exclude #[cfg(test)] mod tests blocks
    #[arg(long = "no-tests")]
    no_tests: bool,
    
    /// Show stats (file count, lines, bytes, tokens, items) instead of content
    #[arg(long)]
    stats: bool,

    /// Filter by file extensions (comma-separated, e.g. --ext rs,ts)
    #[arg(long, value_delimiter = ',')]
    ext: Vec<String>,

    /// Show class with method signatures collapsed (use with a class symbol)
    #[arg(long)]
    signatures: bool,

    /// Truncate expanded symbol output after N lines
    #[arg(long = "max-lines")]
    max_lines: Option<usize>,

    /// Search for pattern and show matches with structural context
    #[arg(long)]
    search: Option<String>,

    /// Case-insensitive search (use with --search)
    #[arg(short = 'i', requires = "search")]
    case_insensitive: bool,

    /// Maximum number of search matches to display (default: 20 for directory search, unlimited for single-file)
    #[arg(long = "max-results", requires = "search")]
    max_results: Option<usize>,
}

#[derive(Subcommand)]
enum Commands {
    /// Edit a symbol in a file
    Edit {
        /// File to edit
        file: String,
        
        /// Symbol name to edit (not needed with --batch)
        #[arg(default_value = "")]
        symbol: String,
        
        /// Replace the symbol with new source
        #[arg(long, conflicts_with_all = ["delete", "replace_body", "batch"])]
        replace: Option<String>,
        
        /// Replace only the body block, preserving signature/attributes
        #[arg(long = "replace-body", conflicts_with_all = ["delete", "replace", "batch"])]
        replace_body: Option<String>,
        
        /// Read replacement from stdin (works with --replace or --replace-body)
        #[arg(long)]
        stdin: bool,
        
        /// Delete the symbol
        #[arg(long, conflicts_with_all = ["replace", "replace_body", "batch"])]
        delete: bool,
        
        /// Apply batch edits from a JSON file
        #[arg(long, conflicts_with_all = ["replace", "replace_body", "delete"])]
        batch: Option<String>,
        
        /// Dry run - print to stdout instead of writing file
        #[arg(long)]
        dry_run: bool,
        
        /// Output JSON metadata about what changed
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Edit { file, symbol, replace, replace_body, stdin, delete, batch, dry_run, json }) => {
            if let Err(e) = handle_edit(&file, &symbol, replace, replace_body, stdin, delete, batch, dry_run, json) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        None => {
            // Default behavior: process path
            let path = match cli.path {
                Some(p) => p,
                None => {
                    eprintln!("Error: PATH is required");
                    process::exit(1);
                }
            };

            // Handle --search mode
            if let Some(pattern) = cli.search {
                let is_dir = Path::new(&path).is_dir();
                let search_opts = search::SearchOptions {
                    pattern,
                    case_insensitive: cli.case_insensitive,
                    depth: cli.depth,
                    ext: cli.ext,
                    max_results: cli.max_results.or(if is_dir { Some(20) } else { None }),
                };
                match search::search_path(&path, &search_opts) {
                    Ok(output) => {
                        print!("{}", output);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                }
                return;
            }
            
            let format = if cli.json {
                OutputFormat::Json
            } else {
                OutputFormat::Plain
            };
            
            let options = ProcessOptions {
                symbols: cli.symbols,
                pub_only: cli.pub_only,
                fns_only: cli.fns,
                types_only: cli.types,
                no_tests: cli.no_tests,
                depth: cli.depth,
                format,
                stats: cli.stats,
                ext: cli.ext,
                signatures: cli.signatures,
                max_lines: cli.max_lines,
            };
            
            match process_path(&path, options) {
                Ok(output) => {
                    print!("{}", output);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

fn handle_edit(
    file: &str,
    symbol: &str,
    replace: Option<String>,
    replace_body: Option<String>,
    stdin: bool,
    delete: bool,
    batch: Option<String>,
    dry_run: bool,
    json: bool,
) -> Result<(), CodeviewError> {
    let path = Path::new(file);
    if !path.exists() {
        return Err(CodeviewError::PathNotFound(file.to_string()));
    }
    
    let source = fs::read_to_string(path)
        .map_err(|e| CodeviewError::ReadError {
            path: file.to_string(),
            source: e,
        })?;
    
    let language = detect_language_from_path(path)?;
    
    // Compute edit metadata before performing the edit (line ranges from original source)
    let mut edit_results: Vec<EditResult> = Vec::new();
    
    let result = if let Some(batch_file) = batch {
        let batch_json = fs::read_to_string(&batch_file)
            .map_err(|e| CodeviewError::ReadError {
                path: batch_file.clone(),
                source: e,
            })?;
        #[derive(serde::Deserialize)]
        struct BatchInput { edits: Vec<BatchEdit> }
        let input: BatchInput = serde_json::from_str(&batch_json)?;
        
        if json {
            for edit in &input.edits {
                let (line_start, line_end) = editor::symbol_line_range(&source, &edit.symbol, language)?;
                let action = match edit.action {
                    editor::BatchAction::Replace => "replaced",
                    editor::BatchAction::ReplaceBody => "replaced_body",
                    editor::BatchAction::Delete => "deleted",
                };
                edit_results.push(EditResult {
                    symbol: edit.symbol.clone(),
                    action: action.to_string(),
                    line_start,
                    line_end,
                });
            }
        }
        
        editor::batch(&source, &input.edits, language)?
    } else if delete {
        if json {
            let (line_start, line_end) = editor::symbol_line_range(&source, symbol, language)?;
            edit_results.push(EditResult {
                symbol: symbol.to_string(),
                action: "deleted".to_string(),
                line_start,
                line_end,
            });
        }
        editor::delete(&source, symbol, language)?
    } else if let Some(body_content) = replace_body {
        let new_body = if stdin {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)
                .map_err(|e| CodeviewError::ParseError(format!("Failed to read stdin: {}", e)))?;
            buf
        } else {
            body_content
        };
        if json {
            let (line_start, line_end) = editor::symbol_line_range(&source, symbol, language)?;
            edit_results.push(EditResult {
                symbol: symbol.to_string(),
                action: "replaced_body".to_string(),
                line_start,
                line_end,
            });
        }
        editor::replace_body(&source, symbol, &new_body, language)?
    } else if let Some(replacement) = replace {
        let new_content = if stdin {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)
                .map_err(|e| CodeviewError::ParseError(format!("Failed to read stdin: {}", e)))?;
            buf
        } else {
            replacement
        };
        if json {
            let (line_start, line_end) = editor::symbol_line_range(&source, symbol, language)?;
            edit_results.push(EditResult {
                symbol: symbol.to_string(),
                action: "replaced".to_string(),
                line_start,
                line_end,
            });
        }
        editor::replace(&source, symbol, &new_content, language)?
    } else {
        return Err(CodeviewError::ParseError(
            "Must specify --replace, --replace-body, --delete, or --batch".to_string()
        ));
    };
    
    if dry_run {
        print!("{}", result);
    } else {
        fs::write(path, &result)
            .map_err(|e| CodeviewError::ReadError {
                path: file.to_string(),
                source: e,
            })?;
    }
    
    if json {
        if edit_results.len() == 1 {
            println!("{}", serde_json::to_string(&edit_results[0]).unwrap());
        } else {
            println!("{}", serde_json::to_string(&edit_results).unwrap());
        }
    }
    
    Ok(())
}

fn detect_language_from_path(path: &Path) -> Result<Language, CodeviewError> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| CodeviewError::NoExtension(path.display().to_string()))?;
    
    match ext {
        "rs" => Ok(Language::Rust),
        "ts" => Ok(Language::TypeScript),
        "tsx" => Ok(Language::Tsx),
        "js" => Ok(Language::JavaScript),
        "jsx" => Ok(Language::Jsx),
        "py" => Ok(Language::Python),
        _ => Err(CodeviewError::UnsupportedExtension(ext.to_string())),
    }
}
