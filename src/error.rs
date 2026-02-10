use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeviewError {
    #[error("Path not found: {0}")]
    PathNotFound(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Unsupported file extension: {0}")]
    UnsupportedExtension(String),
    
    #[error("No file extension found for path: {0}")]
    NoExtension(String),
    
    #[error("Failed to read {path}: {source}")]
    ReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
}
