//! Compare the working single PDF vs broken merged PDF

use lopdf::{Document, Object};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comparing Single vs Merged PDF ===\n");

    let single_path = Path::new("target/test_single_output.pdf");
    let merged_path = Path::new("target/test_merged_output.pdf");

    let mut single_doc = Document::load(single_path)?;
    let mut merged_doc = Document::load(merged_path)?;

    single_doc.decompress();
    merged_doc.decompress();

    println!("SINGLE PDF (WORKING):");
    println!("{}", "=".repeat(60));
    inspect_first_page(&single_doc)?;

    println!("\n\nMERGED PDF (BROKEN):");
    println!("{}", "=".repeat(60));
    inspect_first_page(&merged_doc)?;

    Ok(())
}

fn inspect_first_page(doc: &Document) -> Result<(), Box<dyn std::error::Error>> {
    let pages = doc.get_pages();

    if let Some(page_id) = pages.values().next() {
        println!("Page ID: {:?}", page_id);

        let page_obj = doc.get_object(*page_id)?;
        if let Object::Dictionary(page_dict) = page_obj {
            if let Ok(contents) = page_dict.get(b"Contents") {
                match contents {
                    Object::Reference(content_id) => {
                        println!("Contents: Single stream {:?}", content_id);
                        if let Ok(Object::Stream(stream)) = doc.get_object(*content_id) {
                            let content_str = String::from_utf8_lossy(&stream.content);
                            let lines: Vec<&str> = content_str.lines().take(20).collect();
                            for line in lines {
                                println!("  {}", line);
                            }
                        }
                    }
                    Object::Array(arr) => {
                        println!("Contents: Array with {} streams", arr.len());
                        for (i, obj) in arr.iter().enumerate() {
                            if let Object::Reference(content_id) = obj {
                                println!("\nStream {} (ID {:?}):", i + 1, content_id);
                                if let Ok(Object::Stream(stream)) = doc.get_object(*content_id) {
                                    let content_str = String::from_utf8_lossy(&stream.content);
                                    let lines: Vec<&str> = content_str.lines().take(15).collect();
                                    for line in lines {
                                        println!("  {}", line);
                                    }
                                }
                            }
                        }
                    }
                    _ => println!("Unexpected Contents format"),
                }
            }
        }
    }

    Ok(())
}
