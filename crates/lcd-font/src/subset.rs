use crate::traits::{Font, GlyphLayout};

/// Contiguous character range wrapper around an existing font.
/// Restricts printable characters to `start..=end` with zero allocation.
#[derive(Copy, Clone, Debug)]
pub struct RangeFont<F> {
    pub base: F,
    pub start: char,
    pub end: char,
}

impl<F: Font> Font for RangeFont<F> {
    #[inline(always)]
    fn width(&self) -> u8 {
        self.base.width()
    }

    #[inline(always)]
    fn height(&self) -> u8 {
        self.base.height()
    }

    #[inline(always)]
    fn spacing(&self) -> u8 {
        self.base.spacing()
    }

    #[inline(always)]
    fn contains(&self, c: char) -> bool {
        c >= self.start && c <= self.end && self.base.contains(c)
    }

    #[inline(always)]
    fn char_width(&self, c: char) -> u8 {
        if self.contains(c) {
            self.base.char_width(c)
        } else {
            0
        }
    }

    #[inline(always)]
    fn render_char<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        put_pixel: P,
    ) -> Option<u8> {
        if self.contains(c) {
            self.base.render_char(c, fg, bg, put_pixel)
        } else {
            None
        }
    }

    #[inline(always)]
    fn render_char_scaled<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        scale: u8,
        put_pixel: P,
    ) -> Option<u8> {
        if self.contains(c) {
            self.base.render_char_scaled(c, fg, bg, scale, put_pixel)
        } else {
            None
        }
    }
}

/// Arbitrary character filter wrapper around an existing font.
/// Restricts printable characters to a static slice of allowed characters.
#[derive(Copy, Clone, Debug)]
pub struct FilteredFont<'a, F> {
    pub base: &'a F,
    pub allowed: &'static [char],
}

impl<'a, F: Font> Font for FilteredFont<'a, F> {
    #[inline(always)]
    fn width(&self) -> u8 {
        self.base.width()
    }

    #[inline(always)]
    fn height(&self) -> u8 {
        self.base.height()
    }

    #[inline(always)]
    fn spacing(&self) -> u8 {
        self.base.spacing()
    }

    #[inline(always)]
    fn contains(&self, c: char) -> bool {
        for &ch in self.allowed {
            if ch == c {
                return self.base.contains(c);
            }
        }
        false
    }

    #[inline(always)]
    fn char_width(&self, c: char) -> u8 {
        if self.contains(c) {
            self.base.char_width(c)
        } else {
            0
        }
    }

    #[inline(always)]
    fn render_char<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        put_pixel: P,
    ) -> Option<u8> {
        if self.contains(c) {
            self.base.render_char(c, fg, bg, put_pixel)
        } else {
            None
        }
    }

    #[inline(always)]
    fn render_char_scaled<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        scale: u8,
        put_pixel: P,
    ) -> Option<u8> {
        if self.contains(c) {
            self.base.render_char_scaled(c, fg, bg, scale, put_pixel)
        } else {
            None
        }
    }
}

/// Custom slice-backed character subset font.
/// Stores only the exact glyph data required for an arbitrary character list.
#[derive(Copy, Clone, Debug)]
pub struct CustomSubsetFont<'a> {
    pub chars: &'a [char],
    pub data: &'a [u8],
    pub width: u8,
    pub height: u8,
    pub spacing: u8,
    pub layout: GlyphLayout,
    pub bytes_per_glyph: usize,
}

impl<'a> CustomSubsetFont<'a> {
    #[inline]
    pub fn index_of(&self, c: char) -> Option<usize> {
        self.chars.iter().position(|&ch| ch == c)
    }

    #[inline]
    pub fn glyph(&self, c: char) -> Option<&'a [u8]> {
        let idx = self.index_of(c)?;
        let start = idx * self.bytes_per_glyph;
        let end = start + self.bytes_per_glyph;
        if end <= self.data.len() {
            Some(&self.data[start..end])
        } else {
            None
        }
    }
}

