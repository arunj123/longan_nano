#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;
use core::fmt::Write;
use longan_nano_bsp::lcd::FONT_5X7;
use longan_nano_bsp::{lcd_color, Board, LCD_WIDTH};
use longan_nano_bsp::hal::usb::MSC_CONFIG_DESC;

struct StrBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StrBuf<N> {
    fn new() -> Self {
        Self { buf: [0; N], len: 0 }
    }
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Write for StrBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remain = N - self.len;
        let copy_len = core::cmp::min(bytes.len(), remain);
        self.buf[self.len..self.len + copy_len].copy_from_slice(&bytes[..copy_len]);
        self.len += copy_len;
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let mut board = Board::take_msc().expect("Failed to initialize board peripherals");

    // 1. Initialize Debug UART0 @ 115200
    let _ = writeln!(board.uart0, "\n===================================================");
    let _ = writeln!(board.uart0, "Longan Nano -- USB MSC SD Card Reader (Rust)");
    let _ = writeln!(
        board.uart0,
        "GD32VF103 RV32IMAC @ {} MHz (USB Clock: 48 MHz)",
        board.clocks.sysclk / 1_000_000
    );
    let _ = writeln!(
        board.uart0,
        "Mass Storage Class (SCSI transparent / Bulk-Only Transport)"
    );
    let _ = writeln!(board.uart0, "===================================================");

    // 2. Initialize ST7735 LCD
    board.lcd.clear(lcd_color::BLACK);
    board.lcd.fill_rect(0, 0, LCD_WIDTH, 14, lcd_color::DARK_NAVY);
    board.lcd.draw_string(6, 3, "USB SD CARD READER", &FONT_5X7, lcd_color::CYAN, lcd_color::DARK_NAVY);

    // 3. Initialize SD Card
    board.lcd.draw_string(4, 18, "Probing SD card...", &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);

    let sd_res = board.sdcard.init(&mut board.delay);
    let sd_ok = sd_res.is_ok();

    if sd_ok {
        board.led_green.on();
        board.lcd.draw_string(4, 18, "SD Card: DETECTED  ", &FONT_5X7, lcd_color::GREEN, lcd_color::BLACK);

        let cap_mb = board.sdcard.sector_count / 2048;
        let mut cap_buf = StrBuf::<32>::new();
        let _ = write!(cap_buf, "Cap: {} MB", cap_mb);
        board.lcd.draw_string(4, 28, cap_buf.as_str(), &FONT_5X7, lcd_color::WHITE, lcd_color::BLACK);

        let _ = writeln!(
            board.uart0,
            "[MSC_DISK] SD Card initialized successfully. Sectors: {} (~{} MB)",
            board.sdcard.sector_count,
            cap_mb
        );

        let mut test_sec = [0u8; 512];

        // Sector 0
        let t0 = board.delay.get_raw_ticks();
        let res_0 = board.sdcard.read_sector(0, &mut test_sec);
        let dt_0 = (board.delay.get_raw_ticks() - t0) / 24; // us at 24MHz mtime
        let _ = writeln!(
            board.uart0,
            "[MAIN] Real SD Sector 0: {:?} ({} us, Sig: 0x{:02X}{:02X})",
            res_0,
            dt_0,
            test_sec[510],
            test_sec[511]
        );

        // Sector 496 (VBR)
        let t1 = board.delay.get_raw_ticks();
        let res_496 = board.sdcard.read_sector(496, &mut test_sec);
        let dt_496 = (board.delay.get_raw_ticks() - t1) / 24;
        let _ = writeln!(
            board.uart0,
            "[MAIN] Real SD Sector 496 (VBR): {:?} ({} us, Sig: 0x{:02X}{:02X})",
            res_496,
            dt_496,
            test_sec[510],
            test_sec[511]
        );

        // Sector 560 (FAT)
        let t2 = board.delay.get_raw_ticks();
        let res_560 = board.sdcard.read_sector(560, &mut test_sec);
        let dt_560 = (board.delay.get_raw_ticks() - t2) / 24;
        let _ = writeln!(
            board.uart0,
            "[MAIN] Real SD Sector 560 (FAT): {:?} ({} us, Sig: 0x{:02X}{:02X})",
            res_560,
            dt_560,
            test_sec[510],
            test_sec[511]
        );

        // Sectors 1024..1031
        for s in 1024..=1031 {
            let ts = board.delay.get_raw_ticks();
            let res_s = board.sdcard.read_sector(s, &mut test_sec);
            let dt_s = (board.delay.get_raw_ticks() - ts) / 24;
            let _ = writeln!(
                board.uart0,
                "[MAIN] Real SD Sector {}: {:?} ({} us, Sig: 0x{:02X}{:02X})",
                s,
                res_s,
                dt_s,
                test_sec[510],
                test_sec[511]
            );
        }
    } else {
        board.led_red.on();
        board.lcd.draw_string(4, 18, "SD Card: NOT FOUND ", &FONT_5X7, lcd_color::RED, lcd_color::BLACK);
        board.lcd.draw_string(4, 28, "Insert card & reset", &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);
        let _ = writeln!(board.uart0, "[MSC_DISK] SD Card init failed: {:?}", sd_res);
    }

    // 4. Initialize USB
    board.lcd.draw_string(4, 40, "USB: Initializing...", &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);

    let _ = write!(board.uart0, "CfgDesc [len={}]: ", MSC_CONFIG_DESC.len());
    for b in MSC_CONFIG_DESC.iter() {
        let _ = write!(board.uart0, "{:02X} ", b);
    }
    let _ = writeln!(board.uart0);
    let _ = writeln!(board.uart0, "[MAIN] USB stack initialized. Waiting for host enumeration...");

    let mut last_heartbeat = board.delay.uptime_ms();
    let mut usb_was_configured = false;

    loop {
        board.usb_msc.poll(&mut board.sdcard);

        let configured = board.usb_msc.is_configured();
        if configured && !usb_was_configured {
            usb_was_configured = true;
            board.lcd.draw_string(4, 40, "USB: CONFIGURED OK   ", &FONT_5X7, lcd_color::GREEN, lcd_color::BLACK);
            let _ = writeln!(board.uart0, "[MAIN] >>> USB CONFIGURED BY HOST! <<<");
        } else if !configured && usb_was_configured {
            usb_was_configured = false;
            board.lcd.draw_string(4, 40, "USB: Disconnected... ", &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);
            let _ = writeln!(board.uart0, "[MAIN] USB disconnected / reset.");
        }

        if board.usb_msc.stats.is_active {
            board.usb_msc.clear_active();
            board.led_blue.toggle();
        }

        let now = board.delay.uptime_ms();
        if now.wrapping_sub(last_heartbeat) >= 1000 {
            last_heartbeat = now;
            if sd_ok {
                board.led_green.toggle();
            }
        }
    }
}
