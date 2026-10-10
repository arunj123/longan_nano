#![no_std]
#![no_main]

use core::fmt::Write;
use panic_halt as _;
use riscv_rt::entry;

use longan_nano_bsp::lcd::FONT_5X7;
use longan_nano_bsp::{lcd_color, Board};

mod hid_consumer {
    pub const VOLUME_UP: u16 = 0x00E9;
    pub const VOLUME_DOWN: u16 = 0x00EA;
    pub const MUTE: u16 = 0x00E2;
    pub const NO_KEY: u16 = 0x0000;
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum HidActionState {
    Idle,
    WaitingForPressConfirm,
    WaitingForReleaseConfirm,
}

const DARK_NAVY: u16 = 0x0010;
const LIGHT_GRAY: u16 = 0xCE79;
const GRAY: u16 = 0x8410;

#[entry]
fn main() -> ! {
    let mut board = Board::take_composite().expect("Board initialization failed");

    writeln!(board.uart0, "\r\n========================================").ok();
    writeln!(board.uart0, " Longan Nano USB Composite Device (Rust)").ok();
    writeln!(
        board.uart0,
        " SYSCLK: {} MHz, USB: 48 MHz",
        board.clocks.sysclk / 1_000_000
    )
    .ok();
    writeln!(board.uart0, " Composite: Std HID (EP1) + Custom HID (EP2)").ok();
    writeln!(board.uart0, " Rotary Encoder: PB10 (CLK), PB11 (DT), PB5 (SW)").ok();
    writeln!(board.uart0, " VID: 0x28E9, PID: 0xABE8").ok();
    writeln!(board.uart0, "========================================").ok();

    // Visual splash screen on boot
    board.lcd.init(&mut board.delay);
    board.lcd.clear(DARK_NAVY);
    board.lcd.rect(0, 0, 160, 80, lcd_color::CYAN);
    board.lcd.fill_rect(1, 1, 158, 13, GRAY);
    board
        .lcd
        .draw_string(28, 4, "LONGAN NANO USB", &FONT_5X7, lcd_color::YELLOW, GRAY);
    board
        .lcd
        .draw_string(8, 22, "HID Composite Stack", &FONT_5X7, lcd_color::WHITE, DARK_NAVY);
    board
        .lcd
        .draw_string(8, 38, "Waiting for Host...", &FONT_5X7, lcd_color::RED, DARK_NAVY);
    board
        .lcd
        .draw_string(8, 54, "Pure Embedded Rust", &FONT_5X7, lcd_color::CYAN, DARK_NAVY);
    board.lcd.draw_string(
        8,
        66,
        "PID 0xABE8 @ 96MHz",
        &FONT_5X7,
        LIGHT_GRAY,
        DARK_NAVY,
    );

    let mut hid_state = HidActionState::Idle;
    let mut prev_usb_configured = false;
    let mut last_1hz = board.delay.uptime_ms();
    let mut last_blink = board.delay.uptime_ms();
    let mut button_was_pressed = false;
    let mut custom_rx_count = 0u32;
    let mut consumer_report_count = 0u32;

    loop {
        board.usb_composite.poll();

        let usb_configured = board.usb_composite.is_configured();
        if usb_configured && !prev_usb_configured {
            let uptime = board.delay.uptime_ms();
            writeln!(
                board.uart0,
                "[USB] >>> Composite Device CONFIGURED [Time: {} ms] <<<",
                uptime
            )
            .ok();
            board.lcd.draw_string(
                8,
                38,
                "USB Configured: OK! ",
                &FONT_5X7,
                lcd_color::GREEN,
                DARK_NAVY,
            );
            board.led_green.on();
            prev_usb_configured = true;
        } else if !usb_configured && prev_usb_configured {
            writeln!(board.uart0, "[USB] >>> USB Disconnected <<<").ok();
            board.lcd.draw_string(
                8,
                38,
                "Waiting for Host... ",
                &FONT_5X7,
                lcd_color::RED,
                DARK_NAVY,
            );
            board.led_green.off();
            prev_usb_configured = false;
        }

        // Handle Rotary Encoder
        let (rotation, sw_pressed) = board.encoder.poll();
        if usb_configured {
            match hid_state {
                HidActionState::Idle => {
                    let mut action_key = hid_consumer::NO_KEY;
                    if rotation > 0 {
                        writeln!(board.uart0, "[INPUT] Volume Up").ok();
                        action_key = hid_consumer::VOLUME_UP;
                    } else if rotation < 0 {
                        writeln!(board.uart0, "[INPUT] Volume Down").ok();
                        action_key = hid_consumer::VOLUME_DOWN;
                    } else if sw_pressed {
                        writeln!(board.uart0, "[INPUT] Mute Toggle").ok();
                        action_key = hid_consumer::MUTE;
                    }

                    if action_key != hid_consumer::NO_KEY {
                        if board.usb_composite.send_consumer_report(action_key) {
                            consumer_report_count = consumer_report_count.wrapping_add(1);
                            hid_state = HidActionState::WaitingForPressConfirm;
                        }
                    }
                }
                HidActionState::WaitingForPressConfirm => {
                    if !board.usb_composite.is_std_hid_busy() {
                        if board.usb_composite.send_consumer_report(hid_consumer::NO_KEY) {
                            hid_state = HidActionState::WaitingForReleaseConfirm;
                        }
                    }
                }
                HidActionState::WaitingForReleaseConfirm => {
                    if !board.usb_composite.is_std_hid_busy() {
                        hid_state = HidActionState::Idle;
                    }
                }
            }
        }

        // Handle PA8 User Button
        let button_now = board.button.is_pressed();
        if button_now && !button_was_pressed {
            writeln!(board.uart0, "[BUTTON] User key pressed: Sending theme request").ok();
            board.led_blue.on();
            if usb_configured {
                let theme_report: [u8; 2] = [0x01, 0x01];
                board.usb_composite.send_custom_report(&theme_report);
            }
            button_was_pressed = true;
        } else if !button_now && button_was_pressed {
            board.led_blue.off();
            button_was_pressed = false;
        }

        // Handle Custom HID Host Packets (Display Manager & Image streaming)
        if board.usb_composite.has_custom_rx() {
            let mut rx_buf = [0u8; 64];
            let len = board.usb_composite.read_custom_rx(&mut rx_buf);
            if len > 0 {
                custom_rx_count = custom_rx_count.wrapping_add(1);
                let cmd = rx_buf[0];
                if cmd == 0x01 && len >= 7 {
                    // DRAW_RECT: [cmd, x, y, w, h, seq_lo, seq_hi]
                    let x = rx_buf[1];
                    let y = rx_buf[2];
                    let w = rx_buf[3];
                    let h = rx_buf[4];
                    writeln!(board.uart0, "[DISPLAY] Draw Rect: ({}, {}) {}x{}", x, y, w, h).ok();
                } else if cmd == 0x02 {
                    // IMAGE_DATA chunk
                    board.led_green.toggle();
                }
            }
        }

        let now = board.delay.uptime_ms();

        // Blink Green LED while waiting for host
        if !usb_configured && now.wrapping_sub(last_blink) >= 200 {
            last_blink = now;
            board.led_green.toggle();
        }

        // 1 Hz UART Telemetry Heartbeat
        if now.wrapping_sub(last_1hz) >= 1000 {
            last_1hz = now;
            let sec = now / 1000;
            let status_str = if usb_configured { "CONFIGURED" } else { "WAITING" };
            writeln!(
                board.uart0,
                "[HEARTBEAT] T+{}s | USB: {} | Consumer Reports: {} | Custom Packets: {}",
                sec, status_str, consumer_report_count, custom_rx_count
            )
            .ok();
        }
    }
}
