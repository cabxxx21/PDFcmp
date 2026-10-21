use std::thread;
use std::path::PathBuf;
use std::fs;
use std::sync::mpsc;
use crate::types::{File, FileStatus, Preset, CompressionBackend};
use pdf_compressor_core::{compress, compress_with_backend, CompressOptions};
use tinyfiledialogs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    FileList,
    Preset,
    Backend,
    OutputPath,
    Help,
}

#[derive(Debug, Clone)]
pub enum CompressionEvent {
    Progress(f32, String),
    FileStatus(usize, FileStatus),
    FileCompressed(usize, u64),
    Done(String),
}

pub struct App {
    pub files: Vec<File>,
    pub focus: Focus,
    pub selected_file: usize,
    pub output_path: String,
    pub preset: Preset,
    pub backend: CompressionBackend,
    pub compression_running: bool,
    pub compression_paused: bool,
    pub overall_progress: f32,
    pub status_message: String,
    pub show_help: bool,
    pub should_quit: bool,
    pub compression_rx: Option<mpsc::Receiver<CompressionEvent>>,
    pub progress_animation_frame: u32,
}

impl App {
    pub fn new() -> Self {
        let files: Vec<File> = Vec::new();

        Self {
            files,
            focus: Focus::FileList,
            selected_file: 0,
            output_path: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("output")
                .to_string_lossy()
                .to_string(),
            preset: Preset::Ebook,
            backend: CompressionBackend::Auto,
            compression_running: false,
            compression_paused: false,
            overall_progress: 0.0,
            status_message: "Press 'o' to open files.".to_string(),
            show_help: false,
            should_quit: false,
            compression_rx: None,
            progress_animation_frame: 0,
        }
    }

    pub fn from_files(file_paths: Vec<PathBuf>) -> Self {
        let mut files: Vec<File> = file_paths
            .into_iter()
            .filter_map(|path| {
                // Check if file exists and is a PDF
                if !path.exists() {
                    eprintln!("Warning: File not found: {:?}", path);
                    return None;
                }

                if !path.extension().map_or(false, |ext| ext == "pdf") {
                    eprintln!("Warning: Not a PDF file: {:?}", path);
                    return None;
                }

                // Get file size
                let size = fs::metadata(&path)
                    .map(|m| m.len())
                    .unwrap_or(0);

                Some(File {
                    path: path.clone(),
                    original_size: size,
                    compressed_size: None,
                    status: FileStatus::Pending,
                })
            })
            .collect();

        // Sort by filename
        files.sort_by(|a, b| a.file_name().cmp(b.file_name()));

        let status_message = if files.is_empty() {
            "No valid PDF files provided. Press q to quit.".to_string()
        } else {
            format!("Loaded {} file(s). Press Enter to start compression.", files.len())
        };

        Self {
            files,
            focus: Focus::FileList,
            selected_file: 0,
            output_path: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("output")
                .to_string_lossy()
                .to_string(),
            preset: Preset::Ebook,
            backend: CompressionBackend::Auto,
            compression_running: false,
            compression_paused: false,
            overall_progress: 0.0,
            status_message,
            show_help: false,
            should_quit: false,
            compression_rx: None,
            progress_animation_frame: 0,
        }
    }

