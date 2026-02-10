use crate::error::CodeviewError;
use crate::extractor::Item;
use super::OutputFormat;
use std::collections::BTreeMap;
use std::fmt::Write;

/// Per-file statistics
struct FileStats {
    path: String,
    lines: usize,
    bytes: usize,
    items: usize,
    kinds: BTreeMap<String, usize>,
}

/// Gather common totals from files + source_sizes.
fn gather_stats(
    files: &[(String, Vec<Item>)],
    source_sizes: &[(usize, usize)],
) -> (Vec<FileStats>, usize, usize, usize, BTreeMap<String, usize>) {
    let mut total_lines = 0usize;
    let mut total_bytes = 0usize;
    let mut total_items = 0usize;
    let mut total_kinds: BTreeMap<String, usize> = BTreeMap::new();

    let file_stats: Vec<FileStats> = files
        .iter()
        .zip(source_sizes.iter())
        .map(|((path, items), &(lines, bytes))| {
            let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
            for item in items {
                let kind = format!("{:?}", item.kind).to_lowercase();
                *kinds.entry(kind.clone()).or_default() += 1;
                *total_kinds.entry(kind).or_default() += 1;
            }
            total_lines += lines;
            total_bytes += bytes;
            total_items += items.len();
            FileStats {
                path: path.clone(),
                lines,
                bytes,
                items: items.len(),
                kinds,
            }
        })
        .collect();

    (file_stats, total_lines, total_bytes, total_items, total_kinds)
}

/// Format stats output in the requested format.
pub fn format_output(
    files: &[(String, Vec<Item>)],
    source_sizes: &[(usize, usize)],
    format: OutputFormat,
) -> Result<String, CodeviewError> {
    match format {
        OutputFormat::Plain => format_plain(files, source_sizes),
        OutputFormat::Json => format_json(files, source_sizes),
    }
}

fn format_plain(
    files: &[(String, Vec<Item>)],
    source_sizes: &[(usize, usize)],
) -> Result<String, CodeviewError> {
    let (file_stats, total_lines, total_bytes, total_items, total_kinds) =
        gather_stats(files, source_sizes);

    let mut out = String::new();
    let file_count = file_stats.iter().filter(|f| f.items > 0 || file_stats.len() == 1).count();

    writeln!(out, "files: {}  lines: {}  bytes: {}  items: {}",
        file_count, total_lines, total_bytes, total_items).unwrap();

    if !total_kinds.is_empty() {
        let kinds_str: Vec<String> = total_kinds
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();
        writeln!(out, "  {}", kinds_str.join("  ")).unwrap();
    }

    if file_stats.len() > 1 {
        writeln!(out).unwrap();
        for f in &file_stats {
            if f.items == 0 {
                continue;
            }
            let kinds_str: Vec<String> = f.kinds
                .iter()
                .map(|(k, v)| format!("{} {}", v, k))
                .collect();
            writeln!(out, "  {} — {} lines, {} bytes, {} items ({})",
                f.path, f.lines, f.bytes, f.items, kinds_str.join(", ")).unwrap();
        }
    }

    Ok(out)
}

fn format_json(
    files: &[(String, Vec<Item>)],
    source_sizes: &[(usize, usize)],
) -> Result<String, CodeviewError> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct StatsOutput {
        files: usize,
        lines: usize,
        bytes: usize,
        items: usize,
        kinds: BTreeMap<String, usize>,
        per_file: Vec<FileStatJson>,
    }

    #[derive(Serialize)]
    struct FileStatJson {
        path: String,
        lines: usize,
        bytes: usize,
        items: usize,
        kinds: BTreeMap<String, usize>,
    }

    let (file_stats, total_lines, total_bytes, total_items, total_kinds) =
        gather_stats(files, source_sizes);

    let per_file: Vec<FileStatJson> = file_stats
        .into_iter()
        .filter(|f| f.items > 0)
        .map(|f| FileStatJson {
            path: f.path,
            lines: f.lines,
            bytes: f.bytes,
            items: f.items,
            kinds: f.kinds,
        })
        .collect();

    let output = StatsOutput {
        files: per_file.len(),
        lines: total_lines,
        bytes: total_bytes,
        items: total_items,
        kinds: total_kinds,
        per_file,
    };

    Ok(serde_json::to_string_pretty(&output)?)
}
