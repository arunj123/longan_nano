#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use longan_nano_bsp::lcd::FONT_5X7;
use longan_nano_bsp::lcd_color;
use longan_nano_bsp::{Board, LCD_HEIGHT, LCD_WIDTH};
use panic_halt as _;
use riscv_rt::entry;

// Small stack-allocated string buffer for formatting numbers without heap
struct StrBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StrBuf<N> {
    fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Write for StrBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = N - self.len;
        let to_copy = bytes.len().min(remaining);
        self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    // Turn on Green LED
    board.led_green.on();

    board.delay.delay_ms(100);

    let _ = writeln!(
        board.uart0,
        "\r\n==================================================\r\n\
         Longan Nano -- Pure Rust ST7735 LCD Test\r\n\
         CPU: GD32VF103 RV32IMAC @ 108 MHz\r\n\
         Standard: Rust 2021 (#![no_std], -Os, lto)\r\n\
         LCD Controller: ST7735 (160x80) via SPI0\r\n\
         Debug UART0: 115200 baud (PA9 TX, PA10 RX)\r\n\
         ==================================================\r\n"
    );

    // Initialize ST7735 LCD
    board.lcd.init(&mut board.delay);

    // Initial Test Pattern matching C++ prj_lcd_test
    board.lcd.clear(lcd_color::DARK_NAVY);

    // Outer border frame
    board.lcd.rect(0, 0, LCD_WIDTH, LCD_HEIGHT, lcd_color::CYAN);

    // Header bar
    board.lcd.fill_rect(1, 1, LCD_WIDTH - 2, 13, lcd_color::GRAY);
    board.lcd.draw_string(
        8,
        4,
        "LONGAN NANO ST7735",
        &FONT_5X7,
        lcd_color::YELLOW,
        lcd_color::GRAY,
    );

    // Subtitles
    board.lcd.draw_string(
        6,
        20,
        "RV32IMAC @ 108MHz",
        &FONT_5X7,
        lcd_color::WHITE,
        lcd_color::DARK_NAVY,
    );
    board.lcd.draw_string(
        6,
        32,
        "Pure Rust Embedded HAL",
        &FONT_5X7,
        lcd_color::GREEN,
        lcd_color::DARK_NAVY,
    );

    // Color Swatches at bottom (8 colors)
    let swatches = [
        lcd_color::RED,
        lcd_color::GREEN,
        lcd_color::BLUE,
        lcd_color::YELLOW,
        lcd_color::CYAN,
        lcd_color::MAGENTA,
        lcd_color::WHITE,
        lcd_color::GRAY,
    ];
    let swatch_w = 18u16;
    let swatch_h = 10u16;
    let swatch_y = 66u16;

    for (i, &color) in swatches.iter().enumerate() {
        let swatch_x = 8 + (i as u16) * (swatch_w + 1);
        board.lcd.fill_rect(swatch_x, swatch_y, swatch_w, swatch_h, color);
        board.lcd.rect(swatch_x, swatch_y, swatch_w, swatch_h, lcd_color::BLACK);
    }

    let _ = writeln!(board.uart0, "LCD initialized and test pattern rendered.");

    let mut heartbeat_count: u32 = 0;
    let mut last_button_state = false;

    loop {
        heartbeat_count = heartbeat_count.wrapping_add(1);
        let ticks = board.delay.get_raw_ticks();

        // Toggle Green LED
        board.led_green.toggle();

        // Update live tick counter on LCD
        let mut buf = StrBuf::<32>::new();
        let _ = write!(buf, "Tick: {:05} s", heartbeat_count / 2);
        board.lcd.draw_string(
            6,
            48,
            buf.as_str(),
            &FONT_5X7,
            lcd_color::CYAN,
            lcd_color::DARK_NAVY,
        );

        // Log heartbeat to UART0
        let _ = writeln!(
            board.uart0,
            "[LCD TEST #{:04}] Running. System ticks: {}",
            heartbeat_count,
            ticks as u32
        );

        // Check user button (PA8)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !last_button_state {
            let _ = writeln!(board.uart0, ">>> User Button Pressed! <<<");
        } else if !button_pressed && last_button_state {
            let _ = writeln!(board.uart0, ">>> User Button Released! <<<");
        }
        last_button_state = button_pressed;

        board.delay.delay_ms(500);
    }
}
