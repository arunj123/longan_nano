/// Glyph bit encoding layout
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GlyphLayout {
    /// 1 byte per column, bit 0 = top row (e.g. Font5x7)
    ColumnMajorLsb,
    /// 1 byte per row, bit 0 = left column (e.g. Font8x16)
    RowMajorLsb,
    /// 1 or more bytes per row, bit 7 = left column (e.g. Font16x24)
    RowMajorMsb,
    /// 1 u32 per row, bit 31 = left column (e.g. Font28)
    RowMajorMsbU32,
}

/// Generic trait for embedded bitmap and rasterized fonts
pub trait Font {
    /// Nominal or maximum glyph width in pixels
    fn width(&self) -> u8;

    /// Line / glyph height in pixels
    fn height(&self) -> u8;

    /// Inter-character horizontal spacing in pixels (defaults to 0)
    fn spacing(&self) -> u8 {
        0
    }

    /// Returns true if this font contains a glyph for character `c`
    fn contains(&self, c: char) -> bool;

    /// Rendered pixel width of character `c` (including inter-char spacing if applicable)
    fn char_width(&self, c: char) -> u8;

    /// Total rendered pixel width of a string
    fn string_width(&self, text: &str) -> u16 {
        let mut total = 0u16;
        for c in text.chars() {
            if self.contains(c) {
                total += self.char_width(c) as u16;
            }
        }
        total
    }

    /// Renders a character pixel-by-pixel in row-major order (row 0 col 0..W, row 1 col 0..W, ...)
    /// by calling `put_pixel(color)` for every pixel in the character cell.
    /// Returns `Some(char_width)` if the glyph exists, or `None` if unmapped.
    fn render_char<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        put_pixel: P,
    ) -> Option<u8>;

    /// Renders a character pixel-by-pixel with integer scaling factor (scale >= 1).
    /// Emits each pixel expanded by `scale` x `scale`.
    fn render_char_scaled<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        scale: u8,
        put_pixel: P,
    ) -> Option<u8>;
}
