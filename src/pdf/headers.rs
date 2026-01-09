//! Direct header/footer writing to PDF pages using lopdf
//!
//! This module provides functionality to add headers and footers directly to PDF pages
//! without creating a separate watermark overlay file. This approach is simpler and more
//! reliable than the overlay method.

use std::path::Path;
use lopdf::{Document, Object, ObjectId, Dictionary, Stream};
use chrono::NaiveDate;
use crate::error::Result;
use crate::date::format_date;

/// Represents a PDF transformation matrix [a b c d e f]
/// where: x' = a*x + c*y + e, y' = b*x + d*y + f
#[derive(Debug, Clone)]
struct TransformMatrix {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl TransformMatrix {
    /// Identity matrix (no transformation)
    fn identity() -> Self {
        Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
    }

    /// Calculate the inverse of this transformation matrix
    fn inverse(&self) -> Self {
        // For a 2D affine transformation matrix:
        // | a  c  e |
        // | b  d  f |
        // | 0  0  1 |
        //
        // The determinant is: det = a*d - b*c
        // The inverse is:
        // | d/det   -c/det   (c*f - d*e)/det |
        // | -b/det   a/det   (b*e - a*f)/det |
        // |   0       0            1         |

        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-10 {
            // Singular matrix, return identity
            return Self::identity();
        }

        Self {
            a: self.d / det,
            b: -self.b / det,
            c: -self.c / det,
            d: self.a / det,
            e: (self.c * self.f - self.d * self.e) / det,
            f: (self.b * self.e - self.a * self.f) / det,
        }
    }

    /// Check if this is (approximately) the identity matrix
    fn is_identity(&self) -> bool {
        (self.a - 1.0).abs() < 0.001 &&
        self.b.abs() < 0.001 &&
        self.c.abs() < 0.001 &&
        (self.d - 1.0).abs() < 0.001 &&
        self.e.abs() < 0.001 &&
        self.f.abs() < 0.001
    }
}

/// Options for adding headers and footers to a PDF
#[derive(Debug, Clone)]
pub struct HeaderFooterOptions {
    /// Title to display on first page (centered at top)
    pub title: Option<String>,
    /// Footer left section content
    pub footer_left: Option<String>,
    /// Footer center section content
    pub footer_center: Option<String>,
    /// Footer right section content
    pub footer_right: Option<String>,
    /// Date to display in footer
    pub date: Option<NaiveDate>,
    /// Whether to show page numbers
    pub show_page_numbers: bool,
    /// Whether to show total page count (e.g., "Page 1 of 10")
    pub show_total_page_count: bool,
    /// Title font size in points
    pub title_font_size: f32,
    /// Footer font size in points
    pub footer_font_size: f32,
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
        }
    }
}

/// Add headers and footers directly to a PDF
///
/// This function loads a PDF, embeds a font, and adds header/footer content streams
/// directly to each page. This is simpler than creating a separate watermark PDF.
///
/// # Example
///
/// ```no_run
/// use pdf_handouts::pdf::{HeaderFooterOptions, add_headers_footers};
/// use std::path::Path;
/// use chrono::NaiveDate;
///
/// let options = HeaderFooterOptions {
///     title: Some("My Document".to_string()),
///     footer_center: Some("Page".to_string()),
///     show_page_numbers: true,
///     ..Default::default()
/// };
///
/// add_headers_footers(
///     Path::new("input.pdf"),
///     Path::new("output.pdf"),
///     &options
/// ).expect("Failed to add headers/footers");
/// ```
pub fn add_headers_footers(
    input_path: &Path,
    output_path: &Path,
    options: &HeaderFooterOptions,
) -> Result<()> {
    // Load the PDF
    let mut doc = Document::load(input_path)?;

    // Decompress for easier content stream parsing
    doc.decompress();

    let page_count = doc.get_pages().len();

    // Embed Liberation Serif font for proper text rendering
    let font_id = embed_liberation_serif(&mut doc)?;

    // Collect page info first (to avoid borrow issues)
    let pages: Vec<(usize, ObjectId)> = doc.get_pages()
        .iter()
        .enumerate()
        .map(|(i, (_num, id))| (i, *id))
        .collect();

    // For each page, detect transformation and create appropriate Form XObject
    for (i, page_id) in pages.iter() {
        let page_number = i + 1;

        // Detect the page's transformation matrix from its content stream
        let transform = detect_page_transformation(&doc, *page_id)?;

        // Generate the content stream for this page's headers/footers
        let content = generate_header_footer_content(
            page_number,
            page_count,
            page_number == 1, // is_first_page
            options,
        );

        // Create a Form XObject with the inverse transformation for this specific page
        let xobject_id = create_form_xobject_with_transform(&mut doc, content, font_id, &transform)?;

        // Add the Form XObject to the page's Resources
        add_xobject_to_page_resources(&mut doc, *page_id, xobject_id)?;

        // Add content stream that invokes the Form XObject
        let invoke_content = format!("q\n/HeaderFooter Do\nQ\n");
        let content_stream_id = doc.add_object(Stream::new(
            Dictionary::new(),
            invoke_content.into_bytes(),
        ));

        // Append the content stream to the page's Contents
        append_content_to_page(&mut doc, *page_id, content_stream_id)?;
    }

    // Save the modified PDF
    doc.compress();
    doc.save(output_path)?;

    Ok(())
}

