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

/// Overlay a watermark PDF onto a source PDF
///
/// This function takes a source PDF and a watermark PDF and overlays the watermark
/// content on top of each page of the source PDF. The watermark PDF should have the
/// same number of pages as the source PDF, with each page containing the headers/footers
/// to overlay.
///
/// # Arguments
///
/// * `source_path` - Path to the source PDF file
/// * `watermark_path` - Path to the watermark PDF file (headers/footers)
/// * `output_path` - Path where the combined PDF will be saved
///
/// # Example
///
/// ```no_run
/// use pdf_handouts::pdf::overlay_watermark;
/// use std::path::Path;
///
/// overlay_watermark(
///     Path::new("source.pdf"),
///     Path::new("watermark.pdf"),
///     Path::new("output.pdf")
/// ).expect("Failed to overlay");
/// ```
pub fn overlay_watermark(
    source_path: &std::path::Path,
    watermark_path: &std::path::Path,
    output_path: &std::path::Path,
) -> Result<()> {
    use std::collections::HashMap;

    // Load both documents
    let mut source_doc = Document::load(source_path)?;
    let watermark_doc = Document::load(watermark_path)?;

    // Get pages from both documents
    let source_pages = source_doc.get_pages();
    let watermark_pages = watermark_doc.get_pages();

    // Verify page counts match
    if source_pages.len() != watermark_pages.len() {
        return Err(Error::General(format!(
            "Page count mismatch: source has {} pages, watermark has {} pages",
            source_pages.len(),
            watermark_pages.len()
        )));
    }

    // Import all objects from watermark document into source document
    // We need to renumber them to avoid ID conflicts
    let watermark_max_id = watermark_doc.max_id;
    let source_max_id = source_doc.max_id;
    let id_offset = source_max_id + 1;

    // Build complete ID map first
    let mut id_map: HashMap<ObjectId, ObjectId> = HashMap::new();
    for (old_id, _) in watermark_doc.objects.iter() {
        let new_id = (old_id.0 + id_offset, old_id.1);
        id_map.insert(*old_id, new_id);
    }

    // Now copy all objects from watermark to source, renumbering references
    for (old_id, object) in watermark_doc.objects.iter() {
        let new_id = id_map[old_id];

        // Clone and renumber references in the object
        let new_object = renumber_object_references(object, &id_map);
        source_doc.objects.insert(new_id, new_object);
    }

    // Update source document's max_id
    source_doc.max_id = watermark_max_id + id_offset;

    // For each page in the source document, overlay the corresponding watermark page
    for (i, (_page_num, source_page_id)) in source_pages.iter().enumerate() {
        let watermark_page_num = (i + 1) as u32;

        // Get the watermark page's content references and resources (now with new IDs)
        if let Some((watermark_content_refs, watermark_resources)) = get_page_content_and_resources(&watermark_doc, watermark_page_num, &id_map)? {
            // Get the source page object
            let page_obj = source_doc.get_object_mut(*source_page_id)?;

            if let Object::Dictionary(ref mut page_dict) = page_obj {
                // 1. Merge content streams
                let existing_content = page_dict.get(b"Contents").ok().cloned();

                match existing_content {
                    Some(Object::Reference(content_id)) => {
                        // Convert single content stream to array and append watermark
                        let mut new_content = vec![Object::Reference(content_id)];
                        new_content.extend(watermark_content_refs);
                        page_dict.set("Contents", Object::Array(new_content));
                    }
                    Some(Object::Array(mut content_array)) => {
                        // Append watermark content to existing array
                        content_array.extend(watermark_content_refs);
                        page_dict.set("Contents", Object::Array(content_array));
                    }
                    _ => {
                        // No existing content, use watermark content
                        page_dict.set("Contents", Object::Array(watermark_content_refs));
                    }
                }

                // 2. Merge Resources dictionaries
                merge_resources(page_dict, &watermark_resources)?;
            }
        }
    }

    // Save the modified document
    source_doc.save(output_path)?;

    Ok(())
}

/// Renumber all object references in an object
fn renumber_object_references(object: &Object, id_map: &std::collections::HashMap<ObjectId, ObjectId>) -> Object {
    match object {
        Object::Reference(old_id) => {
            if let Some(new_id) = id_map.get(old_id) {
                Object::Reference(*new_id)
            } else {
                Object::Reference(*old_id)
            }
        }
        Object::Array(arr) => {
            Object::Array(arr.iter().map(|obj| renumber_object_references(obj, id_map)).collect())
        }
        Object::Dictionary(dict) => {
            let mut new_dict = Dictionary::new();
            for (key, value) in dict.iter() {
                new_dict.set(key.clone(), renumber_object_references(value, id_map));
            }
            Object::Dictionary(new_dict)
        }
        Object::Stream(stream) => {
            let mut new_dict = Dictionary::new();
            for (key, value) in stream.dict.iter() {
                new_dict.set(key.clone(), renumber_object_references(value, id_map));
            }
            Object::Stream(lopdf::Stream {
                dict: new_dict,
                content: stream.content.clone(),
                allows_compression: stream.allows_compression,
                start_position: stream.start_position,
            })
        }
        _ => object.clone(),
    }
}

