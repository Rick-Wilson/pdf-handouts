# PDF Handouts Project - Status and Design

**Last Updated**: January 10, 2026
**Status**: v0.1.0 released - refactoring headers/footers to use XObject overlay approach

## Project Overview

Cross-platform Rust library and CLI tool for merging PDFs and adding headers/footers. Replaces the original PowerShell scripts that depended on PDFtk and wkhtmltopdf.

## Current Release: v0.1.0

GitHub release with binaries for:
- macOS (x86_64, aarch64)
- Linux (x86_64)
- Windows (x86_64)

### What Works

1. **PDF Merging** - Fully functional
2. **CLI Tool** - All commands working (merge, headers, build, info)
3. **Date Parsing** - Flexible expressions ("today", "Tuesday+3", "2026-01-14")
4. **Font Specification** - Color, size, bold/italic support
5. **Headers/Footers** - Works for standard PDFs, issues with Google Docs PDFs

### Known Issue: Google Docs PDFs

Headers/footers don't render correctly on Google Docs PDF exports due to coordinate transformation conflicts. See "Analysis and Solution" below.

## Analysis: Why Old System Worked

### Comparing PDF Structures

Analyzed a working PDF from the old PowerShell/PDFtk system:
- File: `Intermediate Bridge 2025-12-09.pdf`
- Created by: PowerShell + wkhtmltopdf (watermark) + PDFtk (stamp)

### Old System Architecture

The old system uses a **3-stream approach** per page:

```
Stream 1: q                              # Save graphics state
Stream 2: [original page content]        # Google Docs content with transforms
Stream 3: Q                              # Restore state
          q 1 0 0 1 0 0 cm /Xi0 Do Q     # Draw overlay XObject
```

Key insight: The XObject `/Xi0` is drawn **after** restoring graphics state with `Q`. This means:
1. The original page's transformation matrix is completely reset
2. The XObject renders in standard page coordinates
3. No conflict with Google Docs' `.24 0 0 -.24 0 792 cm` transform

### XObject Overlay Details

The watermark XObjects (`/Xi0`, `/Xi1`, etc.) are Form XObjects with:

```
/Type /XObject
/Subtype /Form
/BBox [0 0 612 792]           # Full page size
/Matrix [1 0 0 1 0 0]         # Identity matrix
/Resources << /Font << /F6 ... /F7 ... >> >>
```

Content includes:
- Title text (on first page)
- 3-column footer (left: location/date, center: presenter, right: page numbers)
- Uses embedded fonts referenced by the XObject's own Resources

### Our Current System

Our current implementation appends content directly to pages:

```
[original content streams]
q                              # Our q/Q wrapper
[header/footer content]
Q
```

**Problem**: The `q` saves the current transformation state, but that state **already includes** Google Docs' transform. Our content inherits the `.24 0 0 -.24 0 792 cm` transformation.

## Solution: XObject Overlay Approach

### Architecture

Adopt the same approach as PDFtk's stamp command:

1. **Wrap original content** in `q`/`Q` to isolate its transformations
2. **Create Form XObject** containing headers/footers with own coordinate system
3. **Append XObject invocation** after the `Q` restore

```
q                              # Save state (NEW)
[original content streams]
Q                              # Restore state (NEW)
q 1 0 0 1 0 0 cm /HdrFtr Do Q  # Draw overlay in clean coordinate space
```

### Implementation Steps

1. **For each page being processed**:
   - Wrap existing Contents in `q`/`Q` pair
   - Create a Form XObject for that page's header/footer
   - Add XObject reference to page's Resources
   - Append XObject invocation to Content streams

2. **Form XObject structure**:
   - BBox: `[0 0 612 792]` (Letter) or detected from MediaBox
   - Matrix: `[1 0 0 1 0 0]` (identity)
   - Resources: Reference to embedded font
   - Stream: Header/footer drawing commands

3. **Font handling**:
   - Embed Liberation Serif font in XObject's Resources
   - XObject is self-contained - doesn't depend on page's fonts

### Why This Will Work

- The `Q` before XObject invocation resets CTM to page default
- Form XObjects render in their own coordinate space
- `1 0 0 1 0 0 cm` ensures identity transformation
- Headers/footers draw at absolute page coordinates

## Technology Stack

### Rust Crates

- **lopdf** (0.38) - PDF manipulation
- **chrono** (0.4) - Date parsing
- **thiserror** (1.0) - Error handling
- **clap** (4.5) - CLI framework

### Embedded Font

Liberation Serif Regular (SIL Open Font License)
- Embedded as CIDFont Type2 with TrueType outlines
- Subset possible for smaller file size

## File Structure

```
pdf-handouts/
├── src/
│   ├── lib.rs
│   ├── pdf/
│   │   ├── mod.rs
│   │   ├── merge.rs        # PDF merging (working)
│   │   ├── metadata.rs     # Page counting (working)
│   │   └── headers.rs      # Headers/footers (refactoring)
│   ├── date.rs             # Date parsing (working)
│   ├── layout.rs           # Layout calculations (working)
│   ├── error.rs            # Error types (working)
│   └── bin/
│       └── pdf-handouts.rs # CLI tool (working)
├── .github/
│   └── workflows/
│       └── release.yml     # Multi-platform builds
├── examples/
│   ├── demo_direct_headers.rs
│   ├── compare_old_new.rs      # PDF structure comparison
│   └── inspect_xobject_detail.rs
├── README.md               # CLI documentation
├── LIBRARY.md              # API documentation
└── BUILD-INFO.md           # Development info
```

## Next Steps

1. **Implement XObject overlay approach** in `headers.rs`
   - Modify `add_headers_footers()` to wrap content in q/Q
   - Create Form XObject per page
   - Append XObject invocation

2. **Test with Google Docs PDFs**
   - Verify headers/footers render correctly
   - Check all 4 test files

3. **Release v0.2.0** with fix

## Test Files

Located in `tests/fixtures/real-world/`:
- `1. NT Ladder - Google Docs.pdf`
- `2. NT Ladder Practice Sheet.pdf`
- `3. ABS4-2 Jacoby Transfers Handouts.pdf`
- `4. thinking-bridge-Responding to 1NT 1-6.pdf`

Reference working PDF:
- `/Users/rick/Documents/Bridge/Stoneridge Creek/Lesson Plans/2025/2025-12 Dec/2025-12-09/Handouts/Intermediate Bridge 2025-12-09.pdf`

## Running Examples

```bash
# Main demo
cargo run --example demo_direct_headers

# Compare PDF structures
cargo run --example compare_old_new -- /path/to/pdf

# Inspect XObject overlays
cargo run --example inspect_xobject_detail -- /path/to/pdf
```

## Git History

- **v0.1.0** - Initial release with CLI, working for standard PDFs
- **main** - Current development branch
