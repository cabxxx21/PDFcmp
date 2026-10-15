use crate::options::CompressOptions;
use crate::stats::CompressionStats;
use crate::error::Result;
use std::path::Path;

mod mupdf;
mod ghostscript;

/// Backend trait for PDF compression engines
pub trait PdfBackend: Send + Sync {
    /// Compress a PDF file
    fn compress(
        &self,
        input: &Path,
        output: &Path,
        options: &CompressOptions,
    ) -> Result<CompressionStats>;

    /// Get backend name/version info
    fn name(&self) -> &'static str;

    /// Check if backend is available
    fn is_available(&self) -> bool {
        true
    }
}

/// Registry of available backends
pub struct BackendRegistry {
    backends: Vec<Box<dyn PdfBackend>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self { backends: Vec::new() }
    }

    pub fn register(&mut self, backend: Box<dyn PdfBackend>) {
        self.backends.push(backend);
    }

    pub fn get(&self, name: &str) -> Option<&dyn PdfBackend> {
        self.backends.iter().find(|b| b.name() == name).map(|b| b.as_ref())
    }

    pub fn default_backend(&self) -> Option<&dyn PdfBackend> {
        // Prefer Ghostscript when available, fall back to others
        self.backends
            .iter()
            .find(|b| b.name() == "ghostscript" && b.is_available())
            .or_else(|| self.backends.iter().find(|b| b.is_available()))
            .map(|b| b.as_ref())
    }

    pub fn available_backends(&self) -> Vec<&'static str> {
        self.backends.iter().filter(|b| b.is_available()).map(|b| b.name()).collect()
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // Register Ghostscript first (preferred for compression)
        registry.register(Box::new(crate::backend::ghostscript::GhostscriptBackend::new()));
        // MuPDF as fallback
        registry.register(Box::new(crate::backend::mupdf::MuPdfBackend::new()));
        registry
    }
}