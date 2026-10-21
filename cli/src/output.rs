use pdf_compressor_core::{BatchSummary, CompressionStats};

/// Print a simple status message for single file compression
pub fn print_compress_start(input: &std::path::Path) {
    eprintln!("Compressing {}...", input.display());
}

/// Print a simple status message for batch processing
pub fn print_batch_start(count: usize) {
    eprintln!("Found {} PDF file(s) to compress", count);
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