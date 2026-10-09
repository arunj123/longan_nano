use crate::traits::{Font, GlyphLayout};

/// 28px Smooth TrueType-Rasterized Font Glyph (variable/proportional width)
#[derive(Copy, Clone, Debug)]
pub struct Glyph28 {
    pub width: u8,
    pub rows: [u32; 28],
}

/// 28px Smooth Bold Numeric & Sensor Font
/// Rasterized directly from Bahnschrift/DIN TrueType vector contours with 1:1 hardware pixel curves.
/// Supported characters: '0'..'9', '-', '.', ' ', 'm', 'A'
#[derive(Copy, Clone, Debug, Default)]
pub struct Font28;

pub static FONT_28: Font28 = Font28;

impl Font28 {
    pub const HEIGHT: u8 = 28;
    pub const LAYOUT: GlyphLayout = GlyphLayout::RowMajorMsbU32;

    /// Looks up glyph for character `c`
    pub fn glyph(&self, c: char) -> Option<&'static Glyph28> {
        get_glyph_28(c)
    }

    /// Calculates total rendered pixel width of a string in 28px font
    pub fn string_width(&self, text: &str) -> u16 {
        string_width_28(text)
    }

    /// Checks if a pixel at (px, py) is set for the given glyph
    pub fn is_pixel_set(&self, glyph: &Glyph28, px: u8, py: u8) -> bool {
        if px < glyph.width && py < 28 {
            (glyph.rows[py as usize] & (1 << (31 - px))) != 0
        } else {
            false
        }
    }
}

impl Font for Font28 {
    #[inline(always)]
    fn width(&self) -> u8 {
        29 // Maximum proportional glyph width ('m')
    }

    #[inline(always)]
    fn height(&self) -> u8 {
        28
    }

    #[inline(always)]
    fn spacing(&self) -> u8 {
        0
    }

    #[inline(always)]
    fn contains(&self, c: char) -> bool {
        get_glyph_28(c).is_some()
    }

    #[inline(always)]
    fn char_width(&self, c: char) -> u8 {
        get_glyph_28(c).map(|g| g.width).unwrap_or(0)
    }

    #[inline(always)]
    fn string_width(&self, text: &str) -> u16 {
        string_width_28(text)
    }

    fn render_char<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        mut put_pixel: P,
    ) -> Option<u8> {
        let glyph = get_glyph_28(c)?;
        let w = glyph.width;
        for row in 0..28 {
            let row_bits = glyph.rows[row];
            for col in 0..w {
                let set = (row_bits & (1 << (31 - col))) != 0;
                put_pixel(if set { fg } else { bg });
            }
        }
        Some(w)
    }

    fn render_char_scaled<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        scale: u8,
        mut put_pixel: P,
    ) -> Option<u8> {
        let glyph = get_glyph_28(c)?;
        let w = glyph.width;
        let scale = scale.max(1);
        for row in 0..28 {
            let row_bits = glyph.rows[row];
            for _ in 0..scale {
                for col in 0..w {
                    let set = (row_bits & (1 << (31 - col))) != 0;
                    let color = if set { fg } else { bg };
                    for _ in 0..scale {
                        put_pixel(color);
                    }
                }
            }
        }
        Some(w * scale)
    }
}

/// Retrieves the static 28px glyph definition for character `c`
pub fn get_glyph_28(c: char) -> Option<&'static Glyph28> {
    match c {
        '0' => Some(&GLYPH_28_0),
        '1' => Some(&GLYPH_28_1),
        '2' => Some(&GLYPH_28_2),
        '3' => Some(&GLYPH_28_3),
        '4' => Some(&GLYPH_28_4),
        '5' => Some(&GLYPH_28_5),
        '6' => Some(&GLYPH_28_6),
        '7' => Some(&GLYPH_28_7),
        '8' => Some(&GLYPH_28_8),
        '9' => Some(&GLYPH_28_9),
        '-' => Some(&GLYPH_28_MINUS),
        '.' => Some(&GLYPH_28_DOT),
        ' ' => Some(&GLYPH_28_SPACE),
        'm' => Some(&GLYPH_28_M),
        'A' => Some(&GLYPH_28_A),
        _ => None,
    }
}

