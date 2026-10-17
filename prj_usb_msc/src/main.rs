#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

pub mod telemetry;

#[cfg(not(test))]
use panic_halt as _;
#[cfg(not(test))]
use riscv_rt::entry;

#[cfg(not(test))]
use core::fmt::Write;
#[cfg(not(test))]
use longan_nano_bsp::{
    Board, Rotation, BKP_REG_ROTATION, BKP_ROTATION_180, BKP_ROTATION_NORMAL,
    BKP_REG_WRITE_PROTECT, BKP_WRITE_PROTECT_RW, BKP_WRITE_PROTECT_RO,
};
#[cfg(not(test))]
use longan_nano_bsp::hal::usb::MSC_CONFIG_DESC;
#[cfg(not(test))]
use crate::telemetry::TelemetryDashboard;

#[cfg(not(test))]
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
    let _ = writeln!(
        board.uart0,
        "Build: 00E2 | Write-Protect (WP) SCSI Support & BKP Persistence | Hot-Swap"
    );
    let _ = writeln!(
        board.uart0,
        "Commands: 'w' Toggle Write-Protect, 'r'/'o' Toggle Display, 's' Status, '?' Help"
    );
    let _ = writeln!(board.uart0, "===================================================");

    // 2. Initialize SD Card
    let sd_res = board.sdcard.init(&mut board.delay);
    let sd_ok = sd_res.is_ok();
    let cap_mb = if sd_ok { board.sdcard.sector_count / 2048 } else { 0 };

    if sd_ok {
        board.usb_msc.set_media_present(true);
        board.usb_msc.unit_attention = false; // Initial boot has no pending UA
        board.led_green.on();
        let _ = writeln!(
            board.uart0,
            "[MSC_DISK] SD Card initialized successfully. Sectors: {} (~{} MB)",
            board.sdcard.sector_count,
            cap_mb
        );

        let mut test_sec = [0u8; 512];
        let _ = writeln!(board.uart0, "[MAIN] Pre-warming 8 KB hot-LBA cache...");
        // Slots 0..7: LBAs 0..7 (MBR & Partition Table)
        for s in 0..8 {
            let t0 = board.delay.get_raw_ticks();
            let res = board.sdcard.read_sector(s, &mut test_sec);
            let dt = (board.delay.get_raw_ticks() - t0) / 24;
            if res.is_ok() {
                board.usb_msc.cache.insert(s as usize, s, &test_sec);
            }
            let _ = writeln!(
                board.uart0,
                "[CACHE] Slot {} <- LBA {}: {:?} ({} us, Sig: 0x{:02X}{:02X})",
                s, s, res, dt, test_sec[510], test_sec[511]
            );
        }

        // Slots 8..15: LBAs 1024..1031 (Partition 1 AU boundary)
        for s in 0..8 {
            let lba = 1024 + s;
            let t0 = board.delay.get_raw_ticks();
            let res = board.sdcard.read_sector(lba, &mut test_sec);
            let dt = (board.delay.get_raw_ticks() - t0) / 24;
            if res.is_ok() {
                board.usb_msc.cache.insert((8 + s) as usize, lba, &test_sec);
            }
            let _ = writeln!(
                board.uart0,
                "[CACHE] Slot {} <- LBA {}: {:?} ({} us, Sig: 0x{:02X}{:02X})",
                8 + s, lba, res, dt, test_sec[510], test_sec[511]
            );
        }
    } else {
        board.usb_msc.set_media_present(false);
        board.led_red.on();
        let _ = writeln!(board.uart0, "[MSC_DISK] SD Card init failed: {:?}", sd_res);
    }

    // 3. Initialize ST7735 LCD & Restore Rotation from BKP
    board.lcd.init(&mut board.delay);

    let bkp_rot = board.bkp.read_reg(BKP_REG_ROTATION);
    let initial_rot = if bkp_rot == BKP_ROTATION_180 {
        Rotation::Rotated180
    } else {
        Rotation::Normal
    };
    board.lcd.set_rotation(initial_rot);
    let _ = writeln!(
        board.uart0,
        "[LCD] Initialized ST7735 160x80 | Rotation: {:?} (BKP: 0x{:04X})",
        initial_rot, bkp_rot
    );

    let bkp_wp = board.bkp.read_reg(BKP_REG_WRITE_PROTECT);
    let initial_wp = bkp_wp == BKP_WRITE_PROTECT_RO;
    board.usb_msc.set_write_protected(initial_wp);
    let _ = writeln!(
        board.uart0,
        "[MSC] Write-Protect: {} (BKP: 0x{:04X})",
        if initial_wp { "READ-ONLY [RO]" } else { "READ-WRITE [RW]" },
        bkp_wp
    );

    let mut dashboard = TelemetryDashboard::new(sd_ok, cap_mb);
    dashboard.draw_layout(&mut board.lcd);
    dashboard.update(&mut board.lcd, &board.usb_msc.stats, false, initial_wp, board.delay.uptime_ms());

    // 4. Initialize USB
    let _ = write!(board.uart0, "CfgDesc [len={}]: ", MSC_CONFIG_DESC.len());
    for b in MSC_CONFIG_DESC.iter() {
        let _ = write!(board.uart0, "{:02X} ", b);
    }
    let _ = writeln!(board.uart0);
    let _ = writeln!(board.uart0, "[MAIN] USB stack initialized. Waiting for host enumeration...");

    let mut last_heartbeat = board.delay.uptime_ms();
    let mut last_sd_check = board.delay.uptime_ms();
    let mut last_btn_check = board.delay.uptime_ms();
    let mut btn_was_down = false;
    let mut btn_pressed_time = 0u32;
    let mut btn_release_time = 0u32;
    let mut btn_waiting_second = false;
    let mut usb_was_configured = false;

    loop {
        board.usb_msc.poll(&mut board.sdcard);

        let configured = board.usb_msc.is_configured();
        if configured && !usb_was_configured {
            usb_was_configured = true;
            let _ = writeln!(board.uart0, "[MAIN] >>> USB CONFIGURED BY HOST! <<<");
        } else if !configured && usb_was_configured {
            usb_was_configured = false;
            let _ = writeln!(board.uart0, "[MAIN] USB disconnected / reset.");
        }

        let now = board.delay.uptime_ms();

        // Dynamic SD Card Hot-Swap Monitoring (when USB bus is idle)
        if board.usb_msc.is_idle() && now.wrapping_sub(last_sd_check) >= 500 {
            last_sd_check = now;
            let current_ready = board.sdcard.check_ready();
            if current_ready != board.usb_msc.is_media_present() {
                board.usb_msc.set_media_present(current_ready);
                let new_cap_mb = if current_ready { board.sdcard.sector_count / 2048 } else { 0 };
                dashboard.update_sd_status(current_ready, new_cap_mb);
                if current_ready {
                    board.led_green.on();
                    board.led_red.off();
                    let _ = writeln!(
                        board.uart0,
                        "[HOT-SWAP] SD Card INSERTED: {} sectors (~{} MB). Unit Attention armed.",
                        board.sdcard.sector_count,
                        new_cap_mb
                    );
                } else {
                    board.led_green.off();
                    board.led_red.on();
                    let _ = writeln!(board.uart0, "[HOT-SWAP] SD Card REMOVED. Media not present.");
                }
            }
        }

        // Handle PA8 button gestures: Single-Click (Rotation) vs Double-Click (Write-Protect)
        if board.usb_msc.is_idle() && now.wrapping_sub(last_btn_check) >= 20 {
            last_btn_check = now;
            let is_down = board.button.is_pressed();
            if is_down && !btn_was_down {
                btn_was_down = true;
                btn_pressed_time = now;
            } else if !is_down && btn_was_down {
                btn_was_down = false;
                let press_duration = now.wrapping_sub(btn_pressed_time);
                if press_duration >= 40 {
                    if btn_waiting_second && now.wrapping_sub(btn_release_time) <= 280 {
                        // Double-click detected!
                        btn_waiting_second = false;
                        let new_wp = !board.usb_msc.is_write_protected();
                        board.usb_msc.set_write_protected(new_wp);
                        let bkp_val = if new_wp { BKP_WRITE_PROTECT_RO } else { BKP_WRITE_PROTECT_RW };
                        board.bkp.write_reg(BKP_REG_WRITE_PROTECT, bkp_val);
                        let _ = writeln!(
                            board.uart0,
                            "[BUTTON] PA8 Double-click -> Toggled Write-Protect to {} (Saved BKP6: 0x{:04X})",
                            if new_wp { "RO" } else { "RW" }, bkp_val
                        );
                        dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, new_wp, now);
                    } else {
                        btn_waiting_second = true;
                        btn_release_time = now;
                    }
                }
            } else if btn_waiting_second && !btn_was_down && now.wrapping_sub(btn_release_time) > 280 {
                // 280 ms timeout expired without second click -> Single-click detected!
                btn_waiting_second = false;
                let new_rot = board.lcd.toggle_rotation();
                let bkp_val = if new_rot == Rotation::Rotated180 {
                    BKP_ROTATION_180
                } else {
                    BKP_ROTATION_NORMAL
                };
                board.bkp.write_reg(BKP_REG_ROTATION, bkp_val);
                let _ = writeln!(
                    board.uart0,
                    "[BUTTON] PA8 Single-click -> Toggled LCD rotation to {:?} (Saved BKP5: 0x{:04X})",
                    new_rot, bkp_val
                );
                dashboard.draw_layout(&mut board.lcd);
                dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, board.usb_msc.is_write_protected(), now);
            }
        }

        // Handle UART0 interactive commands (when USB bus is idle)
        if board.usb_msc.is_idle() {
            if let Some(cmd) = board.uart0.read_byte() {
                match cmd {
                    b'w' | b'W' => {
                        let new_wp = !board.usb_msc.is_write_protected();
                        board.usb_msc.set_write_protected(new_wp);
                        let bkp_val = if new_wp { BKP_WRITE_PROTECT_RO } else { BKP_WRITE_PROTECT_RW };
                        board.bkp.write_reg(BKP_REG_WRITE_PROTECT, bkp_val);
                        let _ = writeln!(
                            board.uart0,
                            "[UART] Command '{}' -> Toggled Write-Protect to {} (Saved BKP6: 0x{:04X})",
                            cmd as char, if new_wp { "RO" } else { "RW" }, bkp_val
                        );
                        dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, new_wp, now);
                    }
                    b'k' => {
                        // Simulated Single-Click: Toggle Display Rotation
                        let new_rot = board.lcd.toggle_rotation();
                        let bkp_val = if new_rot == Rotation::Rotated180 {
                            BKP_ROTATION_180
                        } else {
                            BKP_ROTATION_NORMAL
                        };
                        board.bkp.write_reg(BKP_REG_ROTATION, bkp_val);
                        let _ = writeln!(
                            board.uart0,
                            "[SIM] Simulated PA8 Single-click -> Toggled LCD rotation to {:?} (Saved BKP5: 0x{:04X})",
                            new_rot, bkp_val
                        );
                        dashboard.draw_layout(&mut board.lcd);
                        dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, board.usb_msc.is_write_protected(), now);
                    }
                    b'K' => {
                        // Simulated Double-Click: Toggle Write-Protect
                        let new_wp = !board.usb_msc.is_write_protected();
                        board.usb_msc.set_write_protected(new_wp);
                        let bkp_val = if new_wp { BKP_WRITE_PROTECT_RO } else { BKP_WRITE_PROTECT_RW };
                        board.bkp.write_reg(BKP_REG_WRITE_PROTECT, bkp_val);
                        let _ = writeln!(
                            board.uart0,
                            "[SIM] Simulated PA8 Double-click -> Toggled Write-Protect to {} (Saved BKP6: 0x{:04X})",
                            if new_wp { "RO" } else { "RW" }, bkp_val
                        );
                        dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, new_wp, now);
                    }
                    b'r' | b'R' | b'o' | b'O' => {
                        let new_rot = board.lcd.toggle_rotation();
                        let bkp_val = if new_rot == Rotation::Rotated180 {
                            BKP_ROTATION_180
                        } else {
                            BKP_ROTATION_NORMAL
                        };
                        board.bkp.write_reg(BKP_REG_ROTATION, bkp_val);
                        let _ = writeln!(
                            board.uart0,
                            "[UART] Command '{}' -> Toggled LCD rotation to {:?} (Saved BKP5: 0x{:04X})",
                            cmd as char, new_rot, bkp_val
                        );
                        dashboard.draw_layout(&mut board.lcd);
                        dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, board.usb_msc.is_write_protected(), now);
                    }
                    b's' | b'S' => {
                        let bkp_rot = board.bkp.read_reg(BKP_REG_ROTATION);
                        let bkp_wp = board.bkp.read_reg(BKP_REG_WRITE_PROTECT);
                        let _ = writeln!(
                            board.uart0,
                            "[STATUS] LCD Rotation: {:?} (BKP5: 0x{:04X}) | Write-Protect: {} (BKP6: 0x{:04X})",
                            board.lcd.rotation(), bkp_rot,
                            if board.usb_msc.is_write_protected() { "RO" } else { "RW" }, bkp_wp
                        );
                    }
                    b'?' => {
                        let _ = writeln!(
                            board.uart0,
                            "[HELP] Commands: 'w' Toggle Write-Protect, 'r'/'o' Toggle Display Rotation, 's' Status & BKP, 'k' Sim Single-Click, 'K' Sim Double-Click, '?' Help"
                        );
                    }
                    _ => {}
                }
            }
        }

        if board.usb_msc.stats.is_active {
            board.usb_msc.clear_active();
            board.led_blue.toggle();
            dashboard.mark_active(now);
        }

        // Live telemetry update: only when USB bus is idle
        if board.usb_msc.is_idle() {
            dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, board.usb_msc.is_write_protected(), now);
        }

        if now.wrapping_sub(last_heartbeat) >= 1000 {
            last_heartbeat = now;
            if board.usb_msc.is_media_present() {
                board.led_green.toggle();
            } else {
                board.led_red.toggle();
            }
        }
    }
}
