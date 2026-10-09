#![no_std]
#![no_main]

use core::fmt::Write;
use panic_halt as _;
use riscv_rt::entry;

use longan_nano_bsp::{
    lcd::{FONT_28, FONT_5X7, FONT_8X16},
    lcd_color, Board, Ina219Data,
};

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

// Graph canvas geometry
const GRAPH_X_MIN: u16 = 22;
const GRAPH_X_MAX: u16 = 158;
const GRAPH_Y_TOP: u16 = 22;
const GRAPH_Y_BASE: u16 = 76;
const GRAPH_HEIGHT: u16 = GRAPH_Y_BASE - GRAPH_Y_TOP; // 54 pixels

#[derive(Copy, Clone, PartialEq)]
enum ScreenMode {
    Graph,
    Text,
}

struct ScaleTier {
    max_val: i32, // In tenths (0.1 mA)
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

fn draw_graph_screen_layout(lcd: &mut longan_nano_bsp::Lcd, tier_idx: usize) {
    lcd.clear(lcd_color::BLACK);

    // Header Bar (Y: 0..19)
    lcd.fill_rect(0, 0, 160, 19, COL_BG_TOP);
    lcd.fill_rect(0, 19, 160, 1, COL_DIVIDER);

    redraw_grid(lcd, tier_idx);
}

fn redraw_grid(lcd: &mut longan_nano_bsp::Lcd, tier_idx: usize) {
    let top_lbl = CURRENT_TIERS[tier_idx].top_lbl;
    let mid_lbl = CURRENT_TIERS[tier_idx].mid_lbl;

    // 1. Left Y-Axis Labels
    lcd.fill_rect(0, 20, 21, 60, lcd_color::BLACK);
    lcd.draw_string(1, 22, top_lbl, &FONT_5X7, rgb565(140, 160, 185), lcd_color::BLACK);
    lcd.draw_string(1, 46, mid_lbl, &FONT_5X7, rgb565(110, 130, 155), lcd_color::BLACK);
    lcd.draw_string(6, 70, " 0",   &FONT_5X7, rgb565(90, 110, 135),  lcd_color::BLACK);

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

fn draw_text_screen_layout(lcd: &mut longan_nano_bsp::Lcd) {
    lcd.clear(lcd_color::BLACK);

    // 1. Top Header Bar (Y: 0..13)
    lcd.fill_rect(0, 0, 160, 13, 0x11E6); // Slate Navy
    lcd.fill_rect(0, 13, 160, 1, 0x1AE7); // Hairline separator
    lcd.draw_string(5, 3, "INA219", &FONT_5X7, lcd_color::WHITE, 0x11E6);

    // 2. Bottom Weather Widget Tiles (Y: 51..78)
    // Left Tile: VOLTAGE
    lcd.fill_rect(3, 51, 76, 27, 0x0944); // Deep Navy Slate tile
    lcd.rect(3, 51, 76, 27, 0x1AE7);      // Subtle border
    lcd.draw_string(7, 53, "VOLTAGE", &FONT_5X7, 0x7FE0, 0x0944); // Mint label

    // Right Tile: POWER
    lcd.fill_rect(81, 51, 76, 27, 0x0944); // Deep Navy Slate tile
    lcd.rect(81, 51, 76, 27, 0x1AE7);      // Subtle border
    lcd.draw_string(85, 53, "POWER", &FONT_5X7, 0xFEA0, 0x0944); // Amber label

    // Baseline
    lcd.fill_rect(0, 79, 160, 1, lcd_color::BLACK);
}

#[entry]
fn main() -> ! {
    let mut board = Board::take_current_monitor().expect("Board initialization failed");

    writeln!(board.uart0, "\r\n========================================").ok();
    writeln!(board.uart0, " Longan Nano INA219 Current Monitor (Rust)").ok();
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

    // State Variables
    let mut screen_mode = ScreenMode::Text;  // Default: Text Dashboard (Weather-style)
    let mut scale_tier: usize = 2;          // Default 50mA tier
    let mut peak_in_sweep: i32 = 0;
    let mut sweep_x: u16 = GRAPH_X_MIN;
    let mut prev_y: u16 = GRAPH_Y_BASE;

    // 64-bit Energy & Charge Accumulators
    let mut mw_ticks: u64 = 0;
    let mut tenth_ma_ticks: u64 = 0;

    // Initial Screen Draw (Weather Text Dashboard)
    draw_text_screen_layout(&mut board.lcd);

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

    // Button tracking (PA8)
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
                // Short press: Toggle screen mode between Graph and Text version!
                screen_mode = match screen_mode {
                    ScreenMode::Graph => ScreenMode::Text,
                    ScreenMode::Text => ScreenMode::Graph,
                };
                if screen_mode == ScreenMode::Graph {
                    draw_graph_screen_layout(&mut board.lcd, scale_tier);
                    sweep_x = GRAPH_X_MIN;
                    prev_y = GRAPH_Y_BASE;
                    peak_in_sweep = 0;
                } else {
                    draw_text_screen_layout(&mut board.lcd);
                }
                writeln!(
                    board.uart0,
                    "[UI] Screen mode toggled to {}",
                    if screen_mode == ScreenMode::Graph { "GRAPH (Oscilloscope)" } else { "TEXT (Cards)" }
                )
                .ok();
            }
        }
        prev_button_pressed = button_pressed;

