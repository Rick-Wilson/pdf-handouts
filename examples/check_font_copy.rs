//! Check if fonts are properly copied to the final PDF

use lopdf::{Document, Object};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Checking Font Copy ===\n");

    let watermark_path = Path::new("target/debug_watermark.pdf");
    let final_path = Path::new("target/debug_final.pdf");

    let watermark_doc = Document::load(watermark_path)?;
    let final_doc = Document::load(final_path)?;

    println!("Watermark PDF:");
    println!("  Objects: {}", watermark_doc.objects.len());
    println!("  Max ID: {}", watermark_doc.max_id);

    // Find the font object in watermark
    if let Ok(Object::Stream(font_stream)) = watermark_doc.get_object((6, 0)) {
        println!("\n  Font object (6, 0) found:");
        println!("    Stream length: {} bytes", font_stream.content.len());
        println!("    Dict: {:?}", font_stream.dict);
    } else if let Ok(font_obj) = watermark_doc.get_object((6, 0)) {
        println!("\n  Font object (6, 0) found (not a stream):");
        println!("    {:?}", font_obj);
    }

    println!("\n\nFinal PDF:");
    println!("  Objects: {}", final_doc.objects.len());
    println!("  Max ID: {}", final_doc.max_id);

    // The font should be at offset + 6 = 55 + 6 = 61
    let expected_font_id = (61, 0);
    println!("\n  Looking for copied font at {:?}", expected_font_id);

    if let Ok(Object::Stream(font_stream)) = final_doc.get_object(expected_font_id) {
        println!("    Font object found!");
        println!("    Stream length: {} bytes", font_stream.content.len());
        println!("    Dict: {:?}", font_stream.dict);
    } else if let Ok(font_obj) = final_doc.get_object(expected_font_id) {
        println!("    Font object found (not a stream):");
        println!("    {:?}", font_obj);
    } else {
        println!("    ❌ Font object NOT found!");
    }

    // Check if the watermark content stream references this font
    println!("\n  Checking watermark content stream (58, 0):");
    if let Ok(Object::Stream(content_stream)) = final_doc.get_object((58, 0)) {
        println!("    Content stream found");
        println!("    Length: {} bytes", content_stream.content.len());

        // Check the Resources in the stream dictionary
        if let Ok(resources) = content_stream.dict.get(b"Resources") {
            println!("    ❌ ERROR: Content stream has Resources in its dictionary!");
            println!("    Resources should be in the page object, not the content stream");
            println!("    Resources: {:?}", resources);
        } else {
            println!("    ✓ Content stream has no Resources (correct)");
        }
    }

    // Check the page's Resources
    println!("\n  Checking page 1 Resources:");
    let pages = final_doc.get_pages();
    if let Some(page_id) = pages.values().next() {
        if let Ok(Object::Dictionary(page_dict)) = final_doc.get_object(*page_id) {
            if let Ok(resources) = page_dict.get(b"Resources") {
                println!("    Page Resources: {:?}", resources);

                // Check if it includes the watermark font
                if let Object::Dictionary(resources_dict) = resources {
                    if let Ok(fonts) = resources_dict.get(b"Font") {
                        println!("    Fonts in page: {:?}", fonts);
                    } else {
                        println!("    ❌ No Font entry in page Resources!");
                    }
                } else if let Object::Reference(res_id) = resources {
                    println!("    Resources is a reference: {:?}", res_id);
                    if let Ok(Object::Dictionary(resources_dict)) = final_doc.get_object(*res_id) {
                        if let Ok(fonts) = resources_dict.get(b"Font") {
                            println!("    Fonts in referenced Resources: {:?}", fonts);
                        }
                    }
                }
            } else {
                println!("    ❌ Page has no Resources!");
            }
        }
    }

    Ok(())
}
