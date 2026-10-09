#![no_std]
#![no_main]

use core::fmt::Write;
use panic_halt as _;
use riscv_rt::entry;

use longan_nano_bsp::{lcd_color, Board, Ina219Data};

// Zero-allocation buffer cursor for no_std text formatting
struct BufferCursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufferCursor<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }
}

impl<'a> Write for BufferCursor<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        let to_copy = core::cmp::min(bytes.len(), remaining);
        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;
        Ok(())
    }
}

// 16-bit RGB565 color constructor
const fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | ((b as u16) >> 3)
}

// Color Palette
const COL_BG_TOP: u16   = rgb565(14, 22, 36);   // Deep Slate Navy
const COL_DIVIDER: u16  = rgb565(35, 60, 90);   // Header divider
const COL_GRAPH_BG: u16 = rgb565(2, 4, 8);      // Near pitch black
const COL_GRID: u16     = rgb565(22, 34, 48);   // Subtle grid dot
const COL_AXIS: u16     = rgb565(45, 65, 95);   // Axis border
const COL_TRACE_I: u16  = rgb565(0, 240, 255);  // Neon Cyan
const COL_FILL_I: u16   = rgb565(0, 24, 38);    // Dark Cyan glow fill
const COL_TRACE_P: u16  = rgb565(255, 215, 0);  // Electric Amber
const COL_FILL_P: u16   = rgb565(42, 30, 0);    // Dark Amber glow fill

// Graph canvas geometry
const GRAPH_X_MIN: u16 = 22;
const GRAPH_X_MAX: u16 = 158;
const GRAPH_Y_TOP: u16 = 22;
const GRAPH_Y_BASE: u16 = 76;
const GRAPH_HEIGHT: u16 = GRAPH_Y_BASE - GRAPH_Y_TOP; // 54 pixels

#[derive(Copy, Clone, PartialEq)]
enum PlotMode {
    Current,
    Power,
}

struct ScaleTier {
    max_val: i32, // In tenths (0.1 mA or 0.1 mW)
    top_lbl: &'static str,
    mid_lbl: &'static str,
    badge: &'static str,
}

static CURRENT_TIERS: [ScaleTier; 8] = [
    ScaleTier { max_val: 100,   top_lbl: "10m",  mid_lbl: " 5m",  badge: "[10m]" },
    ScaleTier { max_val: 250,   top_lbl: "25m",  mid_lbl: "12m",  badge: "[25m]" },
    ScaleTier { max_val: 500,   top_lbl: "50m",  mid_lbl: "25m",  badge: "[50m]" },
    ScaleTier { max_val: 1000,  top_lbl: "100m", mid_lbl: "50m",  badge: "[100m]"},
    ScaleTier { max_val: 2500,  top_lbl: "250m", mid_lbl: "125m", badge: "[250m]"},
    ScaleTier { max_val: 5000,  top_lbl: "500m", mid_lbl: "250m", badge: "[500m]"},
    ScaleTier { max_val: 10000, top_lbl: " 1A",  mid_lbl: "500m", badge: " [1A] "},
    ScaleTier { max_val: 32000, top_lbl: "3.2A", mid_lbl: "1.6A", badge: "[3.2A]"},
];

static POWER_TIERS: [ScaleTier; 7] = [
    ScaleTier { max_val: 100,    top_lbl: "10m",  mid_lbl: " 5m",  badge: "[10mW]" },
    ScaleTier { max_val: 500,    top_lbl: "50m",  mid_lbl: "25m",  badge: "[50mW]" },
    ScaleTier { max_val: 1000,   top_lbl: "100m", mid_lbl: "50m",  badge: "[100m]"},
    ScaleTier { max_val: 5000,   top_lbl: "500m", mid_lbl: "250m", badge: "[500m]"},
    ScaleTier { max_val: 10000,  top_lbl: " 1W",  mid_lbl: "500m", badge: " [1W] "},
    ScaleTier { max_val: 50000,  top_lbl: " 5W",  mid_lbl: "2.5W", badge: " [5W] "},
    ScaleTier { max_val: 100000, top_lbl: "10W",  mid_lbl: " 5W",  badge: "[10W] "},
];

