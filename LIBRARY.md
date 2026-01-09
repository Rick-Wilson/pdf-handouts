# PDF Handouts Library

A Rust library for merging PDFs and adding custom headers and footers.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pdf-handouts = "0.1"
```

## Quick Start

```rust
use pdf_handouts::pdf::{merge_pdfs, add_headers_footers, MergeOptions, HeaderFooterOptions};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Merge PDFs
    let merge_opts = MergeOptions {
        input_paths: vec![
            PathBuf::from("1. intro.pdf"),
            PathBuf::from("2. main.pdf"),
        ],
        output_path: PathBuf::from("merged.pdf"),
    };
    merge_pdfs(&merge_opts)?;

    // Add headers/footers
    let header_opts = HeaderFooterOptions {
        title: Some("My Document".to_string()),
        footer_right: Some("Page [page] of [pages]".to_string()),
        ..Default::default()
    };
    add_headers_footers(
        std::path::Path::new("merged.pdf"),
        std::path::Path::new("final.pdf"),
        &header_opts
    )?;

    Ok(())
}
```

## Modules

### `pdf_handouts::pdf`

Core PDF manipulation functions.

### `pdf_handouts::date`

Date expression parsing and formatting.

### `pdf_handouts::layout`

Page dimensions and layout calculations.

### `pdf_handouts::error`

Error types and result aliases.

---

## PDF Module

### `merge_pdfs`

Merge multiple PDF files into a single PDF.

```rust
use pdf_handouts::pdf::{merge_pdfs, MergeOptions};
use std::path::PathBuf;

let options = MergeOptions {
    input_paths: vec![
        PathBuf::from("file1.pdf"),
        PathBuf::from("file2.pdf"),
        PathBuf::from("file3.pdf"),
    ],
    output_path: PathBuf::from("merged.pdf"),
};

merge_pdfs(&options)?;
```

### `add_headers_footers`

Add headers and footers to an existing PDF.

```rust
use pdf_handouts::pdf::{add_headers_footers, HeaderFooterOptions, FontSpec};
use chrono::NaiveDate;
use std::path::Path;

let options = HeaderFooterOptions {
    // Title (first page only, centered at top)
    title: Some("Workshop Handout".to_string()),

    // Footer sections (all pages)
    footer_left: Some("Acme Corp|[font italic]Engineering[/font]".to_string()),
    footer_center: Some("Confidential".to_string()),
    footer_right: Some("Page [page] of [pages]|[date]".to_string()),

    // Date for [date] placeholder
    date: Some(NaiveDate::from_ymd_opt(2026, 1, 14).unwrap()),

    // Legacy page number options (prefer using [page] placeholder)
    show_page_numbers: false,
    show_total_page_count: false,

    // Legacy font sizes (prefer using FontSpec)
    title_font_size: 24.0,
    footer_font_size: 14.0,

    // Font specifications (override legacy sizes)
    header_font: Some(FontSpec::parse("24pt #333333")),
    footer_font: Some(FontSpec::parse("14pt #555555")),
};

add_headers_footers(
    Path::new("input.pdf"),
    Path::new("output.pdf"),
    &options
)?;
```

### `MergeOptions`

```rust
pub struct MergeOptions {
    /// Input PDF file paths in the order they should be merged
    pub input_paths: Vec<PathBuf>,
    /// Output PDF file path
    pub output_path: PathBuf,
}
```

### `HeaderFooterOptions`

```rust
pub struct HeaderFooterOptions {
    /// Title text (centered at top of first page only)
    pub title: Option<String>,

    /// Footer left section content
    pub footer_left: Option<String>,

    /// Footer center section content
    pub footer_center: Option<String>,

    /// Footer right section content
    pub footer_right: Option<String>,

    /// Date to use for [date] placeholder
    pub date: Option<NaiveDate>,

    /// Whether to show page numbers (legacy - use [page] placeholder instead)
    pub show_page_numbers: bool,

    /// Whether to show total page count (legacy - use [pages] placeholder instead)
    pub show_total_page_count: bool,

    /// Title font size in points (legacy - use header_font instead)
    pub title_font_size: f32,

    /// Footer font size in points (legacy - use footer_font instead)
    pub footer_font_size: f32,

