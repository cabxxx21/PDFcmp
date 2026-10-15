//! PDF Compressor Core Library
//!
//! Provides the core compression logic, backend abstraction, and data structures.

pub mod backend;
pub mod error;
pub mod options;
pub mod stats;
pub mod util;

pub use backend::{BackendRegistry, PdfBackend};
pub use error::{CompressionError, Result};
pub use options::{CompressOptions, DownsampleFactor, FontSubset, ImageQuality, Preset, StreamCompression, TargetDpi};
pub use stats::{BatchResult, BatchSummary, CompressionStats};
pub use util::{atomic_write, default_output_path, ensure_output_dir, file_size, find_pdfs, format_bytes, is_pdf_file, validate_paths};

/// High-level compression function using default backend
pub fn compress(input: &std::path::Path, output: &std::path::Path, options: &CompressOptions) -> Result<CompressionStats> {
    let registry = BackendRegistry::default();
    let backend = registry.default_backend()
        .ok_or_else(|| CompressionError::Backend("No backend available".into()))?;
    backend.compress(input, output, options)
}

/// Compress with specific backend by name
pub fn compress_with_backend(
    backend_name: &str,
    input: &std::path::Path,
    output: &std::path::Path,
    options: &CompressOptions,
) -> Result<CompressionStats> {
    let registry = BackendRegistry::default();
    let backend = registry.get(backend_name)
        .ok_or_else(|| CompressionError::Backend(format!("Backend '{}' not found", backend_name)))?;
    backend.compress(input, output, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compress_function() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("test.pdf");
        let output = dir.path().join("out.pdf");

        // Create a minimal valid PDF without font references
        let minimal_pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n193\n%%EOF";
        fs::write(&input, minimal_pdf).unwrap();

        let options = CompressOptions::preset(Preset::Screen);
        let result = compress(&input, &output, &options);

        // For very small PDFs, compression might increase size - that's OK
        match result {
            Ok(stats) => {
                assert!(stats.compressed_size > 0);
                assert_eq!(stats.pages, 1);
            }
            Err(e) => {
                // SizeIncreased is acceptable for tiny PDFs
                let err_str = e.to_string();
                assert!(err_str.contains("SizeIncreased") || err_str.contains("larger"));
            }
        }
    }
}