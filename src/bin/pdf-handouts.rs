//! PDF Handouts CLI tool
//!
//! A command-line tool for merging PDFs and adding headers/footers.

use clap::{Parser, Subcommand};
use glob::glob;
use std::path::PathBuf;
use std::process;

use pdf_handouts::pdf::{
    merge_pdfs, add_headers_footers,
    MergeOptions, HeaderFooterOptions, FontSpec,
};
use pdf_handouts::date::{parse_date_expression, resolve_date};

/// PDF Handouts - Merge PDFs and add headers/footers
#[derive(Parser)]
#[command(name = "pdf-handouts")]
#[command(author, version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    # Merge PDFs and add a footer
    pdf-handouts build -o output.pdf --footer-center \"Page [page]\" *.pdf

    # Merge numbered PDFs in order
    pdf-handouts build -o handout.pdf \"[0-9]*.pdf\"

    # Add header and footer with date
    pdf-handouts headers input.pdf -o output.pdf --title \"My Document\" --footer-right \"[date]\" --date today

    # Merge and open result
    pdf-handouts build -o output.pdf --open file1.pdf file2.pdf")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Merge multiple PDF files into one
    Merge {
        /// Input PDF files (in order). Supports glob patterns like "*.pdf"
        #[arg(required = true)]
        inputs: Vec<String>,

        /// Output PDF file path
        #[arg(short, long)]
        output: PathBuf,

        /// Open the output file after creation
        #[arg(long)]
        open: bool,
    },

    /// Add headers and footers to a PDF
    Headers {
        /// Input PDF file
        input: PathBuf,

        /// Output PDF file path
        #[arg(short, long)]
        output: PathBuf,

        /// Title text (displayed centered at top of first page)
        #[arg(long)]
        title: Option<String>,

        /// Footer left section (use | or [br] for line breaks)
        #[arg(long)]
        footer_left: Option<String>,

        /// Footer center section (use | or [br] for line breaks)
        #[arg(long)]
        footer_center: Option<String>,

        /// Footer right section (use | or [br] for line breaks)
        #[arg(long)]
        footer_right: Option<String>,

        /// Date for [date] placeholder (e.g., "today", "next tuesday", "2026-01-14")
        #[arg(long)]
        date: Option<String>,

        /// Font specification for both header and footer
        /// Format: "[bold] [italic] [size[pt]] [family] [#rrggbb]"
        /// Example: "14pt Liberation_Serif #333333"
        #[arg(long)]
        font: Option<String>,

        /// Font specification for header only (overrides --font)
        #[arg(long)]
        header_font: Option<String>,

        /// Font specification for footer only (overrides --font)
        #[arg(long)]
        footer_font: Option<String>,

        /// Open the output file after creation
        #[arg(long)]
        open: bool,
    },

    /// Merge PDFs and add headers/footers in one step
    Build {
        /// Input PDF files (in order). Supports glob patterns like "*.pdf"
        #[arg(required = true)]
        inputs: Vec<String>,

        /// Output PDF file path
        #[arg(short, long)]
        output: PathBuf,

        /// Title text (displayed centered at top of first page)
        #[arg(long)]
        title: Option<String>,

        /// Footer left section (use | or [br] for line breaks)
        #[arg(long)]
        footer_left: Option<String>,

        /// Footer center section (use | or [br] for line breaks)
        #[arg(long)]
        footer_center: Option<String>,

        /// Footer right section (use | or [br] for line breaks)
        #[arg(long)]
        footer_right: Option<String>,

        /// Date for [date] placeholder (e.g., "today", "next tuesday", "2026-01-14")
        #[arg(long)]
        date: Option<String>,

        /// Font specification for both header and footer
        /// Format: "[bold] [italic] [size[pt]] [family] [#rrggbb]"
        /// Example: "14pt Liberation_Serif #333333"
        #[arg(long)]
        font: Option<String>,

        /// Font specification for header only (overrides --font)
        #[arg(long)]
        header_font: Option<String>,

        /// Font specification for footer only (overrides --font)
        #[arg(long)]
        footer_font: Option<String>,

        /// Open the output file after creation
        #[arg(long)]
        open: bool,
    },

    /// Show information about a PDF file
    Info {
        /// PDF file to inspect
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Merge { inputs, output, open } => {
            cmd_merge(inputs, output, open)
        }
        Commands::Headers {
            input, output, title, footer_left, footer_center, footer_right,
            date, font, header_font, footer_font, open,
        } => {
            cmd_headers(
                input, output, title, footer_left, footer_center, footer_right,
                date, font, header_font, footer_font, open,
            )
        }
        Commands::Build {
            inputs, output, title, footer_left, footer_center, footer_right,
            date, font, header_font, footer_font, open,
        } => {
            cmd_build(
                inputs, output, title, footer_left, footer_center, footer_right,
                date, font, header_font, footer_font, open,
            )
        }
        Commands::Info { input } => {
            cmd_info(input)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Expand glob patterns in input paths
fn expand_globs(patterns: Vec<String>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();

    for pattern in patterns {
        // Check if pattern contains glob characters
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            let mut matched = false;
            for entry in glob(&pattern)? {
                match entry {
                    Ok(path) => {
                        paths.push(path);
                        matched = true;
                    }
                    Err(e) => eprintln!("Warning: glob error for {}: {}", pattern, e),
                }
            }
            if !matched {
                return Err(format!("No files matched pattern: {}", pattern).into());
            }
        } else {
            // No glob characters, treat as literal path
            paths.push(PathBuf::from(pattern));
        }
    }

    // Sort paths for consistent ordering
    paths.sort();

    Ok(paths)
}

/// Open a file with the system default application
fn open_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()?;
    }
    Ok(())
}

