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
use crate::model::{Accumulators, HistogramData, InaReading, SessionStats, WaveformHistory};
use crate::telemetry::TelemetryStreamer;
use crate::ui::{GraphScreen, HeroScreen, HistogramScreen, ScreenMode, SdStatus, StatsScreen};

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
            "[SD] Card detected & ready ({} sectors)",
            sectors
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
    let mut screen_mode = ScreenMode::Hero;

    // Draw initial layout
    hero_screen.draw_layout(&mut board.lcd);

    // Core data containers
    let mut accum = Accumulators::new();
    let mut history = WaveformHistory::new();
    let mut histogram = HistogramData::new();
    let mut stats = SessionStats::new(board.delay.uptime_ms());
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

        // 2. Interactive Button Controls (PA8)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !prev_button_pressed {
            button_press_start_ms = now;
            board.led_blue.on(); // Visual click feedback
        } else if !button_pressed && prev_button_pressed {
            let duration = now.wrapping_sub(button_press_start_ms);
            board.led_blue.off();

            if duration >= 1500 {
                // Long press (>= 1.5s): Zero Tare Calibration + Reset Session Stats
                if ina_present {
                    if let Ok(raw_data) = board.ina219.read_all() {
                        let current_raw = raw_data.current_tenth_ma;
                        board.ina219.set_tare(current_raw);
                        writeln!(
                            board.uart0,
                            "[SYS] Tare Zero Offset Calibrated: {}.{} mA",
                            current_raw / 10,
                            current_raw.unsigned_abs() % 10
                        )
                        .ok();
                    }
                }
                accum.reset();
                history.clear();
                histogram.clear();
                stats.reset(now);
                writeln!(board.uart0, "[SYS] Session Stats, Histogram & Accumulators Reset!").ok();

                // Re-draw active screen layout to clear residual data
                match screen_mode {
                    ScreenMode::Hero => hero_screen.draw_layout(&mut board.lcd),
                    ScreenMode::Graph => graph_screen.draw_layout(&mut board.lcd),
                    ScreenMode::Stats => stats_screen.draw_layout(&mut board.lcd),
                    ScreenMode::Histogram => histogram_screen.draw_layout(&mut board.lcd),
                }
            } else if duration >= 50 {
                // Short press (50ms..1499ms): Cycle screen mode
                screen_mode = match screen_mode {
                    ScreenMode::Hero => ScreenMode::Graph,
                    ScreenMode::Graph => ScreenMode::Stats,
                    ScreenMode::Stats => ScreenMode::Histogram,
                    ScreenMode::Histogram => ScreenMode::Hero,
                };

                match screen_mode {
                    ScreenMode::Hero => hero_screen.draw_layout(&mut board.lcd),
                    ScreenMode::Graph => {
                        graph_screen.draw_layout(&mut board.lcd);
                        // Replot existing historical data at current scale tier
                        graph_screen.replot_history(&mut board.lcd, &history);
                    }
                    ScreenMode::Stats => stats_screen.draw_layout(&mut board.lcd),
                    ScreenMode::Histogram => histogram_screen.draw_layout(&mut board.lcd),
                }

                writeln!(
                    board.uart0,
                    "[UI] Screen mode switched to {}",
                    match screen_mode {
                        ScreenMode::Hero => "HERO (Large Number Cards)",
                        ScreenMode::Graph => "GRAPH (Oscilloscope)",
                        ScreenMode::Stats => "STATS (Dashboard)",
                        ScreenMode::Histogram => "HISTOGRAM (Current Profile)",
                    }
                )
                .ok();
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
                        writeln!(board.uart0, "[I2C] INA219 read error - lost connection").ok();
                    }
                }
            } else {
                // Hot-unplug recovery retry every 1.5 seconds
                if now.wrapping_sub(last_reinit_attempt_ms) >= 1500 {
                    last_reinit_attempt_ms = now;
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

            // Update DSP Session Stats
            stats.update(reading.voltage_mv, reading.current_tenth_ma, reading.power_tenth_mw);

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
                    graph_screen.update(&mut board.lcd, &reading, &accum, &history, ina_present);
                }
                ScreenMode::Stats => {
                    stats_screen.update(
                        &mut board.lcd,
                        &reading,
                        &stats,
                        &accum,
                        datalogger.status,
                        datalogger.total_logged_rows,
                        now,
                    );
                }
                ScreenMode::Histogram => {
                    histogram_screen.update(&mut board.lcd, &histogram);
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
                        "[SD] Card hot-plug inserted & ready ({} sectors)",
                        sectors
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
            };

            writeln!(
                board.uart0,
                "[HEARTBEAT] T+{}s | Mode: {} | Sensor: {} | USB: {} | SD: {} (Lines: {})",
                sec, mode_str, sensor_str, cfg_str, sd_str, datalogger.total_logged_rows
            )
            .ok();
        }
    }
}