        let now = board.delay.uptime_ms();

        // 2. 10 Hz Periodic Measurement, Screen Rendering & USB HID Streaming
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

            // Data extraction & signed handling with near-zero deadband to eliminate ADC jitter
            let raw_c_tenth = data.current_tenth_ma;
            // Deadband: clamp ADC noise within [-1, +1] (i.e. -0.1mA to +0.1mA) to rock-solid 0
            let c_tenth = if raw_c_tenth >= -1 && raw_c_tenth <= 1 {
                0
            } else {
                raw_c_tenth
            };
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
            // 3. Screen Rendering (Graph vs Text)
            // ----------------------------------------------------
            match screen_mode {
                ScreenMode::Graph => {
                    // Top Line 1 (Instantaneous V, I, P, Status)
                    let mut v_buf = [0u8; 12];
                    let mut v_cur = BufferCursor::new(&mut v_buf);
                    write!(
                        v_cur,
                        "{:>2}.{:02}V ",
                        v_mv / 1000,
                        (v_mv % 1000) / 10
                    )
                    .ok();
                    board.lcd.draw_string(2, 1, v_cur.as_str(), &FONT_5X7, lcd_color::GREEN, COL_BG_TOP);

                    let mut c_buf = [0u8; 14];
                    let mut c_cur = BufferCursor::new(&mut c_buf);
                    let whole_c = abs_c_tenth / 10;
                    let frac_c = abs_c_tenth % 10;
                    if is_reverse {
                        write!(c_cur, "-{:>3}.{}mA", whole_c, frac_c).ok();
                    } else {
                        write!(c_cur, " {:>3}.{}mA", whole_c, frac_c).ok();
                    }
                    board.lcd.draw_string(42, 1, c_cur.as_str(), &FONT_5X7, lcd_color::CYAN, COL_BG_TOP);

                    let mut p_buf = [0u8; 14];
                    let mut p_cur = BufferCursor::new(&mut p_buf);
                    if p_tenth < 100000 {
                        write!(p_cur, " {:>3}.{} mW", p_tenth / 10, p_tenth % 10).ok();
                    } else {
                        let whole_w = (p_tenth / 10) / 1000;
                        let frac_w = ((p_tenth / 10) % 1000) / 10;
                        write!(p_cur, " {:>3}.{:02} W", whole_w, frac_w).ok();
                    }
                    board.lcd.draw_string(88, 1, p_cur.as_str(), &FONT_5X7, lcd_color::YELLOW, COL_BG_TOP);

                    // Status Badge (Top-Right): [LIVE], [REV], or [ERR]
                    if !ina_present {
                        board.lcd.draw_string(132, 1, "[ERR] ", &FONT_5X7, lcd_color::RED, COL_BG_TOP);
                    } else if is_reverse {
                        board.lcd.draw_string(132, 1, "[REV] ", &FONT_5X7, lcd_color::RED, COL_BG_TOP);
                    } else {
                        board.lcd.draw_string(132, 1, "[LIVE]", &FONT_5X7, lcd_color::GREEN, COL_BG_TOP);
                    }

                    // Top Line 2 (Energy Consumed, Charge, Tier)
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
                    board.lcd.draw_string(2, 10, e_cur.as_str(), &FONT_5X7, rgb565(255, 160, 40), COL_BG_TOP);

                    let mut q_buf = [0u8; 16];
                    let mut q_cur = BufferCursor::new(&mut q_buf);
                    let total_mah = (tenth_ma_ticks / 360000) as u32;
                    let frac_mah = ((tenth_ma_ticks % 360000) / 3600) as u32;
                    write!(q_cur, "Q:{:>2}.{:02}mAh", total_mah, frac_mah).ok();
                    board.lcd.draw_string(64, 10, q_cur.as_str(), &FONT_5X7, rgb565(100, 255, 140), COL_BG_TOP);

                    // Scale badge
                    board.lcd.draw_string(126, 10, CURRENT_TIERS[scale_tier].badge, &FONT_5X7, lcd_color::WHITE, COL_BG_TOP);

                    // Oscilloscope Waveform Sweep (Current)
                    let plot_val: i32 = abs_c_tenth as i32;
                    if plot_val > peak_in_sweep {
                        peak_in_sweep = plot_val;
                    }

                    // Dynamic Auto-scaling
                    let num_tiers = CURRENT_TIERS.len();
                    let current_max = CURRENT_TIERS[scale_tier].max_val;
                    let mut scale_changed = false;

                    if plot_val > current_max && scale_tier + 1 < num_tiers {
                        while scale_tier + 1 < num_tiers {
                            if plot_val > CURRENT_TIERS[scale_tier].max_val {
                                scale_tier += 1;
                            } else {
                                break;
                            }
                        }
                        scale_changed = true;
                    }

                    if sweep_x == GRAPH_X_MIN && !scale_changed {
                        if scale_tier > 0 {
                            let lower_ceiling = CURRENT_TIERS[scale_tier - 1].max_val;
                            if peak_in_sweep < (lower_ceiling * 65 / 100) {
                                scale_tier -= 1;
                                scale_changed = true;
                            }
                        }
                        peak_in_sweep = 0;
                    }

                    if scale_changed {
                        redraw_grid(&mut board.lcd, scale_tier);
                    }

                    let scale_max = CURRENT_TIERS[scale_tier].max_val;
                    let y_calc = (GRAPH_Y_BASE as i32) - ((plot_val * GRAPH_HEIGHT as i32) / scale_max);
                    let y_curr = if y_calc < GRAPH_Y_TOP as i32 {
                        GRAPH_Y_TOP
                    } else if y_calc > GRAPH_Y_BASE as i32 {
                        GRAPH_Y_BASE
                    } else {
                        y_calc as u16
                    };

                    // Erase lookahead (2 pixels ahead)
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
                                col_buf[idx] = COL_TRACE_I; // Vibrant trace segment
                            } else if y > y_max && y < GRAPH_Y_BASE {
                                col_buf[idx] = COL_FILL_I;  // Glowing area under curve
                            } else if y == GRAPH_Y_BASE {
                                col_buf[idx] = COL_AXIS;    // Baseline
                            } else if (y == 36 || y == 49 || y == 63) && (sweep_x % 4 == 0) {
                                col_buf[idx] = COL_GRID;    // Dotted grid point
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
                }

                ScreenMode::Text => {
                    // Header Bar with Energy & Status Badge
                    let mut e_buf = [0u8; 16];
                    let mut e_cur = BufferCursor::new(&mut e_buf);
                    let total_mwh = (mw_ticks / 36000) as u32;
                    let frac_mwh = ((mw_ticks % 36000) / 360) as u32;
                    if total_mwh == 0 {
                        let uwh = ((mw_ticks * 100) / 360) as u32;
                        write!(e_cur, "E:{:>3}uWh", uwh).ok();
                    } else {
                        write!(e_cur, "E:{:>2}.{:02}mWh", total_mwh, frac_mwh).ok();
                    }
                    board.lcd.draw_string(54, 3, e_cur.as_str(), &FONT_5X7, rgb565(255, 180, 50), 0x11E6);

                    // Status Badge in text header
                    if !ina_present {
                        board.lcd.fill_rect(122, 2, 34, 10, rgb565(50, 0, 0));
                        board.lcd.rect(122, 2, 34, 10, lcd_color::RED);
                        board.lcd.draw_string(128, 3, "ERR ", &FONT_5X7, lcd_color::RED, rgb565(50, 0, 0));
                    } else if is_reverse {
                        board.lcd.fill_rect(122, 2, 34, 10, rgb565(50, 0, 0));
                        board.lcd.rect(122, 2, 34, 10, lcd_color::RED);
                        board.lcd.draw_string(128, 3, "REV ", &FONT_5X7, lcd_color::RED, rgb565(50, 0, 0));
                    } else {
                        board.lcd.fill_rect(122, 2, 34, 10, 0x0162);
                        board.lcd.rect(122, 2, 34, 10, 0x05E2);
                        board.lcd.draw_string(128, 3, "LIVE", &FONT_5X7, lcd_color::GREEN, 0x0162);
                    }

                    // 1. HERO CURRENT DISPLAY (Smooth 28px font, unit small at right edge)
                    let mut c_buf = [0u8; 16];
                    let mut c_cur = BufferCursor::new(&mut c_buf);
                    let whole_c = abs_c_tenth / 10;
                    let frac_c = abs_c_tenth % 10;
                    let unit_str = "mA";

                    if is_reverse {
                        write!(c_cur, "-{}.{}", whole_c, frac_c).ok();
                    } else {
                        write!(c_cur, "{}.{}", whole_c, frac_c).ok();
                    }

                    let curr_col = if is_reverse {
                        rgb565(255, 80, 80)
                    } else {
                        COL_TRACE_I
                    };

                    let s = c_cur.as_str();
                    let str_w = FONT_28.string_width(s);
                    // Center the numeric digits within the available 138px zone (X: 0..137)
                    let x = if str_w < 136 { (136 - str_w) / 2 } else { 0 };

                    // Clear margins around the 28px hero area (Y: 14..50, height = 37)
                    if x > 0 {
                        board.lcd.fill_rect(0, 14, x, 37, lcd_color::BLACK);
                    }
                    if x + str_w < 138 {
                        board.lcd.fill_rect(x + str_w, 14, 138 - (x + str_w), 37, lcd_color::BLACK);
                    }
                    board.lcd.fill_rect(x, 14, str_w, 4, lcd_color::BLACK);
                    board.lcd.fill_rect(x, 46, str_w, 5, lcd_color::BLACK);

                    // Render smooth 28px hero digits (1:1 TrueType curves, crisp, high-res)
                    board.lcd.draw_string(x, 18, s, &FONT_28, curr_col, lcd_color::BLACK);

                    // Render small unit attached to the right edge (X: 140..156, baseline aligned at Y: 27)
                    board.lcd.fill_rect(138, 14, 2, 37, lcd_color::BLACK);
                    board.lcd.fill_rect(140, 14, 20, 13, lcd_color::BLACK);
                    board.lcd.fill_rect(140, 43, 20, 8, lcd_color::BLACK);
                    board.lcd.fill_rect(156, 27, 4, 16, lcd_color::BLACK);

                    board.lcd.draw_string(140, 27, unit_str, &FONT_8X16, curr_col, lcd_color::BLACK);

                    // 2. BOTTOM TILE: VOLTAGE (Font 8x16)
                    let mut v_buf = [0u8; 16];
                    let mut v_cur = BufferCursor::new(&mut v_buf);
                    write!(v_cur, "{:>2}.{:03}V", v_mv / 1000, v_mv % 1000).ok();
                    // 7 chars * 8 px = 56 px, centered in tile at X = 13, Y = 62
                    board.lcd.draw_string(13, 62, v_cur.as_str(), &FONT_8X16, 0xD7FA, 0x0944);

                    // 3. BOTTOM TILE: POWER (Font 8x16)
                    let mut p_buf = [0u8; 16];
                    let mut p_cur = BufferCursor::new(&mut p_buf);
                    if p_tenth < 100000 {
                        write!(p_cur, "{:>4}.{}mW", p_tenth / 10, p_tenth % 10).ok();
                    } else {
                        let whole_w = (p_tenth / 10) / 1000;
                        let frac_w = ((p_tenth / 10) % 1000) / 10;
                        write!(p_cur, "{:>3}.{:02} W", whole_w, frac_w).ok();
                    }
                    // 7 chars * 8 px = 56 px, centered in tile at X = 91, Y = 62
                    board.lcd.draw_string(91, 62, p_cur.as_str(), &FONT_8X16, lcd_color::YELLOW, 0x0944);
                }
            }

            // ----------------------------------------------------
            // 4. USB HID Telemetry Streaming (9 bytes, continuous)
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

        // 5. 1 Hz Diagnostic Telemetry over UART0
        if now.wrapping_sub(last_1hz) >= 1000 {
            last_1hz = now;
            let sec = now / 1000;
            let cfg_str = if board.usb_hid.is_configured() {
                "CFG"
            } else {
                "DISC"
            };
            let sensor_str = if ina_present { "ONLINE" } else { "OFFLINE" };
            let mode_str = if screen_mode == ScreenMode::Graph { "GRAPH" } else { "TEXT" };
            writeln!(
                board.uart0,
                "[HEARTBEAT] T+{}s | Mode: {} | Sensor: {} | USB: {} | HID Reports: {}",
                sec, mode_str, sensor_str, cfg_str, report_count
            )
            .ok();
        }
    }
}