/// Use Helvetica (standard PDF font - simpler than embedding)
#[allow(dead_code)]
fn use_helvetica_font(doc: &mut Document) -> Result<ObjectId> {
    // Create a simple Type1 font using Helvetica (one of the 14 standard PDF fonts)
    let mut font = Dictionary::new();
    font.set("Type", Object::Name(b"Font".to_vec()));
    font.set("Subtype", Object::Name(b"Type1".to_vec()));
    font.set("BaseFont", Object::Name(b"Helvetica".to_vec()));

    let font_id = doc.add_object(Object::Dictionary(font));
    Ok(font_id)
}

/// Embed Liberation Serif TrueType font with WinAnsiEncoding
///
/// This embeds the font data directly in the PDF so it renders correctly
/// on any system, regardless of whether the font is installed.
fn embed_liberation_serif(doc: &mut Document) -> Result<ObjectId> {
    // Load the embedded font data
    const LIBERATION_SERIF: &[u8] = include_bytes!("../../assets/fonts/LiberationSerif-Regular.ttf");

    // Create font stream object (the actual TTF data)
    let mut font_stream_dict = Dictionary::new();
    font_stream_dict.set("Length1", Object::Integer(LIBERATION_SERIF.len() as i64));

    let font_stream = Stream {
        dict: font_stream_dict,
        content: LIBERATION_SERIF.to_vec(),
        allows_compression: true,
        start_position: None,
    };
    let font_stream_id = doc.add_object(Object::Stream(font_stream));

    // Create font descriptor with metrics for Liberation Serif
    let mut font_descriptor = Dictionary::new();
    font_descriptor.set("Type", Object::Name(b"FontDescriptor".to_vec()));
    font_descriptor.set("FontName", Object::Name(b"LiberationSerif".to_vec()));
    font_descriptor.set("FontFamily", Object::String(b"Liberation Serif".to_vec(), lopdf::StringFormat::Literal));
    font_descriptor.set("Flags", Object::Integer(34)); // Serif + Nonsymbolic
    font_descriptor.set("FontBBox", Object::Array(vec![
        Object::Integer(-543),
        Object::Integer(-303),
        Object::Integer(1300),
        Object::Integer(981),
    ]));
    font_descriptor.set("ItalicAngle", Object::Integer(0));
    font_descriptor.set("Ascent", Object::Integer(891));
    font_descriptor.set("Descent", Object::Integer(-216));
    font_descriptor.set("CapHeight", Object::Integer(662));
    font_descriptor.set("XHeight", Object::Integer(450));
    font_descriptor.set("StemV", Object::Integer(84));
    font_descriptor.set("FontFile2", Object::Reference(font_stream_id));

    let font_descriptor_id = doc.add_object(Object::Dictionary(font_descriptor));

    // Create TrueType font with WinAnsiEncoding
    // WinAnsiEncoding allows us to use simple single-byte text strings
    let mut font = Dictionary::new();
    font.set("Type", Object::Name(b"Font".to_vec()));
    font.set("Subtype", Object::Name(b"TrueType".to_vec()));
    font.set("BaseFont", Object::Name(b"LiberationSerif".to_vec()));
    font.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
    font.set("FontDescriptor", Object::Reference(font_descriptor_id));

    // First and last character codes (standard ASCII printable range)
    font.set("FirstChar", Object::Integer(32));
    font.set("LastChar", Object::Integer(255));

    // Widths array - Liberation Serif glyph widths for chars 32-255
    // These are approximate widths in 1/1000ths of the font size
    let widths = create_liberation_serif_widths();
    font.set("Widths", Object::Array(widths));

    let font_id = doc.add_object(Object::Dictionary(font));
    Ok(font_id)
}