impl<'a> Font for CustomSubsetFont<'a> {
    #[inline(always)]
    fn width(&self) -> u8 {
        self.width
    }

    #[inline(always)]
    fn height(&self) -> u8 {
        self.height
    }

    #[inline(always)]
    fn spacing(&self) -> u8 {
        self.spacing
    }

    #[inline(always)]
    fn contains(&self, c: char) -> bool {
        self.index_of(c).is_some()
    }

    #[inline(always)]
    fn char_width(&self, c: char) -> u8 {
        if self.contains(c) {
            self.width + self.spacing
        } else {
            0
        }
    }

    fn render_char<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        mut put_pixel: P,
    ) -> Option<u8> {
        let glyph = self.glyph(c)?;
        let bpg = self.bytes_per_glyph;
        match self.layout {
            GlyphLayout::ColumnMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                let glyph_h = if self.height > 7 { 7 } else { self.height as usize };
                for row in 0..h {
                    for col in 0..w {
                        let set = if row < glyph_h && col < bpg {
                            (glyph[col] & (1 << row)) != 0
                        } else {
                            false
                        };
                        put_pixel(if set { fg } else { bg });
                    }
                    for _ in 0..self.spacing {
                        put_pixel(bg);
                    }
                }
            }
            GlyphLayout::RowMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                for row in 0..h {
                    let row_byte = if row < bpg { glyph[row] } else { 0 };
                    for col in 0..w {
                        let set = (row_byte & (1 << col)) != 0;
                        put_pixel(if set { fg } else { bg });
                    }
                    for _ in 0..self.spacing {
                        put_pixel(bg);
                    }
                }
            }
            GlyphLayout::RowMajorMsb => {
                let h = self.height as usize;
                let bytes_per_row = bpg / h;
                for row in 0..h {
                    for b in 0..bytes_per_row {
                        let byte_val = glyph[row * bytes_per_row + b];
                        for bit in 0..8 {
                            let set = (byte_val & (1 << (7 - bit))) != 0;
                            put_pixel(if set { fg } else { bg });
                        }
                    }
                    for _ in 0..self.spacing {
                        put_pixel(bg);
                    }
                }
            }
            GlyphLayout::RowMajorMsbU32 | GlyphLayout::RowMajorNibble => {}
        }
        Some(self.width + self.spacing)
    }

    fn render_char_scaled<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        scale: u8,
        mut put_pixel: P,
    ) -> Option<u8> {
        let scale = scale.max(1);
        let glyph = self.glyph(c)?;
        let bpg = self.bytes_per_glyph;
        match self.layout {
            GlyphLayout::ColumnMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                let glyph_h = if self.height > 7 { 7 } else { self.height as usize };
                for row in 0..h {
                    for _ in 0..scale {
                        for col in 0..w {
                            let set = if row < glyph_h && col < bpg {
                                (glyph[col] & (1 << row)) != 0
                            } else {
                                false
                            };
                            let color = if set { fg } else { bg };
                            for _ in 0..scale {
                                put_pixel(color);
                            }
                        }
                        for _ in 0..(self.spacing as usize * scale as usize) {
                            put_pixel(bg);
                        }
                    }
                }
            }
            GlyphLayout::RowMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                for row in 0..h {
                    let row_byte = if row < bpg { glyph[row] } else { 0 };
                    for _ in 0..scale {
                        for col in 0..w {
                            let set = (row_byte & (1 << col)) != 0;
                            let color = if set { fg } else { bg };
                            for _ in 0..scale {
                                put_pixel(color);
                            }
                        }
                        for _ in 0..(self.spacing as usize * scale as usize) {
                            put_pixel(bg);
                        }
                    }
                }
            }
            GlyphLayout::RowMajorMsb => {
                let h = self.height as usize;
                let bytes_per_row = bpg / h;
                for row in 0..h {
                    for _ in 0..scale {
                        for b in 0..bytes_per_row {
                            let byte_val = glyph[row * bytes_per_row + b];
                            for bit in 0..8 {
                                let set = (byte_val & (1 << (7 - bit))) != 0;
                                let color = if set { fg } else { bg };
                                for _ in 0..scale {
                                    put_pixel(color);
                                }
                            }
                        }
                        for _ in 0..(self.spacing as usize * scale as usize) {
                            put_pixel(bg);
                        }
                    }
                }
            }
            GlyphLayout::RowMajorMsbU32 | GlyphLayout::RowMajorNibble => {}
        }
        Some((self.width + self.spacing) * scale)
    }
}