    /// Header font specification (overrides title_font_size)
    pub header_font: Option<FontSpec>,

    /// Footer font specification (overrides footer_font_size)
    pub footer_font: Option<FontSpec>,
}

impl Default for HeaderFooterOptions {
    fn default() -> Self {
        Self {
            title: None,
            footer_left: None,
            footer_center: None,
            footer_right: None,
            date: None,
            show_page_numbers: true,
            show_total_page_count: false,
            title_font_size: 24.0,
            footer_font_size: 14.0,
            header_font: None,
            footer_font: None,
        }
    }
}
```

### `FontSpec`

Font specification for styling headers and footers.

```rust
pub struct FontSpec {
    /// Bold weight
    pub bold: bool,
    /// Italic style
    pub italic: bool,
    /// Font size in points
    pub size: Option<f32>,
    /// Font family name
    pub family: Option<String>,
    /// RGB color (0.0-1.0 for each component)
    pub color: Option<(f32, f32, f32)>,
}
```

**Parsing from string:**

```rust
use pdf_handouts::pdf::FontSpec;

// Size only
let spec = FontSpec::parse("14pt");

// Bold with size
let spec = FontSpec::parse("bold 16pt");

// Full specification
let spec = FontSpec::parse("bold italic 24pt Liberation_Serif #333333");

// Just color
let spec = FontSpec::parse("#ff0000");
```

**Creating programmatically:**

```rust
let spec = FontSpec {
    bold: true,
    italic: false,
    size: Some(14.0),
    family: Some("Liberation Serif".to_string()),
    color: Some((0.2, 0.2, 0.2)), // Dark gray
};
```

### Text Placeholders

The following placeholders are expanded in footer text:

| Placeholder | Replaced With |
|-------------|---------------|
| `[page]` | Current page number |
| `[pages]` | Total page count |
| `[date]` | Formatted date (from `options.date`) |

### Line Breaks

Use `|` or `[br]` in footer text to create line breaks:

```rust
footer_left: Some("Line 1|Line 2".to_string()),
// or
footer_left: Some("Line 1[br]Line 2".to_string()),
```

### Inline Font Tags

Use `[font]...[/font]` for inline styling:

```rust
footer_left: Some("Normal [font italic]italic[/font] normal".to_string()),
footer_center: Some("[font bold]Bold text[/font]".to_string()),
```

### Metadata Functions

```rust
use pdf_handouts::pdf::{count_pages, extract_metadata};
use std::path::Path;

// Count pages
let count = count_pages(Path::new("document.pdf"))?;
println!("Pages: {}", count);

// Extract metadata
let metadata = extract_metadata(Path::new("document.pdf"))?;
println!("Pages: {}", metadata.page_count);
println!("Title: {:?}", metadata.title);
println!("Author: {:?}", metadata.author);
```

### `PdfMetadata`

```rust
pub struct PdfMetadata {
    pub page_count: usize,
    pub title: Option<String>,
    pub author: Option<String>,
}
```

---

## Date Module

### `parse_date_expression`

Parse a date expression string into a `DateExpression`.

```rust
use pdf_handouts::date::{parse_date_expression, DateExpression};

// Empty → None
let expr = parse_date_expression("")?;
assert_eq!(expr, DateExpression::None);

// "today" → Today
let expr = parse_date_expression("today")?;
assert_eq!(expr, DateExpression::Today);

// ISO date
let expr = parse_date_expression("2026-01-14")?;
// → DateExpression::Explicit(NaiveDate)

// US date
let expr = parse_date_expression("01/14/2026")?;
// → DateExpression::Explicit(NaiveDate)

// Weekday
let expr = parse_date_expression("Tuesday")?;
// → DateExpression::DayOfWeek { day: Tue, offset: 0 }

// Weekday with offset
let expr = parse_date_expression("Tuesday+3")?;
// → DateExpression::DayOfWeek { day: Tue, offset: 3 }
```

### `resolve_date`

Resolve a `DateExpression` to an actual `NaiveDate`.

```rust
use pdf_handouts::date::{parse_date_expression, resolve_date};

let expr = parse_date_expression("next tuesday")?;
let date = resolve_date(&expr);
// → Some(NaiveDate) for the next Tuesday
```

### `format_date`

Format a date as "Month day, year".

```rust
use pdf_handouts::date::format_date;
use chrono::NaiveDate;