/// Create widths array for Liberation Serif (chars 32-255)
/// Values are in 1/1000ths of the em square
fn create_liberation_serif_widths() -> Vec<Object> {
    // Liberation Serif widths for WinAnsiEncoding characters 32-255
    // These are approximate values based on typical serif font metrics
    let widths: Vec<i64> = vec![
        250,  // 32 space
        333,  // 33 !
        408,  // 34 "
        500,  // 35 #
        500,  // 36 $
        833,  // 37 %
        778,  // 38 &
        180,  // 39 '
        333,  // 40 (
        333,  // 41 )
        500,  // 42 *
        564,  // 43 +
        250,  // 44 ,
        333,  // 45 -
        250,  // 46 .
        278,  // 47 /
        500,  // 48 0
        500,  // 49 1
        500,  // 50 2
        500,  // 51 3
        500,  // 52 4
        500,  // 53 5
        500,  // 54 6
        500,  // 55 7
        500,  // 56 8
        500,  // 57 9
        278,  // 58 :
        278,  // 59 ;
        564,  // 60 <
        564,  // 61 =
        564,  // 62 >
        444,  // 63 ?
        921,  // 64 @
        722,  // 65 A
        667,  // 66 B
        667,  // 67 C
        722,  // 68 D
        611,  // 69 E
        556,  // 70 F
        722,  // 71 G
        722,  // 72 H
        333,  // 73 I
        389,  // 74 J
        722,  // 75 K
        611,  // 76 L
        889,  // 77 M
        722,  // 78 N
        722,  // 79 O
        556,  // 80 P
        722,  // 81 Q
        667,  // 82 R
        556,  // 83 S
        611,  // 84 T
        722,  // 85 U
        722,  // 86 V
        944,  // 87 W
        722,  // 88 X
        722,  // 89 Y
        611,  // 90 Z
        333,  // 91 [
        278,  // 92 \
        333,  // 93 ]
        469,  // 94 ^
        500,  // 95 _
        333,  // 96 `
        444,  // 97 a
        500,  // 98 b
        444,  // 99 c
        500,  // 100 d
        444,  // 101 e
        333,  // 102 f
        500,  // 103 g
        500,  // 104 h
        278,  // 105 i
        278,  // 106 j
        500,  // 107 k
        278,  // 108 l
        778,  // 109 m
        500,  // 110 n
        500,  // 111 o
        500,  // 112 p
        500,  // 113 q
        333,  // 114 r
        389,  // 115 s
        278,  // 116 t
        500,  // 117 u
        500,  // 118 v
        722,  // 119 w
        500,  // 120 x
        500,  // 121 y
        444,  // 122 z
        480,  // 123 {
        200,  // 124 |
        480,  // 125 }
        541,  // 126 ~
        350,  // 127 DEL (placeholder)
        500,  // 128 Euro (placeholder)
        350,  // 129 undefined
        333,  // 130 single low quote
        500,  // 131 f with hook
        444,  // 132 double low quote
        1000, // 133 ellipsis
        500,  // 134 dagger
        500,  // 135 double dagger
        333,  // 136 circumflex
        1000, // 137 per mille
        556,  // 138 S caron
        333,  // 139 single left angle quote
        889,  // 140 OE
        350,  // 141 undefined
        611,  // 142 Z caron
        350,  // 143 undefined
        350,  // 144 undefined
        333,  // 145 left single quote
        333,  // 146 right single quote
        444,  // 147 left double quote
        444,  // 148 right double quote
        350,  // 149 bullet
        500,  // 150 en dash
        1000, // 151 em dash
        333,  // 152 tilde
        980,  // 153 trademark
        389,  // 154 s caron
        333,  // 155 single right angle quote
        722,  // 156 oe
        350,  // 157 undefined
        444,  // 158 z caron
        722,  // 159 Y dieresis
        250,  // 160 non-breaking space
        333,  // 161 inverted !
        500,  // 162 cent
        500,  // 163 pound
        500,  // 164 currency
        500,  // 165 yen
        200,  // 166 broken bar
        500,  // 167 section
        333,  // 168 dieresis
        760,  // 169 copyright
        276,  // 170 feminine ordinal
        500,  // 171 left guillemet
        564,  // 172 not
        333,  // 173 soft hyphen
        760,  // 174 registered
        333,  // 175 macron
        400,  // 176 degree
        564,  // 177 plus minus
        300,  // 178 superscript 2
        300,  // 179 superscript 3
        333,  // 180 acute
        500,  // 181 mu
        453,  // 182 pilcrow
        250,  // 183 middle dot
        333,  // 184 cedilla
        300,  // 185 superscript 1
        310,  // 186 masculine ordinal
        500,  // 187 right guillemet
        750,  // 188 1/4
        750,  // 189 1/2
        750,  // 190 3/4
        444,  // 191 inverted ?
        722,  // 192 A grave
        722,  // 193 A acute
        722,  // 194 A circumflex
        722,  // 195 A tilde
        722,  // 196 A dieresis
        722,  // 197 A ring
        889,  // 198 AE
        667,  // 199 C cedilla
        611,  // 200 E grave
        611,  // 201 E acute
        611,  // 202 E circumflex
        611,  // 203 E dieresis
        333,  // 204 I grave
        333,  // 205 I acute
        333,  // 206 I circumflex
        333,  // 207 I dieresis
        722,  // 208 Eth
        722,  // 209 N tilde
        722,  // 210 O grave
        722,  // 211 O acute
        722,  // 212 O circumflex
        722,  // 213 O tilde
        722,  // 214 O dieresis
        564,  // 215 multiply
        722,  // 216 O stroke
        722,  // 217 U grave
        722,  // 218 U acute
        722,  // 219 U circumflex
        722,  // 220 U dieresis
        722,  // 221 Y acute
        556,  // 222 Thorn
        500,  // 223 sharp s
        444,  // 224 a grave
        444,  // 225 a acute
        444,  // 226 a circumflex
        444,  // 227 a tilde
        444,  // 228 a dieresis
        444,  // 229 a ring
        667,  // 230 ae
        444,  // 231 c cedilla
        444,  // 232 e grave
        444,  // 233 e acute
        444,  // 234 e circumflex
        444,  // 235 e dieresis
        278,  // 236 i grave
        278,  // 237 i acute
        278,  // 238 i circumflex
        278,  // 239 i dieresis
        500,  // 240 eth
        500,  // 241 n tilde
        500,  // 242 o grave
        500,  // 243 o acute
        500,  // 244 o circumflex
        500,  // 245 o tilde
        500,  // 246 o dieresis
        564,  // 247 divide
        500,  // 248 o stroke
        500,  // 249 u grave
        500,  // 250 u acute
        500,  // 251 u circumflex
        500,  // 252 u dieresis
        500,  // 253 y acute
        500,  // 254 thorn
        500,  // 255 y dieresis
    ];

    widths.into_iter().map(Object::Integer).collect()
}

