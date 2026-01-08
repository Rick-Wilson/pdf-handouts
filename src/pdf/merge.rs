//! PDF merging functionality using lopdf

use std::collections::BTreeMap;
use std::path::PathBuf;
use lopdf::{Document, Object, ObjectId, Dictionary};
use crate::error::{Error, Result};

/// Options for merging PDFs
#[derive(Debug, Clone)]
pub struct MergeOptions {
    /// Input PDF file paths in the order they should be merged
    pub input_paths: Vec<PathBuf>,
    /// Output PDF file path
    pub output_path: PathBuf,
}

/// Merge multiple PDF files into a single PDF
///
/// Based on the lopdf merge example:
/// https://github.com/J-F-Liu/lopdf/blob/main/examples/merge.rs
///
/// # Example
///
/// ```no_run
/// use pdf_handouts::pdf::{MergeOptions, merge_pdfs};
/// use std::path::PathBuf;
///
/// let options = MergeOptions {
///     input_paths: vec![
///         PathBuf::from("1. first.pdf"),
///         PathBuf::from("2. second.pdf"),
///     ],
///     output_path: PathBuf::from("merged.pdf"),
/// };
///
/// merge_pdfs(&options).expect("Failed to merge");
/// ```
pub fn merge_pdfs(options: &MergeOptions) -> Result<()> {
    if options.input_paths.is_empty() {
        return Err(Error::General("No input files provided".to_string()));
    }

    // Validate all input files exist
    for path in &options.input_paths {
        if !path.exists() {
            return Err(Error::FileNotFound(path.clone()));
        }
    }

    // Load all documents
    let mut documents: Vec<Document> = Vec::new();
    for path in &options.input_paths {
        let doc = Document::load(path)?;

        // Validate document has pages
        if doc.get_pages().is_empty() {
            return Err(Error::EmptyPdf(path.clone()));
        }

        documents.push(doc);
    }

    // Define a starting max_id for merged document
    let mut max_id = 1;
    let mut page_ids: Vec<(u32, u16)> = Vec::new();
    let mut objects: BTreeMap<ObjectId, Object> = BTreeMap::new();

    for mut doc in documents {
        // Renumber objects in this document to avoid conflicts
        doc.renumber_objects_with(max_id);

        // Update max_id for next document
        max_id = doc.max_id + 1;

        // Collect page IDs from this document
        let pages = doc.get_pages();
        page_ids.extend(pages.into_iter().map(|(_, id)| id));

        // Collect all objects from this document
        objects.extend(doc.objects);
    }

    // Create new document with merged content
    // Initialize with a minimal catalog and pages structure
    let mut merged_doc = Document::with_version("1.5");

    // Add all collected objects FIRST
    merged_doc.objects.extend(objects);

    // CRITICAL: Update max_id to reflect the highest object ID we just added
    // Otherwise new_object_id() will return IDs that collide with existing objects
    merged_doc.max_id = max_id - 1;

    // Now create catalog and pages with IDs that won't conflict
    // (they'll be higher than any object from the source PDFs)
    let pages_id = merged_doc.new_object_id();

    // Create Kids array with all page references
    let kids: Vec<Object> = page_ids
        .iter()
        .map(|&id| Object::Reference(id))
        .collect();

    // Create Pages object
    let mut pages_object = Dictionary::new();
    pages_object.set("Type", Object::Name(b"Pages".to_vec()));
    pages_object.set("Count", Object::Integer(page_ids.len() as i64));
    pages_object.set("Kids", Object::Array(kids));

    // Create Catalog
    let catalog_id = merged_doc.new_object_id();
    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));

    // Insert catalog and pages into merged document
    merged_doc.objects.insert(catalog_id, Object::Dictionary(catalog));
    merged_doc.objects.insert(pages_id, Object::Dictionary(pages_object));

    // Set the catalog as the root
    merged_doc.trailer.set("Root", Object::Reference(catalog_id));

    // Update parent references for all pages
    for &page_id in &page_ids {
        if let Ok(page_object) = merged_doc.get_object_mut(page_id) {
            if let Object::Dictionary(ref mut dict) = page_object {
                dict.set("Parent", Object::Reference(pages_id));
            }
        }
    }

    // Compress and save
    merged_doc.compress();
    merged_doc.save(&options.output_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_merge_options_creation() {
        let options = MergeOptions {
            input_paths: vec![
                PathBuf::from("test1.pdf"),
                PathBuf::from("test2.pdf"),
            ],
            output_path: PathBuf::from("merged.pdf"),
        };

        assert_eq!(options.input_paths.len(), 2);
        assert_eq!(options.output_path, Path::new("merged.pdf"));
    }

    // Note: Integration tests with actual PDFs will be in tests/ directory
}
