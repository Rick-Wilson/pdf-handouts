# Original Design: PDF Handouts Tool

## Overview

This document describes the PowerShell-based PDF handout generation system that merges source PDFs and adds customizable headers and footers. The core scripts are:
- `PackageSCHandouts.ps1` - Convenience wrapper with Stoneridge Creek defaults
- `Package Handouts.ps1` - Main orchestrator
- `Handouts Template Generator.ps1` - Watermark generator

**Location**: `/Users/rick/Documents/Bridge/Tools`

## System Architecture

### Script Hierarchy

```
┌─────────────────────────────┐
│  PackageSCHandouts.ps1      │  (Top-level entry point - Stoneridge Creek specific)
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  Package Handouts.ps1       │  (Main orchestrator)
└─────────────┬───────────────┘
              │
              ├──► Uses pdftk to merge input PDFs
              │
              ├──► Counts pages in merged PDF
              │
              └──► Calls ▼
                   ┌─────────────────────────────┐
                   │  Handouts Template          │
                   │  Generator.ps1              │
                   └─────────────┬───────────────┘
                                 │
                                 ├──► Generates HTML template with headers/footers
                                 ├──► Uses wkhtmltopdf to convert HTML to PDF watermark
                                 └──► Returns watermark PDF
              │
              └──► Uses pdftk to stamp watermark onto merged PDF
```

## Core Components

### 1. PackageSCHandouts.ps1

**Purpose**: Top-level convenience wrapper for Stoneridge Creek bridge classes.

**Parameters**:
- `$Title` - Title to display on first page
- `$InputPath` - Glob pattern for input PDFs (default: "?.*.pdf")
- `$OutputPath` - Output filename (default: "Handouts.pdf")

