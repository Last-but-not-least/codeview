use clap::Parser;
use codeview::{process_path, ProcessOptions, OutputFormat};
use std::process;

#[derive(Parser)]
#[command(name = "codeview")]
#[command(about = "Code context extractor using Tree-sitter", long_about = None, version)]
struct Cli {
    /// File or directory to analyze
    #[arg(value_name = "PATH")]
    path: String,

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

fn main() {
    let cli = Cli::parse();

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

    let result = process_path(&cli.path, options);

    match result {
        Ok(output) => {
            print!("{}", output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
