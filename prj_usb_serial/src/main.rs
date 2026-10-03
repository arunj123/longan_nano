#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use riscv_rt::entry;

use longan_nano_bsp::Board;

#[entry]
fn main() -> ! {
    let mut board = Board::take_usb().expect("Failed to initialize board with USB");
    let mut usb = board.usb.take().expect("USB peripheral not initialized");

    // Settle time for UART monitor
    board.delay.delay_ms(100);

    writeln!(board.uart0, "\r\n==================================================").ok();
    writeln!(board.uart0, "  Longan Nano -- USB CDC ACM Serial Device (Rust)   ").ok();
    writeln!(board.uart0, "  CPU: GD32VF103 RV32IMAC @ 96 MHz (48 MHz USB)    ").ok();
    writeln!(board.uart0, "  Standard: Rust 2021 (no_std, -Os, LTO)            ").ok();
    writeln!(board.uart0, "  Debug UART0: 115200 baud (PA9 TX, PA10 RX)        ").ok();
    writeln!(board.uart0, "==================================================\r\n").ok();

    let mut heartbeat_counter = 0u32;
    let mut led_step = 0u32;
    let mut last_button_state = false;
    let mut last_usb_state = false;

    let mtime_freq = board.clocks.mtime_freq as u64;

    let mut last_led_ticks = board.delay.get_raw_ticks();
    let mut last_heartbeat_ticks = board.delay.get_raw_ticks();

    let led_interval_ticks = (mtime_freq * 200) / 1000;       // 200 ms
    let heartbeat_interval_ticks = mtime_freq;                 // 1000 ms

    let mut rx_buf = [0u8; 64];

    loop {
        // High-frequency non-blocking USB polling engine
        usb.poll();

        let now_ticks = board.delay.get_raw_ticks();
        let now_ms = board.delay.uptime_ms();

        // USB CDC-ACM Echo: if bytes received from host, echo them back
        if usb.has_rx() {
            let n = usb.read(&mut rx_buf);
            if n > 0 {
                // Echo back over USB CDC
                usb.write(&rx_buf[..n]);

                // Also log receipt to UART0
                write!(board.uart0, ">>> USB CDC-ACM Recv: \"").ok();
                for &b in &rx_buf[..n] {
                    if b >= 0x20 && b <= 0x7E {
                        let _ = board.uart0.write_byte(b);
                    } else if b == b'\r' || b == b'\n' {
                        let _ = board.uart0.write_byte(b);
                    } else {
                        write!(board.uart0, "\\x{:02X}", b).ok();
                    }
                }
                writeln!(board.uart0, "\" ({} bytes) <<<", n).ok();
            }
        }

        // Check user button (PA8)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !last_button_state {
            writeln!(board.uart0, ">>> User Button Pressed! [Time: {} ms] <<<", now_ms).ok();
        } else if !button_pressed && last_button_state {
            writeln!(board.uart0, ">>> User Button Released! [Time: {} ms] <<<", now_ms).ok();
        }
        last_button_state = button_pressed;

        // Check USB configured state changes
        let usb_configured = usb.is_configured();
        if usb_configured != last_usb_state {
            writeln!(
                board.uart0,
                ">>> USB State: {} [Time: {} ms] <<<",
                if usb_configured {
                    "CONFIGURED (CDC Ready)"
                } else {
                    "DISCONNECTED / RESET"
                },
                now_ms
            )
            .ok();
            last_usb_state = usb_configured;
        }

        // Non-blocking RGB LED animation every 200 ms
        if now_ticks - last_led_ticks >= led_interval_ticks {
            last_led_ticks = now_ticks;
            match led_step % 3 {
                0 => {
                    board.led_red.on();
                    board.led_green.off();
                    board.led_blue.off();
                }
                1 => {
                    board.led_red.off();
                    board.led_green.on();
                    board.led_blue.off();
                }
                _ => {
                    board.led_red.off();
                    board.led_green.off();
                    board.led_blue.on();
                }
            }
            led_step = led_step.wrapping_add(1);
        }

        // Non-blocking heartbeat status output every 1000 ms
        if now_ticks - last_heartbeat_ticks >= heartbeat_interval_ticks {
            last_heartbeat_ticks = now_ticks;
            writeln!(
                board.uart0,
                "[Heartbeat #{:04}] USB: {} | Time: {} ms",
                heartbeat_counter,
                if usb_configured { "CONFIGURED" } else { "ENUMERATING..." },
                now_ms
            )
            .ok();
            heartbeat_counter = heartbeat_counter.wrapping_add(1);
        }
    }
}
