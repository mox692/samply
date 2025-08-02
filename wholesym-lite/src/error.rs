use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Object file parsing error: {0}")]
    Object(#[from] object::Error),
    
    #[error("DWARF parsing error: {0}")]
    Gimli(#[from] addr2line::gimli::Error),
    
    #[error("File not found: {path}")]
    FileNotFound { path: std::path::PathBuf },
    
    #[error("Unsupported file format")]
    UnsupportedFormat,
    
    #[error("No debug information found")]
    NoDebugInfo,
}