/// Detect the transformation matrix applied at the start of a page's content stream
///
/// PDF content streams may start with a `cm` operator that transforms the coordinate system.
/// This function parses the beginning of the content stream to find such a transformation.
/// If no transformation is found, returns identity matrix.
fn detect_page_transformation(doc: &Document, page_id: ObjectId) -> Result<TransformMatrix> {
    let page_obj = doc.get_object(page_id)?;

    if let Object::Dictionary(page_dict) = page_obj {
        if let Ok(contents) = page_dict.get(b"Contents") {
            // Get content stream ID(s)
            let content_ids: Vec<ObjectId> = match contents {
                Object::Reference(id) => vec![*id],
                Object::Array(arr) => arr.iter().filter_map(|o| {
                    if let Object::Reference(id) = o { Some(*id) } else { None }
                }).collect(),
                _ => vec![],
            };

            // Check the first content stream for a transformation
            if let Some(content_id) = content_ids.first() {
                if let Ok(Object::Stream(stream)) = doc.get_object(*content_id) {
                    let content_str = String::from_utf8_lossy(&stream.content);
                    return Ok(parse_initial_transformation(&content_str));
                }
            }
        }
    }

    Ok(TransformMatrix::identity())
}

/// Parse the initial transformation matrix from a content stream
///
/// Looks for patterns like:
/// - `.24 0 0 -.24 0 792 cm` (Google Docs - NOT wrapped in q/Q)
/// - `0.75 0 0 -0.75 0 792 cm` (some PDFs - NOT wrapped)
/// - `q ... 0.12 0 0 0.12 0 0 cm` (wrapped in q - ignored, returns identity)
///
/// The key distinction is whether the transform is wrapped in q/Q or not.
/// If wrapped, the graphics state is restored before our appended content runs.
/// If NOT wrapped, the transform persists and we need to counteract it.
fn parse_initial_transformation(content: &str) -> TransformMatrix {
    // Look for 'cm' operator with 6 numbers before it
    // The pattern is: num num num num num num cm

    let content = content.trim();

    // Find the first 'cm' operator
    if let Some(cm_pos) = content.find(" cm") {
        // Get the text before 'cm'
        let before_cm = &content[..cm_pos];

        // Check if there's a 'q' (graphics state save) before this cm
        // If the content starts with 'q' or has 'q' before the matrix numbers,
        // then the transform is wrapped and will be restored by a matching Q
        let parts: Vec<&str> = before_cm.split_whitespace().collect();

        // Check if any of the parts before the 6 matrix numbers is 'q'
        if parts.len() >= 6 {
            let start = parts.len() - 6;

            // Check if there's a 'q' in the parts before the matrix
            let has_q_before = parts[..start].iter().any(|&p| p == "q");

            // Also check if the very beginning is 'q' (common pattern)
            let starts_with_q = content.starts_with("q ");

            if has_q_before || starts_with_q {
                // Transform is wrapped in graphics state save/restore
                // The CTM will be restored to identity (or previous state) before
                // our appended content runs, so we don't need to counteract it
                return TransformMatrix::identity();
            }

            // Parse the matrix numbers
            let nums: Vec<f32> = parts[start..]
                .iter()
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();

            if nums.len() == 6 {
                return TransformMatrix {
                    a: nums[0],
                    b: nums[1],
                    c: nums[2],
                    d: nums[3],
                    e: nums[4],
                    f: nums[5],
                };
            }
        }
    }

    // No transformation found, return identity
    TransformMatrix::identity()
}

