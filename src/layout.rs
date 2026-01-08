//! Page layout calculations

/// Simple length type in millimeters
/// We'll integrate with krilla's types when implementing PDF creation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length(pub f64);

impl Length {
    /// Create a length from millimeters
    pub fn from_mm(mm: f64) -> Self {
        Length(mm)
    }

    /// Create a length from inches
    pub fn from_inches(inches: f64) -> Self {
        Length(inches * 25.4)
    }

    /// Get the value in millimeters
    pub fn mm(&self) -> f64 {
        self.0
    }

    /// Get the value in points (1/72 inch)
    pub fn pt(&self) -> f64 {
        self.0 * 72.0 / 25.4
    }
}

/// Page dimensions
#[derive(Debug, Clone, Copy)]
pub struct PageDimensions {
    pub width: Length,
    pub height: Length,
}

impl PageDimensions {
    /// US Letter size (8.5" × 11")
    pub fn letter() -> Self {
        Self {
            width: Length::from_mm(215.9),
            height: Length::from_mm(279.4),
        }
    }

    /// A4 size (210mm × 297mm)
    pub fn a4() -> Self {
        Self {
            width: Length::from_mm(210.0),
            height: Length::from_mm(297.0),
        }
    }
}

/// Margins for page content
#[derive(Debug, Clone, Copy)]
pub struct Margins {
    pub top: Length,
    pub bottom: Length,
    pub left: Length,
    pub right: Length,
}

impl Margins {
    /// Create margins with same value on all sides
    pub fn uniform(margin: Length) -> Self {
        Self {
            top: margin,
            bottom: margin,
            left: margin,
            right: margin,
        }
    }
}

/// Footer layout configuration
#[derive(Debug, Clone)]
pub struct FooterLayout {
    pub left: String,
    pub center: String,
    pub right: String,
}

/// Calculate the safe content area that won't overlap headers/footers
///
/// Returns (left, top, right, bottom) coordinates measured from the page origin.
/// The coordinate system has origin at bottom-left of the page.
pub fn calculate_safe_area(
    page: &PageDimensions,
    header_height: Length,
    footer_height: Length,
) -> (Length, Length, Length, Length) {
    let left = Length::from_mm(0.0);
    let bottom = footer_height;
    let right = page.width;
    let top = Length::from_mm(page.height.mm() - header_height.mm());

    (left, top, right, bottom)
}

/// Standard margins for typical document layouts
impl Margins {
    /// Standard 1-inch margins on all sides
    pub fn standard() -> Self {
        Self::uniform(Length::from_inches(1.0))
    }

    /// Narrow margins (0.5 inches)
    pub fn narrow() -> Self {
        Self::uniform(Length::from_inches(0.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_conversions() {
        let len = Length::from_inches(1.0);
        assert!((len.mm() - 25.4).abs() < 0.01);
        assert!((len.pt() - 72.0).abs() < 0.01);
    }

    #[test]
    fn test_letter_size() {
        let letter = PageDimensions::letter();
        // 8.5 inches = 215.9 mm
        assert!((letter.width.mm() - 215.9).abs() < 0.1);
        // 11 inches = 279.4 mm
        assert!((letter.height.mm() - 279.4).abs() < 0.1);
    }

    #[test]
    fn test_calculate_safe_area() {
        let page = PageDimensions::letter();
        let header_height = Length::from_mm(20.0);
        let footer_height = Length::from_mm(15.0);

        let (left, top, right, bottom) = calculate_safe_area(&page, header_height, footer_height);

        // Left should be at origin
        assert_eq!(left.mm(), 0.0);
        // Right should be at page width
        assert_eq!(right.mm(), page.width.mm());
        // Bottom should be above footer
        assert_eq!(bottom.mm(), 15.0);
        // Top should be below header
        assert_eq!(top.mm(), page.height.mm() - 20.0);
    }

    #[test]
    fn test_standard_margins() {
        let margins = Margins::standard();
        assert_eq!(margins.top.mm(), 25.4); // 1 inch
        assert_eq!(margins.bottom.mm(), 25.4);
        assert_eq!(margins.left.mm(), 25.4);
        assert_eq!(margins.right.mm(), 25.4);
    }
}
