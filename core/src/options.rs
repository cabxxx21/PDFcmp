use serde::{Deserialize, Serialize};
use std::fmt;

/// Compression quality presets matching Ghostscript/mupdf conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    /// Screen viewing only (72 DPI, aggressive compression)
    Screen,
    /// E-book reader (150 DPI, balanced)
    Ebook,
    /// Home/office printing (300 DPI, good quality)
    Printer,
    /// Pre-press / professional print (300 DPI, minimal loss)
    Prepress,
    /// Custom settings
    Custom,
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Preset::Screen => write!(f, "screen"),
            Preset::Ebook => write!(f, "ebook"),
            Preset::Printer => write!(f, "printer"),
            Preset::Prepress => write!(f, "prepress"),
            Preset::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for Preset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "screen" => Ok(Preset::Screen),
            "ebook" => Ok(Preset::Ebook),
            "printer" => Ok(Preset::Printer),
            "prepress" => Ok(Preset::Prepress),
            "custom" => Ok(Preset::Custom),
            _ => Err(format!("Unknown preset: {}. Valid: screen, ebook, printer, prepress, custom", s)),
        }
    }
}

/// Image compression quality (1-100, JPEG quality)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImageQuality(pub u8);

impl ImageQuality {
    pub fn new(q: u8) -> Result<Self, String> {
        if (1..=100).contains(&q) {
            Ok(Self(q))
        } else {
            Err(format!("Image quality must be 1-100, got {}", q))
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for ImageQuality {
    fn default() -> Self {
        Self(75)
    }
}

/// Downsample factor for images (>= 1.0)
/// 1.0 = no downsample, 2.0 = half resolution in each dimension
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DownsampleFactor(pub f32);

impl DownsampleFactor {
    pub fn new(f: f32) -> Result<Self, String> {
        if f >= 1.0 {
            Ok(Self(f))
        } else {
            Err(format!("Downsample factor must be >= 1.0, got {}", f))
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for DownsampleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Target DPI for image downsampling
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TargetDpi(pub u32);

impl TargetDpi {
    pub fn new(dpi: u32) -> Result<Self, String> {
        if dpi > 0 {
            Ok(Self(dpi))
        } else {
            Err("DPI must be > 0".into())
        }
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for TargetDpi {
    fn default() -> Self {
        Self(150)
    }
}

/// Font subsetting strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontSubset {
    /// Subset all embedded fonts (remove unused glyphs)
    All,
    /// Only subset fonts that are not standard 14 fonts
    NonStandard,
    /// Don't subset fonts
    None,
}

impl Default for FontSubset {
    fn default() -> Self {
        Self::All
    }
}

/// Stream compression method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamCompression {
    /// FlateDecode (zlib/deflate) - standard, lossless
    Flate,
    /// JBIG2 - for monochrome images, lossless
    Jbig2,
    /// Auto - let backend choose
    Auto,
}

impl Default for StreamCompression {
    fn default() -> Self {
        Self::Auto
    }
}

/// Complete compression options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressOptions {
    /// Quality preset (overrides individual settings if not Custom)
    pub preset: Preset,

    /// JPEG quality for color images (1-100)
    pub image_quality: ImageQuality,

    /// JPEG quality for grayscale images (1-100)
    pub gray_quality: ImageQuality,

    /// Downsample factor for color images
    pub downsample_color: DownsampleFactor,

    /// Downsample factor for grayscale images
    pub downsample_gray: DownsampleFactor,

    /// Downsample factor for monochrome images
    pub downsample_mono: DownsampleFactor,

    /// Target DPI for downsampling (0 = use downsample factors)
    pub target_dpi: TargetDpi,

    /// Font subsetting strategy
    pub font_subset: FontSubset,

    /// Stream compression method
    pub stream_compression: StreamCompression,

    /// Linearize output (Fast Web View)
    pub linearize: bool,

    /// Garbage collect unused objects
    pub garbage_collect: bool,

    /// Compress streams (FlateDecode)
    pub compress_streams: bool,

    /// Preserve PDF version
    pub preserve_version: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self::preset(Preset::Ebook)
    }
}

impl CompressOptions {
    /// Create options from a preset (non-custom values ignored)
    pub fn preset(preset: Preset) -> Self {
        match preset {
            Preset::Screen => Self {
                preset: Preset::Screen,
                image_quality: ImageQuality::new(50).unwrap(),
                gray_quality: ImageQuality::new(50).unwrap(),
                downsample_color: DownsampleFactor::new(2.0).unwrap(),
                downsample_gray: DownsampleFactor::new(2.0).unwrap(),
                downsample_mono: DownsampleFactor::new(2.0).unwrap(),
                target_dpi: TargetDpi::new(72).unwrap(),
                font_subset: FontSubset::All,
                stream_compression: StreamCompression::Auto,
                linearize: true,
                garbage_collect: true,
                compress_streams: true,
                preserve_version: false,
            },
            Preset::Ebook => Self {
                preset: Preset::Ebook,
                image_quality: ImageQuality::new(70).unwrap(),
                gray_quality: ImageQuality::new(70).unwrap(),
                downsample_color: DownsampleFactor::new(1.5).unwrap(),
                downsample_gray: DownsampleFactor::new(1.5).unwrap(),
                downsample_mono: DownsampleFactor::new(1.5).unwrap(),
                target_dpi: TargetDpi::new(150).unwrap(),
                font_subset: FontSubset::All,
                stream_compression: StreamCompression::Auto,
                linearize: true,
                garbage_collect: true,
                compress_streams: true,
                preserve_version: false,
            },
            Preset::Printer => Self {
                preset: Preset::Printer,
                image_quality: ImageQuality::new(85).unwrap(),
                gray_quality: ImageQuality::new(85).unwrap(),
                downsample_color: DownsampleFactor::new(1.0).unwrap(),
                downsample_gray: DownsampleFactor::new(1.0).unwrap(),
                downsample_mono: DownsampleFactor::new(1.0).unwrap(),
                target_dpi: TargetDpi::new(300).unwrap(),
                font_subset: FontSubset::NonStandard,
                stream_compression: StreamCompression::Auto,
                linearize: true,
                garbage_collect: true,
                compress_streams: true,
                preserve_version: false,
            },
            Preset::Prepress => Self {
                preset: Preset::Prepress,
                image_quality: ImageQuality::new(100).unwrap(),
                gray_quality: ImageQuality::new(100).unwrap(),
                downsample_color: DownsampleFactor::new(1.0).unwrap(),
                downsample_gray: DownsampleFactor::new(1.0).unwrap(),
                downsample_mono: DownsampleFactor::new(1.0).unwrap(),
                target_dpi: TargetDpi::new(300).unwrap(),
                font_subset: FontSubset::None,
                stream_compression: StreamCompression::Flate,
                linearize: false,
                garbage_collect: true,
                compress_streams: true,
                preserve_version: true,
            },
            Preset::Custom => Self {
                preset: Preset::Custom,
                ..Default::default()
            },
        }
    }

    /// Get effective options for backend (preset applied)
    pub fn effective(&self) -> Self {
        if self.preset != Preset::Custom {
            Self::preset(self.preset)
        } else {
            self.clone()
        }
    }

    /// Validate custom options
    pub fn validate(&self) -> Result<(), String> {
        if self.preset == Preset::Custom {
            self.image_quality.value(); // validates range
            self.gray_quality.value();
            self.downsample_color.value();
            self.downsample_gray.value();
            self.downsample_mono.value();
            self.target_dpi.value();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_screen_has_correct_values() {
        let opts = CompressOptions::preset(Preset::Screen);
        assert_eq!(opts.image_quality.value(), 50);
        assert_eq!(opts.target_dpi.value(), 72);
        assert_eq!(opts.downsample_color.value(), 2.0);
    }

    #[test]
    fn preset_ebook_has_correct_values() {
        let opts = CompressOptions::preset(Preset::Ebook);
        assert_eq!(opts.image_quality.value(), 70);
        assert_eq!(opts.target_dpi.value(), 150);
    }

    #[test]
    fn custom_preset_allows_override() {
        let mut opts = CompressOptions::preset(Preset::Custom);
        opts.image_quality = ImageQuality::new(90).unwrap();
        assert_eq!(opts.image_quality.value(), 90);
    }

    #[test]
    fn invalid_quality_rejected() {
        assert!(ImageQuality::new(0).is_err());
        assert!(ImageQuality::new(101).is_err());
    }

    #[test]
    fn invalid_downsample_rejected() {
        assert!(DownsampleFactor::new(0.5).is_err());
        assert!(DownsampleFactor::new(1.0).is_ok());
    }
}