// -----------------------------------------------------------------------------
// Pre-built Pure-Numeric 0..=9 Flash Subsets (Minimum flash memory consumption)
// -----------------------------------------------------------------------------

/// Minimal 5x7 Numeric-only Font ('0'..='9', 50 bytes total flash)
#[derive(Copy, Clone, Debug, Default)]
pub struct NumericFont5x7;

pub static NUMERIC_5X7: NumericFont5x7 = NumericFont5x7;

static NUMERIC_5X7_RAW: [u8; 50] = [
    0x3E, 0x51, 0x49, 0x45, 0x3E, // '0'
    0x00, 0x42, 0x7F, 0x40, 0x00, // '1'
    0x42, 0x61, 0x51, 0x49, 0x46, // '2'
    0x21, 0x41, 0x45, 0x4B, 0x31, // '3'
    0x18, 0x14, 0x12, 0x7F, 0x10, // '4'
    0x27, 0x45, 0x45, 0x45, 0x39, // '5'
    0x3C, 0x4A, 0x49, 0x49, 0x30, // '6'
    0x01, 0x71, 0x09, 0x05, 0x03, // '7'
    0x36, 0x49, 0x49, 0x49, 0x36, // '8'
    0x06, 0x49, 0x49, 0x29, 0x1E, // '9'
];

impl Font for NumericFont5x7 {
    #[inline(always)]
    fn width(&self) -> u8 { 5 }
    #[inline(always)]
    fn height(&self) -> u8 { 8 }
    #[inline(always)]
    fn spacing(&self) -> u8 { 1 }
    #[inline(always)]
    fn contains(&self, c: char) -> bool { ('0'..='9').contains(&c) }
    #[inline(always)]
    fn char_width(&self, c: char) -> u8 { if self.contains(c) { 6 } else { 0 } }

    fn render_char<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, mut put_pixel: P) -> Option<u8> {
        if !self.contains(c) { return None; }
        let idx = (c as usize - '0' as usize) * 5;
        let glyph = &NUMERIC_5X7_RAW[idx..idx + 5];
        for row in 0..8 {
            for col in 0..5 {
                let set = if row < 7 { (glyph[col] & (1 << row)) != 0 } else { false };
                put_pixel(if set { fg } else { bg });
            }
            put_pixel(bg);
        }
        Some(6)
    }

    fn render_char_scaled<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, scale: u8, mut put_pixel: P) -> Option<u8> {
        if !self.contains(c) { return None; }
        let scale = scale.max(1);
        let idx = (c as usize - '0' as usize) * 5;
        let glyph = &NUMERIC_5X7_RAW[idx..idx + 5];
        for row in 0..8 {
            for _ in 0..scale {
                for col in 0..5 {
                    let set = if row < 7 { (glyph[col] & (1 << row)) != 0 } else { false };
                    let color = if set { fg } else { bg };
                    for _ in 0..scale { put_pixel(color); }
                }
                for _ in 0..scale { put_pixel(bg); }
            }
        }
        Some(6 * scale)
    }
}

/// Minimal 8x16 Numeric-only Font ('0'..='9', 160 bytes total flash)
#[derive(Copy, Clone, Debug, Default)]
pub struct NumericFont8x16;

