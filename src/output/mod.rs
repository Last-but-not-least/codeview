pub mod plain;
pub mod json;
pub mod stats;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Plain,
    Json,
}
