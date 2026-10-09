from PIL import ImageFont, Image, ImageDraw
import os

font_path = os.path.join(os.environ["WINDIR"], "Fonts", "bahnschrift.ttf")
font = ImageFont.truetype(font_path, 34)

char_map = [
    ("0", "0"), ("1", "1"), ("2", "2"), ("3", "3"), ("4", "4"),
    ("5", "5"), ("6", "6"), ("7", "7"), ("8", "8"), ("9", "9"),
    ("-", "MINUS"), (".", "DOT"), (" ", "SPACE"), ("m", "M"), ("A", "A")
]

H = 28
code_lines = []

for ch, name in char_map:
    w = int(font.getlength(ch))
    if ch == " ":
        w = 9
    im = Image.new("1", (w, H), 0)
    draw = ImageDraw.Draw(im)
    draw.text((0, 0), ch, font=font, fill=1)
    
    rows = []
    for y in range(H):
        row_val = 0
        for x in range(w):
            if im.getpixel((x, y)):
                row_val |= (1 << (31 - x))
        rows.append(f"0x{row_val:08X}")
    
    rows_str = ", ".join(rows)
    code_lines.append(f"static GLYPH_28_{name}: Glyph28 = Glyph28 {{ width: {w}, rows: [{rows_str}] }};")

glyphs_str = "\n".join(code_lines) + "\n"

content = f"""use crate::traits::{{Font, GlyphLayout}};

/// 28px Smooth TrueType-Rasterized Font Glyph (variable/proportional width)
#[derive(Copy, Clone, Debug)]
pub struct Glyph28 {{
    pub width: u8,
    pub rows: [u32; 28],
}}

/// 28px Smooth Bold Numeric & Sensor Font
/// Rasterized directly from Bahnschrift/DIN TrueType vector contours with 1:1 hardware pixel curves.
/// Supported characters: '0'..'9', '-', '.', ' ', 'm', 'A'
#[derive(Copy, Clone, Debug, Default)]
pub struct Font28;

pub static FONT_28: Font28 = Font28;

impl Font28 {{
    pub const HEIGHT: u8 = 28;
    pub const LAYOUT: GlyphLayout = GlyphLayout::RowMajorMsbU32;

    /// Looks up glyph for character `c`
    pub fn glyph(&self, c: char) -> Option<&'static Glyph28> {{
        get_glyph_28(c)
    }}

    /// Calculates total rendered pixel width of a string in 28px font
    pub fn string_width(&self, text: &str) -> u16 {{
        string_width_28(text)
    }}

    /// Checks if a pixel at (px, py) is set for the given glyph
    pub fn is_pixel_set(&self, glyph: &Glyph28, px: u8, py: u8) -> bool {{
        if px < glyph.width && py < 28 {{
            (glyph.rows[py as usize] & (1 << (31 - px))) != 0
        }} else {{
            false
        }}
    }}
}}

impl Font for Font28 {{
    #[inline(always)]
    fn width(&self) -> u8 {{
        29 // Maximum proportional glyph width ('m')
    }}

    #[inline(always)]
    fn height(&self) -> u8 {{
        28
    }}

    #[inline(always)]
    fn spacing(&self) -> u8 {{
        0
    }}

    #[inline(always)]
    fn contains(&self, c: char) -> bool {{
        get_glyph_28(c).is_some()
    }}

    #[inline(always)]
    fn char_width(&self, c: char) -> u8 {{
        get_glyph_28(c).map(|g| g.width).unwrap_or(0)
    }}

    #[inline(always)]
    fn string_width(&self, text: &str) -> u16 {{
        string_width_28(text)
    }}

    fn render_char<P: FnMut(u16)>(
        &self,
        c: char,
        fg: u16,
        bg: u16,
        mut put_pixel: P,
    ) -> Option<u8> {{
        let glyph = get_glyph_28(c)?;
        let w = glyph.width;
        for row in 0..28 {{
            let row_bits = glyph.rows[row];
            for col in 0..w {{
                let set = (row_bits & (1 << (31 - col))) != 0;
                put_pixel(if set {{ fg }} else {{ bg }});
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
        let glyph = get_glyph_28(c)?;
        let w = glyph.width;
        let scale = scale.max(1);
        for row in 0..28 {{
            let row_bits = glyph.rows[row];
            for _ in 0..scale {{
                for col in 0..w {{
                    let set = (row_bits & (1 << (31 - col))) != 0;
                    let color = if set {{ fg }} else {{ bg }};
                    for _ in 0..scale {{
                        put_pixel(color);
                    }}
                }}
            }}
        }}
        Some(w * scale)
    }}
}}

/// Retrieves the static 28px glyph definition for character `c`
pub fn get_glyph_28(c: char) -> Option<&'static Glyph28> {{
    match c {{
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
    }}
}}

/// Computes the rendered pixel width of a string formatted in 28px font
pub fn string_width_28(text: &str) -> u16 {{
    let mut total = 0u16;
    for c in text.chars() {{
        if let Some(glyph) = get_glyph_28(c) {{
            total += glyph.width as u16;
        }}
    }}
    total
}}

{glyphs_str}"""

target_file = os.path.join(os.path.dirname(os.path.dirname(__file__)), "crates", "lcd-font", "src", "font28.rs")
with open(target_file, "w", encoding="utf-8") as f:
    f.write(content)

print(f"Updated {target_file} successfully with 28px font")
