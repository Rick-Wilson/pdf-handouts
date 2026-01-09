//! PDF manipulation module

pub mod merge;
pub mod metadata;
pub mod create;
pub mod headers;

// Re-export commonly used items
pub use merge::{merge_pdfs, overlay_watermark, MergeOptions};
pub use metadata::{count_pages, extract_metadata, PdfMetadata};
pub use create::{create_watermark_pdf, WatermarkOptions};
pub use headers::{add_headers_footers, HeaderFooterOptions};
