use crate::error::CodeviewError;
use crate::extractor::Item;
use serde::Serialize;
use serde_json;

#[derive(Serialize)]
struct JsonOutput {
    files: Vec<FileOutput>,
}

#[derive(Serialize)]
struct FileOutput {
    path: String,
    items: Vec<JsonItem>,
}

#[derive(Serialize)]
struct JsonItem {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    visibility: String,
    line_start: usize,
    line_end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    content: String,
}

/// Format items as JSON
pub fn format_output(files: &[(String, Vec<Item>)]) -> Result<String, CodeviewError> {
    let files_output: Vec<FileOutput> = files
        .iter()
        .map(|(path, items)| {
            let json_items: Vec<JsonItem> = items
                .iter()
                .map(|item| JsonItem {
                    kind: format!("{:?}", item.kind).to_lowercase(),
                    name: item.name.clone(),
                    visibility: format!("{:?}", item.visibility).to_lowercase(),
                    line_start: item.line_start,
                    line_end: item.line_end,
                    signature: item.signature.clone(),
                    body: item.body.clone(),
                    content: item.content.clone(),
                })
                .collect();

            FileOutput {
                path: path.clone(),
                items: json_items,
            }
        })
        .collect();

    let output = JsonOutput {
        files: files_output,
    };

    Ok(serde_json::to_string_pretty(&output)?)
}
