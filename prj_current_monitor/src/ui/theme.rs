/// 16-bit RGB565 color constructor
pub const fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | ((b as u16) >> 3)
}

// Palette: Deep Slate & Neon Accents
pub const COL_BG_TOP: u16    = rgb565(14, 22, 36);   // Deep Slate Navy
pub const COL_DIVIDER: u16   = rgb565(35, 60, 90);   // Header divider
pub const COL_GRAPH_BG: u16  = rgb565(2, 4, 8);      // Near pitch black
pub const COL_GRID: u16      = rgb565(22, 34, 48);   // Subtle grid dot
pub const COL_AXIS: u16      = rgb565(45, 65, 95);   // Axis border
pub const COL_CYAN: u16      = rgb565(0, 240, 255);  // Neon Cyan
#[allow(dead_code)]
pub const COL_FILL_CYAN: u16 = rgb565(0, 24, 38);    // Dark Cyan glow fill
pub const COL_AMBER: u16     = rgb565(255, 170, 30); // Bright Amber
pub const COL_MINT: u16      = rgb565(90, 240, 150); // Fresh Mint Green
pub const COL_RED: u16       = rgb565(255, 70, 70);  // Vivid Red
pub const COL_CARD_BG: u16   = 0x0944;               // Dark tile background
pub const COL_CARD_BORDER: u16 = 0x1AE7;             // Tile border line
pub const COL_WHITE: u16     = 0xFFFF;
pub const COL_BLACK: u16     = 0x0000;
pub const COL_TEXT_MUTED: u16 = rgb565(140, 160, 185);

// 8x8 USB Plug Icon
pub const ICON_USB: [u8; 8] = [0x3C, 0x5A, 0x42, 0xFF, 0xFF, 0x7E, 0x3C, 0x18];

// 8x8 MicroSD Card Icon
pub const ICON_SD: [u8; 8] = [0x7E, 0x7E, 0x3E, 0x3E, 0x7E, 0x7E, 0x54, 0x00];

// 8x8 Recording Dot Icon
pub const ICON_REC: [u8; 8] = [0x00, 0x18, 0x3C, 0x7E, 0x7E, 0x3C, 0x18, 0x00];
