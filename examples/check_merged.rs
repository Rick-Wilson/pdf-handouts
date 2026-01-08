//! Check the merged PDF page count

use pdf_handouts::pdf::count_pages;
use std::path::Path;

fn main() {
    let path = Path::new("test_merged_output.pdf");

    if !path.exists() {
        eprintln!("File not found: test_merged_output.pdf");
        eprintln!("Run: cargo run --example create_test_merge");
        return;
    }

    match count_pages(path) {
        Ok(count) => {
            println!("test_merged_output.pdf: {} pages", count);
            println!("\nExpected breakdown:");
            println!("  1. NT Ladder - Google Docs.pdf:              1 page");
            println!("  2. NT Ladder Practice Sheet.pdf:             1 page");
            println!("  3. ABS4-2 Jacoby Transfers Handouts.pdf:     6 pages");
            println!("  4. thinking-bridge-Responding to 1NT 1-6.pdf: 6 pages");
            println!("  Total:                                       14 pages");

            if count == 14 {
                println!("\n✓ Page count matches expected total!");
            } else {
                println!("\n✗ Page count mismatch! Expected 14, got {}", count);
            }
        }
        Err(e) => eprintln!("Error counting pages: {}", e),
    }
}
