#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

mod app;
pub mod button;
pub mod fmt;
mod logger;
pub mod model;
mod telemetry;
mod ui;

use core::cell::RefCell;
use core::fmt::Write;
#[cfg(not(test))]
use panic_halt as _;
#[cfg(not(test))]
use riscv_rt::entry;

use longan_nano_bsp::{
    Board, Rotation, UsbMode, BKP_REG_ROTATION, BKP_ROTATION_180,
};

use crate::app::{
    App, AppCommand, BKP_MAGIC_KEY, BKP_REG_CONFIG, BKP_REG_LIMIT, BKP_REG_MAGIC, BKP_REG_TARE,
};
use crate::button::{ButtonEvent, ButtonState};
use crate::logger::SdDatalogger;
use crate::model::{BatteryProfile, InaReading};
use crate::telemetry::HidCommand;
use crate::ui::{ScreenMode, SdStatus, TriggerMode, TriggerState, UsbDriveState};

#[cfg(not(test))]
#[entry]
fn main() -> ! {
    let mut board = Board::take_current_monitor().expect("Board initialization failed");

    writeln!(board.uart0, "\r\n========================================").ok();
    writeln!(board.uart0, " Longan Nano Current Monitor (Rust)").ok();
    writeln!(
        board.uart0,
        " SYSCLK: {} MHz | I2C0: 400 kHz Fast Mode | SPI0: 24 MHz",
        board.clocks.sysclk / 1_000_000
    )
    .ok();
    writeln!(board.uart0, " MicroSD: SPI1 | USB: Dual Mode (HID / MSC)").ok();
    let init_epoch = board.rtc.get_epoch();
    let init_dt = board.rtc.get_datetime();
    writeln!(
        board.uart0,
        " RTC: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC (Epoch: {}, Synced: {})",
        init_dt.year, init_dt.month, init_dt.day, init_dt.hour, init_dt.minute, init_dt.second,
        init_epoch,
        if board.rtc.is_synced() { "YES" } else { "NO" }
    ).ok();
    writeln!(board.uart0, " Commands: '1'..'6' Mode, 'm' Cycle, 'u' UsbDrive, 'b' Bat, 't' Tare, 'c' Clock, 'r' Rotate SD, 'o' Rotate LCD, 's' Dump, '?' Help").ok();
    writeln!(board.uart0, "========================================").ok();

    // Initialize ST7735 160x80 LCD
    board.lcd.init(&mut board.delay);

    // Restore LCD Rotation from BKP
    let bkp_rot = board.rtc.read_backup_reg(BKP_REG_ROTATION);
    let initial_rot = if bkp_rot == BKP_ROTATION_180 {
        Rotation::Rotated180
    } else {
        Rotation::Normal
    };
    board.lcd.set_rotation(initial_rot);

    // MicroSD card wrapper
    let sdcard = board.sdcard.take().expect("SdCard not found");
    let sdcard_cell = RefCell::new(sdcard);
    let mut datalogger = SdDatalogger::new(&sdcard_cell, board.delay.uptime_ms());

    // Initial probe of MicroSD card
    if datalogger.check_probe(board.delay.uptime_ms(), &mut board.delay) {
        let sectors = sdcard_cell.borrow().sector_count;
        writeln!(
            board.uart0,
            "[SD] Card detected & ready ({} sectors) | File: {}",
            sectors,
            datalogger.file_name_str()
        )
        .ok();
    } else {
        writeln!(
            board.uart0,
            "[SD] Card not detected at boot (Running in untethered mode)"
        )
        .ok();
    }

    let mut app = App::new(datalogger, &sdcard_cell, board.delay.uptime_ms());

    // Check NVRAM configuration in BKP registers
    if board.rtc.read_backup_reg(BKP_REG_MAGIC) == BKP_MAGIC_KEY {
        let cfg = board.rtc.read_backup_reg(BKP_REG_CONFIG);
        let mode_idx = (cfg >> 8) & 0xFF;
        let bat_idx = (cfg & 0xFF) as u8;
        let tare_tenth = board.rtc.read_backup_reg(BKP_REG_TARE) as i16;
        app.current_limit_ma = board.rtc.read_backup_reg(BKP_REG_LIMIT);

        app.screen_mode = match mode_idx {
            1 => ScreenMode::Graph,
            2 => ScreenMode::Stats,
            3 => ScreenMode::Histogram,
            4 => ScreenMode::BigDigit,
            5 => ScreenMode::UsbDrive,
            _ => ScreenMode::Hero,
        };
        app.battery.profile = BatteryProfile::from_id(bat_idx);
        board.ina219.set_tare(tare_tenth);
        writeln!(
            board.uart0,
            "[NVRAM] Restored config from BKP: Mode {}, Bat {}, Tare {}.{} mA, Limit {} mA",
            mode_idx + 1,
            app.battery.profile.name(),
            tare_tenth / 10,
            tare_tenth.unsigned_abs() % 10,
            app.current_limit_ma
        ).ok();
    } else {
        app.save_nvram(&mut board);
        writeln!(board.uart0, "[NVRAM] Initialized default configuration in BKP registers").ok();
    }

    // Draw initial layout
    app.draw_current_layout(&mut board);

    // Initial INA219 configuration (0x3E7F: 32V, ±320mV, 128x shunt avg, continuous)
    // Cal = 4096 for 0.1 ohm shunt (0.1 mA LSB, 3.2A max)
    app.ina_present = match board.ina219.init(4096) {
        Ok(_) => {
            writeln!(
                board.uart0,
                "[I2C] INA219 initialized (400 kHz Fast Mode, 128x Shunt Averaging, Cal: 4096)"
            )
            .ok();
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
            board.led_red.on();
            false
        }
    };

    loop {
        let now = board.delay.uptime_ms();

        // 0. Feed Free Watchdog Timer (FWDGT)
        board.fwdgt.feed();

        // 1. High-frequency non-blocking USB polling (every loop iteration)
        match &mut board.usb {
            UsbMode::Hid(ref mut hid) => {
                hid.poll();
                let usb_configured = hid.is_configured();
                if usb_configured && !app.prev_usb_configured {
                    writeln!(
                        board.uart0,
                        "[USB] >>> USB HID State: CONFIGURED [Time: {} ms] <<<",
                        now
                    )
                    .ok();
                    app.prev_usb_configured = true;
                } else if !usb_configured && app.prev_usb_configured {
                    writeln!(board.uart0, "[USB] >>> USB HID State: DISCONNECTED <<<").ok();
                    app.prev_usb_configured = false;
                }
            }
            UsbMode::Msc(ref mut msc) => {
                let mut sd = sdcard_cell.borrow_mut();
                msc.poll(&mut *sd);
                let usb_configured = msc.is_configured();
                if usb_configured && !app.prev_usb_configured {
                    writeln!(
                        board.uart0,
                        "[USB] >>> USB MSC State: CONFIGURED [Time: {} ms] <<<",
                        now
                    )
                    .ok();
                    app.prev_usb_configured = true;
                } else if !usb_configured && app.prev_usb_configured {
                    writeln!(board.uart0, "[USB] >>> USB MSC State: DISCONNECTED <<<").ok();
                    app.prev_usb_configured = false;
                }
            }
            UsbMode::None => {}
        }

        // 1.5 Rotary Encoder Polling (PB10/PB11/PB5) -> Unified AppCommand
        let (rot, enc_pressed) = board.encoder.poll();
        if rot != 0 {
            let cmd = if app.screen_mode == ScreenMode::Graph {
                AppCommand::AdjustTriggerLevel(if rot > 0 { 100 } else { -100 })
            } else if rot > 0 {
                AppCommand::NextScreen
            } else {
                AppCommand::PrevScreen
            };
            app.execute_command(&mut board, cmd, now);
        }

        if enc_pressed {
            let cmd = match app.screen_mode {
                ScreenMode::UsbDrive => AppCommand::ToggleUsbDrive,
                ScreenMode::Graph => AppCommand::CycleTriggerOrRearm { show_banner: false },
                _ => AppCommand::CycleBatteryProfile { show_banner: false },
            };
            app.execute_command(&mut board, cmd, now);
        }

        // 2. Remote UART0 Interactive Command Console -> Unified AppCommand
        while let Some(ch) = board.uart0.read_byte() {
            if app.cmd_len > 0 {
                // Buffering a multi-byte command (started with 'T' or 'L')
                match ch {
                    b'0'..=b'9' | b' ' | b':' => {
                        if app.cmd_len < app.cmd_buf.len() {
                            app.cmd_buf[app.cmd_len] = ch;
                            app.cmd_len += 1;
                        }
                    }
                    b'\r' | b'\n' => {
                        let mut pending_cmd = None;
                        if app.cmd_len >= 2 && app.cmd_buf[0] == b'T' {
                            let line = &app.cmd_buf[..app.cmd_len];
                            let mut idx = 1;
                            while idx < app.cmd_len && (line[idx] == b' ' || line[idx] == b':') {
                                idx += 1;
                            }
                            let mut parsed_epoch = 0u32;
                            let mut has_digits = false;
                            while idx < app.cmd_len && line[idx].is_ascii_digit() {
                                parsed_epoch = parsed_epoch
                                    .saturating_mul(10)
                                    .saturating_add((line[idx] - b'0') as u32);
                                has_digits = true;
                                idx += 1;
                            }
                            if has_digits && parsed_epoch > 0 {
                                pending_cmd = Some(AppCommand::SetRtcEpoch(parsed_epoch));
                            }
                        } else if app.cmd_len == 1 && app.cmd_buf[0] == b'T' {
                            pending_cmd = Some(AppCommand::TareAndReset { show_banner: false });
                        } else if app.cmd_len >= 2 && app.cmd_buf[0] == b'L' {
                            let line = &app.cmd_buf[..app.cmd_len];
                            let mut idx = 1;
                            while idx < app.cmd_len && (line[idx] == b' ' || line[idx] == b':') {
                                idx += 1;
                            }
                            let mut parsed_limit = 0u16;
                            let mut has_digits = false;
                            while idx < app.cmd_len && line[idx].is_ascii_digit() {
                                parsed_limit = parsed_limit
                                    .saturating_mul(10)
                                    .saturating_add((line[idx] - b'0') as u16);
                                has_digits = true;
                                idx += 1;
                            }
                            if has_digits {
                                pending_cmd = Some(AppCommand::SetCurrentLimit(parsed_limit));
                            }
                        } else if app.cmd_len == 1 && app.cmd_buf[0] == b'L' {
                            pending_cmd = Some(AppCommand::PrintCurrentLimit);
                        }
                        app.cmd_len = 0;
                        if let Some(cmd) = pending_cmd {
                            app.execute_command(&mut board, cmd, now);
                        }
                    }
                    _ => {
                        app.cmd_len = 0;
                    }
                }
            } else {
                let maybe_cmd = match ch {
                    b'1' => Some(AppCommand::SetScreen(ScreenMode::Hero)),
                    b'2' => Some(AppCommand::SetScreen(ScreenMode::Graph)),
                    b'3' => Some(AppCommand::SetScreen(ScreenMode::Stats)),
                    b'4' => Some(AppCommand::SetScreen(ScreenMode::Histogram)),
                    b'5' => Some(AppCommand::SetScreen(ScreenMode::BigDigit)),
                    b'6' => Some(AppCommand::SetScreen(ScreenMode::UsbDrive)),
                    b'u' | b'U' => Some(AppCommand::ToggleUsbDrive),
                    b'm' | b'M' => Some(AppCommand::NextScreen),
                    b'g' | b'G' => Some(AppCommand::CycleTriggerOrRearm { show_banner: false }),
                    b'b' | b'B' => Some(AppCommand::CycleBatteryProfile { show_banner: false }),
                    b't' => Some(AppCommand::TareAndReset { show_banner: false }),
                    b'c' | b'C' => Some(AppCommand::PrintRtcTime),
                    b'l' => Some(AppCommand::PrintCurrentLimit),
                    b'L' => {
                        app.cmd_buf[0] = b'L';
                        app.cmd_len = 1;
                        None
                    }
                    b'T' => {
                        app.cmd_buf[0] = b'T';
                        app.cmd_len = 1;
                        None
                    }
                    b'r' | b'R' | b'n' | b'N' => Some(AppCommand::RotateSdLog { show_banner: false }),
                    b'o' | b'O' => Some(AppCommand::ToggleLcdRotation),
                    b'f' | b'F' => Some(AppCommand::FlushSd),
                    b's' | b'S' => Some(AppCommand::DumpSession),
                    b'k' => Some(AppCommand::SimButtonTap),
                    b'K' => Some(AppCommand::SimButtonDoubleTap),
                    b'P' => Some(AppCommand::SimButtonHoldTare),
                    b'p' => Some(AppCommand::SimButtonHoldAbort),
                    b'?' | b'h' | b'H' => Some(AppCommand::PrintHelp),
                    b'\r' | b'\n' => None,
                    _ => {
                        writeln!(
                            board.uart0,
                            "[CMD] Unknown key: '{}' (Send '?' for help)",
                            ch as char
                        )
                        .ok();
                        None
                    }
                };
                if let Some(cmd) = maybe_cmd {
                    app.execute_command(&mut board, cmd, now);
                }
            }
        }

        // 3. Bidirectional USB HID OUT Command Polling -> Unified AppCommand
        let hid_cmd = board
            .usb_hid()
            .and_then(|hid| app.telemetry.poll_command(hid));
        if let Some(cmd) = hid_cmd {
            let app_cmd = match cmd {
                HidCommand::SetMode(0) => Some(AppCommand::SetScreen(ScreenMode::Hero)),
                HidCommand::SetMode(1) => Some(AppCommand::SetScreen(ScreenMode::Graph)),
                HidCommand::SetMode(2) => Some(AppCommand::SetScreen(ScreenMode::Stats)),
                HidCommand::SetMode(3) => Some(AppCommand::SetScreen(ScreenMode::Histogram)),
                HidCommand::SetMode(4) => Some(AppCommand::SetScreen(ScreenMode::BigDigit)),
                HidCommand::SetMode(5) => Some(AppCommand::SetScreen(ScreenMode::UsbDrive)),
                HidCommand::SetMode(0xFF) => Some(AppCommand::NextScreen),
                HidCommand::SetMode(_) => None,
                HidCommand::TareZero => Some(AppCommand::TareAndReset { show_banner: false }),
                HidCommand::FlushSd => Some(AppCommand::FlushSd),
                HidCommand::RotateLog => Some(AppCommand::RotateSdLog { show_banner: false }),
                HidCommand::RequestSummary => Some(AppCommand::DumpSession),
                HidCommand::SetBatteryProfile(p) => {
                    Some(AppCommand::SetBatteryProfile(BatteryProfile::from_id(p)))
                }
                HidCommand::SetEpoch(epoch) => Some(AppCommand::SetRtcEpoch(epoch)),
                HidCommand::SetCurrentLimit(limit) => Some(AppCommand::SetCurrentLimit(limit)),
                HidCommand::Unknown(c) => {
                    writeln!(board.uart0, "[HID-CMD] Unknown command opcode: 0x{:02X}", c).ok();
                    None
                }
            };
            if let Some(c) = app_cmd {
                app.execute_command(&mut board, c, now);
            }
        }

        // 4. Interactive Button Controls (PA8)
        let raw_pressed = board.button.is_pressed();
        let is_button_down = app.button.is_down(raw_pressed, now);

        // Tactile/Visual LED Blue feedback
        if is_button_down && app.button.state != ButtonState::TareExecuted {
            board.led_blue.on();
        } else {
            board.led_blue.off();
        }

        match app.button.update(raw_pressed, now) {
            ButtonEvent::None => {}

            ButtonEvent::SingleClick => {
                if app.screen_mode == ScreenMode::Graph
                    && app.graph_screen.trigger_mode == TriggerMode::Single
                    && app.graph_screen.trigger_state == TriggerState::Halted
                {
                    app.execute_command(
                        &mut board,
                        AppCommand::CycleTriggerOrRearm { show_banner: false },
                        now,
                    );
                } else {
                    let was_active_usb = app.screen_mode == ScreenMode::UsbDrive
                        && app.usb_drive_screen.state == UsbDriveState::Active;
                    app.execute_command(&mut board, AppCommand::NextScreen, now);
                    if was_active_usb {
                        writeln!(
                            board.uart0,
                            "[BUTTON] Single-click: Exited USB Drive -> Advance Screen -> {}",
                            app.screen_mode_name()
                        )
                        .ok();
                    } else {
                        writeln!(
                            board.uart0,
                            "[BUTTON] Single-click: Advance Screen -> {}",
                            app.screen_mode_name()
                        )
                        .ok();
                    }
                }
            }

            ButtonEvent::DoubleClick => {
                let cmd = match app.screen_mode {
                    ScreenMode::UsbDrive => AppCommand::ToggleUsbDrive,
                    ScreenMode::Graph => AppCommand::CycleTriggerOrRearm { show_banner: true },
                    ScreenMode::Stats | ScreenMode::BigDigit => {
                        AppCommand::CycleBatteryProfile { show_banner: true }
                    }
                    ScreenMode::Hero | ScreenMode::Histogram => {
                        AppCommand::RotateSdLog { show_banner: true }
                    }
                };
                app.execute_command(&mut board, cmd, now);
            }

            ButtonEvent::HoldProgress { step, duration_ms } => {
                app.button.overlay.show_hold_progress(step);
                app.render_top_bar_overlay(&mut board.lcd);
                writeln!(
                    board.uart0,
                    "[BUTTON] Holding PA8: {} ms [Step {}/10]",
                    duration_ms, step
                )
                .ok();
            }

            ButtonEvent::HoldAborted => {
                app.button.overlay.clear();
                app.draw_current_layout(&mut board);
                writeln!(board.uart0, "[BUTTON] Hold aborted (< 1500 ms)").ok();
            }

            ButtonEvent::TareTrigger => {
                board.led_blue.off();
                if app.screen_mode == ScreenMode::UsbDrive {
                    app.execute_command(&mut board, AppCommand::ToggleUsbDrive, now);
                } else {
                    app.execute_command(
                        &mut board,
                        AppCommand::TareAndReset { show_banner: true },
                        now,
                    );
                    writeln!(
                        board.uart0,
                        "[BUTTON] Long press (1500 ms): Tare Zero & Reset triggered!"
                    )
                    .ok();
                }
            }
        }

        // Overlay expiration check
        if app.button.overlay.is_banner_expired(now) {
            app.button.overlay.clear();
            app.draw_current_layout(&mut board);
        }

        // 3. Polite Green LED Heartbeat: Subtle 35ms pulse every 2.5s (2500ms)
        let hb_phase = now.wrapping_sub(app.last_heartbeat_ms);
        if hb_phase < 35 {
            board.led_green.on();
        } else {
            board.led_green.off();
            if hb_phase >= 2500 {
                app.last_heartbeat_ms = now;
            }
        }

        // 4. Periodic 10 Hz Measurement, Analytics & Display Rendering
        // Triggered every 100 ms (10.0 Hz). Conversion (~76.6 ms) is guaranteed complete,
        // avoiding redundant I2C pre-polling and reducing bus transactions to 2.
        let dt_ms = now.wrapping_sub(app.last_10hz_ms);
        if dt_ms >= 100 {
            app.last_10hz_ms = if dt_ms >= 200 { now } else { app.last_10hz_ms.wrapping_add(100) };

            let dt_sample_ms = now.wrapping_sub(app.last_sample_ms);
            app.last_sample_ms = now;

            let mut reading = InaReading::default();
            let mut sample_valid = false;

            if app.ina_present {
                match board.ina219.read_all() {
                    Ok(d) => {
                        sample_valid = true;
                        app.i2c_err_count = 0;
                        // Apply near-zero deadband to eliminate ADC quantization jitter
                        let c_tenth = if d.current_tenth_ma >= -1 && d.current_tenth_ma <= 1 {
                            0
                        } else {
                            d.current_tenth_ma
                        };
                        reading = InaReading {
                            voltage_mv: d.voltage_mv,
                            current_tenth_ma: c_tenth,
                            power_tenth_mw: d.power_tenth_mw,
                            overflow: d.overflow,
                            is_reverse: c_tenth < 0,
                        };
                        app.sec_agg.record(&reading);
                        board.led_red.off();
                    }
                    Err(_) => {
                        app.i2c_err_count = app.i2c_err_count.saturating_add(1);
                        board.ina219.recover_bus(&mut board.delay);
                        if app.i2c_err_count >= 3 {
                            app.ina_present = false;
                            board.led_red.on();
                            writeln!(board.uart0, "[I2C] INA219 offline after 3 read errors - bus recovered").ok();
                        } else {
                            reading = app.last_reading;
                            writeln!(
                                board.uart0,
                                "[I2C] INA219 transient read error ({}/3) - bus recovered",
                                app.i2c_err_count
                            )
                            .ok();
                        }
                    }
                }
            } else {
                // Hot-unplug recovery retry every 1.5 seconds
                if now.wrapping_sub(app.last_reinit_attempt_ms) >= 1500 {
                    app.last_reinit_attempt_ms = now;
                    board.ina219.recover_bus(&mut board.delay);
                    if board.ina219.init(4096).is_ok() {
                        app.ina_present = true;
                        app.i2c_err_count = 0;
                        app.last_sample_ms = now;
                        board.led_red.off();
                        writeln!(board.uart0, "[I2C] INA219 hot-plug reconnected!").ok();
                    }
                }
            }

            // Update waveform circular history buffer (pause when Single capture is halted)
            if sample_valid && !(app.graph_screen.trigger_mode == TriggerMode::Single && app.graph_screen.trigger_state == TriggerState::Halted) {
                app.history.push(reading.current_tenth_ma);
            }

            // Only record histogram and session statistics when sample is valid
            if sample_valid {
                app.histogram.record(reading.abs_current_tenth());
                app.stats.update(reading.voltage_mv, reading.current_tenth_ma, reading.power_tenth_mw);
            }

            // Update Battery Fuel Gauge
            app.battery.update(reading.voltage_mv, &app.accum, app.stats.avg_current_tenth_ma());

            // Check Over-Current (using absolute current magnitude and saturation overflow) and Low-Voltage Alert
            let is_over_current = app.current_limit_ma > 0
                && (reading.abs_current_tenth() >= ((app.current_limit_ma as u32) * 10)
                    || (app.current_limit_ma >= 3200 && reading.overflow));
            let is_low_voltage = app.battery.is_low_voltage;
            app.over_current = is_over_current;
            let is_alert = is_over_current || is_low_voltage;
            if is_alert || !app.ina_present {
                board.led_red.on();
            } else {
                board.led_red.off();
            }

            app.last_reading = reading;

            // Drift-free millisecond energy & charge integration
            if sample_valid && reading.voltage_mv > 500 && reading.current_tenth_ma != 0 {
                app.accum.update(reading.power_tenth_mw, reading.current_tenth_ma, dt_sample_ms.min(500));
            }

            let usb_configured = board.is_usb_configured();

            // Screen Rendering
            match app.screen_mode {
                ScreenMode::Hero => {
                    app.hero_screen.update(
                        &mut board.lcd,
                        &reading,
                        &app.accum,
                        app.ina_present,
                        usb_configured,
                        app.datalogger.status,
                        app.over_current,
                    );
                }
                ScreenMode::Graph => {
                    app.graph_screen.update(
                        &mut board.lcd,
                        &reading,
                        &app.accum,
                        &app.history,
                        app.ina_present,
                        app.datalogger.status,
                        usb_configured,
                    );
                }
                ScreenMode::Stats => {
                    let cur_dt = if board.rtc.is_synced() {
                        Some(board.rtc.get_datetime())
                    } else {
                        None
                    };
                    app.stats_screen.update(
                        &mut board.lcd,
                        &reading,
                        &app.stats,
                        &app.accum,
                        &app.battery,
                        app.datalogger.status,
                        app.datalogger.total_logged_rows,
                        now,
                        usb_configured,
                        cur_dt,
                    );
                }
                ScreenMode::Histogram => {
                    app.histogram_screen.update(&mut board.lcd, &app.histogram);
                }
                ScreenMode::BigDigit => {
                    app.big_digit_screen.update(
                        &mut board.lcd,
                        &reading,
                        &app.accum,
                        &app.battery,
                        app.ina_present,
                        usb_configured,
                        app.datalogger.status,
                        app.over_current,
                    );
                }
                ScreenMode::UsbDrive => {
                    // Check if active MSC is idle before SPI0 LCD draw to prevent USB BOT timeouts
                    let can_render = match &board.usb {
                        UsbMode::Msc(msc) => msc.is_idle(),
                        _ => true,
                    };
                    if can_render {
                        let msc_stats = match &board.usb {
                            UsbMode::Msc(msc) => Some(&msc.stats),
                            _ => None,
                        };
                        let sectors = sdcard_cell.borrow().sector_count;
                        let filename = app.datalogger.file_name_str();
                        app.usb_drive_screen.update(
                            &mut board.lcd,
                            app.datalogger.status,
                            sectors,
                            filename,
                            msc_stats,
                        );
                    }
                }
            }

            // Re-assert top bar overlay if active so normal screen render doesn't blank it
            if app.button.overlay.is_active {
                app.render_top_bar_overlay(&mut board.lcd);
            }

            // USB HID Telemetry Streaming (9 bytes @ 10 Hz)
            if let Some(hid) = board.usb_hid() {
                app.telemetry.send_reading(hid, &reading, app.ina_present, app.datalogger.status, app.over_current);
            }
        }

        // 5. 1 Hz MicroSD CSV Datalogger & Heartbeat Logging
        let dt_1hz_ms = now.wrapping_sub(app.last_1hz_ms);
        if dt_1hz_ms >= 1000 {
            app.last_1hz_ms = if dt_1hz_ms >= 2000 { now } else { app.last_1hz_ms.wrapping_add(1000) };

            // Sample internal MCU die temperature once per second (relocated from 10 Hz loop)
            let mcu_temp_tenth = board.mcu_temp.read_temperature_tenth_c();
            app.stats.update_temp(mcu_temp_tenth);

            // Only probe and log if not in Active USB Drive mode (exclusive host access)
            let is_msc_active = app.screen_mode == ScreenMode::UsbDrive
                && app.usb_drive_screen.state == UsbDriveState::Active;

            if !is_msc_active {
                // Probe card if missing (with exponential backoff)
                if app.datalogger.status == SdStatus::NoCard || app.datalogger.status == SdStatus::WriteError {
                    if app.datalogger.check_probe(now, &mut board.delay) {
                        let sectors = sdcard_cell.borrow().sector_count;
                        writeln!(
                            board.uart0,
                            "[SD] Card hot-plug inserted & ready ({} sectors) | File: {}",
                            sectors,
                            app.datalogger.file_name_str()
                        )
                        .ok();
                    }
                }

                // Log representative row to sector buffer using true 1s arithmetic average from sample aggregator
                if app.ina_present {
                    if let Ok(true) = board.ina219.check_and_restore_calibration() {
                        writeln!(board.uart0, "[I2C] INA219 brownout detected - restored calibration!").ok();
                    }

                    let cur_dt = if board.rtc.is_synced() {
                        Some(board.rtc.get_datetime())
                    } else {
                        None
                    };
                    let agg_reading = app.sec_agg.finish_and_reset(&app.last_reading);
                    app.datalogger.log_sample(now, &agg_reading, &app.accum, cur_dt);
                }
            }

            // UART0 Heartbeat & Diagnostic Output (silenced during Active USB Drive mode to avoid xHCI NAK stalls)
            if !is_msc_active {
                let sec = now / 1000;
                let cfg_str = if board.is_usb_configured() { "CFG" } else { "DISC" };
                let sensor_str = if app.ina_present { "ONLINE" } else { "OFFLINE" };
                let sd_str = match app.datalogger.status {
                    SdStatus::Logging => "LOGGING",
                    SdStatus::Ready => "READY",
                    SdStatus::NoCard => "NO_CARD",
                    SdStatus::WriteError => "ERROR",
                };
                let mode_str = match app.screen_mode {
                    ScreenMode::Hero => "HERO",
                    ScreenMode::Graph => "GRAPH",
                    ScreenMode::Stats => "STATS",
                    ScreenMode::Histogram => "HISTO",
                    ScreenMode::BigDigit => "BIG",
                    ScreenMode::UsbDrive => "USB_DRIVE",
                };
                let temp_tenth = app.stats.mcu_temp_tenth_c;

                if board.rtc.is_synced() {
                    let dt = board.rtc.get_datetime();
                    writeln!(
                        board.uart0,
                        "[HEARTBEAT] {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC (T+{}s) | Mode: {} | Sensor: {} | MCU: {}.{}C | BATT: {}% | USB: {} | SD: {} ({} : {} rows)",
                        dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
                        sec, mode_str, sensor_str, temp_tenth / 10, temp_tenth.unsigned_abs() % 10,
                        app.battery.soc_pct, cfg_str, sd_str,
                        app.datalogger.file_name_str(), app.datalogger.total_logged_rows
                    )
                    .ok();
                } else {
                    writeln!(
                        board.uart0,
                        "[HEARTBEAT] T+{}s | Mode: {} | Sensor: {} | MCU: {}.{}C | BATT: {}% | USB: {} | SD: {} ({} : {} rows)",
                        sec, mode_str, sensor_str, temp_tenth / 10, temp_tenth.unsigned_abs() % 10,
                        app.battery.soc_pct, cfg_str, sd_str,
                        app.datalogger.file_name_str(), app.datalogger.total_logged_rows
                    )
                    .ok();
                }
            }
        }
    }
}

#[cfg(test)]
fn main() {}