let date = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
let formatted = format_date(&date);
assert_eq!(formatted, "January 14, 2026");
```

### `DateExpression`

```rust
pub enum DateExpression {
    /// Use today's date
    Today,
    /// Use an explicit date
    Explicit(NaiveDate),
    /// Use next occurrence of a weekday with optional offset
    DayOfWeek { day: Weekday, offset: u32 },
    /// No date
    None,
}
```

---

## Layout Module

### Page Dimensions

```rust
use pdf_handouts::layout::PageDimensions;

// Standard sizes
let letter = PageDimensions::letter();  // 8.5" × 11"
let a4 = PageDimensions::a4();          // 210mm × 297mm

// Custom dimensions
let custom = PageDimensions::new(
    Length::from_inches(8.5),
    Length::from_inches(14.0)
);
```

### Length

```rust
use pdf_handouts::layout::Length;

// Create from different units
let pt = Length::from_pt(72.0);    // 72 points = 1 inch
let inch = Length::from_inches(1.0);
let mm = Length::from_mm(25.4);    // 25.4mm = 1 inch

// All are equivalent
assert_eq!(pt.pt(), inch.pt());
assert_eq!(inch.pt(), mm.pt());

// Convert between units
let length = Length::from_inches(2.0);
println!("{} points", length.pt());    // 144.0
println!("{} inches", length.inches()); // 2.0
println!("{} mm", length.mm());         // 50.8
```

### Margins

```rust
use pdf_handouts::layout::{Margins, Length};

// Uniform margins
let margins = Margins::uniform(Length::from_inches(0.5));

// Individual margins
let margins = Margins {
    top: Length::from_inches(1.0),
    bottom: Length::from_inches(0.75),
    left: Length::from_inches(0.5),
    right: Length::from_inches(0.5),
};

// Standard margins
let standard = Margins::standard(); // 0.5" all around
```

---

## Error Handling

All functions return `Result<T, Error>`.

```rust
use pdf_handouts::error::{Error, Result};

fn example() -> Result<()> {
    // Errors are automatically converted from:
    // - lopdf::Error (PDF parsing errors)
    // - std::io::Error (file I/O errors)

    // Domain-specific errors:
    // - Error::FileNotFound(PathBuf)
    // - Error::EmptyPdf(PathBuf)
    // - Error::General(String)
    // - Error::InvalidDateExpression(String)

    Ok(())
}
```

---

## Complete Example

```rust
use pdf_handouts::pdf::{
    merge_pdfs, add_headers_footers,
    MergeOptions, HeaderFooterOptions, FontSpec,
};
use pdf_handouts::date::{parse_date_expression, resolve_date};
use std::path::{Path, PathBuf};

fn create_handout() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Merge input PDFs
    let merge_options = MergeOptions {
        input_paths: vec![
            PathBuf::from("lesson1.pdf"),
            PathBuf::from("lesson2.pdf"),
            PathBuf::from("exercises.pdf"),
        ],
        output_path: PathBuf::from("temp_merged.pdf"),
    };
    merge_pdfs(&merge_options)?;

    // Step 2: Parse date expression
    let date_expr = parse_date_expression("next tuesday")?;
    let date = resolve_date(&date_expr);

    // Step 3: Add headers and footers
    let header_options = HeaderFooterOptions {
        title: Some("Workshop Handout".to_string()),
        footer_left: Some("Acme Corp|[font italic]Training Dept[/font]".to_string()),
        footer_center: Some("Presented by:|Jane Smith".to_string()),
        footer_right: Some("Page [page] of [pages]|[date]".to_string()),
        date,
        show_page_numbers: false,
        show_total_page_count: false,
        title_font_size: 24.0,
        footer_font_size: 14.0,
        header_font: Some(FontSpec::parse("24pt #333333")),
        footer_font: Some(FontSpec::parse("14pt #555555")),
    };

    add_headers_footers(
        Path::new("temp_merged.pdf"),
        Path::new("final_handout.pdf"),
        &header_options,
    )?;

    // Clean up temp file
    std::fs::remove_file("temp_merged.pdf")?;

    println!("Created: final_handout.pdf");
    Ok(())
}
```

## License

MIT OR Apache-2.0
