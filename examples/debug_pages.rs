//! Debug page counting

use pdf_handouts::pdf::count_pages;
use std::path::Path;

fn main() {
    let test_files = vec![
        "tests/fixtures/real-world/1. NT Ladder - Google Docs.pdf",
        "tests/fixtures/real-world/2. NT Ladder Practice Sheet.pdf",
        "tests/fixtures/real-world/3. ABS4-2 Jacoby Transfers Handouts.pdf",
        "tests/fixtures/real-world/4. thinking-bridge-Responding to 1NT 1-6.pdf",
        "debug_merged.pdf",
    ];

    for test_file in test_files {
        let path = Path::new(test_file);

        if !path.exists() {
            eprintln!("File not found: {}", test_file);
            continue;
        }

        match count_pages(path) {
            Ok(count) => println!("{}: {} pages", test_file, count),
            Err(e) => eprintln!("{}: ERROR - {}", test_file, e),
        }
    }
}