/// Embed Liberation Serif font into the PDF and return its object ID (UNUSED - keeping for reference)
#[allow(dead_code)]
fn embed_liberation_serif_old(doc: &mut Document) -> Result<ObjectId> {
    // Load the embedded font data
    const LIBERATION_SERIF: &[u8] = include_bytes!("../../assets/fonts/LiberationSerif-Regular.ttf");

    // Create font stream object
    let font_stream_id = doc.add_object(Stream::new(
        Dictionary::from_iter(vec![
            ("Length1", Object::Integer(LIBERATION_SERIF.len() as i64)),
        ]),
        LIBERATION_SERIF.to_vec(),
    ));

    // Create font descriptor
    let mut font_descriptor = Dictionary::new();
    font_descriptor.set("Type", Object::Name(b"FontDescriptor".to_vec()));
    font_descriptor.set("FontName", Object::Name(b"LiberationSerif".to_vec()));
    font_descriptor.set("Flags", Object::Integer(32)); // Symbolic
    font_descriptor.set("FontBBox", Object::Array(vec![
        Object::Integer(-543),
        Object::Integer(-303),
        Object::Integer(1277),
        Object::Integer(981),
    ]));
    font_descriptor.set("ItalicAngle", Object::Integer(0));
    font_descriptor.set("Ascent", Object::Integer(891));
    font_descriptor.set("Descent", Object::Integer(-216));
    font_descriptor.set("CapHeight", Object::Integer(981));
    font_descriptor.set("StemV", Object::Integer(80));
    font_descriptor.set("FontFile2", Object::Reference(font_stream_id));

    let font_descriptor_id = doc.add_object(Object::Dictionary(font_descriptor));

    // Create CIDFont dictionary
    let mut cid_font = Dictionary::new();
    cid_font.set("Type", Object::Name(b"Font".to_vec()));
    cid_font.set("Subtype", Object::Name(b"CIDFontType2".to_vec()));
    cid_font.set("BaseFont", Object::Name(b"LiberationSerif".to_vec()));
    cid_font.set("CIDSystemInfo", Object::Dictionary(Dictionary::from_iter(vec![
        ("Registry", Object::String(b"Adobe".to_vec(), lopdf::StringFormat::Literal)),
        ("Ordering", Object::String(b"Identity".to_vec(), lopdf::StringFormat::Literal)),
        ("Supplement", Object::Integer(0)),
    ])));
    cid_font.set("FontDescriptor", Object::Reference(font_descriptor_id));
    cid_font.set("DW", Object::Integer(1000)); // Default width

    let cid_font_id = doc.add_object(Object::Dictionary(cid_font));

    // Create a basic ToUnicode CMap for ASCII characters
    let to_unicode_cmap = create_basic_tounicode_cmap();
    let to_unicode_id = doc.add_object(Stream::new(
        Dictionary::new(),
        to_unicode_cmap.into_bytes(),
    ));

    // Create Type0 font dictionary
    let mut font = Dictionary::new();
    font.set("Type", Object::Name(b"Font".to_vec()));
    font.set("Subtype", Object::Name(b"Type0".to_vec()));
    font.set("BaseFont", Object::Name(b"LiberationSerif".to_vec()));
    font.set("Encoding", Object::Name(b"Identity-H".to_vec()));
    font.set("DescendantFonts", Object::Array(vec![Object::Reference(cid_font_id)]));
    font.set("ToUnicode", Object::Reference(to_unicode_id));

    let font_id = doc.add_object(Object::Dictionary(font));

    Ok(font_id)
}

