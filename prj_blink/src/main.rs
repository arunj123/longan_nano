#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use embedded_hal::delay::DelayNs;
use longan_nano_bsp::Board;
#[cfg(not(test))]
use panic_halt as _;
#[cfg(not(test))]
use riscv_rt::entry;

#[cfg(not(test))]
#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    loop {
        // Red
        board.led_red.on();
        board.led_green.off();
        board.led_blue.off();
        board.delay.delay_ms(500);

        // Green
        board.led_red.off();
        board.led_green.on();
        board.led_blue.off();
        board.delay.delay_ms(500);

        // Blue
        board.led_red.off();
        board.led_green.off();
        board.led_blue.on();
        board.delay.delay_ms(500);
    }
}