**Behavior**:
- Hardcodes Stoneridge Creek-specific footer information:
  - FooterLeft: "Stoneridge Creek"
  - Date: Tuesday (next Tuesday's date)
  - FooterCenter: "Presented by:<br>Rick Wilson"
- Passes parameters to `Package Handouts.ps1`

**Example Usage**:
```powershell
.\PackageSCHandouts.ps1 -Title "Two Over One Introduction" -InputPath "?.*.pdf"
```

### 2. Package Handouts.ps1

**Purpose**: Main orchestrator that merges PDFs and applies headers/footers.

**Parameters**:
- `$inputPath` - Path or pattern to input PDF(s)
- `$outputPath` - Output filename
- `$Title` - Optional title for first page
- `$Date` - Date specification (see Date Parsing below)
- `$FooterLeft`, `$FooterCenter`, `$FooterRight` - Footer content
- `$Font` - Font family (default: "Garamond, serif")
- `$TitleFontSize` - Title font size (default: "24px")
- `$FooterFontSize` - Footer font size (default: "16px")
- `$ShowTotalPageCount` - Switch to show "Page n of m"
- `$NoPageNumber` - Switch to suppress page numbers
- `$KeepHTML` - Switch to preserve intermediate HTML files

**Process Flow**:

1. **Merge Input PDFs**
   - Uses `pdftk` to concatenate all matching input PDFs
   - Creates temporary file: `TempMerge.pdf`
   - Command: `pdftk "$inputPath" cat output TempMerge.pdf`

2. **Count Pages**
   - Uses `pdftk dump_data` to extract metadata
   - Parses `NumberOfPages:` field from output
   - Stores page count for watermark generation

3. **Generate Watermark**
   - Calls `Handouts Template Generator.ps1`
   - Passes page count and all footer parameters
   - Generates `Watermark.pdf` with exact number of pages needed

4. **Apply Watermark**
   - Uses `pdftk multistamp` to overlay watermark
   - Command: `pdftk TempMerge.pdf multistamp Watermark.pdf output $outputPath`
   - The multistamp operation merges page 1 of source with page 1 of watermark, page 2 with page 2, etc.
   - If source has more pages than watermark, last watermark page is reused

5. **Cleanup**
   - Removes `TempMerge.pdf` and `Watermark.pdf` temporary files

### 3. Handouts Template Generator.ps1

**Purpose**: Generates a multi-page watermark PDF containing headers and footers.

**Parameters**:
- `$outputPath` - Base name for output (no extension)
- `$Title` - Title for first page
- `$Date` - Date expression
- `$NumberOfPages` - How many pages to generate (default: 50)
- `$FooterLeft`, `$FooterCenter`, `$FooterRight` - Footer sections
- `$Font` - Font family (default: "Garamond, serif")
- `$TitleFontSize` - Title size (default: "24px")
- `$FooterFontSize` - Footer size (default: "16px")
- `$ShowTotalPageCount` - Include total in page numbers
- `$NoPageNumber` - Suppress page numbers entirely
- `$PDF` - Switch to generate PDF (otherwise HTML only)
- `$KeepHTML` - Preserve HTML file after PDF generation

**Process Flow**:

1. **Date Parsing** (via `Get-ReportDate` function)
   - Blank/null → no date
   - `"today"` → current date
   - Date string (`"2024-11-20"`) → parsed date
   - Day of week (`"Tuesday"`) → next occurrence
   - Day+offset (`"Tuesday+3"`) → 4th upcoming Tuesday
   - Formats as "Month day, year" (e.g., "November 20, 2024")

2. **HTML Generation**
   - Creates HTML document with CSS for page layout
   - Page size: 7.5" × 10.8" (with margins: 8.5" × 11" letter)
   - Generates `$NumberOfPages` pages
   - First page gets title in body area
   - All pages get footer with three columns

3. **Footer Layout**
   ```
   ┌──────────────────────────────────────────────┐
   │                  Page Body                    │
   │              (10.4" height)                   │
   │                                               │
   │  First page only: Title centered              │
   ├──────────────────────────────────────────────┤
   │ Footer (0.4" height)                         │
   │                                               │
   │ Left (25%)     │  Center (50%)  │ Right (25%) │
   │ FooterLeft     │  FooterCenter  │ FooterRight │
   │ Date           │                │ Page n [of m]│
   └──────────────────────────────────────────────┘
   ```

4. **PDF Conversion**
   - Uses `wkhtmltopdf` (installed at `C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf`)
   - Arguments: `--page-size Letter --margin-top 0mm --margin-bottom 0mm --margin-left 0mm --margin-right 0mm --quiet --disable-smart-shrinking`
   - Converts HTML to PDF with precise page dimensions
   - Optionally deletes HTML file unless `$KeepHTML` is set

## External Tool Dependencies

### PDFtk (PDF Toolkit)

**Purpose**: PDF manipulation (merge, stamp, metadata extraction)

**Installation**: https://www.pdflabs.com/tools/pdftk-the-pdf-toolkit/ (~$4.00 USD)

**Key Commands Used**:

1. **Concatenation**:
   ```bash
   pdftk input1.pdf input2.pdf cat output merged.pdf
   ```

2. **Metadata Extraction**:
   ```bash
   pdftk input.pdf dump_data
   ```
   Output includes: `NumberOfPages: 42`

3. **Multistamp** (overlay watermark):
   ```bash
   pdftk source.pdf multistamp watermark.pdf output result.pdf
   ```
   - Overlays watermark page 1 on source page 1, etc.
   - Repeats last watermark page if source has more pages

### wkhtmltopdf

**Purpose**: HTML to PDF conversion with precise rendering control

**Installation Path**: `C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf`

**Critical Arguments**:
- `--page-size Letter` - US Letter (8.5" × 11")
- `--margin-top 0mm` - No top margin
- `--margin-bottom 0mm` - No bottom margin
- `--margin-left 0mm` - No left margin
- `--margin-right 0mm` - No right margin
- `--quiet` - Suppress output
- `--disable-smart-shrinking` - Preserve exact dimensions
- `--print-media-type` - Use print CSS media rules

## File Naming Conventions

### Input Files

The system expects input PDFs to be named with numeric prefixes:

- `1.*.pdf` - First document to merge
- `2.*.pdf` - Second document
- `3.*.pdf` - Third document
- etc.

The glob pattern `?.*.pdf` matches single-digit prefixes (1-9). For more files, use patterns like `??.*.pdf` (10-99) or `*.pdf`.

**Examples**:
- `1. Introduction.pdf`
- `2. Advanced Topics.pdf`
- `3. Practice Problems.pdf`

### Output Files

- `Handouts.pdf` - Default output name
- `TempMerge.pdf` - Temporary merged source (deleted after processing)
- `Watermark.pdf` - Temporary watermark overlay (deleted after processing)
- `*.html` - Intermediate HTML (optionally kept with `-KeepHTML`)

## HTML/CSS Template Structure

### Page Dimensions

```css
.reportcontainer {
    width: 7.5in;
    height: 10.8in;
    margin-left: 0.5in;
    page-break-after: always;
}
.reportbody {
    height: 10.4in;  /* Main content area */
}
.reportfooter {
    height: 0.4in;   /* Footer area */
}
```

**Total**: 7.5" + 0.5" left margin = 8" width (with 0.5" right margin = 8.5" total)
**Total**: 10.8" height (with margins = 11" total for US Letter)

