use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Failed to load PNG: {0}")]
    LoadError(String),

    #[error("Failed to save PNG: {0}")]
    SaveError(String),

    #[error("Invalid image format: {0}")]
    InvalidFormat(String),

    #[error("Image data size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: usize, actual: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Failed to load library: {0}")]
    LoadError(String),

    #[error("Failed to find symbol 'process_image': {0}")]
    SymbolNotFound(String),

    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    #[error("Plugin processing failed: {0}")]
    ProcessingFailed(String),
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Image error: {0}")]
    Image(#[from] ImageError),

    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::InvalidArgs(s.to_string())
    }
}
