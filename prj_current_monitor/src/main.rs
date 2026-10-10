#![no_std]
#![no_main]

mod fmt;
mod logger;
mod model;
mod telemetry;
mod ui;

use core::cell::RefCell;
use core::fmt::Write;
use panic_halt as _;
use riscv_rt::entry;

use longan_nano_bsp::Board;

use crate::logger::SdDatalogger;
use crate::model::{Accumulators, BatteryProfile, BatteryState, HistogramData, InaReading, SessionStats, WaveformHistory};
use crate::telemetry::{HidCommand, TelemetryStreamer};
use crate::ui::{BigDigitScreen, GraphScreen, HeroScreen, HistogramScreen, ScreenMode, SdStatus, StatsScreen};

macro_rules! set_screen_mode {
    ($new_mode:expr, $screen_mode:expr, $board:expr, $hero:expr, $graph:expr, $stats:expr, $histo:expr, $big_digit:expr, $hist:expr) => {{
        $screen_mode = $new_mode;
        match $screen_mode {
            ScreenMode::Hero => $hero.draw_layout(&mut $board.lcd),
            ScreenMode::Graph => {
                $graph.draw_layout(&mut $board.lcd);
                $graph.replot_history(&mut $board.lcd, &$hist);
            }
            ScreenMode::Stats => $stats.draw_layout(&mut $board.lcd),
            ScreenMode::Histogram => $histo.draw_layout(&mut $board.lcd),
            ScreenMode::BigDigit => $big_digit.draw_layout(&mut $board.lcd),
        }
        writeln!(
            $board.uart0,
            "[UI] Screen mode: {}",
            match $screen_mode {
                ScreenMode::Hero => "1: HERO",
                ScreenMode::Graph => "2: GRAPH (Oscilloscope)",
                ScreenMode::Stats => "3: STATS (Dashboard)",
                ScreenMode::Histogram => "4: HISTOGRAM",
                ScreenMode::BigDigit => "5: BIG DIGIT",
            }
        ).ok();
    }};
}

macro_rules! tare_and_reset {
    ($now:expr, $ina_present:expr, $board:expr, $datalogger:expr, $accum:expr, $hist:expr, $histo:expr, $stats:expr, $screen_mode:expr, $hero:expr, $graph:expr, $stats_s:expr, $histo_s:expr, $big_digit:expr) => {{
        if $ina_present {
            if let Ok(raw_data) = $board.ina219.read_all() {
                let current_raw = raw_data.current_tenth_ma;
                $board.ina219.set_tare(current_raw);
                writeln!(
                    $board.uart0,
                    "[SYS] Tare Zero Offset Calibrated: {}.{} mA",
                    current_raw / 10,
                    current_raw.unsigned_abs() % 10
                ).ok();
            }
        }
        $datalogger.flush_buffer();
        $datalogger.rotate_session();
        $accum.reset();
        $hist.clear();
        $histo.clear();
        $stats.reset($now);
        writeln!(
            $board.uart0,
            "[SYS] Session Reset! Rotated to: {}",
            $datalogger.file_name_str()
        ).ok();

        match $screen_mode {
            ScreenMode::Hero => $hero.draw_layout(&mut $board.lcd),
            ScreenMode::Graph => $graph.draw_layout(&mut $board.lcd),
            ScreenMode::Stats => $stats_s.draw_layout(&mut $board.lcd),
            ScreenMode::Histogram => $histo_s.draw_layout(&mut $board.lcd),
            ScreenMode::BigDigit => $big_digit.draw_layout(&mut $board.lcd),
        }
    }};
}

