#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use longan_nano_bsp::Board;
use panic_halt as _;
use riscv_rt::entry;

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
