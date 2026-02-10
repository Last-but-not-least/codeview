use clap::{Parser, Subcommand};
use codeview::{editor, process_path, ProcessOptions, OutputFormat, Language, CodeviewError};
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
}

#[derive(Subcommand)]
enum Commands {
    /// Edit a symbol in a file
    Edit {
        /// File to edit
        file: String,
        
        /// Symbol name to edit
        symbol: String,
        
        /// Replace the symbol with new source
        #[arg(long, conflicts_with = "delete")]
        replace: Option<String>,
        
        /// Read replacement from stdin
        #[arg(long, requires = "replace")]
        stdin: bool,
        
        /// Delete the symbol
        #[arg(long, conflicts_with = "replace")]
        delete: bool,
        
        /// Dry run - print to stdout instead of writing file
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Edit { file, symbol, replace, stdin, delete, dry_run }) => {
            if let Err(e) = handle_edit(&file, &symbol, replace, stdin, delete, dry_run) {
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
    stdin: bool,
    delete: bool,
    dry_run: bool,
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
    
    let result = if delete {
        editor::delete(&source, symbol, language)?
    } else if let Some(replacement) = replace {
        let new_content = if stdin {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)
                .map_err(|e| CodeviewError::ParseError(format!("Failed to read stdin: {}", e)))?;
            buf
        } else {
            replacement
        };
        editor::replace(&source, symbol, &new_content, language)?
    } else {
        return Err(CodeviewError::ParseError(
            "Must specify either --replace or --delete".to_string()
        ));
    };
    
    if dry_run {
        print!("{}", result);
    } else {
        fs::write(path, result)
            .map_err(|e| CodeviewError::ReadError {
                path: file.to_string(),
                source: e,
            })?;
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
        _ => Err(CodeviewError::UnsupportedExtension(ext.to_string())),
    }
}