pub static NUMERIC_8X16: NumericFont8x16 = NumericFont8x16;

static NUMERIC_8X16_RAW: [u8; 160] = [
    0x00, 0x00, 0x00, 0x18, 0x24, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x24, 0x18, 0x00, 0x00, // '0'
    0x00, 0x00, 0x00, 0x08, 0x0E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00, 0x00, // '1'
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x20, 0x20, 0x10, 0x08, 0x04, 0x42, 0x7E, 0x00, 0x00, // '2'
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x20, 0x18, 0x20, 0x40, 0x40, 0x42, 0x22, 0x1C, 0x00, 0x00, // '3'
    0x00, 0x00, 0x00, 0x20, 0x30, 0x28, 0x24, 0x24, 0x22, 0x22, 0x7E, 0x20, 0x20, 0x78, 0x00, 0x00, // '4'
    0x00, 0x00, 0x00, 0x7E, 0x02, 0x02, 0x02, 0x1A, 0x26, 0x40, 0x40, 0x42, 0x22, 0x1C, 0x00, 0x00, // '5'
    0x00, 0x00, 0x00, 0x38, 0x24, 0x02, 0x02, 0x1A, 0x26, 0x42, 0x42, 0x42, 0x24, 0x18, 0x00, 0x00, // '6'
    0x00, 0x00, 0x00, 0x7E, 0x22, 0x22, 0x10, 0x10, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00, // '7'
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x24, 0x18, 0x24, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, // '8'
    0x00, 0x00, 0x00, 0x18, 0x24, 0x42, 0x42, 0x42, 0x64, 0x58, 0x40, 0x40, 0x24, 0x1C, 0x00, 0x00, // '9'
];

impl Font for NumericFont8x16 {
    #[inline(always)]
    fn width(&self) -> u8 { 8 }
    #[inline(always)]
    fn height(&self) -> u8 { 16 }
    #[inline(always)]
    fn spacing(&self) -> u8 { 0 }
    #[inline(always)]
    fn contains(&self, c: char) -> bool { ('0'..='9').contains(&c) }
    #[inline(always)]
    fn char_width(&self, c: char) -> u8 { if self.contains(c) { 8 } else { 0 } }

    fn render_char<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, mut put_pixel: P) -> Option<u8> {
        if !self.contains(c) { return None; }
        let idx = (c as usize - '0' as usize) * 16;
        let glyph = &NUMERIC_8X16_RAW[idx..idx + 16];
        for row in 0..16 {
            let row_byte = glyph[row];
            for col in 0..8 {
                let set = (row_byte & (1 << col)) != 0;
                put_pixel(if set { fg } else { bg });
            }
        }
        Some(8)
    }

    fn render_char_scaled<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, scale: u8, mut put_pixel: P) -> Option<u8> {
        if !self.contains(c) { return None; }
        let scale = scale.max(1);
        let idx = (c as usize - '0' as usize) * 16;
        let glyph = &NUMERIC_8X16_RAW[idx..idx + 16];
        for row in 0..16 {
            let row_byte = glyph[row];
            for _ in 0..scale {
                for col in 0..8 {
                    let set = (row_byte & (1 << col)) != 0;
                    let color = if set { fg } else { bg };
                    for _ in 0..scale { put_pixel(color); }
                }
            }
        }
        Some(8 * scale)
    }
}

/// Minimal 16x24 Numeric-only Font ('0'..='9', 480 bytes total flash)
#[derive(Copy, Clone, Debug, Default)]
pub struct NumericFont16x24;

pub static NUMERIC_16X24: NumericFont16x24 = NumericFont16x24;

