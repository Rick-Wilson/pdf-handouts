//! Inspect the actual content stream from the watermark PDF

use lopdf::{Document, Object};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Inspecting Watermark Content Streams ===\n");

    let watermark_path = Path::new("target/debug_watermark.pdf");
    let watermark_doc = Document::load(watermark_path)?;

    let pages = watermark_doc.get_pages();

    for (page_num, page_id) in &pages {
        println!("Page {}:", page_num);

        let page_obj = watermark_doc.get_object(*page_id)?;
        if let Object::Dictionary(ref page_dict) = page_obj {
            if let Ok(Object::Reference(content_id)) = page_dict.get(b"Contents") {
                println!("  Content ID: {:?}", content_id);

                if let Ok(Object::Stream(stream)) = watermark_doc.get_object(*content_id) {
                    println!("  Stream length: {} bytes", stream.content.len());
                    println!("  Allows compression: {}", stream.allows_compression);

                    // The content is the decompressed data
                    let decoded = stream.content.clone();

                    // Print the actual PDF content stream operators
                    let content_str = String::from_utf8_lossy(&decoded);
                    println!("\n  Content stream:");
                    println!("  {}", "=".repeat(60));
                    println!("{}", content_str);
                    println!("  {}", "=".repeat(60));
                }
            }

            // Also check Resources
            if let Ok(resources) = page_dict.get(b"Resources") {
                println!("\n  Resources: {:?}", resources);

                if let Object::Dictionary(ref resources_dict) = resources {
                    if let Ok(fonts) = resources_dict.get(b"Font") {
                        println!("  Fonts: {:?}", fonts);
                    }
                }
            }
        }

        println!("\n");
    }

    Ok(())
}
