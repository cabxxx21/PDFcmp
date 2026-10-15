# PDF Compressor (`pdfcmp`)

A simple, beginner-friendly PDF compressor for Linux. Built with Rust and Ghostscript (with MuPDF fallback), distributed as a single ~8MB binary.

## Quick Start

```bash
# Basic usage - compresses to input_compressed.pdf
pdfcmp document.pdf

# Compress with screen preset (aggressive)
pdfcmp document.pdf -p screen

# Compress to specific output
pdfcmp large.pdf -o small.pdf -p ebook

# Batch compress directory
pdfcmp ~/Documents -r

# Force overwrite existing file
pdfcmp file.pdf -f
```

### Installation

**Option 1: Build from source**
```bash
git clone https://github.com/cabxxx21/PDFcmp
cd PDFcmp
cargo install --path cli
```

**Option 2: Copy binary manually**
```bash
cp target/release/pdfcmp ~/.local/bin/pdfcmp
# Make sure ~/.local/bin is in your PATH
```

**Option 3: Use full path**
```bash
# Add to your shell config (~/.bashrc or ~/.zshrc)
export PATH="$PATH:/path/to/pdfcompressor/target/release"
```

## Features

- ✅ **Simple CLI** - No subcommands needed
- ✅ **Ghostscript backend** - 80%+ compression on image-heavy PDFs
- ✅ **MuPDF fallback** - Works offline without Ghostscript
- ✅ **Auto-detect mode** - Single file vs batch (directory)
- ✅ **4 presets** - screen/ebook/printer/prepress
- ✅ **Safety first** - Never overwrites originals without `--force`
- ✅ **Accurate stats** - Shows size, savings %, duration
- ✅ **Parallel batch** - Process multiple files at once

## Presets

| Preset | Quality | Use Case | Typical Savings |
|--------|---------|----------|-----------------|
| `screen` | 50% | Screen viewing only | 70-90% |
| `ebook` | 70% | E-readers, tablets | 60-80% |
| `printer` | 85% | Home/office printing | 30-50% |
| `prepress` | 100% | Professional print | 10-30% |

**Default**: `ebook` (balanced quality/size)

## Usage Examples

### Single File Compression

```bash
# Basic (ebook preset, auto output name)
pdfcmp report.pdf

# Aggressive (screen preset for email)
pdfcmp large.pdf -p screen -o email.pdf

# High quality (printer preset)
pdfcmp contract.pdf -p printer
```

### Batch Processing

```bash
# Compress all PDFs in directory
pdfcmp /home/user/PDFs

# Recursive (includes subdirectories)
pdfcmp ~/Documents -r

# Custom output directory
pdfcmp /input -o /output -r

# Parallel jobs (default: CPU count)
pdfcmp ~/PDFs -j 4
```

### Advanced Options

```bash
# Force overwrite
pdfcmp file.pdf -f

# Ghostscript backend (recommended)
pdfcmp file.pdf -b ghostscript

# MuPDF backend (if Ghostscript not installed)
pdfcmp file.pdf -b mupdf

# Verbose logging for debugging
pdfcmp file.pdf -v

# JSON output for scripting
pdfcmp file.pdf --json
```

## Real Compression Results

| File | Size | Preset | Result | Savings |
|------|------|--------|--------|---------|
| jembut.pdf | 10.91 MB | ebook | 8.08 MB | 26% |
| jembut.pdf | 10.91 MB | screen | 3.88 MB | 64% |
| memek.pdf | 24.78 MB | screen | 5.13 MB | 79% |
| avrbook.pdf | 49.00 MB | default | 5.91 MB | 88% |

## How It Works

### Architecture

```
┌─────────────────────────────────────────────┐
│           User Command (pdfcmp)            │
│  CLI parsing, validation, progress bar     │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│           Rust (core library)              │
│  Backend selector, safety checks, stats    │
└──────────────┬──────────────────────────────┘
               │
               ├─ Ghostscript (preferred)
               │   • Image downsampling
               │   • JPEG recompression
               │   • Color optimization
               │
               └─ MuPDF (fallback)
                   • Stream compression
                   • Garbage collection
                   • Font compression
```

### Backend Comparison

**Ghostscript** (primary):
- Uses `gs -sDEVICE=pdfwrite` subprocess
- Aggressive image compression (downsamples, recompresses JPEG)
- Best for image-heavy PDFs (books, scanned docs, presentations)
- Requires `ghostscript` package installed (`gs` command)
- ~70-90% compression on typical PDFs