/// Get content references and resources from a watermark page with remapped IDs
fn get_page_content_and_resources(
    doc: &Document,
    page_num: u32,
    id_map: &std::collections::HashMap<ObjectId, ObjectId>,
) -> Result<Option<(Vec<Object>, Object)>> {
    let pages = doc.get_pages();

    for (pg_num, page_id) in pages {
        if pg_num == page_num {
            let page_obj = doc.get_object(page_id)?;

            if let Object::Dictionary(ref page_dict) = page_obj {
                // Get content references
                let content_refs = if let Ok(content) = page_dict.get(b"Contents") {
                    // Remap the content references
                    let remapped_content = renumber_object_references(content, id_map);

                    // Convert to array if it's a single reference
                    match remapped_content {
                        Object::Reference(id) => vec![Object::Reference(id)],
                        Object::Array(arr) => arr,
                        _ => vec![remapped_content],
                    }
                } else {
                    vec![]
                };

                // Get resources (remapped)
                let resources = if let Ok(res) = page_dict.get(b"Resources") {
                    renumber_object_references(res, id_map)
                } else {
                    // If no Resources, create empty dictionary
                    Object::Dictionary(Dictionary::new())
                };

                return Ok(Some((content_refs, resources)));
            }
        }
    }

    Ok(None)
}

/// Merge watermark resources into the page's resources dictionary
fn merge_resources(page_dict: &mut Dictionary, watermark_resources: &Object) -> Result<()> {
    // Get existing resources (or create empty if none)
    let mut merged_resources = if let Ok(existing) = page_dict.get(b"Resources") {
        existing.clone()
    } else {
        Object::Dictionary(Dictionary::new())
    };

    // If existing resources is a dictionary, merge in watermark resources
    if let (Object::Dictionary(ref mut merged_dict), Object::Dictionary(watermark_dict)) =
        (&mut merged_resources, watermark_resources) {

        // Merge each resource type (Font, ExtGState, XObject, etc.)
        for (key, value) in watermark_dict.iter() {
            if let Ok(existing_value) = merged_dict.get(key) {
                // If both have this resource type, merge the subdictionaries
                if let (Object::Dictionary(existing_subdict), Object::Dictionary(watermark_subdict)) =
                    (existing_value.clone(), value) {

                    let mut merged_subdict = existing_subdict.clone();
                    // Merge the subdictionary entries
                    for (subkey, subvalue) in watermark_subdict.iter() {
                        merged_subdict.set(subkey.clone(), subvalue.clone());
                    }

                    merged_dict.set(key.clone(), Object::Dictionary(merged_subdict));
                } else {
                    // Not both dictionaries, watermark overwrites
                    merged_dict.set(key.clone(), value.clone());
                }
            } else {
                // Key doesn't exist in merged, add it
                merged_dict.set(key.clone(), value.clone());
            }
        }
    }

    // Update the page's Resources
    page_dict.set("Resources", merged_resources);

    Ok(())
}

/// Wrap page content streams in q/Q to isolate transformations
///
/// This ensures that any transformation matrices in the original content
/// don't affect content streams we add later (like headers/footers).
fn wrap_page_content_in_graphics_state(doc: &mut Document, page_id: ObjectId) -> Result<()> {
    // First, get the content references (immutable borrow)
    let content_refs: Vec<ObjectId> = {
        let page_obj = doc.get_object(page_id)?;
        if let Object::Dictionary(page_dict) = page_obj {
            if let Ok(contents) = page_dict.get(b"Contents") {
                match contents {
                    Object::Reference(id) => vec![*id],
                    Object::Array(arr) => {
                        arr.iter()
                            .filter_map(|obj| {
                                if let Object::Reference(id) = obj {
                                    Some(*id)
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                    _ => return Ok(()),
                }
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    };

    // Read all content streams and concatenate them
    let mut combined_content = String::from("q\n"); // Save graphics state

    for content_id in content_refs {
        if let Ok(Object::Stream(stream)) = doc.get_object(content_id) {
            let content_str = String::from_utf8_lossy(&stream.content);
            combined_content.push_str(&content_str);
            combined_content.push('\n');
        }
    }

    combined_content.push_str("Q\n"); // Restore graphics state

    // Create a new single content stream with wrapped content
    let wrapped_stream_id = doc.add_object(lopdf::Stream::new(
        Dictionary::new(),
        combined_content.into_bytes(),
    ));

    // Replace the Contents with single wrapped stream (mutable borrow)
    let page_obj = doc.get_object_mut(page_id)?;
    if let Object::Dictionary(ref mut page_dict) = page_obj {
        page_dict.set("Contents", Object::Reference(wrapped_stream_id));
    }

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
