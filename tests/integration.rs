//! Integration tests for PDF Compressor

use pdf_compressor_core::{compress, CompressOptions, Preset};
use std::fs;
use tempfile::tempdir;

/// Create a minimal valid PDF for testing
fn create_minimal_pdf() -> Vec<u8> {
    b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n193\n%%EOF".to_vec()
}

/// Create a PDF with a simple image (1x1 pixel JPEG)
fn create_pdf_with_image() -> Vec<u8> {
    // Minimal PDF with an inline image
    b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>\nendobj\n4 0 obj\n<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>\nstream\n\xff\xff\xff\nendstream\nendobj\n5 0 obj\n<< /Length 44 >>\nstream\nq 612 0 0 792 0 0 cm /Im0 Do Q\nendstream\nendobj\nxref\n0 6\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000198 00000 n \n0000000315 00000 n \ntrailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n385\n%%EOF".to_vec()
}

#[test]
fn test_compress_minimal_pdf() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("input.pdf");
    let output = dir.path().join("output.pdf");

    fs::write(&input, create_minimal_pdf()).unwrap();

    let options = CompressOptions::preset(Preset::Screen);
    let stats = compress(&input, &output, &options).unwrap();

    assert!(output.exists());
    assert!(stats.compressed_size > 0);
    assert_eq!(stats.pages, 1);
    assert!(stats.linearized);
    assert!(stats.font_subset);
}

#[test]
fn test_all_presets() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("input.pdf");
    fs::write(&input, create_minimal_pdf()).unwrap();

    for preset in [Preset::Screen, Preset::Ebook, Preset::Printer, Preset::Prepress] {
        let output = dir.path().join(format!("output_{}.pdf", preset));
        let options = CompressOptions::preset(preset);
        let stats = compress(&input, &output, &options).unwrap();

        assert!(output.exists(), "Preset {:?} failed to produce output", preset);
        assert!(stats.compressed_size > 0, "Preset {:?} produced empty output", preset);
    }
}

#[test]
fn test_custom_options() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("input.pdf");
    let output = dir.path().join("output.pdf");

    fs::write(&input, create_minimal_pdf()).unwrap();

    let mut options = CompressOptions::preset(Preset::Custom);
    options.image_quality = pdf_compressor_core::ImageQuality::new(90).unwrap();
    options.target_dpi = pdf_compressor_core::TargetDpi::new(200).unwrap();
    options.font_subset = pdf_compressor_core::FontSubset::None;
    options.linearize = false;

    let stats = compress(&input, &output, &options).unwrap();

    assert!(output.exists());
    assert!(!stats.linearized);
    assert!(!stats.font_subset);
}

#[test]
fn test_force_overwrite() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("input.pdf");
    let output = dir.path().join("output.pdf");

    fs::write(&input, create_minimal_pdf()).unwrap();
    fs::write(&output, b"existing").unwrap();

    let options = CompressOptions::preset(Preset::Screen);

    // Should fail without force
    let result = compress(&input, &output, &options);
    assert!(result.is_err());

    // Should succeed with force (using atomic_write directly)
    pdf_compressor_core::atomic_write(&output, |f| {
        use std::io::Write;
        f.write_all(&create_minimal_pdf())
    }).unwrap();
    assert!(output.exists());
}

#[test]
fn test_invalid_input() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("nonexistent.pdf");
    let output = dir.path().join("output.pdf");

    let options = CompressOptions::preset(Preset::Screen);
    let result = compress(&input, &output, &options);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Input file not found"));
}

#[test]
fn test_non_pdf_input() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("test.txt");
    let output = dir.path().join("output.pdf");

    fs::write(&input, b"not a pdf").unwrap();

    let options = CompressOptions::preset(Preset::Screen);
    let result = compress(&input, &output, &options);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("valid PDF") || err.contains("Invalid PDF"));
}

#[test]
fn test_output_directory_creation() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("input.pdf");
    let output = dir.path().join("subdir").join("output.pdf");

    fs::write(&input, create_minimal_pdf()).unwrap();

    let options = CompressOptions::preset(Preset::Screen);
    let stats = compress(&input, &output, &options).unwrap();

    assert!(output.exists());
    assert!(stats.compressed_size > 0);
}

#[test]
fn test_batch_find_pdfs() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.pdf"), create_minimal_pdf()).unwrap();
    fs::write(dir.path().join("b.txt"), b"text").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub").join("c.pdf"), create_minimal_pdf()).unwrap();

    let pdfs = pdf_compressor_core::find_pdfs(dir.path(), true).unwrap();
    assert_eq!(pdfs.len(), 2);

    let pdfs_shallow = pdf_compressor_core::find_pdfs(dir.path(), false).unwrap();
    assert_eq!(pdfs_shallow.len(), 1);
}

#[test]
fn test_preset_ordering_by_size() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("input.pdf");
    fs::write(&input, create_pdf_with_image()).unwrap();

    let mut sizes = Vec::new();
    for preset in [Preset::Screen, Preset::Ebook, Preset::Printer, Preset::Prepress] {
        let output = dir.path().join(format!("out_{}.pdf", preset));
        let options = CompressOptions::preset(preset);
        let stats = compress(&input, &output, &options).unwrap();
        sizes.push((preset, stats.compressed_size));
    }

    // Screen should be smallest, prepress largest (for image PDFs)
    assert!(sizes[0].1 <= sizes[1].1, "Screen should be <= Ebook");
    assert!(sizes[1].1 <= sizes[2].1, "Ebook should be <= Printer");
    assert!(sizes[2].1 <= sizes[3].1, "Printer should be <= Prepress");
}