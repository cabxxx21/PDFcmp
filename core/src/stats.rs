use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Original file size in bytes
    pub original_size: u64,

    /// Compressed file size in bytes
    pub compressed_size: u64,

    /// Compression ratio (compressed/original), 0.0-1.0+
    pub ratio: f64,

    /// Space savings percentage (100 * (1 - ratio))
    pub savings_percent: f64,

    /// Time taken for compression
    pub duration: Duration,

    /// Number of pages processed
    pub pages: u32,

    /// Whether linearization was applied
    pub linearized: bool,

    /// Whether font subsetting was applied
    pub font_subset: bool,

    /// Image compression details
    pub images_compressed: u32,

    /// Original image count
    pub images_total: u32,
}

impl CompressionStats {
    pub fn new(original_size: u64, compressed_size: u64, duration: Duration) -> Self {
        let ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            1.0
        };
        let savings_percent = (1.0 - ratio) * 100.0;

        Self {
            original_size,
            compressed_size,
            ratio,
            savings_percent,
            duration,
            pages: 0,
            linearized: false,
            font_subset: false,
            images_compressed: 0,
            images_total: 0,
        }
    }

    pub fn with_pages(mut self, pages: u32) -> Self {
        self.pages = pages;
        self
    }

    pub fn with_linearized(mut self, v: bool) -> Self {
        self.linearized = v;
        self
    }

    pub fn with_font_subset(mut self, v: bool) -> Self {
        self.font_subset = v;
        self
    }

    pub fn with_images(mut self, compressed: u32, total: u32) -> Self {
        self.images_compressed = compressed;
        self.images_total = total;
        self
    }

    /// Human-readable original size
    pub fn original_size_human(&self) -> String {
        human_bytes(self.original_size)
    }

    /// Human-readable compressed size
    pub fn compressed_size_human(&self) -> String {
        human_bytes(self.compressed_size)
    }

    /// Human-readable savings
    pub fn savings_human(&self) -> String {
        human_bytes(self.original_size.saturating_sub(self.compressed_size))
    }

    /// Format duration as human-readable string
    pub fn duration_human(&self) -> String {
        humantime::format_duration(self.duration).to_string()
    }

    /// Whether compression actually reduced size
    pub fn is_smaller(&self) -> bool {
        self.compressed_size < self.original_size
    }
}

impl fmt::Display for CompressionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Compression Results:")?;
        writeln!(f, "  Original size:    {} ({} bytes)", self.original_size_human(), self.original_size)?;
        writeln!(f, "  Compressed size:  {} ({} bytes)", self.compressed_size_human(), self.compressed_size)?;
        writeln!(f, "  Savings:          {} ({:.1}%)", self.savings_human(), self.savings_percent)?;
        writeln!(f, "  Ratio:            {:.3}", self.ratio)?;
        writeln!(f, "  Duration:         {}", self.duration_human())?;
        if self.pages > 0 {
            writeln!(f, "  Pages:            {}", self.pages)?;
        }
        if self.images_total > 0 {
            writeln!(f, "  Images:           {}/{} compressed", self.images_compressed, self.images_total)?;
        }
        if self.linearized {
            writeln!(f, "  Linearized:       yes")?;
        }
        if self.font_subset {
            writeln!(f, "  Font subset:      yes")?;
        }
        Ok(())
    }
}

/// Batch compression summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub total_duration: Duration,
    pub results: Vec<BatchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub input_path: String,
    pub output_path: String,
    pub stats: Option<CompressionStats>,
    pub error: Option<String>,
}

impl BatchSummary {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
            total_original_size: 0,
            total_compressed_size: 0,
            total_duration: Duration::ZERO,
            results: Vec::new(),
        }
    }

    pub fn add_success(&mut self, input: String, output: String, stats: CompressionStats) {
        self.total_files += 1;
        self.successful += 1;
        self.total_original_size += stats.original_size;
        self.total_compressed_size += stats.compressed_size;
        self.total_duration += stats.duration;
        self.results.push(BatchResult {
            input_path: input,
            output_path: output,
            stats: Some(stats),
            error: None,
        });
    }

    pub fn add_failure(&mut self, input: String, error: String) {
        self.total_files += 1;
        self.failed += 1;
        self.results.push(BatchResult {
            input_path: input,
            output_path: String::new(),
            stats: None,
            error: Some(error),
        });
    }

    pub fn add_skipped(&mut self, input: String, reason: String) {
        self.total_files += 1;
        self.skipped += 1;
        self.results.push(BatchResult {
            input_path: input,
            output_path: String::new(),
            stats: None,
            error: Some(reason),
        });
    }

    pub fn overall_savings_percent(&self) -> f64 {
        if self.total_original_size > 0 {
            (1.0 - self.total_compressed_size as f64 / self.total_original_size as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for BatchSummary {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BatchSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Batch Compression Summary:")?;
        writeln!(f, "  Total files:      {}", self.total_files)?;
        writeln!(f, "  Successful:       {}", self.successful)?;
        writeln!(f, "  Failed:           {}", self.failed)?;
        writeln!(f, "  Skipped:          {}", self.skipped)?;
        writeln!(f, "  Total original:   {}", human_bytes(self.total_original_size))?;
        writeln!(f, "  Total compressed: {}", human_bytes(self.total_compressed_size))?;
        writeln!(f, "  Overall savings:  {:.1}%", self.overall_savings_percent())?;
        writeln!(f, "  Total time:       {}", humantime::format_duration(self.total_duration))?;
        Ok(())
    }
}

fn human_bytes(bytes: u64) -> String {
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
    use std::time::Duration;

    #[test]
    fn stats_calculation() {
        let stats = CompressionStats::new(1000, 500, Duration::from_secs(1));
        assert_eq!(stats.ratio, 0.5);
        assert_eq!(stats.savings_percent, 50.0);
        assert!(stats.is_smaller());
    }

    #[test]
    fn stats_no_reduction() {
        let stats = CompressionStats::new(1000, 1200, Duration::from_secs(1));
        assert!(!stats.is_smaller());
        assert!(stats.savings_percent < 0.0);
    }

    #[test]
    fn human_bytes_formatting() {
        assert_eq!(human_bytes(500), "500 B");
        assert_eq!(human_bytes(1536), "1.50 KB");
        assert_eq!(human_bytes(1_572_864), "1.50 MB");
    }

    #[test]
    fn batch_summary() {
        let mut summary = BatchSummary::new();
        let stats = CompressionStats::new(1000, 500, Duration::from_secs(1));
        summary.add_success("a.pdf".into(), "a_out.pdf".into(), stats);
        summary.add_failure("b.pdf".into(), "error".into());
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.successful, 1);
        assert_eq!(summary.failed, 1);
    }
}