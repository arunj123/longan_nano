import os
from PIL import Image, ImageFont, ImageDraw

def generate_font48():
    font_path = os.path.join(os.environ["WINDIR"], "Fonts", "bahnschrift.ttf")
    font = ImageFont.truetype(font_path, 65)
    font.set_variation_by_axes([600, 75]) # SemiBold, Condensed

    H = 48
    W_DIGIT = 28
    W_DOT = 14

    b0 = font.getbbox('0')
    draw_y_offset = -b0[1]

    char_map = [
        ('0', '0', W_DIGIT),
        ('1', '1', W_DIGIT),
        ('2', '2', W_DIGIT),
        ('3', '3', W_DIGIT),
        ('4', '4', W_DIGIT),
        ('5', '5', W_DIGIT),
        ('6', '6', W_DIGIT),
        ('7', '7', W_DIGIT),
        ('8', '8', W_DIGIT),
        ('9', '9', W_DIGIT),
        ('-', 'MINUS', W_DIGIT),
        ('.', 'DOT', W_DOT),
        (' ', 'SPACE', W_DIGIT),
    ]

    code_sections = []
    glyph_defs = []

    for ch, name, w_cell in char_map:
        im = Image.new('L', (w_cell, H), 0)
        draw = ImageDraw.Draw(im)

        if ch != ' ':
            b = font.getbbox(ch)
            gw = b[2] - b[0]
            char_x = (w_cell - gw) // 2 - b[0]

            if ch == '-':
                draw.text((char_x, -11), ch, font=font, fill=255)
            elif ch == '.':
                draw.text((char_x, draw_y_offset), ch, font=font, fill=255)
            else:
                draw.text((char_x, draw_y_offset), ch, font=font, fill=255)

        # Convert to 4-bit nibbles (2 pixels per byte, row-major)
        data_bytes = []
        for y in range(H):
            for x in range(0, w_cell, 2):
                a0 = min(15, (im.getpixel((x, y)) + 8) // 17)
                a1 = min(15, (im.getpixel((x + 1, y)) + 8) // 17) if (x + 1 < w_cell) else 0
                data_bytes.append((a0 << 4) | a1)

        # Format byte array
        byte_chunks = []
        bytes_per_row = (w_cell + 1) // 2
        for r in range(H):
            row_slice = data_bytes[r * bytes_per_row : (r + 1) * bytes_per_row]
            row_str = ", ".join(f"0x{b:02X}" for b in row_slice)
            byte_chunks.append(f"    {row_str},")

        joined_bytes = "\n".join(byte_chunks)
        array_name = f"DATA_48_{name}"
        total_len = len(data_bytes)
        code_sections.append(f"static {array_name}: [u8; {total_len}] = [\n{joined_bytes}\n];")
        glyph_defs.append(f"static GLYPH_48_{name}: Glyph48 = Glyph48 {{ width: {w_cell}, data: &{array_name} }};")

    data_code = "\n\n".join(code_sections)
    defs_code = "\n".join(glyph_defs)

    content = f"""use crate::traits::{{Font, GlyphLayout}};

/// 16-entry blend table mapping 4-bit alpha levels (0..15) from background (0) to foreground (15) in RGB565.
#[inline(always)]
pub fn blend_lut(fg: u16, bg: u16) -> [u16; 16] {{
    let fg_r = ((fg >> 11) & 0x1F) as u32;
    let fg_g = ((fg >> 5) & 0x3F) as u32;
    let fg_b = (fg & 0x1F) as u32;

    let bg_r = ((bg >> 11) & 0x1F) as u32;
    let bg_g = ((bg >> 5) & 0x3F) as u32;
    let bg_b = (bg & 0x1F) as u32;

    let mut lut = [0u16; 16];
    let mut a = 0u32;
    while a < 16 {{
        let inv = 15 - a;
        let r = ((bg_r * inv + fg_r * a + 7) / 15) as u16;
        let g = ((bg_g * inv + fg_g * a + 7) / 15) as u16;
        let b = ((bg_b * inv + fg_b * a + 7) / 15) as u16;
        lut[a as usize] = (r << 11) | (g << 5) | b;
        a += 1;
    }}
    lut
}}

/// 48px Smooth TrueType Anti-Aliased Glyph (4-bit alpha per pixel)
#[derive(Copy, Clone, Debug)]
pub struct Glyph48 {{
    pub width: u8,
    pub data: &'static [u8],
}}

/// 48px Precision Technical Bench-Meter Font
/// Rasterized directly from Bahnschrift SemiCondensed TrueType vector contours with 1:1 hardware pixel curves.
/// Features tabular fixed-width numerals (28px) for zero-jitter bench-top display.
/// Supported characters: '0'..'9', '-', '.', ' '
#[derive(Copy, Clone, Debug, Default)]
pub struct Font48;

pub static FONT_48: Font48 = Font48;

impl Font48 {{
    pub const HEIGHT: u8 = 48;
    pub const DIGIT_WIDTH: u8 = 28;
    pub const DOT_WIDTH: u8 = 14;
    pub const LAYOUT: GlyphLayout = GlyphLayout::RowMajorNibble;

    /// Looks up glyph for character c
    pub fn glyph(&self, c: char) -> Option<&'static Glyph48> {{
        get_glyph_48(c)
    }}

    /// Calculates total rendered pixel width of a string in 48px font
    pub fn string_width(&self, text: &str) -> u16 {{
        string_width_48(text)
    }}

    /// Checks if a pixel at (px, py) is set (alpha >= 50%) for the given glyph
    pub fn is_pixel_set(&self, glyph: &Glyph48, px: u8, py: u8) -> bool {{
        if px < glyph.width && py < 48 {{
            let row_bytes = ((glyph.width + 1) / 2) as usize;
            let byte_idx = (py as usize) * row_bytes + (px as usize / 2);
            let b = glyph.data[byte_idx];
            let a = if (px & 1) == 0 {{ b >> 4 }} else {{ b & 0x0F }};
            a >= 8
        }} else {{
            false
        }}
    }}
}}

impl Font for Font48 {{
    #[inline(always)]
    fn width(&self) -> u8 {{
        28 // Tabular digit width
    }}

    #[inline(always)]
    fn height(&self) -> u8 {{
        48
    }}

    #[inline(always)]
    fn spacing(&self) -> u8 {{
        0
    }}

    #[inline(always)]
    fn contains(&self, c: char) -> bool {{
        get_glyph_48(c).is_some()
    }}

    #[inline(always)]
    fn char_width(&self, c: char) -> u8 {{
        get_glyph_48(c).map(|g| g.width).unwrap_or(0)
    }}

    #[inline(always)]
    fn string_width(&self, text: &str) -> u16 {{
        string_width_48(text)
    }}

    fn render_char<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        mut put_pixel: P,
    ) -> Option<u8> {{
        let glyph = get_glyph_48(c)?;
        let w = glyph.width;
        let lut = blend_lut(fg, bg);
        let row_bytes = ((w + 1) / 2) as usize;

        for row in 0..48 {{
            let row_offset = row * row_bytes;
            let mut col = 0;
            let mut byte_idx = 0;
            while col < w {{
                let b = glyph.data[row_offset + byte_idx];
                byte_idx += 1;
                let a0 = (b >> 4) as usize;
                put_pixel(lut[a0]);
                col += 1;
                if col < w {{
                    let a1 = (b & 0x0F) as usize;
                    put_pixel(lut[a1]);
                    col += 1;
                }}
            }}
        }}
        Some(w)
    }}

    fn render_char_scaled<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        scale: u8,
        mut put_pixel: P,
    ) -> Option<u8> {{
        let glyph = get_glyph_48(c)?;
        let w = glyph.width;
        let scale = scale.max(1);
        let lut = blend_lut(fg, bg);
        let row_bytes = ((w + 1) / 2) as usize;

        for row in 0..48 {{
            let row_offset = row * row_bytes;
            for _ in 0..scale {{
                let mut col = 0;
                let mut byte_idx = 0;
                while col < w {{
                    let b = glyph.data[row_offset + byte_idx];
                    byte_idx += 1;
                    let a0 = (b >> 4) as usize;
                    let col0 = lut[a0];
                    for _ in 0..scale {{
                        put_pixel(col0);
                    }}
                    col += 1;
                    if col < w {{
                        let a1 = (b & 0x0F) as usize;
                        let col1 = lut[a1];
                        for _ in 0..scale {{
                            put_pixel(col1);
                        }}
                        col += 1;
                    }}
                }}
            }}
        }}
        Some(w * scale)
    }}
}}

/// Retrieves the static 48px glyph definition for character c
pub fn get_glyph_48(c: char) -> Option<&'static Glyph48> {{
    match c {{
        '0' => Some(&GLYPH_48_0),
        '1' => Some(&GLYPH_48_1),
        '2' => Some(&GLYPH_48_2),
        '3' => Some(&GLYPH_48_3),
        '4' => Some(&GLYPH_48_4),
        '5' => Some(&GLYPH_48_5),
        '6' => Some(&GLYPH_48_6),
        '7' => Some(&GLYPH_48_7),
        '8' => Some(&GLYPH_48_8),
        '9' => Some(&GLYPH_48_9),
        '-' => Some(&GLYPH_48_MINUS),
        '.' => Some(&GLYPH_48_DOT),
        ' ' => Some(&GLYPH_48_SPACE),
        _ => None,
    }}
}}

/// Computes the rendered pixel width of a string formatted in 48px font
pub fn string_width_48(text: &str) -> u16 {{
    let mut total = 0u16;
    for c in text.chars() {{
        if let Some(glyph) = get_glyph_48(c) {{
            total += glyph.width as u16;
        }}
    }}
    total
}}

{data_code}

{defs_code}
"""

    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target_file = os.path.join(root, "crates", "lcd-font", "src", "font48.rs")
    with open(target_file, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Generated {target_file} successfully.")

if __name__ == "__main__":
    generate_font48()