static NUMERIC_16X24_RAW: [u8; 480] = [
    // '0'
    0x07, 0xC0, 0x0F, 0xE0, 0x1C, 0x70, 0x1C, 0x70, 0x18, 0x70, 0x18, 0x30, 0x18, 0x30, 0x18, 0x30,
    0x18, 0x30, 0x1C, 0x70, 0x1C, 0x70, 0x1C, 0x70, 0x0F, 0xE0, 0x07, 0xC0, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '1'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xC0, 0x01, 0xC0, 0x1F, 0xC0,
    0x1F, 0xC0, 0x01, 0xC0, 0x01, 0xC0, 0x01, 0xC0, 0x01, 0xC0, 0x01, 0xC0, 0x01, 0xC0, 0x01, 0xC0,
    0x01, 0xC0, 0x01, 0xC0, 0x01, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '2'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0xE0, 0x1F, 0xF0, 0x1C, 0x70,
    0x1C, 0x70, 0x18, 0x70, 0x00, 0x70, 0x00, 0xF0, 0x01, 0xE0, 0x03, 0xC0, 0x07, 0x00, 0x0E, 0x00,
    0x1F, 0xF0, 0x1F, 0xF0, 0x1F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '3'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0xE0, 0x1F, 0xE0, 0x1C, 0x70,
    0x18, 0x70, 0x00, 0x70, 0x00, 0xE0, 0x03, 0xC0, 0x03, 0xF0, 0x00, 0x70, 0x00, 0x70, 0x18, 0x70,
    0x1C, 0x70, 0x1F, 0xF0, 0x0F, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '4'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xE0, 0x01, 0xE0, 0x03, 0xE0,
    0x07, 0xE0, 0x06, 0xE0, 0x0C, 0xE0, 0x0C, 0xE0, 0x18, 0xE0, 0x30, 0xE0, 0x3F, 0xF0, 0x3F, 0xF0,
    0x00, 0xE0, 0x00, 0xE0, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '5'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0xF0, 0x0F, 0xF0, 0x0C, 0x00,
    0x0C, 0x00, 0x1D, 0x80, 0x1F, 0xE0, 0x1F, 0xF0, 0x1C, 0x70, 0x00, 0x70, 0x00, 0x30, 0x18, 0x70,
    0x1C, 0x70, 0x1F, 0xE0, 0x0F, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '6'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xE0, 0x0F, 0xF0, 0x1C, 0x70,
    0x1C, 0x00, 0x1C, 0x00, 0x1B, 0xE0, 0x1F, 0xF0, 0x1C, 0x70, 0x1C, 0x30, 0x18, 0x30, 0x1C, 0x30,
    0x1C, 0x70, 0x0F, 0xE0, 0x07, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '7'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F, 0xF8, 0x1F, 0xF8, 0x00, 0x70,
    0x00, 0x60, 0x00, 0xE0, 0x01, 0xC0, 0x01, 0xC0, 0x03, 0x80, 0x03, 0x80, 0x03, 0x00, 0x07, 0x00,
    0x07, 0x00, 0x07, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '8'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0xE0, 0x1F, 0xF0, 0x1C, 0x70,
    0x18, 0x30, 0x1C, 0x70, 0x0F, 0xE0, 0x0F, 0xE0, 0x1E, 0xF0, 0x1C, 0x70, 0x38, 0x30, 0x38, 0x30,
    0x1C, 0x70, 0x1F, 0xF0, 0x0F, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '9'
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0xC0, 0x1F, 0xE0, 0x1C, 0x70,
    0x18, 0x70, 0x18, 0x70, 0x18, 0x70, 0x1C, 0x70, 0x1F, 0xF0, 0x0F, 0xB0, 0x00, 0x70, 0x00, 0x70,
    0x1C, 0x70, 0x1F, 0xE0, 0x0F, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

impl Font for NumericFont16x24 {
    #[inline(always)]
    fn width(&self) -> u8 { 16 }
    #[inline(always)]
    fn height(&self) -> u8 { 24 }
    #[inline(always)]
    fn spacing(&self) -> u8 { 0 }
    #[inline(always)]
    fn contains(&self, c: char) -> bool { ('0'..='9').contains(&c) }
    #[inline(always)]
    fn char_width(&self, c: char) -> u8 { if self.contains(c) { 16 } else { 0 } }

    fn render_char<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, mut put_pixel: P) -> Option<u8> {
        if !self.contains(c) { return None; }
        let idx = (c as usize - '0' as usize) * 48;
        let glyph = &NUMERIC_16X24_RAW[idx..idx + 48];
        for row in 0..24 {
            let b0 = glyph[row * 2];
            let b1 = glyph[row * 2 + 1];
            for col in 0..8 {
                let set = (b0 & (1 << (7 - col))) != 0;
                put_pixel(if set { fg } else { bg });
            }
            for col in 0..8 {
                let set = (b1 & (1 << (7 - col))) != 0;
                put_pixel(if set { fg } else { bg });
            }
        }
        Some(16)
    }

    fn render_char_scaled<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, scale: u8, mut put_pixel: P) -> Option<u8> {
        if !self.contains(c) { return None; }
        let scale = scale.max(1);
        let idx = (c as usize - '0' as usize) * 48;
        let glyph = &NUMERIC_16X24_RAW[idx..idx + 48];
        for row in 0..24 {
            let b0 = glyph[row * 2];
            let b1 = glyph[row * 2 + 1];
            for _ in 0..scale {
                for col in 0..8 {
                    let set = (b0 & (1 << (7 - col))) != 0;
                    let color = if set { fg } else { bg };
                    for _ in 0..scale { put_pixel(color); }
                }
                for col in 0..8 {
                    let set = (b1 & (1 << (7 - col))) != 0;
                    let color = if set { fg } else { bg };
                    for _ in 0..scale { put_pixel(color); }
                }
            }
        }
        Some(16 * scale)
    }
}

