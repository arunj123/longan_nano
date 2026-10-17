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
use longan_nano_bsp::Board;
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
    let _ = writeln!(board.uart0, "Build: 00DE | Live LCD Telemetry Dashboard Enabled");
    let _ = writeln!(board.uart0, "===================================================");

    // 2. Initialize SD Card
    let sd_res = board.sdcard.init(&mut board.delay);
    let sd_ok = sd_res.is_ok();
    let cap_mb = if sd_ok { board.sdcard.sector_count / 2048 } else { 0 };

    if sd_ok {
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
        board.led_red.on();
        let _ = writeln!(board.uart0, "[MSC_DISK] SD Card init failed: {:?}", sd_res);
    }

    // 3. Initialize ST7735 LCD Live Telemetry Dashboard
    let mut dashboard = TelemetryDashboard::new(sd_ok, cap_mb);
    dashboard.draw_layout(&mut board.lcd);
    dashboard.update(&mut board.lcd, &board.usb_msc.stats, false, board.delay.uptime_ms());

    // 4. Initialize USB
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
            let _ = writeln!(board.uart0, "[MAIN] >>> USB CONFIGURED BY HOST! <<<");
        } else if !configured && usb_was_configured {
            usb_was_configured = false;
            let _ = writeln!(board.uart0, "[MAIN] USB disconnected / reset.");
        }

        let now = board.delay.uptime_ms();

        if board.usb_msc.stats.is_active {
            board.usb_msc.clear_active();
            board.led_blue.toggle();
            dashboard.mark_active(now);
        }

        // Live telemetry update: only when USB bus is idle
        if board.usb_msc.is_idle() {
            dashboard.update(&mut board.lcd, &board.usb_msc.stats, configured, now);
        }

        if now.wrapping_sub(last_heartbeat) >= 1000 {
            last_heartbeat = now;
            if sd_ok {
                board.led_green.toggle();
            }
        }
    }
}
