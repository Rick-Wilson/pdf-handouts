# PDF Handouts

A cross-platform command-line tool for merging PDFs and adding custom headers and footers.

## Installation

```bash
# Build from source
cargo build --release

# The binary will be at target/release/pdf-handouts
```

## Quick Start

```bash
# Merge multiple PDFs and add headers/footers in one step
pdf-handouts build \
  "1. intro.pdf" "2. main.pdf" "3. appendix.pdf" \
  -o handout.pdf \
  --title "Workshop Handout" \
  --footer-left "Acme Corp" \
  --footer-right "Page [page] of [pages]" \
  --date today
```

## Commands

### `build` - Merge and add headers/footers

The most common workflow: merge multiple PDFs and add headers/footers in one step.

```bash
pdf-handouts build [OPTIONS] --output <OUTPUT> <INPUTS>...
```

**Arguments:**
- `<INPUTS>...` - Input PDF files in order

**Options:**
- `-o, --output <OUTPUT>` - Output PDF file path (required)
- `--title <TITLE>` - Title text (centered at top of first page)
- `--footer-left <TEXT>` - Footer left section
- `--footer-center <TEXT>` - Footer center section
- `--footer-right <TEXT>` - Footer right section
- `--date <DATE>` - Date for `[date]` placeholder
- `--font <SPEC>` - Font specification for both header and footer
- `--header-font <SPEC>` - Font specification for header only
- `--footer-font <SPEC>` - Font specification for footer only

**Example:**
```bash
pdf-handouts build \
  "lesson1.pdf" "lesson2.pdf" "exercises.pdf" \
  -o "complete-handout.pdf" \
  --title "Bridge Class Handout" \
  --footer-left "Stoneridge Creek|[font italic]Community Center[/font]" \
  --footer-center "Presented by:|Rick Wilson" \
  --footer-right "Page [page] of [pages]|[date]" \
  --date "next tuesday" \
  --font "14pt #333333" \
  --header-font "24pt #222222"
```

### `merge` - Merge PDFs only

Merge multiple PDFs into one without adding headers/footers.

```bash
pdf-handouts merge [OPTIONS] --output <OUTPUT> <INPUTS>...
```

**Example:**
```bash
pdf-handouts merge file1.pdf file2.pdf file3.pdf -o merged.pdf
```

### `headers` - Add headers/footers to existing PDF

Add headers and footers to an already-merged PDF.

```bash
pdf-handouts headers [OPTIONS] --output <OUTPUT> <INPUT>
```

**Example:**
```bash
pdf-handouts headers merged.pdf -o final.pdf \
  --title "My Document" \
  --footer-right "Page [page]"
```

### `info` - Show PDF information

Display page count and metadata for a PDF file.

```bash
pdf-handouts info <INPUT>
```

**Example:**
```bash
pdf-handouts info document.pdf
# Output:
# File: document.pdf
# Pages: 14
# Title: My Document
# Author: John Doe
```

## Text Formatting

### Placeholders

Use these placeholders in footer text - they're replaced with actual values:

| Placeholder | Description |
|-------------|-------------|
| `[page]` | Current page number |
| `[pages]` | Total page count |
| `[date]` | Formatted date (requires `--date`) |

**Example:**
```bash
--footer-right "Page [page] of [pages]|[date]"
# Output: "Page 3 of 14" and "January 14, 2026"
```

### Line Breaks

Use `|` or `[br]` to create multi-line footers:

```bash
--footer-left "Acme Corp|Engineering Division"
# Creates:
#   Acme Corp
#   Engineering Division
```

### Inline Font Styling

Use `[font]...[/font]` tags for inline styling:

| Tag | Effect |
|-----|--------|
| `[font italic]...[/font]` | Italic text |
| `[font bold]...[/font]` | Bold text |
| `[font bold italic]...[/font]` | Bold italic text |

**Example:**
```bash
--footer-left "Company Name|[font italic]Department[/font]"
```

## Font Specification

The `--font`, `--header-font`, and `--footer-font` options accept a font specification string:

```
[bold] [italic] [size[pt]] [family_name] [#rrggbb]
```

All components are optional. Order doesn't matter.

| Component | Description | Example |
|-----------|-------------|---------|
| `bold` | Bold weight | `bold` |
| `italic` | Italic style | `italic` |
| `size` | Font size in points | `14pt` or `14` |
| `family` | Font family (use underscores for spaces) | `Liberation_Serif` |
| `#rrggbb` | Hex color | `#333333` or `#f00` |

**Examples:**
```bash
--font "14pt"                           # 14pt default font
--font "bold 16pt"                      # Bold 16pt
--font "italic 12pt Liberation_Serif"   # Italic 12pt Liberation Serif
--font "24pt #333333"                   # 24pt dark gray
--font "bold italic 18pt #0000ff"       # Bold italic 18pt blue
```

### Font Hierarchy

- `--font` sets the base font for both header and footer
- `--header-font` overrides `--font` for the header only
- `--footer-font` overrides `--font` for the footer only

```bash
pdf-handouts build input.pdf -o output.pdf \
  --font "14pt #333333" \           # Base: 14pt dark gray
  --header-font "24pt #000000"      # Header: 24pt black (overrides base)
```

## Date Expressions

The `--date` option accepts flexible date expressions:

| Expression | Description |
|------------|-------------|
| `today` | Current date |
| `2026-01-14` | ISO format date |
| `01/14/2026` | US format date |
| `Tuesday` | Next Tuesday (or today if Tuesday) |
| `Tuesday+1` | Tuesday after next |
| `Tuesday+3` | 4th upcoming Tuesday |

**Example:**
```bash
--date "next tuesday"    # Next occurrence of Tuesday
--date "2026-01-14"      # Specific date
--date "today"           # Current date
```

## Complete Example

```bash
# Create a workshop handout from multiple source PDFs
pdf-handouts build \
  "1. NT Ladder - Google Docs.pdf" \
  "2. NT Ladder Practice Sheet.pdf" \
  "3. ABS4-2 Jacoby Transfers Handouts.pdf" \
  "4. thinking-bridge-Responding to 1NT 1-6.pdf" \
  -o "Bridge-Workshop-Handout.pdf" \
  --title "Bridge Class Handout" \
  --footer-left "Stoneridge Creek|[font italic]Community Center[/font]" \
  --footer-center "Presented by:|Rick Wilson" \
  --footer-right "Page [page] of [pages]|[date]" \
  --date "next tuesday" \
  --header-font "24pt #333333" \
  --footer-font "14pt #555555"
```

## Library Usage

This tool is also available as a Rust library. See [LIBRARY.md](LIBRARY.md) for API documentation.

## License

MIT OR Apache-2.0