/// Create a basic ToUnicode CMap for ASCII characters (32-126)
fn create_basic_tounicode_cmap() -> String {
    r#"/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
/CIDSystemInfo
<< /Registry (Adobe)
/Ordering (UCS)
/Supplement 0
>> def
/CMapName /Adobe-Identity-UCS def
/CMapType 2 def
1 begincodespacerange
<0000> <FFFF>
endcodespacerange
95 beginbfchar
<0020> <0020>
<0021> <0021>
<0022> <0022>
<0023> <0023>
<0024> <0024>
<0025> <0025>
<0026> <0026>
<0027> <0027>
<0028> <0028>
<0029> <0029>
<002A> <002A>
<002B> <002B>
<002C> <002C>
<002D> <002D>
<002E> <002E>
<002F> <002F>
<0030> <0030>
<0031> <0031>
<0032> <0032>
<0033> <0033>
<0034> <0034>
<0035> <0035>
<0036> <0036>
<0037> <0037>
<0038> <0038>
<0039> <0039>
<003A> <003A>
<003B> <003B>
<003C> <003C>
<003D> <003D>
<003E> <003E>
<003F> <003F>
<0040> <0040>
<0041> <0041>
<0042> <0042>
<0043> <0043>
<0044> <0044>
<0045> <0045>
<0046> <0046>
<0047> <0047>
<0048> <0048>
<0049> <0049>
<004A> <004A>
<004B> <004B>
<004C> <004C>
<004D> <004D>
<004E> <004E>
<004F> <004F>
<0050> <0050>
<0051> <0051>
<0052> <0052>
<0053> <0053>
<0054> <0054>
<0055> <0055>
<0056> <0056>
<0057> <0057>
<0058> <0058>
<0059> <0059>
<005A> <005A>
<005B> <005B>
<005C> <005C>
<005D> <005D>
<005E> <005E>
<005F> <005F>
<0060> <0060>
<0061> <0061>
<0062> <0062>
<0063> <0063>
<0064> <0064>
<0065> <0065>
<0066> <0066>
<0067> <0067>
<0068> <0068>
<0069> <0069>
<006A> <006A>
<006B> <006B>
<006C> <006C>
<006D> <006D>
<006E> <006E>
<006F> <006F>
<0070> <0070>
<0071> <0071>
<0072> <0072>
<0073> <0073>
<0074> <0074>
<0075> <0075>
<0076> <0076>
<0077> <0077>
<0078> <0078>
<0079> <0079>
<007A> <007A>
<007B> <007B>
<007C> <007C>
<007D> <007D>
<007E> <007E>
endbfchar
endcmap
CMapName currentdict /CMap defineresource pop
end
end
"#.to_string()
}

/// Generate PDF content stream operators for headers/footers
fn generate_header_footer_content(
    page_num: usize,
    total_pages: usize,
    is_first_page: bool,
    options: &HeaderFooterOptions,
) -> String {
    let mut content = String::new();

    // Page dimensions (US Letter: 612pt × 792pt)
    let page_width = 612.0;
    let page_height = 792.0;

    // Form XObjects have their own coordinate system, so we don't need to worry
    // about transformations from the page content. Just set up our graphics state.
    content.push_str("0 g\n"); // gray fill color (0 = black)

    // Add title on first page
    if is_first_page {
        if let Some(ref title) = options.title {
            // Position title 50pt from top of page (PDF coordinates: bottom-left origin)
            let title_y = page_height - 50.0;
            let title_width = estimate_text_width(title, options.title_font_size);
            let title_x = (page_width - title_width) / 2.0; // Center

            content.push_str("BT\n");
            content.push_str("0 Tr\n"); // Fill text
            content.push_str(&format!("/F1 {} Tf\n", options.title_font_size));
            content.push_str(&format!("1 0 0 1 {} {} Tm\n", title_x, title_y));
            content.push_str(&format!("({}) Tj\n", escape_pdf_string(title)));
            content.push_str("ET\n");
        }
    }

    // Add footers
    // We position footer lines starting from the bottom of the page, with the
    // first line at the top of the footer area and subsequent lines below it.
    let line_height = options.footer_font_size * 1.2;

    // Footer left
    if let Some(ref left_text) = options.footer_left {
        let lines = parse_multiline_text(left_text);
        let num_lines = lines.len();
        // Calculate top of footer area: start high enough to fit all lines above the margin
        let footer_top = 30.0 + ((num_lines - 1) as f32 * line_height);
        for (i, line) in lines.iter().enumerate() {
            // First line at top, subsequent lines below (Y decreases)
            let y = footer_top - (i as f32 * line_height);
            content.push_str("BT\n");
            content.push_str(&format!("/F1 {} Tf\n", options.footer_font_size));
            content.push_str(&format!("1 0 0 1 50 {} Tm\n", y));
            content.push_str(&format!("({}) Tj\n", escape_pdf_string(line)));
            content.push_str("ET\n");
        }
    }

    // Footer center
    if let Some(ref center_text) = options.footer_center {
        let lines = parse_multiline_text(center_text);
        let num_lines = lines.len();
        let footer_top = 30.0 + ((num_lines - 1) as f32 * line_height);
        for (i, line) in lines.iter().enumerate() {
            let y = footer_top - (i as f32 * line_height);
            let text_width = estimate_text_width(line, options.footer_font_size);
            let x = (page_width - text_width) / 2.0;

            content.push_str("BT\n");
            content.push_str(&format!("/F1 {} Tf\n", options.footer_font_size));
            content.push_str(&format!("1 0 0 1 {} {} Tm\n", x, y));
            content.push_str(&format!("({}) Tj\n", escape_pdf_string(line)));
            content.push_str("ET\n");
        }
    }

    // Footer right (page numbers and date)
    let mut right_parts = Vec::new();

    if let Some(ref right_text) = options.footer_right {
        right_parts.push(right_text.clone());
    }

    if options.show_page_numbers {
        let page_text = if options.show_total_page_count {
            format!("Page {} of {}", page_num, total_pages)
        } else {
            format!("Page {}", page_num)
        };
        right_parts.push(page_text);
    }

    if let Some(ref date) = options.date {
        right_parts.push(format_date(date));
    }

    let num_lines = right_parts.len();
    let footer_top = 30.0 + ((num_lines.saturating_sub(1)) as f32 * line_height);
    for (i, line) in right_parts.iter().enumerate() {
        let y = footer_top - (i as f32 * line_height);
        let text_width = estimate_text_width(line, options.footer_font_size);
        let x = page_width - 50.0 - text_width; // Right-aligned with margin

        content.push_str("BT\n");
        content.push_str(&format!("/F1 {} Tf\n", options.footer_font_size));
        content.push_str(&format!("1 0 0 1 {} {} Tm\n", x, y));
        content.push_str(&format!("({}) Tj\n", escape_pdf_string(line)));
        content.push_str("ET\n");
    }

    content
}