/// Compile-time or static character subset font.
/// Embeds exactly `N` characters and `B` bytes of raw glyph data in flash memory.
#[derive(Copy, Clone, Debug)]
pub struct SubsetFont<const N: usize, const B: usize> {
    pub chars: [char; N],
    pub data: [u8; B],
    pub width: u8,
    pub height: u8,
    pub spacing: u8,
    pub layout: GlyphLayout,
    pub bytes_per_glyph: usize,
}

impl<const N: usize, const B: usize> SubsetFont<N, B> {
    pub const fn new(
        chars: [char; N],
        data: [u8; B],
        width: u8,
        height: u8,
        spacing: u8,
        layout: GlyphLayout,
        bytes_per_glyph: usize,
    ) -> Self {
        Self {
            chars,
            data,
            width,
            height,
            spacing,
            layout,
            bytes_per_glyph,
        }
    }

    #[inline]
    pub fn index_of(&self, c: char) -> Option<usize> {
        let mut i = 0;
        while i < N {
            if self.chars[i] == c {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    #[inline]
    pub fn glyph(&self, c: char) -> Option<&[u8]> {
        let idx = self.index_of(c)?;
        let start = idx * self.bytes_per_glyph;
        let end = start + self.bytes_per_glyph;
        if end <= self.data.len() {
            Some(&self.data[start..end])
        } else {
            None
        }
    }
}

impl<const N: usize, const B: usize> Font for SubsetFont<N, B> {
    #[inline(always)]
    fn width(&self) -> u8 { self.width }
    #[inline(always)]
    fn height(&self) -> u8 { self.height }
    #[inline(always)]
    fn spacing(&self) -> u8 { self.spacing }
    #[inline(always)]
    fn contains(&self, c: char) -> bool { self.index_of(c).is_some() }
    #[inline(always)]
    fn char_width(&self, c: char) -> u8 {
        if self.contains(c) { self.width + self.spacing } else { 0 }
    }

    fn render_char<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, mut put_pixel: P) -> Option<u8> {
        let glyph = self.glyph(c)?;
        let bpg = self.bytes_per_glyph;
        match self.layout {
            GlyphLayout::ColumnMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                let glyph_h = if self.height > 7 { 7 } else { self.height as usize };
                for row in 0..h {
                    for col in 0..w {
                        let set = if row < glyph_h && col < bpg {
                            (glyph[col] & (1 << row)) != 0
                        } else {
                            false
                        };
                        put_pixel(if set { fg } else { bg });
                    }
                    for _ in 0..self.spacing {
                        put_pixel(bg);
                    }
                }
            }
            GlyphLayout::RowMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                for row in 0..h {
                    let row_byte = if row < bpg { glyph[row] } else { 0 };
                    for col in 0..w {
                        let set = (row_byte & (1 << col)) != 0;
                        put_pixel(if set { fg } else { bg });
                    }
                    for _ in 0..self.spacing {
                        put_pixel(bg);
                    }
                }
            }
            GlyphLayout::RowMajorMsb => {
                let h = self.height as usize;
                let bytes_per_row = bpg / h;
                for row in 0..h {
                    for b in 0..bytes_per_row {
                        let byte_val = glyph[row * bytes_per_row + b];
                        for bit in 0..8 {
                            let set = (byte_val & (1 << (7 - bit))) != 0;
                            put_pixel(if set { fg } else { bg });
                        }
                    }
                    for _ in 0..self.spacing {
                        put_pixel(bg);
                    }
                }
            }
            GlyphLayout::RowMajorMsbU32 | GlyphLayout::RowMajorNibble => {}
        }
        Some(self.width + self.spacing)
    }

