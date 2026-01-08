# PDF Handouts

A cross-platform Rust library and CLI tool for merging PDFs and adding custom headers and footers.

This project ports the functionality from Windows PowerShell scripts (PackageSCHandouts.ps1, Package Handouts.ps1) to pure Rust, eliminating dependencies on PDFtk and wkhtmltopdf.

## Project Status

**Current Phase**: Sprint 1 - Foundation ✅ | Sprint 2 - Date & Layout (In Progress)

- [x] Project initialization with Cargo
- [x] Dependencies configured (lopdf, krilla, clap, chrono)
- [x] Basic module structure created
- [x] Error types implemented with thiserror
- [x] Layout types and tests working
- [x] PDF merging implementation ✅
- [x] Metadata extraction (page counting) ✅
- [x] Integration tests with real-world PDFs ✅
- [x] Date parsing module ✅
- [ ] Complete layout calculations (in progress)
- [ ] Header/footer PDF generation with krilla
- [ ] CLI tool with clap

## Architecture

```
pdf-handouts/
├── src/
│   ├── lib.rs              # Library entry point ✅
│   ├── error.rs            # Error types ✅
│   ├── pdf/
│   │   ├── mod.rs          # PDF module ✅
│   │   ├── merge.rs        # PDF merging ✅
│   │   ├── metadata.rs     # Metadata extraction ✅
│   │   └── create.rs       # Header/footer generation (TODO)
│   ├── date.rs             # Date parsing ✅
│   ├── layout.rs           # Layout calculations ✅
│   └── bin/
│       └── pdf-handouts.rs # CLI tool (placeholder)
├── scripts/                # Wrapper scripts (TODO)
├── tests/                  # Integration tests ✅
└── examples/               # Usage examples ✅
```

## Technology Stack

- **PDF Merging**: [lopdf](https://github.com/J-F-Liu/lopdf) 0.34 - Low-level PDF operations
- **PDF Creation**: [krilla](https://github.com/LaurenzV/krilla) 0.3 - Modern PDF generation with excellent testing
- **CLI**: [clap](https://github.com/clap-rs/clap) 4.5 - Command-line argument parsing
- **Date Handling**: [chrono](https://github.com/chronotope/chrono) 0.4 - Date/time library
- **Error Handling**: [thiserror](https://github.com/dtolnay/thiserror) 1.0 - Derive Error trait

## Building

```bash
# Check the project compiles
cargo check

# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test
```

## Current Capabilities

### PDF Merging ✅

```rust
use pdf_handouts::pdf::{MergeOptions, merge_pdfs};
use std::path::PathBuf;

let options = MergeOptions {
    input_paths: vec![
        PathBuf::from("1. first.pdf"),
        PathBuf::from("2. second.pdf"),
    ],
    output_path: PathBuf::from("merged.pdf"),
};

merge_pdfs(&options)?;
```

### Metadata Extraction ✅

```rust
use pdf_handouts::pdf::{count_pages, extract_metadata};
use std::path::Path;

// Count pages
let count = count_pages(Path::new("document.pdf"))?;

// Extract full metadata
let metadata = extract_metadata(Path::new("document.pdf"))?;
println!("Pages: {}", metadata.page_count);
println!("Title: {:?}", metadata.title);
println!("Author: {:?}", metadata.author);
```

### Date Parsing ✅

```rust
use pdf_handouts::date::{parse_date_expression, resolve_date, format_date};

// Parse flexible date expressions
let expr = parse_date_expression("Tuesday")?;
let date = resolve_date(&expr).unwrap();
let formatted = format_date(&date);
// Output: "January 13, 2026"

// Supported formats:
// - "" (empty) → None
// - "today" → Current date
// - "2024-11-20" → ISO format
// - "11/20/2024" → US format
// - "Tuesday" → Next Tuesday (or today if today is Tuesday)
// - "Tuesday+3" → 4th upcoming Tuesday (next + 3 weeks)
```

### Layout Module ✅

```rust
use pdf_handouts::layout::*;

// Create page dimensions
let letter = PageDimensions::letter();  // US Letter (8.5" × 11")
let a4 = PageDimensions::a4();          // A4 (210mm × 297mm)

// Work with lengths
let margin = Length::from_mm(25.4);  // 1 inch
assert_eq!(margin.pt(), 72.0);       // 72 points

// Create margins
let margins = Margins::uniform(Length::from_mm(12.7));
```

### Error Handling ✅

```rust
use pdf_handouts::error::{Error, Result};

// All operations return Result<T, Error>
fn example() -> Result<()> {
    // Errors are automatically converted from:
    // - lopdf::Error
    // - std::io::Error
    // Custom errors for domain-specific issues
    Ok(())
}
```

## Planned Features

See [ORIGINAL_DESIGN_PDF_HANDOUTS.md](ORIGINAL_DESIGN_PDF_HANDOUTS.md) for the original PowerShell implementation details.

### Phase 1: Core Library ✅
- [x] Project setup
- [x] PDF merging with lopdf
- [x] Page counting and metadata extraction
- [x] Flexible date expression parsing

### Phase 2: Header/Footer Generation
- [ ] Watermark PDF creation with krilla
- [ ] Three-column footer layout (25% / 50% / 25%)
- [ ] Title on first page
- [ ] Page numbering with optional total count
- [ ] Date formatting

### Phase 3: CLI Tool
- [ ] Command-line interface with clap
- [ ] Glob pattern support for input files
- [ ] All PowerShell script parameters supported
- [ ] Bash and PowerShell wrapper scripts for Stoneridge Creek defaults

### Future Enhancements
- Content scaling to avoid header/footer overlap
- Image file input support (JPG, PNG, TIFF)
- Configuration file support
- GUI for iPad app integration

## Original Design Reference

This project replaces three PowerShell scripts:

1. **PackageSCHandouts.ps1** - Wrapper with Stoneridge Creek defaults
2. **Package Handouts.ps1** - Main orchestrator
3. **Handouts Template Generator.ps1** - Watermark generator

The original implementation used:
- PDFtk for merging and stamping
- wkhtmltopdf for HTML→PDF conversion
- Windows-specific paths and tools

Our Rust implementation eliminates all external dependencies and works cross-platform.

## License

MIT OR Apache-2.0

## Author

Rick Wilson