fn redraw_grid(lcd: &mut longan_nano_bsp::Lcd, mode: PlotMode, tier_idx: usize) {
    let (top_lbl, mid_lbl) = match mode {
        PlotMode::Current => (CURRENT_TIERS[tier_idx].top_lbl, CURRENT_TIERS[tier_idx].mid_lbl),
        PlotMode::Power => (POWER_TIERS[tier_idx].top_lbl, POWER_TIERS[tier_idx].mid_lbl),
    };

    // 1. Left Y-Axis Labels
    lcd.fill_rect(0, 20, 21, 60, lcd_color::BLACK);
    lcd.draw_string(1, 22, top_lbl, rgb565(140, 160, 185), lcd_color::BLACK);
    lcd.draw_string(1, 46, mid_lbl, rgb565(110, 130, 155), lcd_color::BLACK);
    lcd.draw_string(6, 70, " 0",   rgb565(90, 110, 135),  lcd_color::BLACK);

    // 2. Vertical Axis Line
    lcd.fill_rect(21, 20, 1, 60, COL_AXIS);

    // 3. Clear Graph Canvas & Draw Baseline
    lcd.fill_rect(GRAPH_X_MIN, 20, GRAPH_X_MAX - GRAPH_X_MIN + 1, 60, COL_GRAPH_BG);
    lcd.fill_rect(GRAPH_X_MIN, GRAPH_Y_BASE, GRAPH_X_MAX - GRAPH_X_MIN + 1, 1, COL_AXIS);

    // 4. Subtle Dotted Grid Lines (25%, 50%, 75%)
    let mut x = GRAPH_X_MIN;
    while x <= GRAPH_X_MAX {
        lcd.set_pixel(x, 36, COL_GRID); // 75%
        lcd.set_pixel(x, 49, COL_GRID); // 50%
        lcd.set_pixel(x, 63, COL_GRID); // 25%
        x += 4;
    }
}

