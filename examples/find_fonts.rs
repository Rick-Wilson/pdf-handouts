//! Find all Font objects in PDF

use lopdf::{Document, Object};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("Usage: find_fonts <pdf>");
    let doc = Document::load(Path::new(&path))?;

    println!("Looking for Font objects...");
    for (id, obj) in &doc.objects {
        if let Object::Dictionary(dict) = obj {
            if let Ok(obj_type) = dict.get(b"Type") {
                if let Object::Name(name) = obj_type {
                    if name == b"Font" {
                        println!("  Font {:?}: {:?}", id, dict);
                    }
                }
            }
        }
    }

    // Also search for Font dictionaries in Resources
    println!("\nSearching all dictionaries for Font entries...");
    for (id, obj) in &doc.objects {
        if let Object::Dictionary(dict) = obj {
            if let Ok(font) = dict.get(b"Font") {
                println!("  {:?} has Font entry: {:?}", id, font);
            }
        }
    }

    Ok(())
}
