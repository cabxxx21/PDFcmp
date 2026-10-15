use crate::backend::PdfBackend;
use crate::options::{CompressOptions, Preset};
use crate::stats::CompressionStats;
use crate::error::{CompressionError, Result};
use crate::util::file_size;
use std::path::Path;
use std::time::Instant;
use std::process::Command;

pub struct GhostscriptBackend;

impl GhostscriptBackend {
    pub fn new() -> Self {
        Self
    }

    fn check_gs_available() -> bool {
        Command::new("gs")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn build_gs_args(&self, input: &Path, output: &Path, options: &CompressOptions) -> Vec<String> {
        let eff = options.effective();

        let preset_arg = match eff.preset {
            Preset::Screen => "/screen",
            Preset::Ebook => "/ebook",
            Preset::Printer => "/printer",
            Preset::Prepress => "/prepress",
            Preset::Custom => "/printer",
        };

        let mut args = vec![
            "gs".to_string(),
            "-sDEVICE=pdfwrite".to_string(),
            "-dCompatibilityLevel=1.4".to_string(),
            "-dNOPAUSE".to_string(),
            "-dQUIET".to_string(),
            "-dBATCH".to_string(),
            format!("-dPDFSETTINGS={}", preset_arg),
            "-dDetectDuplicateImages=true".to_string(),
            "-dCompressFonts=true".to_string(),
            "-dCompressStreams=true".to_string(),
        ];

        // Add explicit image compression parameters for aggressive presets
        match eff.preset {
            Preset::Screen | Preset::Ebook => {
                // For screen and ebook, add aggressive image compression
                args.push("-dDownsampleColorImages=true".to_string());
                args.push(format!("-dColorImageResolution={}", eff.target_dpi.value()));
                args.push(format!("-dColorImageDownsampleThreshold=1.0"));

                args.push("-dDownsampleGrayImages=true".to_string());
                args.push(format!("-dGrayImageResolution={}", eff.target_dpi.value()));
                args.push(format!("-dGrayImageDownsampleThreshold=1.0"));

                args.push("-dDownsampleMonoImages=true".to_string());
                args.push(format!("-dMonoImageResolution={}", eff.target_dpi.value()));
                args.push(format!("-dMonoImageDownsampleThreshold=1.0"));

                // JPEG quality settings
                args.push(format!("-dColorImageQuality={}", eff.image_quality.value()));
                args.push(format!("-dGrayImageQuality={}", eff.gray_quality.value()));
            }
            _ => {
                // For printer/prepress, use conservative defaults
                args.push("-dDownsampleColorImages=true".to_string());
                args.push(format!("-dColorImageResolution={}", eff.target_dpi.value()));
                args.push(format!("-dColorImageDownsampleThreshold=1.5"));

                args.push("-dDownsampleGrayImages=true".to_string());
                args.push(format!("-dGrayImageResolution={}", eff.target_dpi.value()));
                args.push(format!("-dGrayImageDownsampleThreshold=1.5"));
            }
        }

        args.push(format!("-sOutputFile={}", output.to_string_lossy()));
        args.push(input.to_string_lossy().to_string());
        args
    }
}

impl PdfBackend for GhostscriptBackend {
    fn name(&self) -> &'static str {
        "ghostscript"
    }

    fn is_available(&self) -> bool {
        Self::check_gs_available()
    }

    fn compress(&self, input: &Path, output: &Path, options: &CompressOptions) -> Result<CompressionStats> {
        if !Self::check_gs_available() {
            return Err(CompressionError::Backend("Ghostscript (gs) not found in PATH".into()));
        }

        let start = Instant::now();
        let original_size = file_size(input)?;

        let args = self.build_gs_args(input, output, options);

        let status = Command::new("gs")
            .args(&args[1..]) // Skip "gs" as it's the program name
            .status()
            .map_err(|e| CompressionError::Backend(format!("Failed to execute gs: {}", e)))?;

        if !status.success() {
            let _ = std::fs::remove_file(output);
            return Err(CompressionError::Backend("Ghostscript compression failed".into()));
        }

        let compressed_size = file_size(output)?;

        if compressed_size > original_size {
            let _ = std::fs::remove_file(output);
            return Err(CompressionError::SizeIncreased {
                original: original_size,
                compressed: compressed_size,
            });
        }

        let duration = start.elapsed();

        Ok(CompressionStats::new(original_size, compressed_size, duration)
            .with_pages(0) // GS doesn't easily return page count
            .with_linearized(false)
            .with_font_subset(options.effective().font_subset != crate::options::FontSubset::None))
    }
}

impl Default for GhostscriptBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gs_available() {
        // Just check if we can run gs --version
        let available = GhostscriptBackend::check_gs_available();
        println!("Ghostscript available: {}", available);
    }
}