/// Computes the rendered pixel width of a string formatted in 28px font
pub fn string_width_28(text: &str) -> u16 {
    let mut total = 0u16;
    for c in text.chars() {
        if let Some(glyph) = get_glyph_28(c) {
            total += glyph.width as u16;
        }
    }
    total
}

static GLYPH_28_0: Glyph28 = Glyph28 { width: 19, rows: [0x00000000, 0x00000000, 0x00000000, 0x03F80000, 0x07FC0000, 0x0FFE0000, 0x1E1F0000, 0x1C0F0000, 0x1C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x3C070000, 0x1C070000, 0x1C0F0000, 0x1E1F0000, 0x0FFE0000, 0x0FFC0000, 0x03F00000, 0x00000000] };
static GLYPH_28_1: Glyph28 = Glyph28 { width: 11, rows: [0x00000000, 0x00000000, 0x00000000, 0x0F000000, 0x1F000000, 0x7F000000, 0x7F000000, 0x77000000, 0x47000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x07000000, 0x00000000] };
static GLYPH_28_2: Glyph28 = Glyph28 { width: 18, rows: [0x00000000, 0x00000000, 0x00000000, 0x03F00000, 0x0FFC0000, 0x1FFE0000, 0x1E1E0000, 0x3C0E0000, 0x3C0F0000, 0x000F0000, 0x000E0000, 0x001E0000, 0x001E0000, 0x003C0000, 0x007C0000, 0x00780000, 0x00F00000, 0x01E00000, 0x03E00000, 0x03C00000, 0x07800000, 0x0F000000, 0x1F000000, 0x1E000000, 0x3FFF0000, 0x3FFF0000, 0x3FFF0000, 0x00000000] };
static GLYPH_28_3: Glyph28 = Glyph28 { width: 18, rows: [0x00000000, 0x00000000, 0x00000000, 0x03F00000, 0x0FF80000, 0x1FFC0000, 0x1E1E0000, 0x3C0E0000, 0x380E0000, 0x000E0000, 0x000E0000, 0x000E0000, 0x001C0000, 0x01F80000, 0x01F00000, 0x01FC0000, 0x001E0000, 0x000E0000, 0x000E0000, 0x000F0000, 0x000F0000, 0x380F0000, 0x3C0E0000, 0x3E1E0000, 0x1FFE0000, 0x0FFC0000, 0x03F00000, 0x00000000] };
static GLYPH_28_4: Glyph28 = Glyph28 { width: 19, rows: [0x00000000, 0x00000000, 0x00000000, 0x003C0000, 0x00380000, 0x00780000, 0x00700000, 0x00F00000, 0x00E00000, 0x01E00000, 0x01C00000, 0x03C00000, 0x03800000, 0x07870000, 0x07070000, 0x0F070000, 0x0E070000, 0x1E070000, 0x1E070000, 0x3C070000, 0x3FFFC000, 0x3FFFC000, 0x3FFFC000, 0x00070000, 0x00070000, 0x00070000, 0x00070000, 0x00000000] };
static GLYPH_28_5: Glyph28 = Glyph28 { width: 19, rows: [0x00000000, 0x00000000, 0x00000000, 0x1FFE0000, 0x1FFE0000, 0x1FFE0000, 0x1C000000, 0x1C000000, 0x1C000000, 0x1C000000, 0x1DF00000, 0x1FFC0000, 0x1FFE0000, 0x1F1E0000, 0x1E0F0000, 0x000F0000, 0x00070000, 0x00070000, 0x00070000, 0x00070000, 0x00070000, 0x1C0F0000, 0x1C0F0000, 0x1E1E0000, 0x0FFE0000, 0x07FC0000, 0x03F00000, 0x00000000] };
static GLYPH_28_6: Glyph28 = Glyph28 { width: 17, rows: [0x00000000, 0x00000000, 0x00000000, 0x00700000, 0x00F00000, 0x00E00000, 0x01E00000, 0x01C00000, 0x03C00000, 0x03800000, 0x07800000, 0x07000000, 0x0F000000, 0x0FF00000, 0x1FFC0000, 0x1FFC0000, 0x3E3E0000, 0x3C1E0000, 0x3C0E0000, 0x380E0000, 0x380E0000, 0x3C0E0000, 0x3C0E0000, 0x1E1E0000, 0x1FFC0000, 0x0FF80000, 0x03F00000, 0x00000000] };
static GLYPH_28_7: Glyph28 = Glyph28 { width: 17, rows: [0x00000000, 0x00000000, 0x00000000, 0x3FFE0000, 0x3FFE0000, 0x3FFE0000, 0x381E0000, 0x381C0000, 0x381C0000, 0x383C0000, 0x003C0000, 0x00380000, 0x00780000, 0x00780000, 0x00700000, 0x00700000, 0x00F00000, 0x00F00000, 0x00E00000, 0x01E00000, 0x01E00000, 0x01C00000, 0x03C00000, 0x03C00000, 0x03800000, 0x03800000, 0x07800000, 0x00000000] };
static GLYPH_28_8: Glyph28 = Glyph28 { width: 19, rows: [0x00000000, 0x00000000, 0x00000000, 0x01F80000, 0x07FC0000, 0x0FFE0000, 0x1F1F0000, 0x1E0F0000, 0x1C070000, 0x1C070000, 0x1C070000, 0x1E0F0000, 0x0F1E0000, 0x07FC0000, 0x07FC0000, 0x0FFE0000, 0x1F0F0000, 0x1C070000, 0x3C078000, 0x3C078000, 0x3C078000, 0x3C078000, 0x1C078000, 0x1E0F0000, 0x0FFF0000, 0x07FC0000, 0x03F80000, 0x00000000] };
static GLYPH_28_9: Glyph28 = Glyph28 { width: 17, rows: [0x00000000, 0x00000000, 0x00000000, 0x03F00000, 0x0FF80000, 0x1FFC0000, 0x1E1E0000, 0x3C1E0000, 0x3C0E0000, 0x380E0000, 0x380E0000, 0x3C0E0000, 0x3C1E0000, 0x1E1E0000, 0x1FFE0000, 0x0FFC0000, 0x07FC0000, 0x00380000, 0x00780000, 0x00700000, 0x00F00000, 0x00E00000, 0x01E00000, 0x01C00000, 0x03C00000, 0x03800000, 0x07800000, 0x00000000] };
static GLYPH_28_MINUS: Glyph28 = Glyph28 { width: 16, rows: [0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x1FFC0000, 0x1FFC0000, 0x1FFC0000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000] };
static GLYPH_28_DOT: Glyph28 = Glyph28 { width: 8, rows: [0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x3C000000, 0x3C000000, 0x3C000000, 0x3C000000, 0x00000000] };
static GLYPH_28_SPACE: Glyph28 = Glyph28 { width: 9, rows: [0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000] };
static GLYPH_28_M: Glyph28 = Glyph28 { width: 29, rows: [0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x1CF83E00, 0x1DFE7F80, 0x1FFEFFC0, 0x1F1FC3C0, 0x1E0F81C0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x1C0701E0, 0x00000000] };
static GLYPH_28_A: Glyph28 = Glyph28 { width: 22, rows: [0x00000000, 0x00000000, 0x00000000, 0x00780000, 0x00780000, 0x00780000, 0x00FC0000, 0x00FC0000, 0x00FC0000, 0x01CE0000, 0x01CE0000, 0x03CF0000, 0x03870000, 0x03870000, 0x07878000, 0x07038000, 0x07038000, 0x0F03C000, 0x0E01C000, 0x1FFFE000, 0x1FFFE000, 0x1FFFE000, 0x3C00F000, 0x3C00F000, 0x38007000, 0x78007800, 0x78003800, 0x00000000] };