macro_rules! dump_session {
    ($now:expr, $screen_mode:expr, $board:expr, $datalogger:expr, $stats:expr, $accum:expr, $battery:expr) => {{
        let uptime_s = $now / 1000;
        let mode_name = match $screen_mode {
            ScreenMode::Hero => "HERO",
            ScreenMode::Graph => "GRAPH",
            ScreenMode::Stats => "STATS",
            ScreenMode::Histogram => "HISTO",
            ScreenMode::BigDigit => "BIG",
        };
        let c_min = if $stats.has_samples { $stats.c_min } else { 0 };
        let c_max = if $stats.has_samples { $stats.c_max } else { 0 };
        let v_min = if $stats.has_samples { $stats.v_min } else { 0 };
        let v_max = if $stats.has_samples { $stats.v_max } else { 0 };
        let p_peak_mw = $stats.p_max / 10;
        let avg_c = $stats.avg_current_tenth_ma();
        let mcu_temp_tenth = $stats.mcu_temp_tenth_c;

        writeln!($board.uart0, "\r\n--- SESSION TELEMETRY SUMMARY ---").ok();
        writeln!($board.uart0, "Uptime: {} s | Mode: {} | MCU Temp: {}.{} C", uptime_s, mode_name, mcu_temp_tenth / 10, mcu_temp_tenth.unsigned_abs() % 10).ok();
        writeln!(
            $board.uart0,
            "Voltage: Min {} mV, Max {} mV",
            v_min, v_max
        ).ok();
        writeln!(
            $board.uart0,
            "Current: Min {}.{} mA, Max {}.{} mA, Avg {}.{} mA",
            c_min / 10,
            c_min.unsigned_abs() % 10,
            c_max / 10,
            c_max.unsigned_abs() % 10,
            avg_c / 10,
            avg_c % 10
        ).ok();
        writeln!(
            $board.uart0,
            "Peak Power: {} mW",
            p_peak_mw
        ).ok();
        writeln!(
            $board.uart0,
            "Energy: {}.{:02} mWh | Charge: {}.{:02} mAh",
            $accum.energy_mwh(), $accum.energy_mwh_frac(),
            $accum.charge_mah(), $accum.charge_mah_frac()
        ).ok();
        writeln!(
            $board.uart0,
            "Battery: Profile {} | SoC: {}% | Rem: {} mAh ({} mins)",
            $battery.profile.name(), $battery.soc_pct, $battery.rem_mah, $battery.time_rem_mins
        ).ok();
        writeln!(
            $board.uart0,
            "SD File: {} (Rows: {}, Status: {:?})",
            $datalogger.file_name_str(),
            $datalogger.total_logged_rows,
            $datalogger.status
        ).ok();
        writeln!(
            $board.uart0,
            "[JSON] {{\"uptime_s\":{},\"mode\":\"{}\",\"v_min\":{},\"v_max\":{},\"c_min\":{},\"c_max\":{},\"c_avg\":{},\"p_peak_mw\":{},\"mwh\":{},\"mah\":{},\"mcu_temp_c\":{}.{},\"bat_soc\":{},\"sd_file\":\"{}\",\"sd_rows\":{}}}",
            uptime_s,
            mode_name,
            v_min,
            v_max,
            c_min,
            c_max,
            avg_c,
            p_peak_mw,
            $accum.energy_mwh(),
            $accum.charge_mah(),
            mcu_temp_tenth / 10,
            mcu_temp_tenth.unsigned_abs() % 10,
            $battery.soc_pct,
            $datalogger.file_name_str(),
            $datalogger.total_logged_rows
        ).ok();
        writeln!($board.uart0, "---------------------------------").ok();
    }};
}