/// Parse text with line break markers
fn parse_multiline_text(text: &str) -> Vec<String> {
    text.split('\n')
        .flat_map(|line| line.split('|'))
        .flat_map(|part| part.split("[br]"))
        .flat_map(|part| part.split("[BR]"))
        .flat_map(|part| part.split("<br>"))
        .flat_map(|part| part.split("<BR>"))
        .flat_map(|part| part.split("<br/>"))
        .flat_map(|part| part.split("<BR/>"))
        .flat_map(|part| part.split("<br />"))
        .flat_map(|part| part.split("<BR />"))
        .map(|s| s.to_string())
        .collect()
}

/// Escape special characters in PDF strings
fn escape_pdf_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

/// Estimate text width for Liberation Serif
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    // Use average character width from the widths table
    // Average width is approximately 480/1000 = 0.48 em
    text.len() as f32 * font_size * 0.48
}

/// Create a Form XObject with coordinate system adjusted for the detected page transformation
fn create_form_xobject_with_transform(
    doc: &mut Document,
    content: String,
    font_id: ObjectId,
    page_transform: &TransformMatrix,
) -> Result<ObjectId> {
    // Create Resources dictionary for the Form XObject
    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_id));
    resources.set("Font", Object::Dictionary(fonts));

    // Create the Form XObject dictionary
    let mut xobject_dict = Dictionary::new();
    xobject_dict.set("Type", Object::Name(b"XObject".to_vec()));
    xobject_dict.set("Subtype", Object::Name(b"Form".to_vec()));
    xobject_dict.set("FormType", Object::Integer(1));

    // BBox defines the Form's coordinate system - use standard Letter size
    // The Form content uses coordinates within this bounding box
    xobject_dict.set("BBox", Object::Array(vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Integer(612),
        Object::Integer(792),
    ]));

    // Calculate the inverse transformation to counteract the page's CTM
    // When our Form XObject is invoked via Do, the page's transformation is in effect.
    // We apply the inverse so our content renders at the correct position.
    let inverse = page_transform.inverse();

    // Only apply a non-identity matrix if the page has a transformation
    if !page_transform.is_identity() {
        xobject_dict.set("Matrix", Object::Array(vec![
            Object::Real(inverse.a),
            Object::Real(inverse.b),
            Object::Real(inverse.c),
            Object::Real(inverse.d),
            Object::Real(inverse.e),
            Object::Real(inverse.f),
        ]));
    } else {
        // Identity matrix for pages without transformation
        xobject_dict.set("Matrix", Object::Array(vec![
            Object::Integer(1),
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(1),
            Object::Integer(0),
            Object::Integer(0),
        ]));
    }

    xobject_dict.set("Resources", Object::Dictionary(resources));

    // Create the stream with the content
    let xobject_stream = Stream {
        dict: xobject_dict,
        content: content.into_bytes(),
        allows_compression: true,
        start_position: None,
    };

    let xobject_id = doc.add_object(Object::Stream(xobject_stream));
    Ok(xobject_id)
}

