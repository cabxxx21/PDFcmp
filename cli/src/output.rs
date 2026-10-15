use indicatif::{ProgressBar, ProgressStyle};
use pdf_compressor_core::{BatchSummary, CompressionStats};
use std::time::Duration;

/// Create a progress bar for single file compression
pub fn create_progress_bar(visible: bool) -> ProgressBar {
    let pb = if visible {
        ProgressBar::new_spinner()
    } else {
        ProgressBar::hidden()
    };
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Create a progress bar for batch processing
pub fn create_batch_progress_bar(total: u64, visible: bool) -> ProgressBar {
    let pb = if visible {
        ProgressBar::new(total)
    } else {
        ProgressBar::hidden()
    };
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-")
    );
    pb
}

/// Print compression stats in human-readable format
pub fn print_stats(stats: &CompressionStats, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(stats).unwrap());
    } else {
        println!("{}", stats);
    }
}

/// Print batch summary
pub fn print_batch_summary(summary: &BatchSummary, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(summary).unwrap());
    } else {
        println!("{}", summary);
        println!();
        println!("Details:");
        for result in &summary.results {
            if let Some(stats) = &result.stats {
                println!("  ✓ {} -> {}", result.input_path, result.output_path);
                println!("    {} -> {} ({:.1}% savings)",
                    pdf_compressor_core::format_bytes(stats.original_size),
                    pdf_compressor_core::format_bytes(stats.compressed_size),
                    stats.savings_percent);
            } else if let Some(err) = &result.error {
                println!("  ✗ {} - {}", result.input_path, err);
            }
        }
    }
}

/// Print presets table
pub fn print_presets() {
    println!("Available Compression Presets:");
    println!();
    println!("{:<12} {:<8} {:<8} {:<6} {:<8} {:<10}", "Preset", "Color Q", "Gray Q", "DPI", "Downsample", "Font Subset");
    println!("{:-<12} {:-<8} {:-<8} {:-<6} {:-<8} {:-<10}", "", "", "", "", "", "");
    println!("{:<12} {:<8} {:<8} {:<6} {:<8} {:<10}", "screen", "50", "50", "72", "2.0x", "all");
    println!("{:<12} {:<8} {:<8} {:<6} {:<8} {:<10}", "ebook", "70", "70", "150", "1.5x", "all");
    println!("{:<12} {:<8} {:<8} {:<6} {:<8} {:<10}", "printer", "85", "85", "300", "1.0x", "non-std");
    println!("{:<12} {:<8} {:<8} {:<6} {:<8} {:<10}", "prepress", "100", "100", "300", "1.0x", "none");
    println!("{:<12} {:<8} {:<8} {:<6} {:<8} {:<10}", "custom", "custom", "custom", "custom", "custom", "custom");
    println!();
    println!("Use --preset <name> to select. Use 'custom' with individual options for fine control.");
}