### Footer Layout (CSS Table Display)

```css
.table { display: table; width: 100%; }
.tr { display: table-row; }
.d1 { display: table-cell; width: 25%; text-align: left; }
.d2 { display: table-cell; width: 50%; text-align: center; }
.d3 { display: table-cell; width: 25%; text-align: right; }
```

### HTML Tags in Parameters

The system supports HTML tags in `$Title` and `$Footer*` parameters:

- `<br>` - Line breaks
- `<b>` - Bold text
- `<i>` - Italic text
- Other basic HTML formatting

**Example**:
```powershell
-FooterCenter "Presented by:<br><b>Rick Wilson</b>"
```

## Date Parsing Details

The `Get-ReportDate` function provides flexible date specification:

### Input Formats

1. **Null/Empty**: Returns `$null` (no date displayed)
   ```powershell
   -Date ""
   ```

2. **"today"**: Current date
   ```powershell
   -Date "today"
   # Output: "January 7, 2026"
   ```

3. **Explicit Date**: Standard date formats
   ```powershell
   -Date "2024-11-20"          # ISO format
   -Date "11/20/2024"          # US format
   -Date "20 November 2024"    # Long format
   # Output: "November 20, 2024"
   ```

4. **Day of Week**: Next occurrence
   ```powershell
   -Date "Tuesday"
   # If today is Monday Jan 6, 2026 → "January 7, 2026"
   # If today is Tuesday Jan 7, 2026 → "January 7, 2026" (today)
   ```

5. **Day of Week + Offset**: Future occurrence
   ```powershell
   -Date "Tuesday+3"
   # Finds 4th upcoming Tuesday (next + 3 more weeks)
   ```

### Output Format

All dates are formatted as: `"Month day, year"`

**Example**: `"November 20, 2024"`

## Typical Usage Scenarios

### Scenario 1: Quick Handout for Stoneridge Creek

```powershell
cd "C:\Bridge\Lesson Materials"
.\PackageSCHandouts.ps1 -Title "Two Over One Introduction"
```

**Result**: Merges all `1.*.pdf`, `2.*.pdf`, etc. files in current directory, adds:
- Title: "Two Over One Introduction" on page 1
- Footer Left: "Stoneridge Creek" and next Tuesday's date
- Footer Center: "Presented by:<br>Rick Wilson"
- Footer Right: Page numbers

### Scenario 2: Custom Handout with Specific Date

```powershell
.\Package Handouts.ps1 `
    -InputPath "lesson*.pdf" `
    -OutputPath "MyHandout.pdf" `
    -Title "Advanced Bidding Techniques" `
    -Date "2024-12-15" `
    -FooterLeft "Bridge Club" `
    -FooterCenter "Instructor: Jane Doe" `
    -FooterRight "Advanced Series" `
    -ShowTotalPageCount
```

**Result**: Custom handout with:
- Specified date: "December 15, 2024"
- Page numbers showing "Page n of m" format
- Custom footer information

### Scenario 3: Minimal Headers/Footers

```powershell
.\Package Handouts.ps1 `
    -InputPath "1.*.pdf 2.*.pdf 3.*.pdf" `
    -OutputPath "SimpleHandout.pdf" `
    -NoPageNumber
