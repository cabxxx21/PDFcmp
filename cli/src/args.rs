use clap::{Parser, ValueEnum};
use pdf_compressor_core::{CompressOptions, Preset};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "pdfcmp",
    version,
    about = "Lightweight PDF compressor for Linux",
    long_about = "pdfcmp - Simple PDF compression tool\n\nExamples:\n  pdfcmp document.pdf              # Compress to document_compressed.pdf\n  pdfcmp document.pdf -p screen   # Use aggressive screen preset\n  pdfcmp /path/to/pdfs -r         # Batch compress directory recursively\n  pdfcmp *.pdf -o /tmp/out        # Compress multiple files to output dir"
)]
pub struct Args {
    /// Input PDF file or directory
    #[arg(value_name = "PATH")]
    pub input: PathBuf,

    /// Output file or directory (default: auto-generated)
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Compression preset [screen|ebook|printer|prepress]
    #[arg(short, long, value_enum, default_value = "ebook")]
    pub preset: PresetArg,

    /// Backend [auto|mupdf|ghostscript]
    #[arg(short, long, value_enum, default_value = "auto")]
    pub backend: BackendArg,

    /// Recursively process subdirectories (for batch mode)
    #[arg(short, long)]
    pub recursive: bool,

    /// Output suffix for batch mode (default: _compressed)
    #[arg(long, default_value = "_compressed")]
    pub suffix: String,

    /// Overwrite output files if they exist
    #[arg(short, long)]
    pub force: bool,

    /// Maximum parallel jobs for batch mode (default: CPU count)
    #[arg(short, long, value_name = "N")]
    pub jobs: Option<usize>,

    /// Continue on error in batch mode
    #[arg(long)]
    pub continue_on_error: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    // Advanced options (hidden from main help)
    #[arg(long, value_name = "QUALITY", hide = true)]
    pub image_quality: Option<u8>,

    #[arg(long, value_name = "QUALITY", hide = true)]
    pub gray_quality: Option<u8>,

    #[arg(long, value_name = "FACTOR", hide = true)]
    pub downsample_color: Option<f32>,

    #[arg(long, value_name = "FACTOR", hide = true)]
    pub downsample_gray: Option<f32>,

    #[arg(long, value_name = "FACTOR", hide = true)]
    pub downsample_mono: Option<f32>,

    #[arg(long, value_name = "DPI", hide = true)]
    pub target_dpi: Option<u32>,

    #[arg(long, value_enum, default_value = "all", hide = true)]
    pub font_subset: FontSubsetArg,

    #[arg(long, value_enum, default_value = "auto", hide = true)]
    pub stream_compression: StreamCompressionArg,

    #[arg(long, default_value = "true", hide = true)]
    pub linearize: bool,

    #[arg(long, conflicts_with = "linearize", hide = true)]
    pub no_linearize: bool,

    #[arg(long, default_value = "true", hide = true)]
    pub garbage_collect: bool,

    #[arg(long, conflicts_with = "garbage_collect", hide = true)]
    pub no_garbage_collect: bool,

    #[arg(long, default_value = "true", hide = true)]
    pub compress_streams: bool,

    #[arg(long, conflicts_with = "compress_streams", hide = true)]
    pub no_compress_streams: bool,

    #[arg(long, hide = true)]
    pub preserve_version: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PresetArg {
    Screen,
    Ebook,
    Printer,
    Prepress,
}

impl From<PresetArg> for Preset {
    fn from(p: PresetArg) -> Self {
        match p {
            PresetArg::Screen => Preset::Screen,
            PresetArg::Ebook => Preset::Ebook,
            PresetArg::Printer => Preset::Printer,
            PresetArg::Prepress => Preset::Prepress,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FontSubsetArg {
    All,
    NonStandard,
    None,
}

impl From<FontSubsetArg> for pdf_compressor_core::FontSubset {
    fn from(f: FontSubsetArg) -> Self {
        match f {
            FontSubsetArg::All => pdf_compressor_core::FontSubset::All,
            FontSubsetArg::NonStandard => pdf_compressor_core::FontSubset::NonStandard,
            FontSubsetArg::None => pdf_compressor_core::FontSubset::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StreamCompressionArg {
    Flate,
    Jbig2,
    Auto,
}

impl From<StreamCompressionArg> for pdf_compressor_core::StreamCompression {
    fn from(s: StreamCompressionArg) -> Self {
        match s {
            StreamCompressionArg::Flate => pdf_compressor_core::StreamCompression::Flate,
            StreamCompressionArg::Jbig2 => pdf_compressor_core::StreamCompression::Jbig2,
            StreamCompressionArg::Auto => pdf_compressor_core::StreamCompression::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BackendArg {
    Auto,
    Mupdf,
    Ghostscript,
}

impl Args {
    /// Check if input is a directory (batch mode)
    pub fn is_batch_mode(&self) -> bool {
        self.input.is_dir()
    }

    /// Convert to CompressOptions
    pub fn to_options(&self) -> anyhow::Result<CompressOptions> {
        // Apply preset
        let preset: Preset = self.preset.into();
        let mut opts = CompressOptions::preset(preset);

        // Override with any custom advanced options provided
        if self.image_quality.is_some()
            || self.gray_quality.is_some()
            || self.downsample_color.is_some()
            || self.downsample_gray.is_some()
            || self.downsample_mono.is_some()
            || self.target_dpi.is_some()
            || self.font_subset != FontSubsetArg::All
            || self.stream_compression != StreamCompressionArg::Auto
            || self.linearize != true
            || self.garbage_collect != true
            || self.compress_streams != true
            || self.preserve_version != false
        {
            // User provided custom options
            opts.preset = Preset::Custom;
        }

        if let Some(q) = self.image_quality {
            opts.image_quality = pdf_compressor_core::ImageQuality::new(q)
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        if let Some(q) = self.gray_quality {
            opts.gray_quality = pdf_compressor_core::ImageQuality::new(q)
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        if let Some(f) = self.downsample_color {
            opts.downsample_color = pdf_compressor_core::DownsampleFactor::new(f)
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        if let Some(f) = self.downsample_gray {
            opts.downsample_gray = pdf_compressor_core::DownsampleFactor::new(f)
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        if let Some(f) = self.downsample_mono {
            opts.downsample_mono = pdf_compressor_core::DownsampleFactor::new(f)
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        if let Some(dpi) = self.target_dpi {
            opts.target_dpi = pdf_compressor_core::TargetDpi::new(dpi)
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        opts.font_subset = self.font_subset.into();
        opts.stream_compression = self.stream_compression.into();
        opts.linearize = self.linearize && !self.no_linearize;
        opts.garbage_collect = self.garbage_collect && !self.no_garbage_collect;
        opts.compress_streams = self.compress_streams && !self.no_compress_streams;
        opts.preserve_version = self.preserve_version;

        opts.validate().map_err(|e| anyhow::anyhow!(e))?;
        Ok(opts)
    }

    /// Get output path for single file compression
    pub fn output_path(&self) -> PathBuf {
        self.output.clone().unwrap_or_else(|| {
            pdf_compressor_core::default_output_path(&self.input, "_compressed")
        })
    }

    /// Get output directory for batch mode
    pub fn output_dir(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }

    /// Get backend name if specified
    pub fn backend_name(&self) -> Option<&'static str> {
        match self.backend {
            BackendArg::Auto => None,
            BackendArg::Mupdf => Some("mupdf"),
            BackendArg::Ghostscript => Some("ghostscript"),
        }
    }
}
