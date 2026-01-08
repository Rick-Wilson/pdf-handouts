//! PDF manipulation module

pub mod merge;
pub mod metadata;
pub mod create;

// Re-export commonly used items
pub use merge::{merge_pdfs, MergeOptions};
pub use metadata::{count_pages, extract_metadata, PdfMetadata};
pub use create::{create_watermark_pdf, WatermarkOptions};