#[entry]
fn main() -> ! {
    let mut board = Board::take_current_monitor().expect("Board initialization failed");

    writeln!(board.uart0, "\r\n========================================").ok();
    writeln!(board.uart0, " Longan Nano INA219 Oscilloscope Monitor (Rust)").ok();
    writeln!(
        board.uart0,
        " SYSCLK: {} MHz, I2C0: 100 kHz (PB6 SCL, PB7 SDA)",
        board.clocks.sysclk / 1_000_000
    )
    .ok();
    writeln!(board.uart0, " USB HID: VID 0x28E9, PID 0x1234").ok();
    writeln!(board.uart0, "========================================").ok();

    // Initialize ST7735 160x80 LCD
    board.lcd.init(&mut board.delay);
    board.lcd.clear(lcd_color::BLACK);

    // Initial Header Bar (Y: 0..19)
    board.lcd.fill_rect(0, 0, 160, 19, COL_BG_TOP);
    board.lcd.fill_rect(0, 19, 160, 1, COL_DIVIDER);

    // State Variables
    let mut plot_mode = PlotMode::Current;
    let mut scale_tier: usize = 2; // Default 50mA tier
    let mut peak_in_sweep: i32 = 0;
    let mut sweep_x: u16 = GRAPH_X_MIN;
    let mut prev_y: u16 = GRAPH_Y_BASE;

    // 64-bit Energy & Charge Accumulators
    let mut mw_ticks: u64 = 0;
    let mut tenth_ma_ticks: u64 = 0;

    redraw_grid(&mut board.lcd, plot_mode, scale_tier);

    // Initial INA219 Calibration (cal = 4096 for 0.1 ohm shunt and 3.2A max)
    let mut ina_present = match board.ina219.init(4096) {
        Ok(_) => {
            writeln!(board.uart0, "[I2C] INA219 sensor detected & initialized").ok();
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
            board.led_red.on(); // Error indicator
            false
        }
    };

    let mut last_10hz = board.delay.uptime_ms();
    let mut last_1hz = board.delay.uptime_ms();
    let mut last_reinit_attempt = board.delay.uptime_ms();
    let mut report_count = 0u32;
    let mut prev_usb_configured = false;

    // Button tracking
    let mut button_press_start = 0u32;
    let mut prev_button_pressed = false;

    loop {
        // High frequency non-blocking USB polling
        board.usb_hid.poll();

        let usb_configured = board.usb_hid.is_configured();
        if usb_configured && !prev_usb_configured {
            let uptime = board.delay.uptime_ms();
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

        // 1. Interactive Button Controls (PA8)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !prev_button_pressed {
            button_press_start = board.delay.uptime_ms();
            board.led_blue.on(); // Visual feedback
        } else if !button_pressed && prev_button_pressed {
            let duration = board.delay.uptime_ms().wrapping_sub(button_press_start);
            board.led_blue.off();
            if duration >= 1500 {
                // Long press: Reset accumulated energy & capacity!
                mw_ticks = 0;
                tenth_ma_ticks = 0;
                writeln!(board.uart0, "[SYS] Energy & Charge accumulators reset").ok();
            } else if duration >= 50 {
                // Short press: Toggle plot mode (Current vs Power)
                plot_mode = match plot_mode {
                    PlotMode::Current => PlotMode::Power,
                    PlotMode::Power => PlotMode::Current,
                };
                scale_tier = 2;
                peak_in_sweep = 0;
                sweep_x = GRAPH_X_MIN;
                prev_y = GRAPH_Y_BASE;
                redraw_grid(&mut board.lcd, plot_mode, scale_tier);
                writeln!(
                    board.uart0,
                    "[UI] Plot mode changed to {}",
                    if plot_mode == PlotMode::Current { "CURRENT" } else { "POWER" }
                )
                .ok();
            }
        }
        prev_button_pressed = button_pressed;

        let now = board.delay.uptime_ms();

        // 2. 10 Hz Periodic Measurement, Oscilloscope Sweep & USB HID Streaming
        if now.wrapping_sub(last_10hz) >= 100 {
            last_10hz = now;

            let mut data = Ina219Data::default();
            if ina_present {
                match board.ina219.read_all() {
                    Ok(readings) => {
                        data = readings;
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
                if now.wrapping_sub(last_reinit_attempt) >= 1500 {
                    last_reinit_attempt = now;
                    if board.ina219.init(4096).is_ok() {
                        ina_present = true;
                        board.led_red.off();
                        writeln!(board.uart0, "[I2C] INA219 hot-plug reconnected!").ok();
                    }
                }
            }

            // Data extraction & signed handling
            let c_tenth = data.current_tenth_ma;
            let is_reverse = c_tenth < 0;
            let abs_c_tenth = if is_reverse { -c_tenth } else { c_tenth };
            let v_mv = data.voltage_mv;
            // Power in tenths of mW: (mV * tenths_of_mA) / 1000
            let p_tenth = (v_mv as u32 * abs_c_tenth as u32) / 1000;

            // Accumulate Energy (mWh) and Charge (mAh)
            if ina_present && v_mv > 500 && abs_c_tenth > 2 {
                mw_ticks = mw_ticks.saturating_add((p_tenth / 10) as u64);
                tenth_ma_ticks = tenth_ma_ticks.saturating_add(abs_c_tenth as u64);
            }

            // ----------------------------------------------------
            // 3. Render Top Line 1 (Instantaneous V, I, P, Status)
            // ----------------------------------------------------
            let mut v_buf = [0u8; 12];
            let mut v_cur = BufferCursor::new(&mut v_buf);
            write!(
                v_cur,
                "{:>2}.{:02}V ",
                v_mv / 1000,
                (v_mv % 1000) / 10
            )
            .ok();
            board.lcd.draw_string(2, 1, v_cur.as_str(), lcd_color::GREEN, COL_BG_TOP);

            let mut c_buf = [0u8; 14];
            let mut c_cur = BufferCursor::new(&mut c_buf);
            let whole_c = abs_c_tenth / 10;
            let frac_c = abs_c_tenth % 10;
            if is_reverse {
                write!(c_cur, "-{:>3}.{}mA", whole_c, frac_c).ok();
            } else {
                write!(c_cur, " {:>3}.{}mA", whole_c, frac_c).ok();
            }
            board.lcd.draw_string(42, 1, c_cur.as_str(), lcd_color::CYAN, COL_BG_TOP);

            let mut p_buf = [0u8; 14];
            let mut p_cur = BufferCursor::new(&mut p_buf);
            if p_tenth < 100000 {
                write!(p_cur, " {:>3}.{} mW", p_tenth / 10, p_tenth % 10).ok();
            } else {
                let whole_w = (p_tenth / 10) / 1000;
                let frac_w = ((p_tenth / 10) % 1000) / 10;
                write!(p_cur, " {:>3}.{:02} W", whole_w, frac_w).ok();
            }
            board.lcd.draw_string(88, 1, p_cur.as_str(), lcd_color::YELLOW, COL_BG_TOP);

            // Status Badge (Top-Right): [LIVE], [REV], or [ERR]
            if !ina_present {
                board.lcd.draw_string(132, 1, "[ERR] ", lcd_color::RED, COL_BG_TOP);
            } else if is_reverse {
                board.lcd.draw_string(132, 1, "[REV] ", lcd_color::RED, COL_BG_TOP);
            } else {
                board.lcd.draw_string(132, 1, "[LIVE]", lcd_color::GREEN, COL_BG_TOP);
            }

            // ----------------------------------------------------
            // 4. Render Top Line 2 (Energy Consumed, Charge, Tier)
            // ----------------------------------------------------
            let mut e_buf = [0u8; 16];
            let mut e_cur = BufferCursor::new(&mut e_buf);
            let total_mwh = (mw_ticks / 36000) as u32;
            let frac_mwh = ((mw_ticks % 36000) / 360) as u32;
            if total_mwh == 0 {
                let uwh = ((mw_ticks * 100) / 360) as u32;
                write!(e_cur, "E:{:>3} uWh ", uwh).ok();
            } else {
                write!(e_cur, "E:{:>2}.{:02}mWh", total_mwh, frac_mwh).ok();
            }
            board.lcd.draw_string(2, 10, e_cur.as_str(), rgb565(255, 160, 40), COL_BG_TOP);

            let mut q_buf = [0u8; 16];
            let mut q_cur = BufferCursor::new(&mut q_buf);
            let total_mah = (tenth_ma_ticks / 360000) as u32;
            let frac_mah = ((tenth_ma_ticks % 360000) / 3600) as u32;
            write!(q_cur, "Q:{:>2}.{:02}mAh", total_mah, frac_mah).ok();
            board.lcd.draw_string(64, 10, q_cur.as_str(), rgb565(100, 255, 140), COL_BG_TOP);

            // Scale badge
            let tier_badge = match plot_mode {
                PlotMode::Current => CURRENT_TIERS[scale_tier].badge,
                PlotMode::Power => POWER_TIERS[scale_tier].badge,
            };
            board.lcd.draw_string(126, 10, tier_badge, lcd_color::WHITE, COL_BG_TOP);

            // ----------------------------------------------------
            // 5. Update Oscilloscope Waveform Sweep
            // ----------------------------------------------------
            let plot_val: i32 = match plot_mode {
                PlotMode::Current => abs_c_tenth as i32,
                PlotMode::Power => p_tenth as i32,
            };

            if plot_val > peak_in_sweep {
                peak_in_sweep = plot_val;
            }

            // Dynamic Auto-scaling
            let (num_tiers, current_max) = match plot_mode {
                PlotMode::Current => (CURRENT_TIERS.len(), CURRENT_TIERS[scale_tier].max_val),
                PlotMode::Power => (POWER_TIERS.len(), POWER_TIERS[scale_tier].max_val),
            };

            let mut scale_changed = false;
            // Step UP immediately if value exceeds ceiling
            if plot_val > current_max && scale_tier + 1 < num_tiers {
                while scale_tier + 1 < num_tiers {
                    let next_max = match plot_mode {
                        PlotMode::Current => CURRENT_TIERS[scale_tier].max_val,
                        PlotMode::Power => POWER_TIERS[scale_tier].max_val,
                    };
                    if plot_val > next_max {
                        scale_tier += 1;
                    } else {
                        break;
                    }
                }
                scale_changed = true;
            }

            // When sweep wraps around: evaluate hysteresis downscale
            if sweep_x == GRAPH_X_MIN && !scale_changed {
                if scale_tier > 0 {
                    let lower_ceiling = match plot_mode {
                        PlotMode::Current => CURRENT_TIERS[scale_tier - 1].max_val,
                        PlotMode::Power => POWER_TIERS[scale_tier - 1].max_val,
                    };
                    // Downscale only if peak was below 65% of lower ceiling
                    if peak_in_sweep < (lower_ceiling * 65 / 100) {
                        scale_tier -= 1;
                        scale_changed = true;
                    }
                }
                peak_in_sweep = 0;
            }

            if scale_changed {
                redraw_grid(&mut board.lcd, plot_mode, scale_tier);
            }

            let scale_max = match plot_mode {
                PlotMode::Current => CURRENT_TIERS[scale_tier].max_val,
                PlotMode::Power => POWER_TIERS[scale_tier].max_val,
            };

            // Calculate Y coordinate on 54-pixel canvas
            let y_calc = (GRAPH_Y_BASE as i32) - ((plot_val * GRAPH_HEIGHT as i32) / scale_max);
            let y_curr = if y_calc < GRAPH_Y_TOP as i32 {
                GRAPH_Y_TOP
            } else if y_calc > GRAPH_Y_BASE as i32 {
                GRAPH_Y_BASE
            } else {
                y_calc as u16
            };

            let trace_col = match plot_mode {
                PlotMode::Current => COL_TRACE_I,
                PlotMode::Power => COL_TRACE_P,
            };
            let fill_col = match plot_mode {
                PlotMode::Current => COL_FILL_I,
                PlotMode::Power => COL_FILL_P,
            };

            // Erase lookahead column head (2 pixels ahead)
            let erase_x1 = if sweep_x + 1 > GRAPH_X_MAX { GRAPH_X_MIN } else { sweep_x + 1 };
            let erase_x2 = if sweep_x + 2 > GRAPH_X_MAX { GRAPH_X_MIN + 1 } else { sweep_x + 2 };

            for ex in [erase_x1, erase_x2] {
                board.lcd.fill_rect(ex, GRAPH_Y_TOP, 1, GRAPH_HEIGHT + 1, COL_GRAPH_BG);
                board.lcd.set_pixel(ex, GRAPH_Y_BASE, COL_AXIS);
                if ex % 4 == 0 {
                    board.lcd.set_pixel(ex, 36, COL_GRID);
                    board.lcd.set_pixel(ex, 49, COL_GRID);
                    board.lcd.set_pixel(ex, 63, COL_GRID);
                }
            }

            // Blit single vertical column (55 pixels)
            let mut col_buf = [0u16; 55];
            let y_min = core::cmp::min(prev_y, y_curr);
            let y_max = core::cmp::max(prev_y, y_curr);

            for y in GRAPH_Y_TOP..=GRAPH_Y_BASE {
                let idx = (y - GRAPH_Y_TOP) as usize;
                if idx < col_buf.len() {
                    if y >= y_min && y <= y_max {
                        col_buf[idx] = trace_col; // Vibrant trace segment
                    } else if y > y_max && y < GRAPH_Y_BASE {
                        col_buf[idx] = fill_col; // Glowing area under curve
                    } else if y == GRAPH_Y_BASE {
                        col_buf[idx] = COL_AXIS; // Baseline
                    } else if (y == 36 || y == 49 || y == 63) && (sweep_x % 4 == 0) {
                        col_buf[idx] = COL_GRID; // Dotted grid point
                    } else {
                        col_buf[idx] = COL_GRAPH_BG;
                    }
                }
            }

            board.lcd.write_pixels(sweep_x, GRAPH_Y_TOP, 1, GRAPH_HEIGHT + 1, &col_buf);

            prev_y = y_curr;
            sweep_x += 1;
            if sweep_x > GRAPH_X_MAX {
                sweep_x = GRAPH_X_MIN;
            }

            // ----------------------------------------------------
            // 6. USB HID Telemetry Streaming (9 bytes)
            // ----------------------------------------------------
            let report: [u8; 9] = [
                0x01,
                (data.voltage_mv & 0xFF) as u8,
                (data.voltage_mv >> 8) as u8,
                (data.current_ma & 0xFF) as u8,
                (data.current_ma >> 8) as u8,
                (data.power_mw & 0xFF) as u8,
                (data.power_mw >> 8) as u8,
                0x00,
                0x00,
            ];

            if board.usb_hid.send_report(&report) {
                report_count = report_count.wrapping_add(1);
            }

            // Green LED heartbeat toggle
            board.led_green.toggle();
        }

        // 7. 1 Hz Diagnostic Telemetry over UART0
        if now.wrapping_sub(last_1hz) >= 1000 {
            last_1hz = now;
            let sec = now / 1000;
            let cfg_str = if board.usb_hid.is_configured() {
                "CFG"
            } else {
                "DISC"
            };
            let sensor_str = if ina_present { "ONLINE" } else { "OFFLINE" };
            writeln!(
                board.uart0,
                "[HEARTBEAT] T+{}s | Sensor: {} | USB: {} | HID Reports: {}",
                sec, sensor_str, cfg_str, report_count
            )
            .ok();
        }
    }
}
