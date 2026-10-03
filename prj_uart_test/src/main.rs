#![no_std]
#![no_main]

use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use longan_nano_bsp::Board;
use panic_halt as _;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    // Settle time for UART lines and monitor
    board.delay.delay_ms(100);

    let _ = writeln!(board.uart0, "\r\n==================================================");
    let _ = writeln!(board.uart0, "  Longan Nano -- Pure Rust Embedded UART Engine   ");
    let _ = writeln!(board.uart0, "  CPU: GD32VF103 RV32IMAC @ 108 MHz               ");
    let _ = writeln!(board.uart0, "  Standard: Rust 2021 (#![no_std], -Os, lto)      ");
    let _ = writeln!(board.uart0, "  USART0: 115200 baud (PA9 TX, PA10 RX)           ");
    let _ = writeln!(board.uart0, "==================================================\r\n");

    let mut counter: u32 = 0;
    let mut last_button_state = false;

    loop {
        let ms = board.delay.uptime_ms();

        // Cycle through RGB LEDs on each heartbeat
        match counter % 3 {
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

        // Check user button (PA8)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !last_button_state {
            let _ = writeln!(board.uart0, ">>> User Button Pressed! [Time: {} ms] <<<", ms);
        } else if !button_pressed && last_button_state {
            let _ = writeln!(board.uart0, ">>> User Button Released! [Time: {} ms] <<<", ms);
        }
        last_button_state = button_pressed;

        // Periodic heartbeat message
        let _ = writeln!(
            board.uart0,
            "[Heartbeat #{:04}] Longan Nano alive! Time: {} ms",
            counter,
            ms
        );
        counter = counter.wrapping_add(1);

        board.delay.delay_ms(500);
    }
}
