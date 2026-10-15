use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{CompressionError, Result};

/// Check if a file is a valid PDF by reading magic bytes
pub fn is_pdf_file(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        use std::io::Read;
        let mut header = [0u8; 5];
        if file.read_exact(&mut header).is_ok() {
            return &header == b"%PDF-";
        }
    }
    false
}

/// Get file size in bytes
pub fn file_size(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path)?.len())
}

/// Ensure output directory exists and is writable
pub fn ensure_output_dir(path: &Path) -> Result<PathBuf> {
    let parent = path.parent().unwrap_or(Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }
    // Test writability
    let test_file = parent.join(".pdf_compressor_write_test");
    match fs::File::create(&test_file) {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            Ok(parent.to_path_buf())
        }
        Err(_) => Err(CompressionError::NoWritableOutputDir(parent.to_path_buf())),
    }
}

/// Generate output path if not provided
pub fn default_output_path(input: &Path, suffix: &str) -> PathBuf {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let parent = input.parent().unwrap_or(Path::new("."));
    parent.join(format!("{}{}.pdf", stem, suffix))
}

/// Atomic write: write to temp file then rename
pub fn atomic_write<F>(target: &Path, write_fn: F) -> Result<()>
where
    F: FnOnce(&mut fs::File) -> std::io::Result<()>,
{
    let parent = target.parent().unwrap_or(Path::new("."));
    let temp_name = format!(
        ".{}.tmp.{}",
        target.file_name().and_then(|s| s.to_str()).unwrap_or("output"),
        std::process::id()
    );
    let temp_path = parent.join(temp_name);

    {
        let mut file = fs::File::create(&temp_path)?;
        write_fn(&mut file)?;
        file.sync_all()?;
    }

    fs::rename(&temp_path, target)?;
    Ok(())
}

/// Find all PDF files in a directory (recursive)
pub fn find_pdfs(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut pdfs = Vec::new();
    let walker = if recursive {
        walkdir::WalkDir::new(dir).into_iter()
    } else {
        walkdir::WalkDir::new(dir).max_depth(1).into_iter()
    };

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_pdf_file(path) {
            pdfs.push(path.to_path_buf());
        }
    }

    pdfs.sort();
    Ok(pdfs)
}

/// Validate input/output paths for compression
pub fn validate_paths(input: &Path, output: &Path, force: bool) -> Result<()> {
    if !input.exists() {
        return Err(CompressionError::InputNotFound(input.to_path_buf()));
    }
    if !input.is_file() {
        return Err(CompressionError::InputNotAFile(input.to_path_buf()));
    }
    if !is_pdf_file(input) {
        return Err(CompressionError::InvalidPdf("Not a valid PDF file (missing %PDF- header)".into()));
    }

    if output.exists() {
        if output.is_dir() {
            return Err(CompressionError::OutputIsDirectory(output.to_path_buf()));
        }
        if !force {
            return Err(CompressionError::OutputExists(output.to_path_buf()));
        }
    }

    ensure_output_dir(output)?;
    Ok(())
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_pdf_file() {
        let dir = tempdir().unwrap();
        let pdf_path = dir.path().join("test.pdf");
        fs::write(&pdf_path, b"%PDF-1.4\n%...\n").unwrap();
        assert!(is_pdf_file(&pdf_path));

        let txt_path = dir.path().join("test.txt");
        fs::write(&txt_path, b"not a pdf").unwrap();
        assert!(!is_pdf_file(&txt_path));
    }

    #[test]
    fn test_validate_paths() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("input.pdf");
        fs::write(&input, b"%PDF-1.4\n").unwrap();

        let output = dir.path().join("output.pdf");
        validate_paths(&input, &output, false).unwrap();

        // Test force overwrite
        fs::write(&output, b"%PDF-1.4\n").unwrap();
        validate_paths(&input, &output, true).unwrap();
        validate_paths(&input, &output, false).unwrap_err();
    }

    #[test]
    fn test_default_output_path() {
        let input = Path::new("/home/user/doc.pdf");
        let out = default_output_path(input, "_compressed");
        assert_eq!(out, Path::new("/home/user/doc_compressed.pdf"));
    }

    #[test]
    fn test_find_pdfs() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.pdf"), b"%PDF-1.4").unwrap();
        fs::write(dir.path().join("b.txt"), b"text").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("c.pdf"), b"%PDF-1.4").unwrap();

        let pdfs = find_pdfs(dir.path(), true).unwrap();
        assert_eq!(pdfs.len(), 2);

        let pdfs_shallow = find_pdfs(dir.path(), false).unwrap();
        assert_eq!(pdfs_shallow.len(), 1);
    }
}