```

**Result**: Merged PDF with no headers or footers (clean merge).

## Known Limitations

### 1. PDFtk Watermark Behavior

**Issue**: Content from source PDFs may overlap with watermark headers/footers.

**Details**:
- No automatic scaling or repositioning of source content
- Headers and footers are simply overlaid on existing content
- If source PDF has content near page margins, it will be obscured

**Impact**: Headers/footers may cover important content or vice versa.

### 2. Page Count Requirement

**Issue**: Must generate enough watermark pages for source document.

**Details**:
- Default 50 pages may be insufficient for very long documents
- Excess watermark pages are simply ignored
- If watermark has fewer pages than source, last watermark page is reused

**Workaround**: System automatically counts pages and generates exact number needed.

### 3. Platform Dependencies

**Issue**: Windows-specific implementation.

**Details**:
- PowerShell scripts (Windows-native)
- Hardcoded path: `C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf`
- Requires manual installation of PDFtk and wkhtmltopdf

**Impact**: Not portable to macOS or Linux without modifications.

### 4. Font Rendering

**Issue**: Relies on system fonts.

**Details**:
- Default font: "Garamond, serif"
- May render differently on systems without Garamond installed
- Fallback to generic serif font if Garamond unavailable

### 5. No Image Input

**Issue**: Only accepts PDF input files.

**Details**:
- Cannot merge image files (JPG, PNG, TIFF, etc.) directly
- Images must first be converted to PDF externally

**Workaround**: Use image-to-PDF converter before running script.

## Design Philosophy

The original system follows a **multi-stage pipeline** approach:

1. **Merge** source PDFs into single document
2. **Generate** watermark with headers/footers via HTML→PDF
3. **Stamp** watermark onto merged document

### Advantages

- **Separation of concerns**: Content and decoration handled independently
- **Reusability**: Watermark can be reused for multiple documents
- **Page-specific footers**: Each page can have unique footer (e.g., incrementing page numbers)
- **HTML flexibility**: Easy to customize layout with HTML/CSS

### Disadvantages

- **Content overlap**: Source content may overlap with headers/footers (no content reflow)
- **External dependencies**: Requires PDFtk and wkhtmltopdf
- **Two-pass processing**: Generate watermark, then stamp (slower)
- **Temporary files**: Creates intermediate files that must be cleaned up

## Future Improvements for Rust Port

Based on the limitations above, the Rust implementation should address:

### 1. Content Scaling

**Goal**: Automatically scale source pages to fit within safe area (avoiding header/footer zones).

**Approach**:
- Calculate safe area: page size minus header/footer heights
- Scale each source page to fit within safe area
- Center scaled content
- Add headers/footers in reserved space

### 2. Image Support

**Goal**: Accept image files (JPG, PNG, TIFF) in addition to PDFs.

**Approach**:
- Detect file type by extension or magic bytes
- Convert images to pages on-the-fly
- Maintain aspect ratio, center on page
- Support multi-page TIFFs

### 3. Cross-Platform

**Goal**: Remove Windows-specific dependencies.

**Approach**:
- Replace PDFtk with pure Rust PDF library (e.g., `lopdf`, `printpdf`)
- Replace wkhtmltopdf with direct PDF generation or Rust HTML renderer
- Use platform-independent paths
- Single self-contained executable

### 4. Self-Contained

**Goal**: All functionality in single executable, no external tools.

**Approach**:
- Bundle all libraries statically
- No external process spawning
- Embed fonts for consistent rendering

### 5. Better Date Parsing

**Goal**: Maintain flexible date expression language.

**Approach**:
- Port `Get-ReportDate` logic to Rust
- Support all existing formats
- Add more natural language (e.g., "next Monday", "2 weeks from today")

### 6. Configuration Files

**Goal**: Allow saved presets for footer templates.

**Approach**:
- TOML or JSON configuration files
- Save commonly used footer combinations
- Command-line override of config values

### 7. Template System

**Goal**: More flexible header/footer layouts.

**Approach**:
- Template language for custom layouts
- Multiple template presets
- User-defined templates

## Technical Dependencies Summary

| Tool | Purpose | Platform | License | Installation |
|------|---------|----------|---------|--------------|
| PowerShell | Scripting | Windows | Built-in | N/A |
| PDFtk | PDF manipulation | Windows/Mac/Linux | Commercial (~$4) | Manual |
| wkhtmltopdf | HTML→PDF | Windows/Mac/Linux | LGPL | Manual |

## Conclusion

This system effectively solves the problem of creating professional bridge handouts with custom headers and footers. The modular design allows flexibility but at the cost of portability and content overlap issues.

**Key strengths**:
- Flexible footer customization
- Sophisticated date parsing
- Clean HTML/CSS templating
- Reliable PDF merging

**Key weaknesses**:
- Platform-specific (Windows)
- Content may overlap headers/footers
- External tool dependencies
- No image file support

The Rust port should maintain the flexibility while addressing cross-platform concerns and adding content scaling capabilities to prevent overlap issues.