/// Merge multiple PDFs into one
fn cmd_merge(inputs: Vec<String>, output: PathBuf, open: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Expand glob patterns
    let inputs = expand_globs(inputs)?;

    // Validate inputs exist
    for path in &inputs {
        if !path.exists() {
            return Err(format!("Input file not found: {}", path.display()).into());
        }
    }

    eprintln!("Merging {} PDF files...", inputs.len());

    let options = MergeOptions {
        input_paths: inputs,
        output_path: output.clone(),
    };

    merge_pdfs(&options)?;

    eprintln!("Merged to: {}", output.display());

    if open {
        open_file(&output)?;
    }

    Ok(())
}

/// Add headers and footers to a PDF
fn cmd_headers(
    input: PathBuf,
    output: PathBuf,
    title: Option<String>,
    footer_left: Option<String>,
    footer_center: Option<String>,
    footer_right: Option<String>,
    date: Option<String>,
    font: Option<String>,
    header_font: Option<String>,
    footer_font: Option<String>,
    open: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !input.exists() {
        return Err(format!("Input file not found: {}", input.display()).into());
    }

    // Parse date expression
    let resolved_date = date.as_deref()
        .and_then(|d| parse_date_expression(d).ok())
        .and_then(|expr| resolve_date(&expr));

    // Parse font specifications
    let base_font = font.as_deref().map(FontSpec::parse);
    let header_spec = header_font.as_deref().map(FontSpec::parse).or_else(|| base_font.clone());
    let footer_spec = footer_font.as_deref().map(FontSpec::parse).or(base_font);

    // Build options
    let options = HeaderFooterOptions {
        title,
        footer_left,
        footer_center,
        footer_right,
        date: resolved_date,
        show_page_numbers: false,
        show_total_page_count: false,
        title_font_size: header_spec.as_ref().and_then(|f| f.size).unwrap_or(24.0),
        footer_font_size: footer_spec.as_ref().and_then(|f| f.size).unwrap_or(14.0),
        header_font: header_spec,
        footer_font: footer_spec,
    };

    eprintln!("Adding headers/footers...");
    add_headers_footers(&input, &output, &options)?;

    eprintln!("Output: {}", output.display());

    if open {
        open_file(&output)?;
    }

    Ok(())
}

/// Merge PDFs and add headers/footers in one step
fn cmd_build(
    inputs: Vec<String>,
    output: PathBuf,
    title: Option<String>,
    footer_left: Option<String>,
    footer_center: Option<String>,
    footer_right: Option<String>,
    date: Option<String>,
    font: Option<String>,
    header_font: Option<String>,
    footer_font: Option<String>,
    open: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Expand glob patterns
    let inputs = expand_globs(inputs)?;

    // Validate inputs exist
    for path in &inputs {
        if !path.exists() {
            return Err(format!("Input file not found: {}", path.display()).into());
        }
    }

    // Create temporary file for merged PDF
    let temp_dir = std::env::temp_dir();
    let temp_merged = temp_dir.join("pdf-handouts-merged-temp.pdf");

    eprintln!("Step 1: Merging {} PDF files...", inputs.len());

    let merge_options = MergeOptions {
        input_paths: inputs,
        output_path: temp_merged.clone(),
    };

    merge_pdfs(&merge_options)?;

    // Parse date expression
    let resolved_date = date.as_deref()
        .and_then(|d| parse_date_expression(d).ok())
        .and_then(|expr| resolve_date(&expr));

    // Parse font specifications
    let base_font = font.as_deref().map(FontSpec::parse);
    let header_spec = header_font.as_deref().map(FontSpec::parse).or_else(|| base_font.clone());
    let footer_spec = footer_font.as_deref().map(FontSpec::parse).or(base_font);

    // Build options
    let options = HeaderFooterOptions {
        title,
        footer_left,
        footer_center,
        footer_right,
        date: resolved_date,
        show_page_numbers: false,
        show_total_page_count: false,
        title_font_size: header_spec.as_ref().and_then(|f| f.size).unwrap_or(24.0),
        footer_font_size: footer_spec.as_ref().and_then(|f| f.size).unwrap_or(14.0),
        header_font: header_spec,
        footer_font: footer_spec,
    };

    eprintln!("Step 2: Adding headers/footers...");
    add_headers_footers(&temp_merged, &output, &options)?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_merged);

    eprintln!("Output: {}", output.display());

    if open {
        open_file(&output)?;
    }

    Ok(())
}

/// Show information about a PDF
fn cmd_info(input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !input.exists() {
        return Err(format!("Input file not found: {}", input.display()).into());
    }

    let metadata = pdf_handouts::pdf::extract_metadata(&input)?;

    println!("File: {}", input.display());
    println!("Pages: {}", metadata.page_count);

    if let Some(title) = metadata.title {
        println!("Title: {}", title);
    }
    if let Some(author) = metadata.author {
        println!("Author: {}", author);
    }

    Ok(())
}
