use gd32vf103_hal::delay::Delay;
use gd32vf103_hal::gpio::{mode, Pin, PortB};
use gd32vf103_hal::spi::Spi0;
use embedded_hal::delay::DelayNs;
pub use lcd_font::*;

pub const LCD_WIDTH: u16 = 160;
pub const LCD_HEIGHT: u16 = 80;

pub mod color {
    pub const BLACK: u16     = 0x0000;
    pub const WHITE: u16     = 0xFFFF;
    pub const RED: u16       = 0xF800;
    pub const GREEN: u16     = 0x07E0;
    pub const BLUE: u16      = 0x001F;
    pub const YELLOW: u16    = 0xFFE0;
    pub const CYAN: u16      = 0x07FF;
    pub const MAGENTA: u16   = 0xF81F;
    pub const GRAY: u16      = 0x8410;
    pub const DARK_NAVY: u16 = 0x000F;
}

pub struct Lcd {
    spi: Spi0,
    cs: Pin<PortB, 2, mode::Output<mode::PushPull>>,
    dc: Pin<PortB, 0, mode::Output<mode::PushPull>>,
    rst: Pin<PortB, 1, mode::Output<mode::PushPull>>,
}

impl Lcd {
    pub fn new(
        spi: Spi0,
        cs: Pin<PortB, 2, mode::Output<mode::PushPull>>,
        dc: Pin<PortB, 0, mode::Output<mode::PushPull>>,
        rst: Pin<PortB, 1, mode::Output<mode::PushPull>>,
    ) -> Self {
        Self { spi, cs, dc, rst }
    }

    #[inline(always)]
    fn mode_cmd(&mut self) {
        use embedded_hal::digital::OutputPin;
        let _ = self.dc.set_low();
    }

    #[inline(always)]
    fn mode_data(&mut self) {
        use embedded_hal::digital::OutputPin;
        let _ = self.dc.set_high();
    }

    #[inline(always)]
    fn cs_enable(&mut self) {
        use embedded_hal::digital::OutputPin;
        let _ = self.cs.set_low();
    }

    #[inline(always)]
    fn cs_disable(&mut self) {
        use embedded_hal::digital::OutputPin;
        let _ = self.cs.set_high();
    }

    fn write_cmd(&mut self, cmd: u8) {
        self.spi.wait_idle();
        self.spi.set_8bit();
        self.mode_cmd();
        self.spi.send_u8(cmd);
    }

    #[allow(dead_code)]
    fn write_data_u8(&mut self, data: u8) {
        self.spi.wait_idle();
        self.spi.set_8bit();
        self.mode_data();
        self.spi.send_u8(data);
    }

    fn write_data_u16(&mut self, data: u16) {
        self.spi.wait_idle();
        self.spi.set_16bit();
        self.mode_data();
        self.spi.send_u16(data);
    }