    pub fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = (self.selected_file + 1) % self.files.len();
        }
    }

    pub fn prev_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = (self.selected_file + self.files.len() - 1) % self.files.len();
        }
    }

    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::FileList => Focus::Preset,
            Focus::Preset => Focus::Backend,
            Focus::Backend => Focus::OutputPath,
            Focus::OutputPath => Focus::FileList,
            Focus::Help => Focus::FileList,
        };
    }

    pub fn prev_focus(&mut self) {
        self.focus = match self.focus {
            Focus::FileList => Focus::OutputPath,
            Focus::Preset => Focus::FileList,
            Focus::Backend => Focus::Preset,
            Focus::OutputPath => Focus::Backend,
            Focus::Help => Focus::FileList,
        };
    }

    pub fn next_preset(&mut self) {
        if !self.compression_running {
            self.preset = self.preset.next();
        }
    }

    pub fn prev_preset(&mut self) {
        if !self.compression_running {
            self.preset = self.preset.prev();
        }
    }

    pub fn next_backend(&mut self) {
        if !self.compression_running {
            self.backend = self.backend.next();
        }
    }

    pub fn prev_backend(&mut self) {
        if !self.compression_running {
            self.backend = self.backend.prev();
        }
    }

    pub fn edit_output_path(&mut self, ch: char) {
        if !self.compression_running && self.focus == Focus::OutputPath {
            self.output_path.push(ch);
        }
    }

    pub fn backspace_output_path(&mut self) {
        if !self.compression_running && self.focus == Focus::OutputPath {
            self.output_path.pop();
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.focus = Focus::Help;
        } else {
            self.focus = Focus::FileList;
        }
    }

    pub fn process_events(&mut self) {
        if let Some(ref rx) = self.compression_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    CompressionEvent::Progress(progress, msg) => {
                        self.overall_progress = progress;
                        self.status_message = msg;
                    }
                    CompressionEvent::FileStatus(idx, status) => {
                        if idx < self.files.len() {
                            self.files[idx].status = status;
                        }
                    }
                    CompressionEvent::FileCompressed(idx, size) => {
                        if idx < self.files.len() {
                            self.files[idx].compressed_size = Some(size);
                        }
                    }
                    CompressionEvent::Done(msg) => {
                        self.compression_running = false;
                        self.status_message = msg;
                    }
                }
            }
        }
    }

    pub fn tick(&mut self) {
        if self.compression_running {
            self.progress_animation_frame = self.progress_animation_frame.wrapping_add(1);
        }
    }

    pub fn start_compression(&mut self) {
        if self.compression_running || self.files.is_empty() {
            return;
        }

        self.compression_running = true;
        self.compression_paused = false;
        self.overall_progress = 0.0;
        self.status_message = "Compressing...".to_string();
        self.progress_animation_frame = 0;

        // Reset all pending files
        for file in &mut self.files {
            if matches!(file.status, FileStatus::Pending) {
                file.status = FileStatus::Compressing(0);
            }
        }

        // Create channel for event communication
        let (tx, rx) = mpsc::channel();
        self.compression_rx = Some(rx);

        // Spawn background thread for real compression
        let files = self.files.clone();
        let preset = self.preset;
        let backend = self.backend;
        let output_path = self.output_path.clone();

        thread::spawn(move || {
            Self::real_compression(files, preset, backend, output_path, tx);
        });
    }

    fn real_compression(
        files: Vec<File>,
        preset: Preset,
        backend: CompressionBackend,
        output_path: String,
        tx: mpsc::Sender<CompressionEvent>,
    ) {
        let total = files.len();
        let mut completed = 0;

        for (i, file) in files.iter().enumerate() {
            // Create output filename
            let output_path_obj = PathBuf::from(&output_path);
            if !output_path_obj.exists() {
                let _ = fs::create_dir_all(&output_path_obj);
            }

            let file_name = file.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("output.pdf");
            let output_file = output_path_obj.join(file_name);

            // Map TUI preset to core preset
            let core_preset = match preset {
                Preset::Screen => pdf_compressor_core::Preset::Screen,
                Preset::Ebook => pdf_compressor_core::Preset::Ebook,
                Preset::Printer => pdf_compressor_core::Preset::Printer,
                Preset::Prepress => pdf_compressor_core::Preset::Prepress,
            };

            let compress_options = CompressOptions::preset(core_preset);

            // Update status
            let _ = tx.send(CompressionEvent::FileStatus(i, FileStatus::Compressing(0)));
            let _ = tx.send(CompressionEvent::Progress(0.0, format!("Compressing {}...", file.file_name())));

            // Perform real compression - use compress() for Auto (default backend), compress_with_backend() for specific
            let result = match backend {
                CompressionBackend::Auto => {
                    compress(&file.path, &output_file, &compress_options)
                }
                _ => {
                    let backend_name = match backend {
                        CompressionBackend::Mupdf => "mupdf",
                        CompressionBackend::Ghostscript => "ghostscript",
                        CompressionBackend::Auto => unreachable!(),
                    };
                    compress_with_backend(backend_name, &file.path, &output_file, &compress_options)
                }
            };

            match result {
                Ok(stats) => {
                    let _ = tx.send(CompressionEvent::FileCompressed(i, stats.compressed_size));
                    let _ = tx.send(CompressionEvent::FileStatus(i, FileStatus::Completed));
                    completed += 1;
                    let progress = (completed as f32 / total as f32) * 100.0;
                    let _ = tx.send(CompressionEvent::Progress(progress, format!(
                        "Compressed {} - Saved {:.0}%",
                        file.file_name(),
                        (1.0 - stats.compressed_size as f64 / stats.original_size as f64) * 100.0
                    )));
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let _ = tx.send(CompressionEvent::FileStatus(i, FileStatus::Failed(error_msg.clone())));
                    completed += 1;
                    let progress = (completed as f32 / total as f32) * 100.0;
                    let _ = tx.send(CompressionEvent::Progress(progress, format!("Error on {}: {}", file.file_name(), error_msg)));
                }
            }
        }

        // All done - calculate total savings
        let total_original: u64 = files.iter().map(|f| f.original_size).sum();
        let total_compressed: u64 = files.iter().filter_map(|f| {
            if matches!(f.status, FileStatus::Completed) {
                f.compressed_size
            } else {
                Some(f.original_size)
            }
        }).sum();

        let completed_count = files.iter().filter(|f| matches!(f.status, FileStatus::Completed)).count();
        let failed_count = files.iter().filter(|f| matches!(f.status, FileStatus::Failed(_))).count();

        let savings = if total_original > 0 {
            (1.0 - total_compressed as f64 / total_original as f64) * 100.0
        } else {
            0.0
        };

        let _ = tx.send(CompressionEvent::Done(format!(
            "Done! {} completed, {} failed. Saved {:.0}% ({})",
            completed_count,
            failed_count,
            savings,
            File::format_size(total_original.saturating_sub(total_compressed))
        )));
    }

    pub fn cancel_compression(&mut self) {
        if self.compression_running {
            self.compression_running = false;
            self.status_message = "Cancelling...".to_string();

            // Mark in-progress files as cancelled
            for file in &mut self.files {
                if matches!(file.status, FileStatus::Compressing(_)) {
                    file.status = FileStatus::Skipped("Cancelled".to_string());
                }
            }
        }
    }

    pub fn open_file_picker(&mut self) {
        if self.compression_running {
            return;
        }

        #[cfg(feature = "native-dialog")]
        {
            let files = rfd::FileDialog::new()
                .add_filter("PDF Files", &["pdf"])
                .set_directory(".")
                .pick_files();

            if let Some(paths) = files {
                self.process_paths(paths);
            }
            return;
        }

        // Fallback: tinyfiledialogs (uses GTK/zenity/kdialog on Linux)
        let filters = Some((&["*.pdf" as &str][..], "PDF files"));
        let paths = tinyfiledialogs::open_file_dialog_multi("Select PDF files", ".", filters);
        if let Some(paths) = paths {
            let paths: Vec<PathBuf> = paths.iter().map(|s| s.into()).collect();
            self.process_paths(paths);
        }
    }

    fn process_paths(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if !path.exists() {
                continue;
            }
            if !path.extension().map_or(false, |ext| ext == "pdf") {
                continue;
            }
            let size = fs::metadata(&path)
                .map(|m| m.len())
                .unwrap_or(0);
            if !self.files.iter().any(|f| f.path == path) {
                self.files.push(File {
                    path: path.clone(),
                    original_size: size,
                    compressed_size: None,
                    status: FileStatus::Pending,
                });
            }
        }
        self.files.sort_by(|a, b| a.file_name().cmp(b.file_name()));
        self.status_message = if self.files.is_empty() {
            "No PDF files selected.".to_string()
        } else {
            format!("Loaded {} file(s).", self.files.len())
        };
    }

    pub fn toggle_auto_backend(&mut self) {
        if self.focus != Focus::Backend {
            return;
        }

        if self.backend != CompressionBackend::Auto {
            self.backend = CompressionBackend::Auto;
            self.status_message = "Backend: Auto (will choose best available)".to_string();
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
