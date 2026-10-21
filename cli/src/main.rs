//! PDF Compressor CLI - Simple, beginner-friendly interface

mod args;
mod batch;
mod output;

use args::Args;
use batch::process_batch;
use output::{print_batch_start, print_batch_summary, print_compress_start, print_stats};
use pdf_compressor_core::{compress, compress_with_backend};
use clap::Parser;
use std::process;

fn main() {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let result = run(args);
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(args: Args) -> anyhow::Result<()> {
    // Auto-detect mode: single file or batch
    if args.is_batch_mode() {
        run_batch(&args)
    } else {
        run_compress(&args)
    }
}

fn run_compress(args: &Args) -> anyhow::Result<()> {
    let options = args.to_options()?;
    let output = args.output_path();

    // Check if input file exists
    if !args.input.exists() {
        anyhow::bail!("Input file not found: {}", args.input.display());
    }

    // Check if input is a PDF
    if !pdf_compressor_core::is_pdf_file(&args.input) {
        anyhow::bail!("Input file is not a PDF: {}", args.input.display());
    }

    // Check if output exists (unless force)
    if output.exists() && !args.force {
        anyhow::bail!(
            "Output file already exists: {}. Use --force to overwrite.",
            output.display()
        );
    }

    // Show progress
    print_compress_start(&args.input);

    let stats = if let Some(backend) = args.backend_name() {
        compress_with_backend(backend, &args.input, &output, &options)?
    } else {
        compress(&args.input, &output, &options)?
    };

    print_stats(&stats, args.json);

    if !stats.is_smaller() {
        eprintln!("Warning: Compressed file is larger than original!");
    }

    Ok(())
}

fn run_batch(args: &Args) -> anyhow::Result<()> {
    // Collect input files from directory
    let pdfs = pdf_compressor_core::find_pdfs(&args.input, args.recursive)?;

    if pdfs.is_empty() {
        eprintln!("No PDF files found in: {}", args.input.display());
        return Ok(());
    }

    print_batch_start(pdfs.len());

    let options = args.to_options()?;
    let output_dir = args.output_dir().map(|p| p.as_path());

    let summary = process_batch(
        &pdfs,
        output_dir,
        &args.suffix,
        &options,
        args.jobs,
        args.force,
        args.continue_on_error,
        args.backend_name(),
    );

    print_batch_summary(&summary, args.json);

    if summary.failed > 0 && !args.continue_on_error {
        process::exit(1);
    }

    Ok(())
}