    /// Initializes the ST7735 controller for 160x80 landscape display.
    pub fn init(&mut self, delay: &mut Delay) {
        use embedded_hal::digital::OutputPin;

        self.mode_cmd();
        let _ = self.rst.set_low();
        self.cs_disable();

        // Hardware reset pulse
        delay.delay_ms(1);
        let _ = self.rst.set_high();
        delay.delay_ms(5);

        self.cs_enable();

        // ST7735 Initialization sequence
        let init_cmds: &[&[u8]] = &[
            &[0x21],                                     // Display Inversion ON
            &[0xb1, 0x05, 0x3a, 0x3a],                   // Frame Rate (Normal)
            &[0xb2, 0x05, 0x3a, 0x3a],                   // Frame Rate (Idle)
            &[0xb3, 0x05, 0x3a, 0x3a, 0x05, 0x3a, 0x3a], // Frame Rate (Partial)
            &[0xb4, 0x03],                               // Inversion Control
            &[0xc0, 0x62, 0x02, 0x04],                   // Power Control 1
            &[0xc1, 0xc0],                               // Power Control 2
            &[0xc2, 0x0d, 0x00],                         // Power Control 3
            &[0xc3, 0x8d, 0x6a],                         // Power Control 4
            &[0xc4, 0x8d, 0xee],                         // Power Control 5
            &[0xc5, 0x0e],                               // VCOM Control 1
            &[0xe0, 0x10, 0x0e, 0x02, 0x03, 0x0e, 0x07, 0x02, 0x07, 0x0a, 0x12, 0x27, 0x37, 0x00, 0x0d, 0x0e, 0x10], // Gamma '+'
            &[0xe1, 0x10, 0x0e, 0x03, 0x03, 0x0f, 0x06, 0x02, 0x08, 0x0a, 0x13, 0x26, 0x36, 0x00, 0x0d, 0x0e, 0x10], // Gamma '-'
            &[0x3a, 0x55],                               // Pixel format: RGB565 (16-bit)
            &[0x36, 0x78],                               // Orientation (Landscape 160x80)
            &[0x29],                                     // Display ON
            &[0x11],                                     // Sleep OUT
        ];

        for cmd in init_cmds {
            self.write_cmd(cmd[0]);
            if cmd.len() > 1 {
                self.spi.wait_idle();
                self.mode_data();
                for &b in &cmd[1..] {
                    self.spi.send_u8(b);
                }
            }
        }

        delay.delay_ms(120);
        self.clear(color::BLACK);
    }

    /// Set address window with Longan Nano ST7735 panel offsets (X+1, Y+26).
    pub fn set_address_window(&mut self, x: u16, y: u16, w: u16, h: u16) {
        let x_start = x + 1;
        let x_end = x + w;
        let y_start = y + 26;
        let y_end = y + h + 25;

        self.write_cmd(0x2a); // CASET
        self.write_data_u16(x_start);
        self.spi.send_u16(x_end);

        self.write_cmd(0x2b); // RASET
        self.write_data_u16(y_start);
        self.spi.send_u16(y_end);

        self.write_cmd(0x2c); // RAMWR
    }

