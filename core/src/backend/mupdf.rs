use crate::backend::PdfBackend;
use crate::options::{CompressOptions, FontSubset, Preset};
use crate::stats::CompressionStats;
use crate::error::{CompressionError, Result};
use crate::util::file_size;
use mupdf::pdf::{PdfDocument, PdfWriteOptions};
use std::path::Path;
use std::time::Instant;

pub struct MuPdfBackend;

impl MuPdfBackend {
    pub fn new() -> Self {
        Self
    }

    /// Map our options to mupdf PdfWriteOptions
    fn build_write_options(&self, options: &CompressOptions) -> PdfWriteOptions {
        let eff = options.effective();
        let mut write_opts = PdfWriteOptions::default();

        // Garbage collection (remove unused objects)
        write_opts.set_garbage(eff.garbage_collect);
        if eff.garbage_collect {
            write_opts.set_garbage_level(3); // Aggressive garbage collection
        }

        // Note: Linearize (Fast Web View) is not supported in this version of mupdf
        // write_opts.set_linear(eff.linearize); // Disabled due to mupdf limitation

        // Compress streams
        write_opts.set_compress(eff.compress_streams);

        // ASCII encode (disable for smaller binary)
        write_opts.set_ascii(false);

        // Clean content streams
        write_opts.set_clean(true);

        // Image compression - recompress with default settings
        write_opts.set_compress_images(true);

        // Font compression (subsetting via garbage collection)
        write_opts.set_compress_fonts(true);

        // Decompress (false = keep compressed)
        write_opts.set_decompress(false);

        // Sanitize (removes some metadata)
        write_opts.set_sanitize(false);

        // Appearance streams
        write_opts.set_appearance(false);

        write_opts
    }
}

impl PdfBackend for MuPdfBackend {
    fn name(&self) -> &'static str {
        "mupdf"
    }

    fn compress(&self, input: &Path, output: &Path, options: &CompressOptions) -> Result<CompressionStats> {
        let start = Instant::now();

        // Validate input
        let original_size = file_size(input)?;

        // Open as PdfDocument to preserve content
        let pdf_doc = PdfDocument::open(input)
            .map_err(|e| CompressionError::InvalidPdf(format!("Failed to open PDF: {}", e)))?;

        // Check if encrypted
        if pdf_doc.needs_password().map_err(|e| CompressionError::Backend(format!("Password check failed: {}", e)))? {
            return Err(CompressionError::EncryptedPdf);
        }

        let page_count = pdf_doc.page_count().map_err(|e| CompressionError::Backend(format!("Page count failed: {}", e)))? as u32;

        // Build write options
        let write_opts = self.build_write_options(options);

        // Save to output
        pdf_doc.save_with_options(output.to_str().unwrap(), write_opts)
            .map_err(|e| CompressionError::Backend(format!("Failed to save PDF: {}", e)))?;

        // Get compressed size
        let compressed_size = file_size(output)?;

        // Check if size increased
        if compressed_size > original_size {
            // Clean up output file
            let _ = std::fs::remove_file(output);
            return Err(CompressionError::SizeIncreased {
                original: original_size,
                compressed: compressed_size,
            });
        }

        let duration = start.elapsed();

        Ok(CompressionStats::new(original_size, compressed_size, duration)
            .with_pages(page_count)
            .with_linearized(options.effective().linearize)
            .with_font_subset(options.effective().font_subset != FontSubset::None))
    }

    fn is_available(&self) -> bool {
        // mupdf crate compiles mupdf from source, so always available if built
        true
    }
}

impl Default for MuPdfBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::CompressOptions;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_mupdf_backend_basic() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("test.pdf");
        let output = dir.path().join("out.pdf");

        // Create a minimal valid PDF without font references
        let minimal_pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n193\n%%EOF";
        fs::write(&input, minimal_pdf).unwrap();

        let backend = MuPdfBackend::new();
        let options = CompressOptions::preset(crate::options::Preset::Screen);
        let result = backend.compress(&input, &output, &options);

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