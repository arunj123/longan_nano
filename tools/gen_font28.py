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

methods = """
    /// Draws a character using the 28px smooth bold numeric font.
    pub fn draw_char_28(&mut self, x: u16, y: u16, glyph: &Glyph28, fg: u16, bg: u16) {
        let w = glyph.width as u16;
        let h = 28u16;
        if x + w > LCD_WIDTH || y + h > LCD_HEIGHT {
            return;
        }

        self.set_address_window(x, y, w, h);
        self.spi.wait_idle();
        self.spi.set_16bit();
        self.mode_data();

        for row in 0..28 {
            let row_bits = glyph.rows[row];
            for col in 0..w {
                let set = (row_bits & (1 << (31 - col))) != 0;
                self.spi.send_u16(if set { fg } else { bg });
            }
        }
    }

    /// Calculates the total rendered pixel width of a string in 28px font.
    pub fn string_width_28(&self, text: &str) -> u16 {
        let mut total = 0u16;
        for c in text.chars() {
            if let Some(glyph) = get_glyph_28(c) {
                total += glyph.width as u16;
            }
        }
        total
    }

    /// Draws a text string horizontally using the 28px smooth bold numeric font.
    pub fn draw_string_28(&mut self, mut x: u16, y: u16, text: &str, fg: u16, bg: u16) {
        for c in text.chars() {
            if let Some(glyph) = get_glyph_28(c) {
                let w = glyph.width as u16;
                if x + w > LCD_WIDTH {
                    break;
                }
                self.draw_char_28(x, y, glyph, fg, bg);
                x += w;
            }
        }
    }
"""

struct_def = """
#[derive(Copy, Clone)]
pub struct Glyph28 {
    pub width: u8,
    pub rows: [u32; 28],
}

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
""" + "\n".join(code_lines) + "\n"

target_file = r"crates\longan-nano-bsp\src\lcd.rs"
with open(target_file, "r") as f:
    content = f.read()

target = "// 5x7 ASCII Bitmap Font Table"
parts = content.split(target)

new_content = parts[0].rstrip()[:-1].rstrip() + "\n" + methods + "}\n\n" + target + parts[1] + "\n\n// 28px Smooth Bold Numeric Font\n" + struct_def

with open(target_file, "w") as f:
    f.write(new_content)

print("Updated crates/longan-nano-bsp/src/lcd.rs successfully with 28px font")