    /// Fills a rectangular region with a solid 16-bit RGB565 color.
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16) {
        if x >= LCD_WIDTH || y >= LCD_HEIGHT || w == 0 || h == 0 {
            return;
        }
        let w = if x + w > LCD_WIDTH { LCD_WIDTH - x } else { w };
        let h = if y + h > LCD_HEIGHT { LCD_HEIGHT - y } else { h };

        self.set_address_window(x, y, w, h);

        self.spi.wait_idle();
        self.spi.set_16bit();
        self.mode_data();

        let total = (w as u32) * (h as u32);
        for _ in 0..total {
            self.spi.send_u16(color);
        }
    }

    /// Writes a slice of 16-bit RGB565 pixel values into a bounding window.
    pub fn write_pixels(&mut self, x: u16, y: u16, w: u16, h: u16, pixels: &[u16]) {
        if x >= LCD_WIDTH || y >= LCD_HEIGHT || w == 0 || h == 0 {
            return;
        }
        let w = if x + w > LCD_WIDTH { LCD_WIDTH - x } else { w };
        let h = if y + h > LCD_HEIGHT { LCD_HEIGHT - y } else { h };

        self.set_address_window(x, y, w, h);

        self.spi.wait_idle();
        self.spi.set_16bit();
        self.mode_data();

        let count = core::cmp::min(pixels.len(), (w as usize) * (h as usize));
        for &c in &pixels[..count] {
            self.spi.send_u16(c);
        }
    }

    /// Draws an outline rectangle.
    pub fn rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16) {
        if w == 0 || h == 0 { return; }
        self.fill_rect(x, y, w, 1, color);             // Top
        self.fill_rect(x, y + h - 1, w, 1, color);     // Bottom
        self.fill_rect(x, y, 1, h, color);             // Left
        self.fill_rect(x + w - 1, y, 1, h, color);     // Right
    }

    /// Sets a single pixel.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) {
        if x < LCD_WIDTH && y < LCD_HEIGHT {
            self.fill_rect(x, y, 1, 1, color);
        }
    }

    /// Draws a horizontal line.
    #[inline]
    pub fn draw_hline(&mut self, x: u16, y: u16, len: u16, color: u16) {
        self.fill_rect(x, y, len, 1, color);
    }

    /// Draws a vertical line.
    #[inline]
    pub fn draw_vline(&mut self, x: u16, y: u16, len: u16, color: u16) {
        self.fill_rect(x, y, 1, len, color);
    }

    /// Clears the entire display to a single color.
    pub fn clear(&mut self, color: u16) {
        self.fill_rect(0, 0, LCD_WIDTH, LCD_HEIGHT, color);
    }

    /// Draws a 1-bit monochrome 8x8 bitmap with foreground and background colors.
    pub fn draw_bitmap_8x8(&mut self, x: u16, y: u16, bitmap: &[u8; 8], fg: u16, bg: u16) {
        if x + 8 > LCD_WIDTH || y + 8 > LCD_HEIGHT {
            return;
        }
        self.set_address_window(x, y, 8, 8);
        self.spi.wait_idle();
        self.spi.set_16bit();
        self.mode_data();

        for &row_bits in bitmap {
            let mut mask = 0x80u8;
            while mask != 0 {
                let color = if (row_bits & mask) != 0 { fg } else { bg };
                self.spi.send_u16(color);
                mask >>= 1;
            }
        }
    }
    /// Draws a character using any font implementing `Font`.
    pub fn draw_char<F: Font>(&mut self, x: u16, y: u16, c: char, font: &F, fg: u16, bg: u16) -> Option<u8> {
        let w = font.char_width(c) as u16;
        let h = font.height() as u16;
        if w == 0 || x + w > LCD_WIDTH || y + h > LCD_HEIGHT {
            return None;
        }

        self.set_address_window(x, y, w, h);
        self.spi.wait_idle();
        self.spi.set_16bit();
        self.mode_data();

        font.render_char(c, fg, bg, |color| self.spi.send_u16(color))
    }

    /// Draws a character using any font implementing `Font` with integer scaling.
    pub fn draw_char_scaled<F: Font>(&mut self, x: u16, y: u16, c: char, font: &F, fg: u16, bg: u16, scale: u8) -> Option<u8> {
        let scale = scale.max(1);
        let w = (font.char_width(c) as u16) * (scale as u16);
        let h = (font.height() as u16) * (scale as u16);
        if w == 0 || x + w > LCD_WIDTH || y + h > LCD_HEIGHT {
            return None;
        }

        self.set_address_window(x, y, w, h);
        self.spi.wait_idle();
        self.spi.set_16bit();
        self.mode_data();

        font.render_char_scaled(c, fg, bg, scale, |color| self.spi.send_u16(color))
    }

    /// Draws a text string horizontally using any font implementing `Font`.
    pub fn draw_string<F: Font>(&mut self, mut x: u16, y: u16, text: &str, font: &F, fg: u16, bg: u16) {
        for c in text.chars() {
            let adv = font.char_width(c) as u16;
            if x + adv > LCD_WIDTH {
                break;
            }
            self.draw_char(x, y, c, font, fg, bg);
            x += adv;
        }
    }

    /// Draws a text string horizontally using any font implementing `Font` with integer scaling.
    pub fn draw_string_scaled<F: Font>(&mut self, mut x: u16, y: u16, text: &str, font: &F, fg: u16, bg: u16, scale: u8) {
        let scale = scale.max(1);
        for c in text.chars() {
            let adv = (font.char_width(c) as u16) * (scale as u16);
            if x + adv > LCD_WIDTH {
                break;
            }
            self.draw_char_scaled(x, y, c, font, fg, bg, scale);
            x += adv;
        }
    }
}
