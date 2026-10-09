#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use longan_nano_bsp::lcd::FONT_5X7;
use longan_nano_bsp::lcd_color;
use longan_nano_bsp::{Board, CardType, LCD_HEIGHT, LCD_WIDTH};
use panic_halt as _;
use riscv_rt::entry;

// Stack-allocated string buffer for formatting without heap
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

fn print_hexdump(uart: &mut gd32vf103_hal::Uart0, data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = (i * 16) as u32;
        let _ = write!(uart, "  {:04X}:  ", offset);
        for (j, &b) in chunk.iter().enumerate() {
            let _ = write!(uart, "{:02X} ", b);
            if j == 7 {
                let _ = write!(uart, " ");
            }
        }
        for _ in chunk.len()..16 {
            let _ = write!(uart, "   ");
        }
        let _ = write!(uart, " |");
        for &b in chunk {
            let c = if (32..=126).contains(&b) { b as char } else { '.' };
            let _ = write!(uart, "{}", c);
        }
        let _ = writeln!(uart, "|");
    }
}

fn run_sd_test(board: &mut Board) {
    let _ = writeln!(board.uart0, "\r\n>>> STARTING SD CARD DIAGNOSTIC SEQUENCE <<<");

    // Clear display area below header
    board.lcd.fill_rect(0, 16, LCD_WIDTH, LCD_HEIGHT - 16, lcd_color::BLACK);
    board.lcd.draw_string(4, 18, "Probing SD card...", &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);

    board.led_red.off();
    board.led_green.off();
    board.led_blue.on(); // Blue = Testing

    match board.sdcard.init(&mut board.delay) {
        Ok(card_type) => {
            board.led_blue.off();
            board.led_green.on(); // Green = Success

            board.lcd.draw_string(4, 18, "SD Init: SUCCESS! ", &FONT_5X7, lcd_color::GREEN, lcd_color::BLACK);

            let type_str = match card_type {
                CardType::SD2HC => "Type: SDHC/SDXC",
                CardType::SD2SC => "Type: SDSC (v2)",
                CardType::SD1 => "Type: SDSC (v1)",
                CardType::Unknown => "Type: Unknown",
            };
            board.lcd.draw_string(4, 30, type_str, &FONT_5X7, lcd_color::WHITE, lcd_color::BLACK);

            let _ = writeln!(board.uart0, "[SD:OK] SD Card Initialized Successfully!");
            let _ = writeln!(board.uart0, "        Card Type: {}", card_type.as_str());
            let _ = writeln!(
                board.uart0,
                "        Sectors  : {} ({} MB)",
                board.sdcard.sector_count,
                board.sdcard.sector_count / 2048
            );

            board.lcd.draw_string(4, 42, "Reading Sector 0...", &FONT_5X7, lcd_color::CYAN, lcd_color::BLACK);
            let _ = writeln!(board.uart0, "[TEST] Reading Sector 0 (Master Boot Record / Boot Sector)...");

            let mut sector_buf = [0u8; 512];
            match board.sdcard.read_sector(0, &mut sector_buf) {
                Ok(()) => {
                    let _ = writeln!(board.uart0, "[TEST] Sector 0 read successfully! Printing Hexdump (512 bytes):");
                    print_hexdump(&mut board.uart0, &sector_buf);

                    let sig = ((sector_buf[510] as u16) << 8) | (sector_buf[511] as u16);
                    let _ = writeln!(board.uart0, "\r\n[CHECK] Boot Record Signature: 0x{:04X} (Expected: 0x55AA)", sig);

                    if sector_buf[510] == 0x55 && sector_buf[511] == 0xAA {
                        let _ = writeln!(board.uart0, "[SUCCESS] >>> 0x55AA VALID BOOT RECORD SIGNATURE CONFIRMED! <<<");
                        board.lcd.draw_string(4, 42, "Sector 0: 0x55AA OK! ", &FONT_5X7, lcd_color::GREEN, lcd_color::BLACK);

                        let mut cap_buf = StrBuf::<32>::new();
                        let _ = write!(cap_buf, "Cap: {} MB (OK)", board.sdcard.sector_count / 2048);
                        board.lcd.draw_string(4, 54, cap_buf.as_str(), &FONT_5X7, lcd_color::GREEN, lcd_color::BLACK);

                        let part_type = sector_buf[446 + 4];
                        let _ = writeln!(board.uart0, "          Partition 1 Type: 0x{:02X}", part_type);
                    } else {
                        let _ = writeln!(board.uart0, "[INFO] Sector 0 read OK (Unpartitioned / Non-MBR format, Sig: 0x{:04X})", sig);
                        board.lcd.draw_string(4, 42, "Sector 0: Read OK    ", &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);

                        let mut cap_buf = StrBuf::<32>::new();
                        let _ = write!(cap_buf, "Cap: {} MB (OK)", board.sdcard.sector_count / 2048);
                        board.lcd.draw_string(4, 54, cap_buf.as_str(), &FONT_5X7, lcd_color::WHITE, lcd_color::BLACK);
                    }
                }
                Err(err) => {
                    let _ = writeln!(board.uart0, "[ERROR] Sector 0 read failed: {}", err.as_str());
                    board.lcd.draw_string(4, 42, "Read Sec 0: FAILED", &FONT_5X7, lcd_color::RED, lcd_color::BLACK);
                }
            }
        }
        Err(err) => {
            board.led_blue.off();
            board.led_red.on(); // Red = Error

            let _ = writeln!(board.uart0, "[SD:FAIL] SD Card Probe Failed: {}", err.as_str());

            board.lcd.draw_string(4, 18, "SD Init: FAILED!   ", &FONT_5X7, lcd_color::RED, lcd_color::BLACK);
            board.lcd.draw_string(4, 30, err.as_str(), &FONT_5X7, lcd_color::RED, lcd_color::BLACK);
            board.lcd.draw_string(4, 44, "Check TF card seating", &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);
            board.lcd.draw_string(4, 56, "Press PA8 to retry", &FONT_5X7, lcd_color::WHITE, lcd_color::BLACK);
        }
    }
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    // Settle delay
    board.delay.delay_ms(100);

    // Title banner on LCD
    board.lcd.init(&mut board.delay);
    board.lcd.clear(lcd_color::BLACK);
    board.lcd.fill_rect(0, 0, LCD_WIDTH, 14, lcd_color::DARK_NAVY);
    board.lcd.draw_string(4, 3, "LONGAN NANO: SD TEST", &FONT_5X7, lcd_color::CYAN, lcd_color::DARK_NAVY);

    let _ = writeln!(board.uart0, "\r\n==================================================");
    let _ = writeln!(board.uart0, "Longan Nano -- Pure Rust Embedded SD Card Engine");
    let _ = writeln!(board.uart0, "CPU: GD32VF103 RV32IMAC @ 108 MHz");
    let _ = writeln!(board.uart0, "Standard: Rust 2021 (#![no_std], -Os, lto)");
    let _ = writeln!(board.uart0, "Interface: SPI1 (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI)");
    let _ = writeln!(board.uart0, "User Button: PA8 (Press to re-run test at any time)");
    let _ = writeln!(board.uart0, "==================================================");

    // Run first test immediately
    run_sd_test(&mut board);

    let mut loop_counter: u32 = 0;
    let mut prev_button_pressed = false;

    loop {
        board.delay.delay_ms(100);
        loop_counter = loop_counter.wrapping_add(1);

        // Heartbeat on Blue LED and UART (every 2 seconds)
        if loop_counter % 20 == 0 {
            board.led_blue.toggle();
            board.delay.delay_ms(20);
            board.led_blue.toggle();

            let _ = writeln!(
                board.uart0,
                "[Heartbeat #{:04}] SD Card test alive! Uptime: {} ms",
                loop_counter / 20,
                board.delay.uptime_ms()
            );
        }

        // Live seconds counter on bottom line of LCD
        if loop_counter % 10 == 0 {
            let mut time_buf = StrBuf::<32>::new();
            let _ = write!(time_buf, "Uptime: {:04} s", loop_counter / 10);
            board.lcd.draw_string(4, 68, time_buf.as_str(), &FONT_5X7, lcd_color::GRAY, lcd_color::BLACK);
        }

        // Check PA8 button (Active Low)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !prev_button_pressed {
            let _ = writeln!(board.uart0, "\r\n[USER] PA8 Button Pressed! Re-running SD Card Probe...");
            run_sd_test(&mut board);
        }
        prev_button_pressed = button_pressed;
    }
}