macro_rules! print_help {
    ($board:expr) => {{
        writeln!($board.uart0, "\r\n=== REMOTE COMMAND CONSOLE ===").ok();
        writeln!($board.uart0, " '1'..'5' : Set Screen (1:Hero, 2:Graph, 3:Stats, 4:Histo, 5:BigDigit)").ok();
        writeln!($board.uart0, " 'm'      : Cycle next screen mode").ok();
        writeln!($board.uart0, " 'b'      : Cycle battery capacity profile").ok();
        writeln!($board.uart0, " 't'      : Zero-Tare calibration & reset session").ok();
        writeln!($board.uart0, " 'r' / 'n': Rotate SD log file to next session").ok();
        writeln!($board.uart0, " 'f'      : Force flush MicroSD sector buffer").ok();
        writeln!($board.uart0, " 's'      : Dump full telemetry summary (JSON)").ok();
        writeln!($board.uart0, " '?' / 'h': Print this command menu").ok();
        writeln!($board.uart0, "==============================").ok();
    }};
}

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
    writeln!(board.uart0, " Commands: '1'..'4' Mode, 'm' Cycle, 't' Tare, 'r' Rotate, 's' Dump, '?' Help").ok();
    writeln!(board.uart0, "========================================").ok();

    // Initialize ST7735 160x80 LCD
    board.lcd.init(&mut board.delay);

    // MicroSD card wrapper
    let sdcard_cell = RefCell::new(board.sdcard);
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

    // UI Screen instances
    let mut hero_screen = HeroScreen::new();
    let mut graph_screen = GraphScreen::new();
    let mut stats_screen = StatsScreen::new();
    let mut histogram_screen = HistogramScreen::new();
    let mut big_digit_screen = BigDigitScreen::new();
    let mut screen_mode = ScreenMode::Hero;

    // Draw initial layout
    hero_screen.draw_layout(&mut board.lcd);

    // Core data containers
    let mut accum = Accumulators::new();
    let mut history = WaveformHistory::new();
    let mut histogram = HistogramData::new();
    let mut stats = SessionStats::new(board.delay.uptime_ms());
    let mut battery = BatteryState::new();
    let mut telemetry = TelemetryStreamer::new();

    // Initial INA219 configuration (0x3E7F: 32V, ±320mV, 128x shunt avg, continuous)
    // Cal = 4096 for 0.1 ohm shunt (0.1 mA LSB, 3.2A max)
    let mut ina_present = match board.ina219.init(4096) {
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

    let mut last_10hz_ms = board.delay.uptime_ms();
    let mut last_1hz_ms = board.delay.uptime_ms();
    let mut last_reinit_attempt_ms = board.delay.uptime_ms();
    let mut last_heartbeat_ms = board.delay.uptime_ms();
    let mut prev_usb_configured = false;

    // User Button tracking (PA8)
    let mut button_press_start_ms = 0u32;
    let mut prev_button_pressed = false;

    loop {
        let now = board.delay.uptime_ms();

        // 0. Feed Free Watchdog Timer (FWDGT)
        board.fwdgt.feed();

        // 1. High-frequency non-blocking USB polling (every loop iteration)
        board.usb_hid.poll();

        let usb_configured = board.usb_hid.is_configured();
        if usb_configured && !prev_usb_configured {
            let uptime = now;
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

        // 2. Remote UART0 Interactive Command Console
        while let Some(ch) = board.uart0.read_byte() {
            match ch {
                b'1' => set_screen_mode!(ScreenMode::Hero, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history),
                b'2' => set_screen_mode!(ScreenMode::Graph, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history),
                b'3' => set_screen_mode!(ScreenMode::Stats, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history),
                b'4' => set_screen_mode!(ScreenMode::Histogram, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history),
                b'5' => set_screen_mode!(ScreenMode::BigDigit, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history),
                b'm' | b'M' => {
                    let next = match screen_mode {
                        ScreenMode::Hero => ScreenMode::Graph,
                        ScreenMode::Graph => ScreenMode::Stats,
                        ScreenMode::Stats => ScreenMode::Histogram,
                        ScreenMode::Histogram => ScreenMode::BigDigit,
                        ScreenMode::BigDigit => ScreenMode::Hero,
                    };
                    set_screen_mode!(next, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history);
                }
                b'b' | b'B' => {
                    battery.profile = battery.profile.next();
                    writeln!(
                        board.uart0,
                        "[BATT] Switched battery profile to: {} (Nominal: {} mAh)",
                        battery.profile.name(),
                        battery.profile.capacity_mah()
                    ).ok();
                }
                b't' | b'T' => tare_and_reset!(now, ina_present, board, datalogger, accum, history, histogram, stats, screen_mode, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen),
                b'r' | b'R' | b'n' | b'N' => {
                    datalogger.rotate_session();
                    writeln!(board.uart0, "[SD] Rotated to new file: {}", datalogger.file_name_str()).ok();
                }
                b'f' | b'F' => {
                    datalogger.flush_buffer();
                    writeln!(board.uart0, "[SD] Flushed buffer to {} (Total rows: {})", datalogger.file_name_str(), datalogger.total_logged_rows).ok();
                }
                b's' | b'S' => dump_session!(now, screen_mode, board, datalogger, stats, accum, battery),
                b'?' | b'h' | b'H' => print_help!(board),
                b'\r' | b'\n' => {}
                _ => {
                    writeln!(board.uart0, "[CMD] Unknown key: '{}' (Send '?' for help)", ch as char).ok();
                }
            }
        }

        // 3. Bidirectional USB HID OUT Command Polling
        if let Some(cmd) = telemetry.poll_command(&mut board.usb_hid) {
            match cmd {
                HidCommand::SetMode(m) => {
                    let next = match m {
                        0 => Some(ScreenMode::Hero),
                        1 => Some(ScreenMode::Graph),
                        2 => Some(ScreenMode::Stats),
                        3 => Some(ScreenMode::Histogram),
                        4 => Some(ScreenMode::BigDigit),
                        0xFF => Some(match screen_mode {
                            ScreenMode::Hero => ScreenMode::Graph,
                            ScreenMode::Graph => ScreenMode::Stats,
                            ScreenMode::Stats => ScreenMode::Histogram,
                            ScreenMode::Histogram => ScreenMode::BigDigit,
                            ScreenMode::BigDigit => ScreenMode::Hero,
                        }),
                        _ => None,
                    };
                    if let Some(mode) = next {
                        set_screen_mode!(mode, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history);
                    }
                }
                HidCommand::TareZero => tare_and_reset!(now, ina_present, board, datalogger, accum, history, histogram, stats, screen_mode, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen),
                HidCommand::FlushSd => {
                    datalogger.flush_buffer();
                    writeln!(board.uart0, "[HID-CMD] Flushed SD buffer to {}", datalogger.file_name_str()).ok();
                }
                HidCommand::RotateLog => {
                    datalogger.rotate_session();
                    writeln!(board.uart0, "[HID-CMD] Rotated log file to {}", datalogger.file_name_str()).ok();
                }
                HidCommand::RequestSummary => dump_session!(now, screen_mode, board, datalogger, stats, accum, battery),
                HidCommand::SetBatteryProfile(p) => {
                    battery.profile = match p {
                        1 => BatteryProfile::Lipo500,
                        2 => BatteryProfile::Lipo1200,
                        3 => BatteryProfile::LiIon2500,
                        4 => BatteryProfile::Alkaline1000,
                        _ => BatteryProfile::None,
                    };
                    writeln!(board.uart0, "[HID-CMD] Battery profile set to: {}", battery.profile.name()).ok();
                }
                HidCommand::Unknown(c) => {
                    writeln!(board.uart0, "[HID-CMD] Unknown command opcode: 0x{:02X}", c).ok();
                }
            }
        }

        // 4. Interactive Button Controls (PA8)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !prev_button_pressed {
            button_press_start_ms = now;
            board.led_blue.on(); // Visual click feedback
        } else if !button_pressed && prev_button_pressed {
            let duration = now.wrapping_sub(button_press_start_ms);
            board.led_blue.off();

            if duration >= 1500 {
                // Long press (>= 1.5s): Zero Tare Calibration + Reset Session Stats
                tare_and_reset!(now, ina_present, board, datalogger, accum, history, histogram, stats, screen_mode, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen);
            } else if duration >= 50 {
                // Short press (50ms..1499ms): Cycle screen mode
                let next = match screen_mode {
                    ScreenMode::Hero => ScreenMode::Graph,
                    ScreenMode::Graph => ScreenMode::Stats,
                    ScreenMode::Stats => ScreenMode::Histogram,
                    ScreenMode::Histogram => ScreenMode::BigDigit,
                    ScreenMode::BigDigit => ScreenMode::Hero,
                };
                set_screen_mode!(next, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history);
            }
        }
        prev_button_pressed = button_pressed;

        // 3. Polite Green LED Heartbeat: Subtle 35ms pulse every 2.5s (2500ms)
        let hb_phase = now.wrapping_sub(last_heartbeat_ms);
        if hb_phase < 35 {
            board.led_green.on();
        } else {
            board.led_green.off();
            if hb_phase >= 2500 {
                last_heartbeat_ms = now;
            }
        }

        // 4. 10 Hz Periodic Measurement, Analytics & Display Rendering
        let dt_ms = now.wrapping_sub(last_10hz_ms);
        if dt_ms >= 100 {
            last_10hz_ms = now;

            let mut reading = InaReading::default();

            if ina_present {
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
                        board.led_red.off();
                    }
                    Err(_) => {
                        ina_present = false;
                        board.led_red.on();
                        board.ina219.recover_bus(&mut board.delay);
                        writeln!(board.uart0, "[I2C] INA219 read error - bus recovered").ok();
                    }
                }
            } else {
                // Hot-unplug recovery retry every 1.5 seconds
                if now.wrapping_sub(last_reinit_attempt_ms) >= 1500 {
                    last_reinit_attempt_ms = now;
                    board.ina219.recover_bus(&mut board.delay);
                    if board.ina219.init(4096).is_ok() {
                        ina_present = true;
                        board.led_red.off();
                        writeln!(board.uart0, "[I2C] INA219 hot-plug reconnected!").ok();
                    }
                }
            }

            // Update waveform circular history buffer
            history.push(reading.current_tenth_ma);

            // Update current distribution histogram
            histogram.record(reading.abs_current_tenth());

            // Sample internal MCU die temperature
            let mcu_temp_tenth = board.mcu_temp.read_temperature_tenth_c();
            stats.update_temp(mcu_temp_tenth);

            // Update DSP Session Stats
            stats.update(reading.voltage_mv, reading.current_tenth_ma, reading.power_tenth_mw);

            // Update Battery Fuel Gauge
            battery.update(reading.voltage_mv, &accum, stats.avg_current_tenth_ma());

            // Drift-free millisecond energy & charge integration
            if ina_present && reading.voltage_mv > 500 && reading.abs_current_tenth() > 2 {
                accum.update(reading.power_tenth_mw, reading.abs_current_tenth(), dt_ms);
            }

            // Screen Rendering
            match screen_mode {
                ScreenMode::Hero => {
                    hero_screen.update(
                        &mut board.lcd,
                        &reading,
                        &accum,
                        ina_present,
                        board.usb_hid.is_configured(),
                        datalogger.status,
                    );
                }
                ScreenMode::Graph => {
                    graph_screen.update(
                        &mut board.lcd,
                        &reading,
                        &accum,
                        &history,
                        ina_present,
                        datalogger.status,
                        board.usb_hid.is_configured(),
                    );
                }
                ScreenMode::Stats => {
                    stats_screen.update(
                        &mut board.lcd,
                        &reading,
                        &stats,
                        &accum,
                        &battery,
                        datalogger.status,
                        datalogger.total_logged_rows,
                        now,
                        board.usb_hid.is_configured(),
                    );
                }
                ScreenMode::Histogram => {
                    histogram_screen.update(&mut board.lcd, &histogram);
                }
                ScreenMode::BigDigit => {
                    big_digit_screen.update(
                        &mut board.lcd,
                        &reading,
                        &accum,
                        &battery,
                        ina_present,
                        board.usb_hid.is_configured(),
                        datalogger.status,
                    );
                }
            }

            // USB HID Telemetry Streaming (9 bytes @ 10 Hz)
            telemetry.send_reading(&mut board.usb_hid, &reading, ina_present, datalogger.status);
        }

        // 5. 1 Hz MicroSD CSV Datalogger & Heartbeat Logging
        if now.wrapping_sub(last_1hz_ms) >= 1000 {
            last_1hz_ms = now;

            // Probe card if missing (with exponential backoff)
            if datalogger.status == SdStatus::NoCard || datalogger.status == SdStatus::WriteError {
                if datalogger.check_probe(now, &mut board.delay) {
                    let sectors = sdcard_cell.borrow().sector_count;
                    writeln!(
                        board.uart0,
                        "[SD] Card hot-plug inserted & ready ({} sectors) | File: {}",
                        sectors,
                        datalogger.file_name_str()
                    )
                    .ok();
                }
            }

            // Log representative row to sector buffer
            if ina_present {
                if let Ok(d) = board.ina219.read_all() {
                    let reading = InaReading {
                        voltage_mv: d.voltage_mv,
                        current_tenth_ma: d.current_tenth_ma,
                        power_tenth_mw: d.power_tenth_mw,
                        overflow: d.overflow,
                        is_reverse: d.current_tenth_ma < 0,
                    };
                    datalogger.log_sample(now, &reading, &accum);
                }
            }

            // UART0 Heartbeat & Diagnostic Output
            let sec = now / 1000;
            let cfg_str = if board.usb_hid.is_configured() { "CFG" } else { "DISC" };
            let sensor_str = if ina_present { "ONLINE" } else { "OFFLINE" };
            let sd_str = match datalogger.status {
                SdStatus::Logging => "LOGGING",
                SdStatus::Ready => "READY",
                SdStatus::NoCard => "NO_CARD",
                SdStatus::WriteError => "ERROR",
            };
            let mode_str = match screen_mode {
                ScreenMode::Hero => "HERO",
                ScreenMode::Graph => "GRAPH",
                ScreenMode::Stats => "STATS",
                ScreenMode::Histogram => "HISTO",
                ScreenMode::BigDigit => "BIG",
            };
            let temp_tenth = stats.mcu_temp_tenth_c;

            writeln!(
                board.uart0,
                "[HEARTBEAT] T+{}s | Mode: {} | Sensor: {} | MCU: {}.{}C | BATT: {}% | USB: {} | SD: {} ({} : {} rows)",
                sec, mode_str, sensor_str, temp_tenth / 10, temp_tenth.unsigned_abs() % 10,
                battery.soc_pct, cfg_str, sd_str,
                datalogger.file_name_str(), datalogger.total_logged_rows
            )
            .ok();
        }
    }
}
