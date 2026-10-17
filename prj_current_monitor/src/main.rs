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

use longan_nano_bsp::Board;

use crate::app::{App, BKP_MAGIC_KEY, BKP_REG_CONFIG, BKP_REG_LIMIT, BKP_REG_MAGIC, BKP_REG_TARE};
use crate::button::{ButtonEvent, ButtonState};
use crate::logger::SdDatalogger;
use crate::model::{BatteryProfile, InaReading};
use crate::telemetry::HidCommand;
use crate::ui::{ScreenMode, SdStatus, TriggerMode, TriggerState};

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
    writeln!(board.uart0, " MicroSD: SPI1 | USB HID: VID 0x28E9, PID 0x1234").ok();
    let init_epoch = board.rtc.get_epoch();
    let init_dt = board.rtc.get_datetime();
    writeln!(
        board.uart0,
        " RTC: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC (Epoch: {}, Synced: {})",
        init_dt.year, init_dt.month, init_dt.day, init_dt.hour, init_dt.minute, init_dt.second,
        init_epoch,
        if board.rtc.is_synced() { "YES" } else { "NO" }
    ).ok();
    writeln!(board.uart0, " Commands: '1'..'5' Mode, 'm' Cycle, 'b' Bat, 't' Tare, 'c' Clock, 'r' Rotate, 's' Dump, '?' Help").ok();
    writeln!(board.uart0, "========================================").ok();

    // Initialize ST7735 160x80 LCD
    board.lcd.init(&mut board.delay);

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

    let mut app = App::new(datalogger, board.delay.uptime_ms());

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
        board.usb_hid.poll();

        let usb_configured = board.usb_hid.is_configured();
        if usb_configured && !app.prev_usb_configured {
            let uptime = now;
            writeln!(
                board.uart0,
                "[USB] >>> USB HID State: CONFIGURED [Time: {} ms] <<<",
                uptime
            )
            .ok();
            app.prev_usb_configured = true;
        } else if !usb_configured && app.prev_usb_configured {
            writeln!(board.uart0, "[USB] >>> USB HID State: DISCONNECTED <<<").ok();
            app.prev_usb_configured = false;
        }

        // 1.5 Rotary Encoder Polling (PB10/PB11/PB5)
        let (rot, enc_pressed) = board.encoder.poll();
        if rot != 0 {
            if app.screen_mode == ScreenMode::Graph {
                // In Graph mode: adjust oscilloscope trigger level (+-10.0 mA per click)
                let delta = if rot > 0 { 100 } else { -100 };
                app.graph_screen.adjust_trigger_level(delta);
                writeln!(
                    board.uart0,
                    "[TRIG] Level: {}.{} mA | Mode: {} | State: {}",
                    app.graph_screen.trigger_level_tenth / 10,
                    app.graph_screen.trigger_level_tenth % 10,
                    app.graph_screen.trigger_mode.name(),
                    app.graph_screen.trigger_state_name(),
                ).ok();
            } else if rot > 0 {
                app.cycle_screen_mode(&mut board);
            } else if rot < 0 {
                app.cycle_screen_mode_rev(&mut board);
            }
        }

        if enc_pressed {
            if app.screen_mode == ScreenMode::Graph {
                // In Graph mode: Cycle trigger mode (AUTO -> NORM -> SINGLE) or Rearm if halted
                if app.graph_screen.trigger_mode == TriggerMode::Single && app.graph_screen.trigger_state == TriggerState::Halted {
                    app.graph_screen.rearm();
                    writeln!(board.uart0, "[TRIG] Single sweep rearmed! Waiting for trigger...").ok();
                } else {
                    app.graph_screen.cycle_trigger_mode();
                    writeln!(
                        board.uart0,
                        "[TRIG] Switched trigger mode to: {} | Level: {}.{} mA",
                        app.graph_screen.trigger_mode.name(),
                        app.graph_screen.trigger_level_tenth / 10,
                        app.graph_screen.trigger_level_tenth % 10,
                    ).ok();
                }
            } else {
                app.battery.profile = app.battery.profile.next();
                app.save_nvram(&mut board);
                writeln!(
                    board.uart0,
                    "[ROTARY] Switched battery profile to: {} (Nominal: {} mAh)",
                    app.battery.profile.name(),
                    app.battery.profile.capacity_mah()
                ).ok();
            }
        }

        // 2. Remote UART0 Interactive Command Console
        while let Some(ch) = board.uart0.read_byte() {
            if app.cmd_len > 0 {
                // We are buffering a multi-byte command (started with 'T' or 'L')
                match ch {
                    b'0'..=b'9' | b' ' | b':' => {
                        if app.cmd_len < app.cmd_buf.len() {
                            app.cmd_buf[app.cmd_len] = ch;
                            app.cmd_len += 1;
                        }
                    }
                    b'\r' | b'\n' => {
                        if app.cmd_len >= 2 && app.cmd_buf[0] == b'T' {
                            let line = &app.cmd_buf[..app.cmd_len];
                            let mut idx = 1;
                            while idx < app.cmd_len && (line[idx] == b' ' || line[idx] == b':') {
                                idx += 1;
                            }
                            let mut parsed_epoch = 0u32;
                            let mut has_digits = false;
                            while idx < app.cmd_len && line[idx] >= b'0' && line[idx] <= b'9' {
                                parsed_epoch = parsed_epoch.saturating_mul(10).saturating_add((line[idx] - b'0') as u32);
                                has_digits = true;
                                idx += 1;
                            }
                            if has_digits && parsed_epoch > 0 {
                                board.rtc.set_epoch(parsed_epoch);
                                let dt = board.rtc.get_datetime();
                                writeln!(
                                    board.uart0,
                                    "[RTC] Set Unix epoch to: {} ({:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC)",
                                    parsed_epoch, dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
                                ).ok();
                            }
                        } else if app.cmd_len == 1 && app.cmd_buf[0] == b'T' {
                            app.tare_and_reset(&mut board, now);
                        } else if app.cmd_len >= 2 && app.cmd_buf[0] == b'L' {
                            let line = &app.cmd_buf[..app.cmd_len];
                            let mut idx = 1;
                            while idx < app.cmd_len && (line[idx] == b' ' || line[idx] == b':') {
                                idx += 1;
                            }
                            let mut parsed_limit = 0u16;
                            let mut has_digits = false;
                            while idx < app.cmd_len && line[idx] >= b'0' && line[idx] <= b'9' {
                                parsed_limit = parsed_limit.saturating_mul(10).saturating_add((line[idx] - b'0') as u16);
                                has_digits = true;
                                idx += 1;
                            }
                            if has_digits {
                                app.current_limit_ma = parsed_limit.min(3200);
                                app.save_nvram(&mut board);
                                writeln!(board.uart0, "[LIMIT] Set Over-Current Alert Limit to: {} mA", app.current_limit_ma).ok();
                            }
                        } else if app.cmd_len == 1 && app.cmd_buf[0] == b'L' {
                            writeln!(
                                board.uart0,
                                "[LIMIT] Over-Current Alert Limit: {} mA (Alert Active: {})",
                                app.current_limit_ma,
                                if app.over_current { "YES" } else { "NO" }
                            ).ok();
                        }
                        app.cmd_len = 0;
                    }
                    _ => {
                        app.cmd_len = 0;
                    }
                }
            } else {
                match ch {
                    b'1' => {
                        app.set_screen_mode(&mut board, ScreenMode::Hero);
                    }
                    b'2' => {
                        app.set_screen_mode(&mut board, ScreenMode::Graph);
                    }
                    b'3' => {
                        app.set_screen_mode(&mut board, ScreenMode::Stats);
                    }
                    b'4' => {
                        app.set_screen_mode(&mut board, ScreenMode::Histogram);
                    }
                    b'5' => {
                        app.set_screen_mode(&mut board, ScreenMode::BigDigit);
                    }
                    b'm' | b'M' => {
                        app.cycle_screen_mode(&mut board);
                    }
                    b'g' | b'G' => {
                        if app.graph_screen.trigger_mode == TriggerMode::Single && app.graph_screen.trigger_state == TriggerState::Halted {
                            app.graph_screen.rearm();
                            writeln!(board.uart0, "[TRIG] Single sweep rearmed! Waiting for trigger...").ok();
                        } else {
                            app.graph_screen.cycle_trigger_mode();
                            writeln!(
                                board.uart0,
                                "[TRIG] Trigger Mode: {} | State: {} | Level: {}.{} mA",
                                app.graph_screen.trigger_mode.name(),
                                app.graph_screen.trigger_state_name(),
                                app.graph_screen.trigger_level_tenth / 10,
                                app.graph_screen.trigger_level_tenth % 10,
                            ).ok();
                        }
                    }
                    b'b' | b'B' => {
                        app.battery.profile = app.battery.profile.next();
                        app.save_nvram(&mut board);
                        writeln!(
                            board.uart0,
                            "[BATT] Switched battery profile to: {} (Nominal: {} mAh)",
                            app.battery.profile.name(),
                            app.battery.profile.capacity_mah()
                        ).ok();
                    }
                    b't' => {
                        app.tare_and_reset(&mut board, now);
                    }
                    b'c' | b'C' => {
                        let epoch = board.rtc.get_epoch();
                        let dt = board.rtc.get_datetime();
                        writeln!(
                            board.uart0,
                            "[RTC] Current Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC | Epoch: {} | Synced: {}",
                            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
                            epoch,
                            if board.rtc.is_synced() { "YES" } else { "NO" }
                        ).ok();
                    }
                    b'l' => {
                        writeln!(
                            board.uart0,
                            "[LIMIT] Over-Current Alert Limit: {} mA (Alert Active: {})",
                            app.current_limit_ma,
                            if app.over_current { "YES" } else { "NO" }
                        ).ok();
                    }
                    b'L' => {
                        app.cmd_buf[0] = b'L';
                        app.cmd_len = 1;
                    }
                    b'T' => {
                        app.cmd_buf[0] = b'T';
                        app.cmd_len = 1;
                    }
                    b'r' | b'R' | b'n' | b'N' => {
                        app.datalogger.rotate_session();
                        writeln!(board.uart0, "[SD] Rotated to new file: {}", app.datalogger.file_name_str()).ok();
                    }
                    b'f' | b'F' => {
                        app.datalogger.flush_buffer();
                        writeln!(board.uart0, "[SD] Flushed buffer to {} (Total rows: {})", app.datalogger.file_name_str(), app.datalogger.total_logged_rows).ok();
                    }
                    b's' | b'S' => {
                        app.dump_session(&mut board, now);
                    }
                    b'k' => {
                        app.button.simulate_tap(now, 80);
                        writeln!(board.uart0, "[SIM] Simulated button tap (80 ms)").ok();
                    }
                    b'K' => {
                        app.button.simulate_double_tap(now, 80, 100);
                        writeln!(board.uart0, "[SIM] Simulated button double-tap (80ms + 100ms gap + 80ms)").ok();
                    }
                    b'P' => {
                        app.button.simulate_hold(now, 1600);
                        writeln!(board.uart0, "[SIM] Simulated button hold (1600 ms)").ok();
                    }
                    b'p' => {
                        app.button.simulate_hold(now, 600);
                        writeln!(board.uart0, "[SIM] Simulated button aborted hold (600 ms)").ok();
                    }
                    b'?' | b'h' | b'H' => {
                        app.print_help(&mut board);
                    }
                    b'\r' | b'\n' => {}
                    _ => {
                        writeln!(board.uart0, "[CMD] Unknown key: '{}' (Send '?' for help)", ch as char).ok();
                    }
                }
            }
        }

        // 3. Bidirectional USB HID OUT Command Polling
        if let Some(cmd) = app.telemetry.poll_command(&mut board.usb_hid) {
            match cmd {
                HidCommand::SetMode(m) => {
                    let next = match m {
                        0 => Some(ScreenMode::Hero),
                        1 => Some(ScreenMode::Graph),
                        2 => Some(ScreenMode::Stats),
                        3 => Some(ScreenMode::Histogram),
                        4 => Some(ScreenMode::BigDigit),
                        0xFF => Some(app.next_screen_mode()),
                        _ => None,
                    };
                    if let Some(mode) = next {
                        app.set_screen_mode(&mut board, mode);
                    }
                }
                HidCommand::TareZero => app.tare_and_reset(&mut board, now),
                HidCommand::FlushSd => {
                    app.datalogger.flush_buffer();
                    writeln!(board.uart0, "[HID-CMD] Flushed SD buffer to {}", app.datalogger.file_name_str()).ok();
                }
                HidCommand::RotateLog => {
                    app.datalogger.rotate_session();
                    writeln!(board.uart0, "[HID-CMD] Rotated log file to {}", app.datalogger.file_name_str()).ok();
                }
                HidCommand::RequestSummary => app.dump_session(&mut board, now),
                HidCommand::SetBatteryProfile(p) => {
                    app.battery.profile = match p {
                        1 => BatteryProfile::Lipo500,
                        2 => BatteryProfile::Lipo1200,
                        3 => BatteryProfile::LiIon2500,
                        4 => BatteryProfile::Alkaline1000,
                        _ => BatteryProfile::None,
                    };
                    app.save_nvram(&mut board);
                    writeln!(board.uart0, "[HID-CMD] Battery profile set to: {}", app.battery.profile.name()).ok();
                }
                HidCommand::SetEpoch(epoch) => {
                    board.rtc.set_epoch(epoch);
                    let dt = board.rtc.get_datetime();
                    writeln!(
                        board.uart0,
                        "[HID-CMD] Synchronized RTC epoch to: {} ({:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC)",
                        epoch, dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
                    ).ok();
                }
                HidCommand::SetCurrentLimit(limit) => {
                    app.current_limit_ma = limit.min(3200);
                    app.save_nvram(&mut board);
                    writeln!(board.uart0, "[HID-CMD] Set Over-Current Alert Limit to: {} mA", app.current_limit_ma).ok();
                }
                HidCommand::Unknown(c) => {
                    writeln!(board.uart0, "[HID-CMD] Unknown command opcode: 0x{:02X}", c).ok();
                }
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

        let btn_event = app.button.update(raw_pressed, now);
        match btn_event {
            ButtonEvent::None => {}

            ButtonEvent::SingleClick => {
                if app.screen_mode == ScreenMode::Graph
                    && app.graph_screen.trigger_mode == TriggerMode::Single
                    && app.graph_screen.trigger_state == TriggerState::Halted
                {
                    app.graph_screen.rearm();
                    writeln!(board.uart0, "[TRIG] Single sweep rearmed! Waiting for trigger...").ok();
                } else {
                    app.cycle_screen_mode(&mut board);
                    writeln!(board.uart0, "[BUTTON] Single-click: Advance Screen -> {}", app.screen_mode_name()).ok();
                }
            }

            ButtonEvent::DoubleClick => {
                match app.screen_mode {
                    ScreenMode::Graph => {
                        app.graph_screen.cycle_trigger_mode();
                        let trig_name = app.graph_screen.trigger_mode.name();
                        writeln!(
                            board.uart0,
                            "[BUTTON] Double-click: Trigger Mode -> {} | Level: {}.{} mA",
                            trig_name,
                            app.graph_screen.trigger_level_tenth / 10,
                            app.graph_screen.trigger_level_tenth % 10,
                        ).ok();
                        let banner = match app.graph_screen.trigger_mode {
                            TriggerMode::Auto => " TRIG MODE: AUTO        ",
                            TriggerMode::Norm => " TRIG MODE: NORMAL      ",
                            TriggerMode::Single => " TRIG MODE: SINGLE      ",
                        };
                        app.button.overlay.show_context_banner(banner, now, 700);
                        app.render_top_bar_overlay(&mut board.lcd);
                    }
                    ScreenMode::Stats | ScreenMode::BigDigit => {
                        app.battery.profile = app.battery.profile.next();
                        app.save_nvram(&mut board);
                        writeln!(
                            board.uart0,
                            "[BUTTON] Double-click: Battery profile -> {} (Nominal: {} mAh)",
                            app.battery.profile.name(),
                            app.battery.profile.capacity_mah(),
                        ).ok();
                        let banner = match app.battery.profile {
                            BatteryProfile::None => " BATT: NONE (Disabled)  ",
                            BatteryProfile::Lipo500 => " BATT: LIPO 500 mAh     ",
                            BatteryProfile::Lipo1200 => " BATT: LIPO 1200 mAh    ",
                            BatteryProfile::LiIon2500 => " BATT: LI-ION 2500 mAh  ",
                            BatteryProfile::Alkaline1000 => " BATT: ALKALINE 1000 mAh",
                        };
                        app.button.overlay.show_context_banner(banner, now, 700);
                        app.render_top_bar_overlay(&mut board.lcd);
                    }
                    ScreenMode::Hero | ScreenMode::Histogram => {
                        app.datalogger.rotate_session();
                        writeln!(
                            board.uart0,
                            "[BUTTON] Double-click: SD Rotated to new file: {}",
                            app.datalogger.file_name_str(),
                        ).ok();
                        app.button.overlay.show_context_banner(" SD: ROTATED LOG FILE   ", now, 700);
                        app.render_top_bar_overlay(&mut board.lcd);
                    }
                }
            }

            ButtonEvent::HoldProgress { step, duration_ms } => {
                app.button.overlay.show_hold_progress(step);
                app.render_top_bar_overlay(&mut board.lcd);
                writeln!(
                    board.uart0,
                    "[BUTTON] Holding PA8: {} ms [Step {}/10]",
                    duration_ms, step
                ).ok();
            }

            ButtonEvent::HoldAborted => {
                app.button.overlay.clear();
                app.draw_current_layout(&mut board);
                writeln!(board.uart0, "[BUTTON] Hold aborted (< 1500 ms)").ok();
            }

            ButtonEvent::TareTrigger => {
                app.tare_and_reset(&mut board, now);
                app.button.overlay.show_tare_confirmed(now, 1000);
                app.render_top_bar_overlay(&mut board.lcd);
                board.led_blue.off();
                writeln!(board.uart0, "[BUTTON] Long press (1500 ms): Tare Zero & Reset triggered!").ok();
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
        // avoiding redundant I2C pre-polling and reducing bus transactions to exactly 3.
        let dt_ms = now.wrapping_sub(app.last_10hz_ms);
        if dt_ms >= 100 {
            app.last_10hz_ms = now;

            let mut reading = InaReading::default();

            if app.ina_present {
                match board.ina219.read_all() {
                    Ok(d) => {
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
                        app.ina_present = false;
                        board.led_red.on();
                        board.ina219.recover_bus(&mut board.delay);
                        writeln!(board.uart0, "[I2C] INA219 read error - bus recovered").ok();
                    }
                }
            } else {
                // Hot-unplug recovery retry every 1.5 seconds
                if now.wrapping_sub(app.last_reinit_attempt_ms) >= 1500 {
                    app.last_reinit_attempt_ms = now;
                    board.ina219.recover_bus(&mut board.delay);
                    if board.ina219.init(4096).is_ok() {
                        app.ina_present = true;
                        board.led_red.off();
                        writeln!(board.uart0, "[I2C] INA219 hot-plug reconnected!").ok();
                    }
                }
            }

            // Update waveform circular history buffer (pause when Single capture is halted)
            if !(app.graph_screen.trigger_mode == TriggerMode::Single && app.graph_screen.trigger_state == TriggerState::Halted) {
                app.history.push(reading.current_tenth_ma);
            }

            // Update current distribution histogram
            app.histogram.record(reading.abs_current_tenth());

            // Sample internal MCU die temperature
            let mcu_temp_tenth = board.mcu_temp.read_temperature_tenth_c();
            app.stats.update_temp(mcu_temp_tenth);

            // Update DSP Session Stats
            app.stats.update(reading.voltage_mv, reading.current_tenth_ma, reading.power_tenth_mw);

            // Update Battery Fuel Gauge
            app.battery.update(reading.voltage_mv, &app.accum, app.stats.avg_current_tenth_ma());

            // Check Over-Current and Low-Voltage Alert
            let is_over_current = app.current_limit_ma > 0 && (reading.current_tenth_ma as i32) > ((app.current_limit_ma as i32) * 10);
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
            if app.ina_present && reading.voltage_mv > 500 && reading.current_tenth_ma != 0 {
                app.accum.update(reading.power_tenth_mw, reading.current_tenth_ma, dt_ms);
            }

            // Screen Rendering
            match app.screen_mode {
                ScreenMode::Hero => {
                    app.hero_screen.update(
                        &mut board.lcd,
                        &reading,
                        &app.accum,
                        app.ina_present,
                        board.usb_hid.is_configured(),
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
                        board.usb_hid.is_configured(),
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
                        board.usb_hid.is_configured(),
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
                        board.usb_hid.is_configured(),
                        app.datalogger.status,
                        app.over_current,
                    );
                }
            }

            // Re-assert top bar overlay if active so normal screen render doesn't blank it
            if app.button.overlay.is_active {
                app.render_top_bar_overlay(&mut board.lcd);
            }

            // USB HID Telemetry Streaming (9 bytes @ 10 Hz)
            app.telemetry.send_reading(&mut board.usb_hid, &reading, app.ina_present, app.datalogger.status, app.over_current);
        }

        // 5. 1 Hz MicroSD CSV Datalogger & Heartbeat Logging
        if now.wrapping_sub(app.last_1hz_ms) >= 1000 {
            app.last_1hz_ms = now;

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

            // UART0 Heartbeat & Diagnostic Output
            let sec = now / 1000;
            let cfg_str = if board.usb_hid.is_configured() { "CFG" } else { "DISC" };
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

#[cfg(test)]
fn main() {}
