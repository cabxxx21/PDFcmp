use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Input file not found: {0}")]
    InputNotFound(PathBuf),

    #[error("Input is not a file: {0}")]
    InputNotAFile(PathBuf),

    #[error("Output path is a directory: {0}")]
    OutputIsDirectory(PathBuf),

    #[error("Output file already exists (use --force to overwrite): {0}")]
    OutputExists(PathBuf),

    #[error("Invalid PDF: {0}")]
    InvalidPdf(String),

    #[error("PDF is encrypted/password-protected")]
    EncryptedPdf,

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid preset: {0}")]
    InvalidPreset(String),

    #[error("Invalid image quality value: {0} (must be 1-100)")]
    InvalidImageQuality(u8),

    #[error("Invalid downsample factor: {0} (must be >= 1.0)")]
    InvalidDownsampleFactor(f32),

    #[error("Invalid DPI: {0} (must be > 0)")]
    InvalidDpi(u32),

    #[error("Compression resulted in larger file: original={original} bytes, compressed={compressed} bytes")]
    SizeIncreased { original: u64, compressed: u64 },

    #[error("No writable output directory: {0}")]
    NoWritableOutputDir(PathBuf),

    #[error("Batch error: {0}")]
    Batch(String),
}

pub type Result<T> = std::result::Result<T, CompressionError>;