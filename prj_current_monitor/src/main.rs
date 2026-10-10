#![no_std]
#![no_main]

mod fmt;
mod logger;
mod model;
mod telemetry;
mod ui;

use core::cell::RefCell;
use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use riscv_rt::entry;

use longan_nano_bsp::Board;

use crate::logger::SdDatalogger;
use crate::model::{Accumulators, BatteryProfile, BatteryState, HistogramData, InaReading, SampleAggregator, SessionStats, WaveformHistory};
use crate::telemetry::{HidCommand, TelemetryStreamer};
use crate::ui::{BigDigitScreen, GraphScreen, HeroScreen, HistogramScreen, ScreenMode, SdStatus, StatsScreen};

const BKP_MAGIC_KEY: u16 = 0x5A5A;
const BKP_REG_CONFIG: usize = 1; // [15..8] screen_mode, [7..0] battery_profile
const BKP_REG_TARE: usize = 2;   // tare_offset_tenth (i16)
const BKP_REG_LIMIT: usize = 3;  // current_limit_ma (u16)
const BKP_REG_MAGIC: usize = 4;  // BKP_MAGIC_KEY

fn save_nvram_config(
    rtc: &mut gd32vf103_hal::Rtc,
    screen_mode: ScreenMode,
    battery_profile: BatteryProfile,
    tare_offset_tenth: i16,
    limit_ma: u16,
) {
    let mode_idx = match screen_mode {
        ScreenMode::Hero => 0u16,
        ScreenMode::Graph => 1u16,
        ScreenMode::Stats => 2u16,
        ScreenMode::Histogram => 3u16,
        ScreenMode::BigDigit => 4u16,
    };
    let config = (mode_idx << 8) | (battery_profile.id() as u16);
    rtc.write_backup_reg(BKP_REG_CONFIG, config);
    rtc.write_backup_reg(BKP_REG_TARE, tare_offset_tenth as u16);
    rtc.write_backup_reg(BKP_REG_LIMIT, limit_ma);
    rtc.write_backup_reg(BKP_REG_MAGIC, BKP_MAGIC_KEY);
}

