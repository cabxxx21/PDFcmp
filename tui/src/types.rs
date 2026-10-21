use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub original_size: u64,
    pub compressed_size: Option<u64>,
    pub status: FileStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Compressing(u8), // 0-100 progress
    Completed,
    Failed(String),
    Skipped(String),
}

impl File {
    pub fn file_name(&self) -> &str {
        self.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;

        if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn original_size_formatted(&self) -> String {
        Self::format_size(self.original_size)
    }

    pub fn compressed_size_formatted(&self) -> String {
        self.compressed_size.map(Self::format_size).unwrap_or_else(|| "—".to_string())
    }

    pub fn compression_ratio(&self) -> Option<String> {
        match (self.original_size, self.compressed_size) {
            (orig, Some(comp)) if orig > 0 => {
                let ratio = (1.0 - (comp as f64 / orig as f64)) * 100.0;
                Some(format!("−{:.0}%", ratio))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Preset {
    Screen,
    Ebook,
    Printer,
    Prepress,
}

impl Preset {
    pub fn label(&self) -> &'static str {
        match self {
            Preset::Screen => "Screen (72 dpi)",
            Preset::Ebook => "Ebook (150 dpi)",
            Preset::Printer => "Printer (300 dpi)",
            Preset::Prepress => "Prepress (300 dpi, high quality)",
        }
    }

    pub fn all() -> &'static [Preset] {
        &[Preset::Screen, Preset::Ebook, Preset::Printer, Preset::Prepress]
    }

    pub fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|p| p == self).unwrap_or(0);
        all[(idx + 1) % all.len()].clone()
    }

    pub fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|p| p == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()].clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionBackend {
    Auto,
    Mupdf,
    Ghostscript,
}

impl CompressionBackend {
    pub fn label(&self) -> &'static str {
        match self {
            CompressionBackend::Auto => "Auto (best available)",
            CompressionBackend::Mupdf => "MuPDF",
            CompressionBackend::Ghostscript => "Ghostscript",
        }
    }

    pub fn all() -> &'static [CompressionBackend] {
        &[CompressionBackend::Auto, CompressionBackend::Mupdf, CompressionBackend::Ghostscript]
    }

    pub fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|b| b == self).unwrap_or(0);
        all[(idx + 1) % all.len()].clone()
    }

    pub fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|b| b == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()].clone()
    }
}