**MuPDF** (fallback):
- Uses `mupdf` crate (statically linked)
- Stream compression, garbage collection, font compression
- Works without external dependencies
- Slower compression, smaller size increase
- Best for text-heavy PDFs or if Ghostscript unavailable

## Safety Features

- ✅ Original files never modified
- ✅ Atomic writes (temp file → rename on success)
- ✅ Cleanup on failure (removes partial output)
- ✅ Size increase detection (error if compressed > original)
- ✅ Encrypted PDF rejection (clear error message)

## Options Reference

```
Usage: pdfcmp [OPTIONS] <PATH>

Arguments:
  <PATH>     Input PDF file or directory (auto-detects batch mode)

Options:
  -o, --output <PATH>      Output file or directory (default: auto-generated)
  -p, --preset <PRESET>    Compression preset [screen|ebook|printer|prepress]
                           [default: ebook]
  -b, --backend <BACKEND>  Backend [auto|mupdf|ghostscript]
                           [default: auto]
  -f, --force              Overwrite output files if they exist
  -r, --recursive          Recursively process subdirectories
      --suffix <SUFFIX>    Output suffix for batch mode [default: _compressed]
  -j, --jobs <N>           Maximum parallel jobs [default: CPU count]
      --continue-on-error  Continue on error in batch mode
      --progress           Show progress bar [default: true]
      --no-progress        Disable progress bar
      --json               Output as JSON
  -v, --verbose            Enable verbose logging
```

## Examples

### Reduce PDF for email attachment
```bash
pdfcmp large_report.pdf -p screen -o email_report.pdf
# Typical: 60-80% reduction for image-heavy PDFs
```

### Archive for long-term storage
```bash
pdfcmp ~/Archive -o ~/Archive_Compressed -r --preset prepress
# Minimal quality loss, preserves fonts and structure
```

### Optimize for e-reader
```bash
pdfcmp textbook.pdf -p ebook -o textbook_ebook.pdf
# Balanced quality/size for 6-10" screens
```

### Maximum compression (accept quality loss)
```bash
pdfcmp draft.pdf -p screen -o draft_tiny.pdf
# Aggressive: ~70-90% reduction
```

## Requirements

### Runtime
- Linux kernel 3.2+ (any modern distro)
- Ghostscript 9.50+ (`gs` command, recommended)
- x86_64, ARM64, or other common architectures

### Build
- Rust 1.75+
- `ghostscript` package (for Ghostscript backend)
- ~1.5 GB RAM for compilation

## Installation by Distribution

| Distribution | Install Ghostscript | Build pdfcmp |
|--------------|---------------------|--------------|
| Ubuntu/Debian | `sudo apt install ghostscript` | `cargo install --path cli` |
| Fedora | `sudo dnf install ghostscript` | `cargo install --path cli` |
| Arch Linux | `sudo pacman -S ghostscript` | `cargo install --path cli` |
| Alpine | `sudo apk add ghostscript` | `cargo install --path cli` |

## Limitations

- ❌ No OCR (can't add searchable text to scanned PDFs)
- ❌ No password removal (encrypted PDFs are rejected)
- ❌ No form flattening (interactive forms preserved as-is)
- ❌ JPEG-only image re-encoding (no WebP/AVIF/HEIC)
- ❌ Single-threaded per file (parallel only across files in batch)

## License

MIT OR Apache-2.0

## Credits

- **Ghostscript** - PDF writing and image compression (GPL v2)
- **MuPDF** - PDF parsing and optimization (AGPL v3)
- **mupdf crate** - Rust bindings to MuPDF
- **clap** - CLI argument parsing
- **indicatif** - Progress bars
- **rayon** - Parallel batch processing

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-thing`)
3. Make changes with tests
4. Run `cargo test` and verify with real PDFs
5. Submit a pull request

## Changelog

### 0.1.0 (Initial Release)
- Simple beginner-friendly CLI (no subcommands)
- Ghostscript backend with aggressive image compression
- MuPDF fallback for offline/dependency-free mode
- Auto-detect single file vs batch mode
- 4 compression presets (screen/ebook/printer/prepress)
- Parallel batch processing
- Safety guarantees (atomic writes, cleanup on failure)
- Real-time progress bars and detailed stats