macro_rules! set_screen_mode {
    ($new_mode:expr, $screen_mode:expr, $board:expr, $hero:expr, $graph:expr, $stats:expr, $histo:expr, $big_digit:expr, $hist:expr, $battery:expr, $current_limit_ma:expr) => {{
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
        save_nvram_config(&mut $board.rtc, $screen_mode, $battery.profile, $board.ina219.tare_offset_tenth, $current_limit_ma);
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
    ($now:expr, $ina_present:expr, $board:expr, $datalogger:expr, $accum:expr, $hist:expr, $histo:expr, $stats:expr, $screen_mode:expr, $hero:expr, $graph:expr, $stats_s:expr, $histo_s:expr, $big_digit:expr, $battery:expr, $current_limit_ma:expr, $sec_agg:expr) => {{
        if $ina_present {
            let mut sum_raw = 0i32;
            let mut valid_samples = 0i32;
            let cur_tare = $board.ina219.tare_offset_tenth as i32;
            for _ in 0..8 {
                if let Ok(raw_data) = $board.ina219.read_all() {
                    let true_raw = (raw_data.current_tenth_ma as i32) + cur_tare;
                    sum_raw += true_raw;
                    valid_samples += 1;
                }
                $board.delay.delay_ms(2);
            }
            if valid_samples > 0 {
                let avg_offset = (sum_raw / valid_samples) as i16;
                $board.ina219.set_tare(avg_offset);
                save_nvram_config(&mut $board.rtc, $screen_mode, $battery.profile, avg_offset, $current_limit_ma);
                writeln!(
                    $board.uart0,
                    "[SYS] Tare Zero Offset Calibrated: {}.{} mA",
                    avg_offset / 10,
                    avg_offset.unsigned_abs() % 10
                ).ok();
            }
        }
        $datalogger.flush_buffer();
        $datalogger.rotate_session();
        $accum.reset();
        $sec_agg.finish_and_reset(&InaReading::default());
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
    ($now:expr, $screen_mode:expr, $board:expr, $datalogger:expr, $stats:expr, $accum:expr, $battery:expr, $current_limit_ma:expr, $over_current:expr) => {{
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
        let rtc_epoch = $board.rtc.get_epoch();
        let rtc_dt = $board.rtc.get_datetime();
        let rtc_synced = $board.rtc.is_synced();
        let vdda_mv = $board.mcu_temp.read_vdda_mv();

        writeln!($board.uart0, "\r\n--- SESSION TELEMETRY SUMMARY ---").ok();
        writeln!($board.uart0, "Uptime: {} s | Mode: {} | MCU Temp: {}.{} C (VDDA: {} mV)", uptime_s, mode_name, mcu_temp_tenth / 10, mcu_temp_tenth.unsigned_abs() % 10, vdda_mv).ok();
        writeln!(
            $board.uart0,
            "Date/Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC | Epoch: {} | Synced: {}",
            rtc_dt.year, rtc_dt.month, rtc_dt.day, rtc_dt.hour, rtc_dt.minute, rtc_dt.second,
            rtc_epoch,
            if rtc_synced { "YES" } else { "NO" }
        ).ok();
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
        let is_over = $over_current;
        let is_low_bat = $battery.is_low_voltage;
        let is_alert = is_over || is_low_bat;
        writeln!(
            $board.uart0,
            "Battery: Profile {} | SoC: {}% | Rem: {} mAh ({} mins)",
            $battery.profile.name(), $battery.soc_pct, $battery.rem_mah, $battery.time_rem_mins
        ).ok();
        writeln!(
            $board.uart0,
            "Alert Limit: {} mA | Over-Current: {} | Low-Voltage: {}",
            $current_limit_ma,
            if is_over { "YES (OVERCURRENT)" } else { "NO" },
            if is_low_bat { "YES (LOW BATTERY)" } else { "NO" }
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
            "[JSON] {{\"uptime_s\":{},\"mode\":\"{}\",\"epoch\":{},\"time\":\"{:04}-{:02}-{:02} {:02}:{:02}:{:02}\",\"rtc_synced\":{},\"v_min\":{},\"v_max\":{},\"c_min\":{},\"c_max\":{},\"c_avg\":{},\"p_peak_mw\":{},\"mwh\":{},\"mah\":{},\"mcu_temp_c\":{}.{},\"vdda_mv\":{},\"bat_soc\":{},\"sd_file\":\"{}\",\"sd_rows\":{},\"limit_ma\":{},\"alert\":{},\"over_current\":{},\"low_voltage\":{}}}",
            uptime_s,
            mode_name,
            rtc_epoch,
            rtc_dt.year, rtc_dt.month, rtc_dt.day, rtc_dt.hour, rtc_dt.minute, rtc_dt.second,
            rtc_synced,
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
            vdda_mv,
            $battery.soc_pct,
            $datalogger.file_name_str(),
            $datalogger.total_logged_rows,
            $current_limit_ma,
            is_alert,
            is_over,
            is_low_bat
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
        writeln!($board.uart0, " 'c'      : Print current RTC time & sync status").ok();
        writeln!($board.uart0, " 'T <ep>' : Set RTC Unix epoch seconds").ok();
        writeln!($board.uart0, " 'l' / 'L': Set overcurrent alert limit (0 = disable)").ok();
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
    let mut current_limit_ma = 0u16;
    let mut over_current = false;

    // Core data containers
    let mut accum = Accumulators::new();
    let mut history = WaveformHistory::new();
    let mut histogram = HistogramData::new();
    let mut stats = SessionStats::new(board.delay.uptime_ms());
    let mut battery = BatteryState::new();
    let mut telemetry = TelemetryStreamer::new();
    let mut sec_agg = SampleAggregator::new();

    // Check NVRAM configuration in BKP registers
    if board.rtc.read_backup_reg(BKP_REG_MAGIC) == BKP_MAGIC_KEY {
        let cfg = board.rtc.read_backup_reg(BKP_REG_CONFIG);
        let mode_idx = (cfg >> 8) & 0xFF;
        let bat_idx = (cfg & 0xFF) as u8;
        let tare_tenth = board.rtc.read_backup_reg(BKP_REG_TARE) as i16;
        current_limit_ma = board.rtc.read_backup_reg(BKP_REG_LIMIT);

        screen_mode = match mode_idx {
            1 => ScreenMode::Graph,
            2 => ScreenMode::Stats,
            3 => ScreenMode::Histogram,
            4 => ScreenMode::BigDigit,
            _ => ScreenMode::Hero,
        };
        battery.profile = BatteryProfile::from_id(bat_idx);
        board.ina219.set_tare(tare_tenth);
        writeln!(
            board.uart0,
            "[NVRAM] Restored config from BKP: Mode {}, Bat {}, Tare {}.{} mA, Limit {} mA",
            mode_idx + 1,
            battery.profile.name(),
            tare_tenth / 10,
            tare_tenth.unsigned_abs() % 10,
            current_limit_ma
        ).ok();
    } else {
        save_nvram_config(&mut board.rtc, screen_mode, battery.profile, board.ina219.tare_offset_tenth, current_limit_ma);
        writeln!(board.uart0, "[NVRAM] Initialized default configuration in BKP registers").ok();
    }

    // Draw initial layout
    match screen_mode {
        ScreenMode::Hero => hero_screen.draw_layout(&mut board.lcd),
        ScreenMode::Graph => graph_screen.draw_layout(&mut board.lcd),
        ScreenMode::Stats => stats_screen.draw_layout(&mut board.lcd),
        ScreenMode::Histogram => histogram_screen.draw_layout(&mut board.lcd),
        ScreenMode::BigDigit => big_digit_screen.draw_layout(&mut board.lcd),
    }

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
    let mut last_reading = InaReading::default();

    // User Button tracking (PA8)
    let mut button_press_start_ms = 0u32;
    let mut prev_button_pressed = false;

    // Line buffer for multi-byte UART commands (e.g. "T 1760091240")
    let mut cmd_buf = [0u8; 32];
    let mut cmd_len = 0usize;

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

        // 1.5 Rotary Encoder Polling (PB10/PB11/PB5)
        let (rot, enc_pressed) = board.encoder.poll();
        if rot > 0 {
            let next = match screen_mode {
                ScreenMode::Hero => ScreenMode::Graph,
                ScreenMode::Graph => ScreenMode::Stats,
                ScreenMode::Stats => ScreenMode::Histogram,
                ScreenMode::Histogram => ScreenMode::BigDigit,
                ScreenMode::BigDigit => ScreenMode::Hero,
            };
            set_screen_mode!(next, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
        } else if rot < 0 {
            let prev = match screen_mode {
                ScreenMode::Hero => ScreenMode::BigDigit,
                ScreenMode::Graph => ScreenMode::Hero,
                ScreenMode::Stats => ScreenMode::Graph,
                ScreenMode::Histogram => ScreenMode::Stats,
                ScreenMode::BigDigit => ScreenMode::Histogram,
            };
            set_screen_mode!(prev, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
        }

        if enc_pressed {
            battery.profile = battery.profile.next();
            save_nvram_config(&mut board.rtc, screen_mode, battery.profile, board.ina219.tare_offset_tenth, current_limit_ma);
            writeln!(
                board.uart0,
                "[ROTARY] Switched battery profile to: {} (Nominal: {} mAh)",
                battery.profile.name(),
                battery.profile.capacity_mah()
            ).ok();
        }

        // 2. Remote UART0 Interactive Command Console
        while let Some(ch) = board.uart0.read_byte() {
            if cmd_len > 0 {
                // We are buffering a multi-byte command (started with 'T' or 'L')
                match ch {
                    b'0'..=b'9' | b' ' | b':' => {
                        if cmd_len < cmd_buf.len() {
                            cmd_buf[cmd_len] = ch;
                            cmd_len += 1;
                        }
                    }
                    b'\r' | b'\n' => {
                        if cmd_len >= 2 && cmd_buf[0] == b'T' {
                            let line = &cmd_buf[..cmd_len];
                            let mut idx = 1;
                            while idx < cmd_len && (line[idx] == b' ' || line[idx] == b':') {
                                idx += 1;
                            }
                            let mut parsed_epoch = 0u32;
                            let mut has_digits = false;
                            while idx < cmd_len && line[idx] >= b'0' && line[idx] <= b'9' {
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
                        } else if cmd_len == 1 && cmd_buf[0] == b'T' {
                            tare_and_reset!(now, ina_present, board, datalogger, accum, history, histogram, stats, screen_mode, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, battery, current_limit_ma, sec_agg);
                        } else if cmd_len >= 2 && cmd_buf[0] == b'L' {
                            let line = &cmd_buf[..cmd_len];
                            let mut idx = 1;
                            while idx < cmd_len && (line[idx] == b' ' || line[idx] == b':') {
                                idx += 1;
                            }
                            let mut parsed_limit = 0u16;
                            let mut has_digits = false;
                            while idx < cmd_len && line[idx] >= b'0' && line[idx] <= b'9' {
                                parsed_limit = parsed_limit.saturating_mul(10).saturating_add((line[idx] - b'0') as u16);
                                has_digits = true;
                                idx += 1;
                            }
                            if has_digits {
                                current_limit_ma = parsed_limit.min(3200);
                                save_nvram_config(&mut board.rtc, screen_mode, battery.profile, board.ina219.tare_offset_tenth, current_limit_ma);
                                writeln!(board.uart0, "[LIMIT] Set Over-Current Alert Limit to: {} mA", current_limit_ma).ok();
                            }
                        } else if cmd_len == 1 && cmd_buf[0] == b'L' {
                            writeln!(
                                board.uart0,
                                "[LIMIT] Over-Current Alert Limit: {} mA (Alert Active: {})",
                                current_limit_ma,
                                if over_current { "YES" } else { "NO" }
                            ).ok();
                        }
                        cmd_len = 0;
                    }
                    _ => {
                        cmd_len = 0;
                    }
                }
            } else {
                match ch {
                    b'1' => {
                        set_screen_mode!(ScreenMode::Hero, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
                    }
                    b'2' => {
                        set_screen_mode!(ScreenMode::Graph, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
                    }
                    b'3' => {
                        set_screen_mode!(ScreenMode::Stats, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
                    }
                    b'4' => {
                        set_screen_mode!(ScreenMode::Histogram, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
                    }
                    b'5' => {
                        set_screen_mode!(ScreenMode::BigDigit, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
                    }
                    b'm' | b'M' => {
                        let next = match screen_mode {
                            ScreenMode::Hero => ScreenMode::Graph,
                            ScreenMode::Graph => ScreenMode::Stats,
                            ScreenMode::Stats => ScreenMode::Histogram,
                            ScreenMode::Histogram => ScreenMode::BigDigit,
                            ScreenMode::BigDigit => ScreenMode::Hero,
                        };
                        set_screen_mode!(next, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
                    }
                    b'b' | b'B' => {
                        battery.profile = battery.profile.next();
                        save_nvram_config(&mut board.rtc, screen_mode, battery.profile, board.ina219.tare_offset_tenth, current_limit_ma);
                        writeln!(
                            board.uart0,
                            "[BATT] Switched battery profile to: {} (Nominal: {} mAh)",
                            battery.profile.name(),
                            battery.profile.capacity_mah()
                        ).ok();
                    }
                    b't' => {
                        tare_and_reset!(now, ina_present, board, datalogger, accum, history, histogram, stats, screen_mode, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, battery, current_limit_ma, sec_agg);
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
                            current_limit_ma,
                            if over_current { "YES" } else { "NO" }
                        ).ok();
                    }
                    b'L' => {
                        cmd_buf[0] = b'L';
                        cmd_len = 1;
                    }
                    b'T' => {
                        cmd_buf[0] = b'T';
                        cmd_len = 1;
                    }
                    b'r' | b'R' | b'n' | b'N' => {
                        datalogger.rotate_session();
                        writeln!(board.uart0, "[SD] Rotated to new file: {}", datalogger.file_name_str()).ok();
                    }
                    b'f' | b'F' => {
                        datalogger.flush_buffer();
                        writeln!(board.uart0, "[SD] Flushed buffer to {} (Total rows: {})", datalogger.file_name_str(), datalogger.total_logged_rows).ok();
                    }
                    b's' | b'S' => {
                        dump_session!(now, screen_mode, board, datalogger, stats, accum, battery, current_limit_ma, over_current);
                    }
                    b'?' | b'h' | b'H' => {
                        print_help!(board);
                    }
                    b'\r' | b'\n' => {}
                    _ => {
                        writeln!(board.uart0, "[CMD] Unknown key: '{}' (Send '?' for help)", ch as char).ok();
                    }
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
                        set_screen_mode!(mode, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
                    }
                }
                HidCommand::TareZero => tare_and_reset!(now, ina_present, board, datalogger, accum, history, histogram, stats, screen_mode, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, battery, current_limit_ma, sec_agg),
                HidCommand::FlushSd => {
                    datalogger.flush_buffer();
                    writeln!(board.uart0, "[HID-CMD] Flushed SD buffer to {}", datalogger.file_name_str()).ok();
                }
                HidCommand::RotateLog => {
                    datalogger.rotate_session();
                    writeln!(board.uart0, "[HID-CMD] Rotated log file to {}", datalogger.file_name_str()).ok();
                }
                HidCommand::RequestSummary => dump_session!(now, screen_mode, board, datalogger, stats, accum, battery, current_limit_ma, over_current),
                HidCommand::SetBatteryProfile(p) => {
                    battery.profile = match p {
                        1 => BatteryProfile::Lipo500,
                        2 => BatteryProfile::Lipo1200,
                        3 => BatteryProfile::LiIon2500,
                        4 => BatteryProfile::Alkaline1000,
                        _ => BatteryProfile::None,
                    };
                    save_nvram_config(&mut board.rtc, screen_mode, battery.profile, board.ina219.tare_offset_tenth, current_limit_ma);
                    writeln!(board.uart0, "[HID-CMD] Battery profile set to: {}", battery.profile.name()).ok();
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
                    current_limit_ma = limit.min(3200);
                    save_nvram_config(&mut board.rtc, screen_mode, battery.profile, board.ina219.tare_offset_tenth, current_limit_ma);
                    writeln!(board.uart0, "[HID-CMD] Set Over-Current Alert Limit to: {} mA", current_limit_ma).ok();
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
                tare_and_reset!(now, ina_present, board, datalogger, accum, history, histogram, stats, screen_mode, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, battery, current_limit_ma, sec_agg);
            } else if duration >= 50 {
                // Short press (50ms..1499ms): Cycle screen mode
                let next = match screen_mode {
                    ScreenMode::Hero => ScreenMode::Graph,
                    ScreenMode::Graph => ScreenMode::Stats,
                    ScreenMode::Stats => ScreenMode::Histogram,
                    ScreenMode::Histogram => ScreenMode::BigDigit,
                    ScreenMode::BigDigit => ScreenMode::Hero,
                };
                set_screen_mode!(next, screen_mode, board, hero_screen, graph_screen, stats_screen, histogram_screen, big_digit_screen, history, battery, current_limit_ma);
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

        // 4. CNVR Synchronized Periodic Measurement, Analytics & Display Rendering
        let dt_ms = now.wrapping_sub(last_10hz_ms);
        let conv_ready = if ina_present && dt_ms >= 75 {
            board.ina219.is_conversion_ready().unwrap_or(false)
        } else {
            false
        };
        if conv_ready || dt_ms >= 100 {
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
                        sec_agg.record(&reading);
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

            // Check Over-Current and Low-Voltage Alert
            let is_over_current = current_limit_ma > 0 && (reading.current_tenth_ma as i32) > ((current_limit_ma as i32) * 10);
            let is_low_voltage = battery.is_low_voltage;
            over_current = is_over_current;
            let is_alert = is_over_current || is_low_voltage;
            if is_alert || !ina_present {
                board.led_red.on();
            } else {
                board.led_red.off();
            }

            last_reading = reading;

            // Drift-free millisecond energy & charge integration
            if ina_present && reading.voltage_mv > 500 && reading.current_tenth_ma != 0 {
                accum.update(reading.power_tenth_mw, reading.current_tenth_ma, dt_ms);
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
                    let cur_dt = if board.rtc.is_synced() {
                        Some(board.rtc.get_datetime())
                    } else {
                        None
                    };
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
                        cur_dt,
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
                        over_current,
                    );
                }
            }

            // USB HID Telemetry Streaming (9 bytes @ 10 Hz)
            telemetry.send_reading(&mut board.usb_hid, &reading, ina_present, datalogger.status, over_current);
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

            // Log representative row to sector buffer using true 1s arithmetic average from sample aggregator
            if ina_present {
                let cur_dt = if board.rtc.is_synced() {
                    Some(board.rtc.get_datetime())
                } else {
                    None
                };
                let agg_reading = sec_agg.finish_and_reset(&last_reading);
                datalogger.log_sample(now, &agg_reading, &accum, cur_dt);
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

            if board.rtc.is_synced() {
                let dt = board.rtc.get_datetime();
                writeln!(
                    board.uart0,
                    "[HEARTBEAT] {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC (T+{}s) | Mode: {} | Sensor: {} | MCU: {}.{}C | BATT: {}% | USB: {} | SD: {} ({} : {} rows)",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
                    sec, mode_str, sensor_str, temp_tenth / 10, temp_tenth.unsigned_abs() % 10,
                    battery.soc_pct, cfg_str, sd_str,
                    datalogger.file_name_str(), datalogger.total_logged_rows
                )
                .ok();
            } else {
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
}
