//! Check if the font object is valid

use lopdf::{Document, Object};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Checking Font Object ===\n");

    let final_path = Path::new("target/demo_final_direct.pdf");
    let doc = Document::load(final_path)?;

    println!("Looking for font object (183, 0)...\n");

    if let Ok(font_obj) = doc.get_object((183, 0)) {
        println!("Font object found:");
        println!("{:#?}", font_obj);

        // Check if it's a valid Type0 font
        if let Object::Dictionary(font_dict) = font_obj {
            if let Ok(font_type) = font_dict.get(b"Type") {
                println!("\nType: {:?}", font_type);
            }
            if let Ok(subtype) = font_dict.get(b"Subtype") {
                println!("Subtype: {:?}", subtype);
            }
            if let Ok(base_font) = font_dict.get(b"BaseFont") {
                println!("BaseFont: {:?}", base_font);
            }
            if let Ok(encoding) = font_dict.get(b"Encoding") {
                println!("Encoding: {:?}", encoding);
            }
            if let Ok(descendant_fonts) = font_dict.get(b"DescendantFonts") {
                println!("DescendantFonts: {:?}", descendant_fonts);

                // Check the descendant font
                if let Object::Array(arr) = descendant_fonts {
                    if let Some(Object::Reference(cid_font_id)) = arr.first() {
                        println!("\nChecking CIDFont ({:?})...", cid_font_id);
                        if let Ok(cid_font) = doc.get_object(*cid_font_id) {
                            println!("{:#?}", cid_font);
                        }
                    }
                }
            }
        }
    } else {
        println!("❌ Font object NOT found!");
    }

    Ok(())
}
