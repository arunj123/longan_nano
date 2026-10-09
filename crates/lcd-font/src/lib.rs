#![no_std]

pub mod traits;
pub mod subset;
pub mod font5x7;
pub mod font8x16;
pub mod font16x24;
pub mod font28;

pub use traits::{Font, GlyphLayout};
pub use subset::{
    CustomSubsetFont, FilteredFont, RangeFont, SubsetFont,
    NumericFont5x7, NumericFont8x16, NumericFont16x24,
    NUMERIC_5X7, NUMERIC_8X16, NUMERIC_16X24,
};
pub use font5x7::{Font5x7, FONT_5X7, FONT_5X7_RAW};
pub use font8x16::{Font8x16, FONT_8X16, FONT_8X16_RAW};
pub use font16x24::{Font16x24, FONT_16X24, FONT_16X24_CHARS, FONT_16X24_RAW};
pub use font28::{Font28, FONT_28, Glyph28, get_glyph_28, string_width_28};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fonts_basic() {
        assert_eq!(FONT_5X7.width(), 5);
        assert_eq!(FONT_5X7.height(), 8);
        assert!(FONT_5X7.contains('A'));
        assert_eq!(FONT_5X7.char_width('A'), 6);

        assert_eq!(FONT_8X16.width(), 8);
        assert_eq!(FONT_8X16.height(), 16);
        assert!(FONT_8X16.contains('0'));

        assert_eq!(FONT_16X24.width(), 16);
        assert_eq!(FONT_16X24.height(), 24);
        assert!(FONT_16X24.contains('V'));

        assert_eq!(FONT_28.height(), 28);
        assert!(FONT_28.contains('5'));
    }

    #[test]
    fn test_numeric_subsets() {
        for c in '0'..='9' {
            assert!(NUMERIC_5X7.contains(c));
            assert!(NUMERIC_8X16.contains(c));
            assert!(NUMERIC_16X24.contains(c));
        }
        assert!(!NUMERIC_5X7.contains('A'));
        assert!(!NUMERIC_8X16.contains(' '));
        assert!(!NUMERIC_16X24.contains('V'));
    }

    #[test]
    fn test_range_font() {
        let range = FONT_5X7.range('0', '9');
        assert!(range.contains('0'));
        assert!(range.contains('9'));
        assert!(!range.contains('A'));
        assert!(!range.contains('/'));
    }

    #[test]
    fn test_filter_font() {
        static ALLOWED: [char; 3] = ['A', 'B', 'C'];
        let filter = FONT_5X7.filter(&ALLOWED);
        assert!(filter.contains('A'));
        assert!(filter.contains('B'));
        assert!(filter.contains('C'));
        assert!(!filter.contains('D'));
        assert!(!filter.contains('0'));
    }

    #[test]
    fn test_subset_font() {
        let subset = SubsetFont::new(
            ['X', 'Y'],
            [0xAA, 0xBB],
            8, 8, 0,
            GlyphLayout::RowMajorLsb,
            1,
        );
        assert!(subset.contains('X'));
        assert!(subset.contains('Y'));
        assert!(!subset.contains('Z'));
        assert_eq!(subset.glyph('X'), Some(&[0xAA][..]));
        assert_eq!(subset.glyph('Y'), Some(&[0xBB][..]));
    }
}
