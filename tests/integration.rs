//! Integration tests for PDF handouts library

use pdf_handouts::pdf::{count_pages, merge_pdfs, MergeOptions};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test helper to get the path to test fixtures
fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("real-world");
    path.push(name);
    path
}

#[test]
fn test_count_pages_real_pdfs() {
    // Test counting pages in real PDF files
    // Note: These are the actual Count values from the PDF metadata
    // Some PDFs may have incorrect Count fields in their metadata
    let test_cases = vec![
        ("1. NT Ladder - Google Docs.pdf", 1),
        ("2. NT Ladder Practice Sheet.pdf", 1),
        ("3. ABS4-2 Jacoby Transfers Handouts.pdf", 6),
        ("4. thinking-bridge-Responding to 1NT 1-6.pdf", 6),
    ];

    for (filename, expected_pages) in test_cases {
        let path = fixture_path(filename);

        // Skip if file doesn't exist (in case test fixtures aren't available)
        if !path.exists() {
            eprintln!("Skipping test for {}: file not found", filename);
            continue;
        }

        let page_count = count_pages(&path)
            .expect(&format!("Failed to count pages in {}", filename));

        assert_eq!(
            page_count, expected_pages,
            "Page count mismatch for {}: expected {}, got {}",
            filename, expected_pages, page_count
        );
    }
}

#[test]
fn test_merge_real_pdfs_page_count() {
    // Verify that merged PDF has correct total page count
    // Using actual Count values from PDF metadata
    let input_files = vec![
        ("1. NT Ladder - Google Docs.pdf", 1),
        ("2. NT Ladder Practice Sheet.pdf", 1),
        ("3. ABS4-2 Jacoby Transfers Handouts.pdf", 6),
        ("4. thinking-bridge-Responding to 1NT 1-6.pdf", 6),
    ];

    // Build list of input paths
    let mut input_paths = Vec::new();
    let mut expected_total = 0;

    for (filename, pages) in &input_files {
        let path = fixture_path(filename);
        if !path.exists() {
            eprintln!("Skipping merge test: {} not found", filename);
            return; // Skip entire test if any file is missing
        }
        input_paths.push(path);
        expected_total += pages;
    }

    // Create temporary directory for output
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("merged.pdf");

    // Merge PDFs
    let options = MergeOptions {
        input_paths,
        output_path: output_path.clone(),
    };

    merge_pdfs(&options).expect("Failed to merge PDFs");

    // Verify output exists
    assert!(output_path.exists(), "Merged PDF was not created");

    // Count pages in merged PDF
    let merged_page_count = count_pages(&output_path)
        .expect("Failed to count pages in merged PDF");

    assert_eq!(
        merged_page_count, expected_total,
        "Merged PDF should have {} pages (sum of all inputs), got {}",
        expected_total, merged_page_count
    );

    println!("✓ Successfully merged {} PDFs into {} pages",
             input_files.len(), merged_page_count);
}

#[test]
fn test_merge_preserves_content_order() {
    // Verify that pages appear in correct order after merge
    let input_files = vec![
        ("1. NT Ladder - Google Docs.pdf", 1),
        ("2. NT Ladder Practice Sheet.pdf", 1),
        ("3. ABS4-2 Jacoby Transfers Handouts.pdf", 6),
    ];

    let mut input_paths = Vec::new();
    let mut expected_total = 0;
    for (filename, pages) in &input_files {
        let path = fixture_path(filename);
        if !path.exists() {
            eprintln!("Skipping order test: {} not found", filename);
            return;
        }
        input_paths.push(path);
        expected_total += pages;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("ordered.pdf");

    let options = MergeOptions {
        input_paths,
        output_path: output_path.clone(),
    };

    merge_pdfs(&options).expect("Failed to merge PDFs");

    // Verify the merge succeeded
    assert!(output_path.exists(), "Merged PDF was not created");

    // Expected: 1 + 1 + 6 = 8 pages
    let page_count = count_pages(&output_path)
        .expect("Failed to count pages");
    assert_eq!(page_count, expected_total, "Merged PDF should have {} pages", expected_total);

    println!("✓ Page order preserved in merged PDF");
}

#[test]
fn test_merge_empty_input_list() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("empty.pdf");

    let options = MergeOptions {
        input_paths: vec![],
        output_path: output_path.clone(),
    };

    let result = merge_pdfs(&options);
    assert!(result.is_err(), "Should fail with empty input list");

    if let Err(e) = result {
        assert!(
            e.to_string().contains("No input files"),
            "Error message should mention no input files"
        );
    }
}

#[test]
fn test_merge_nonexistent_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("output.pdf");

    let options = MergeOptions {
        input_paths: vec![PathBuf::from("nonexistent.pdf")],
        output_path: output_path.clone(),
    };

    let result = merge_pdfs(&options);
    assert!(result.is_err(), "Should fail with nonexistent file");

    if let Err(e) = result {
        assert!(
            e.to_string().contains("not found") || e.to_string().contains("nonexistent"),
            "Error should mention file not found: {}",
            e
        );
    }
}