    fn render_char_scaled<P: FnMut(u16)>(&self, c: char, fg: u16, bg: u16, scale: u8, mut put_pixel: P) -> Option<u8> {
        let scale = scale.max(1);
        let glyph = self.glyph(c)?;
        let bpg = self.bytes_per_glyph;
        match self.layout {
            GlyphLayout::ColumnMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                let glyph_h = if self.height > 7 { 7 } else { self.height as usize };
                for row in 0..h {
                    for _ in 0..scale {
                        for col in 0..w {
                            let set = if row < glyph_h && col < bpg {
                                (glyph[col] & (1 << row)) != 0
                            } else {
                                false
                            };
                            let color = if set { fg } else { bg };
                            for _ in 0..scale {
                                put_pixel(color);
                            }
                        }
                        for _ in 0..(self.spacing as usize * scale as usize) {
                            put_pixel(bg);
                        }
                    }
                }
            }
            GlyphLayout::RowMajorLsb => {
                let w = self.width as usize;
                let h = self.height as usize;
                for row in 0..h {
                    let row_byte = if row < bpg { glyph[row] } else { 0 };
                    for _ in 0..scale {
                        for col in 0..w {
                            let set = (row_byte & (1 << col)) != 0;
                            let color = if set { fg } else { bg };
                            for _ in 0..scale {
                                put_pixel(color);
                            }
                        }
                        for _ in 0..(self.spacing as usize * scale as usize) {
                            put_pixel(bg);
                        }
                    }
                }
            }
            GlyphLayout::RowMajorMsb => {
                let h = self.height as usize;
                let bytes_per_row = bpg / h;
                for row in 0..h {
                    for _ in 0..scale {
                        for b in 0..bytes_per_row {
                            let byte_val = glyph[row * bytes_per_row + b];
                            for bit in 0..8 {
                                let set = (byte_val & (1 << (7 - bit))) != 0;
                                let color = if set { fg } else { bg };
                                for _ in 0..scale {
                                    put_pixel(color);
                                }
                            }
                        }
                        for _ in 0..(self.spacing as usize * scale as usize) {
                            put_pixel(bg);
                        }
                    }
                }
            }
            GlyphLayout::RowMajorMsbU32 | GlyphLayout::RowMajorNibble => {}
        }
        Some((self.width + self.spacing) * scale)
    }
}