/// Add XObject reference to page's Resources dictionary
fn add_xobject_to_page_resources(doc: &mut Document, page_id: ObjectId, xobject_id: ObjectId) -> Result<()> {
    // First, get the resources dictionary (may need to dereference)
    let resources_dict = {
        let page_obj = doc.get_object(page_id)?;
        if let Object::Dictionary(page_dict) = page_obj {
            if let Ok(res) = page_dict.get(b"Resources") {
                match res {
                    Object::Dictionary(dict) => dict.clone(),
                    Object::Reference(res_id) => {
                        // Dereference to get the actual Resources dictionary
                        if let Ok(Object::Dictionary(dict)) = doc.get_object(*res_id) {
                            dict.clone()
                        } else {
                            Dictionary::new()
                        }
                    }
                    _ => Dictionary::new(),
                }
            } else {
                Dictionary::new()
            }
        } else {
            Dictionary::new()
        }
    };

    // Now modify the page with the updated resources
    let page_obj = doc.get_object_mut(page_id)?;

    if let Object::Dictionary(ref mut page_dict) = page_obj {
        let mut new_resources = resources_dict;

        // Get or create XObject subdictionary
        let mut xobjects = if let Ok(Object::Dictionary(xo)) = new_resources.get(b"XObject") {
            xo.clone()
        } else {
            Dictionary::new()
        };

        // Add our Form XObject as /HeaderFooter
        xobjects.set("HeaderFooter", Object::Reference(xobject_id));

        new_resources.set("XObject", Object::Dictionary(xobjects));

        // Set the Resources directly on the page (not as a reference)
        // This ensures the page has its own copy with our XObject
        page_dict.set("Resources", Object::Dictionary(new_resources));
    }

    Ok(())
}

/// Add font reference to page's Resources dictionary
fn add_font_to_page_resources(doc: &mut Document, page_id: ObjectId, font_id: ObjectId) -> Result<()> {
    let page_obj = doc.get_object_mut(page_id)?;

    if let Object::Dictionary(ref mut page_dict) = page_obj {
        // Get or create Resources dictionary
        let mut resources = if let Ok(res) = page_dict.get(b"Resources") {
            res.clone()
        } else {
            Object::Dictionary(Dictionary::new())
        };

        if let Object::Dictionary(ref mut resources_dict) = resources {
            // Get or create Font subdictionary
            let mut fonts = if let Ok(Object::Dictionary(f)) = resources_dict.get(b"Font") {
                f.clone()
            } else {
                Dictionary::new()
            };

            // Add our font as /F1
            fonts.set("F1", Object::Reference(font_id));

            resources_dict.set("Font", Object::Dictionary(fonts));
            page_dict.set("Resources", resources);
        }
    }

    Ok(())
}

/// Prepend a content stream to a page's Contents
///
/// We prepend so our headers/footers are drawn BEFORE any page transformations,
/// giving them an independent coordinate system.
fn prepend_content_to_page(doc: &mut Document, page_id: ObjectId, new_content_id: ObjectId) -> Result<()> {
    let page_obj = doc.get_object_mut(page_id)?;

    if let Object::Dictionary(ref mut page_dict) = page_obj {
        let existing_content = page_dict.get(b"Contents").ok().cloned();

        match existing_content {
            Some(Object::Reference(content_id)) => {
                // Convert single reference to array, PREPEND our content
                let new_contents = vec![
                    Object::Reference(new_content_id),
                    Object::Reference(content_id),
                ];
                page_dict.set("Contents", Object::Array(new_contents));
            }
            Some(Object::Array(mut content_array)) => {
                // PREPEND to existing array
                content_array.insert(0, Object::Reference(new_content_id));
                page_dict.set("Contents", Object::Array(content_array));
            }
            _ => {
                // No existing content, create new array
                page_dict.set("Contents", Object::Array(vec![Object::Reference(new_content_id)]));
            }
        }
    }

    Ok(())
}

/// Append a content stream to a page's Contents
///
/// We append our content after the original content so our headers/footers are drawn
/// on top (not covered by background fills).
fn append_content_to_page(doc: &mut Document, page_id: ObjectId, new_content_id: ObjectId) -> Result<()> {
    let page_obj = doc.get_object_mut(page_id)?;

    if let Object::Dictionary(ref mut page_dict) = page_obj {
        let existing_content = page_dict.get(b"Contents").ok().cloned();

        match existing_content {
            Some(Object::Reference(content_id)) => {
                // Convert single reference to array, append our content
                let new_contents = vec![
                    Object::Reference(content_id),
                    Object::Reference(new_content_id),
                ];
                page_dict.set("Contents", Object::Array(new_contents));
            }
            Some(Object::Array(mut content_array)) => {
                // Append to existing array
                content_array.push(Object::Reference(new_content_id));
                page_dict.set("Contents", Object::Array(content_array));
            }
            _ => {
                // No existing content, create new array
                page_dict.set("Contents", Object::Array(vec![Object::Reference(new_content_id)]));
            }
        }
    }

    Ok(())
}
