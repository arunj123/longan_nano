#![no_std]
#![no_main]

use core::fmt::Write;
use panic_halt as _;
use riscv_rt::entry;

use longan_nano_bsp::{lcd_color, Board, Ina219Data};

struct BufferCursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufferCursor<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }
}

impl<'a> Write for BufferCursor<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        let to_copy = core::cmp::min(bytes.len(), remaining);
        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let mut board = Board::take_current_monitor().expect("Board initialization failed");

    writeln!(board.uart0, "\r\n========================================").ok();
    writeln!(board.uart0, " Longan Nano INA219 Current Monitor (Rust)").ok();
    writeln!(
        board.uart0,
        " SYSCLK: {} MHz, I2C0: 100 kHz (PB6 SCL, PB7 SDA)",
        board.clocks.sysclk / 1_000_000
    )
    .ok();
    writeln!(board.uart0, " USB HID: VID 0x28E9, PID 0x1234").ok();
    writeln!(board.uart0, "========================================").ok();

    // Initialize ST7735 160x80 LCD
    board.lcd.init(&mut board.delay);
    board.lcd.clear(lcd_color::BLACK);
    board.lcd.draw_string(10, 0, "INA219 Monitor", lcd_color::WHITE, lcd_color::BLACK);
    board.lcd.draw_string(10, 20, "Voltage: ", lcd_color::GREEN, lcd_color::BLACK);
    board.lcd.draw_string(10, 35, "Current: ", lcd_color::CYAN, lcd_color::BLACK);
    board.lcd.draw_string(10, 50, "Power:   ", lcd_color::YELLOW, lcd_color::BLACK);

    // Initial INA219 Calibration (cal = 4096 for 0.1 ohm shunt and 3.2A max)
    let mut ina_present = match board.ina219.init(4096) {
        Ok(_) => {
            writeln!(board.uart0, "[I2C] INA219 sensor detected & initialized").ok();
            board.led_red.off();
            true
        }
        Err(e) => {
            writeln!(
                board.uart0,
                "[I2C] INA219 init failed: {:?} (Continuing in disconnected mode)",
                e
            )
            .ok();
            board.led_red.on(); // Error indicator
            false
        }
    };

    let mut last_10hz = board.delay.uptime_ms();
    let mut last_1hz = board.delay.uptime_ms();
    let mut report_count = 0u32;
    let mut prev_usb_configured = false;

    loop {
        // High frequency non-blocking USB polling
        board.usb_hid.poll();

        let usb_configured = board.usb_hid.is_configured();
        if usb_configured && !prev_usb_configured {
            let uptime = board.delay.uptime_ms();
            writeln!(
                board.uart0,
                "[USB] >>> USB HID State: CONFIGURED [Time: {} ms] <<<",
                uptime
            )
            .ok();
            prev_usb_configured = true;
        } else if !usb_configured && prev_usb_configured {
            writeln!(board.uart0, "[USB] >>> USB HID State: DISCONNECTED <<<").ok();
            prev_usb_configured = false;
        }

        // Track user button (PA8)
        if board.button.is_pressed() {
            board.led_blue.on();
        } else {
            board.led_blue.off();
        }

        let now = board.delay.uptime_ms();

        // 10 Hz Sensor Reading, LCD Update, and USB HID Streaming
        if now.wrapping_sub(last_10hz) >= 100 {
            last_10hz = now;

            let mut data = Ina219Data::default();
            if ina_present {
                match board.ina219.read_all() {
                    Ok(readings) => {
                        data = readings;
                        board.led_red.off();
                    }
                    Err(_) => {
                        board.led_red.on();
                    }
                }
            } else {
                // Periodically retry initializing if sensor was hot-plugged
                if board.ina219.init(4096).is_ok() {
                    ina_present = true;
                    board.led_red.off();
                    writeln!(board.uart0, "[I2C] INA219 hot-plug detected!").ok();
                }
            }

            // Update LCD digits
            let mut v_buf = [0u8; 16];
            let mut v_cur = BufferCursor::new(&mut v_buf);
            write!(v_cur, "{:>5} mV ", data.voltage_mv).ok();
            board
                .lcd
                .draw_string(70, 20, v_cur.as_str(), lcd_color::WHITE, lcd_color::BLACK);

            let mut c_buf = [0u8; 16];
            let mut c_cur = BufferCursor::new(&mut c_buf);
            write!(c_cur, "{:>5} mA ", data.current_ma).ok();
            board
                .lcd
                .draw_string(70, 35, c_cur.as_str(), lcd_color::WHITE, lcd_color::BLACK);

            let mut p_buf = [0u8; 16];
            let mut p_cur = BufferCursor::new(&mut p_buf);
            write!(p_cur, "{:>5} mW ", data.power_mw).ok();
            board
                .lcd
                .draw_string(70, 50, p_cur.as_str(), lcd_color::WHITE, lcd_color::BLACK);

            // Stream 9-byte USB HID report: [Report ID 0x01, V_L, V_H, C_L, C_H, P_L, P_H, Pad, Pad]
            let report: [u8; 9] = [
                0x01,
                (data.voltage_mv & 0xFF) as u8,
                (data.voltage_mv >> 8) as u8,
                (data.current_ma & 0xFF) as u8,
                (data.current_ma >> 8) as u8,
                (data.power_mw & 0xFF) as u8,
                (data.power_mw >> 8) as u8,
                0x00,
                0x00,
            ];

            if board.usb_hid.send_report(&report) {
                report_count = report_count.wrapping_add(1);
            }

            // Toggle Green LED to indicate running monitoring loop
            board.led_green.toggle();
        }

        // 1 Hz UART Telemetry
        if now.wrapping_sub(last_1hz) >= 1000 {
            last_1hz = now;
            let sec = now / 1000;
            let cfg_str = if board.usb_hid.is_configured() {
                "CFG"
            } else {
                "DISC"
            };
            let sensor_str = if ina_present { "ONLINE" } else { "OFFLINE" };
            writeln!(
                board.uart0,
                "[HEARTBEAT] T+{}s | Sensor: {} | USB: {} | HID Reports: {}",
                sec, sensor_str, cfg_str, report_count
            )
            .ok();
        }
    }
}
