use longan_nano_bsp::{
    lcd::FONT_5X7,
    Lcd,
};
use crate::fmt::{fmt_charge_auto, fmt_energy_auto, BufferCursor, DirtyField};
use crate::model::{Accumulators, CURRENT_TIERS, InaReading, WaveformHistory};
use crate::ui::theme::*;
use crate::ui::SdStatus;

pub const GRAPH_X_MIN: u16 = 22;
pub const GRAPH_X_MAX: u16 = 158;
pub const GRAPH_Y_TOP: u16 = 22;
pub const GRAPH_Y_BASE: u16 = 76;
pub const GRAPH_HEIGHT: u16 = GRAPH_Y_BASE - GRAPH_Y_TOP; // 54 pixels

pub struct GraphScreen {
    pub scale_tier: usize,
    pub sweep_x: u16,
    pub prev_y: u16,
    peak_in_sweep: i32,
    v_field: DirtyField<12>,
    i_field: DirtyField<12>,
    p_field: DirtyField<12>,
    e_field: DirtyField<16>,
    q_field: DirtyField<16>,
    scale_field: DirtyField<8>,
    badge_field: DirtyField<8>,
    prev_usb_cfg: Option<bool>,
    prev_sd_status: Option<SdStatus>,
}

impl GraphScreen {
    pub const fn new() -> Self {
        Self {
            scale_tier: 2, // 50mA tier
            sweep_x: GRAPH_X_MIN,
            prev_y: GRAPH_Y_BASE,
            peak_in_sweep: 0,
            v_field: DirtyField::new(),
            i_field: DirtyField::new(),
            p_field: DirtyField::new(),
            e_field: DirtyField::new(),
            q_field: DirtyField::new(),
            scale_field: DirtyField::new(),
            badge_field: DirtyField::new(),
            prev_usb_cfg: None,
            prev_sd_status: None,
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);
        lcd.fill_rect(0, 0, 160, 19, COL_BG_TOP);
        lcd.fill_rect(0, 19, 160, 1, COL_DIVIDER);

        self.redraw_grid(lcd);
        self.sweep_x = GRAPH_X_MIN;
        self.prev_y = GRAPH_Y_BASE;
        self.peak_in_sweep = 0;

        self.v_field.invalidate();
        self.i_field.invalidate();
        self.p_field.invalidate();
        self.e_field.invalidate();
        self.q_field.invalidate();
        self.scale_field.invalidate();
        self.badge_field.invalidate();
        self.prev_usb_cfg = None;
        self.prev_sd_status = None;
    }

    pub fn redraw_grid(&mut self, lcd: &mut Lcd) {
        let tier = &CURRENT_TIERS[self.scale_tier];

        lcd.fill_rect(0, 20, 21, 60, COL_BLACK);
        lcd.draw_string(1, 22, tier.top_lbl, &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);
        lcd.draw_string(1, 46, tier.mid_lbl, &FONT_5X7, rgb565(110, 130, 155), COL_BLACK);
        lcd.draw_string(6, 70, " 0",   &FONT_5X7, rgb565(90, 110, 135),  COL_BLACK);

        lcd.fill_rect(21, 20, 1, 60, COL_AXIS);
        lcd.fill_rect(GRAPH_X_MIN, 20, GRAPH_X_MAX - GRAPH_X_MIN + 1, 60, COL_GRAPH_BG);
        lcd.fill_rect(GRAPH_X_MIN, GRAPH_Y_BASE, GRAPH_X_MAX - GRAPH_X_MIN + 1, 1, COL_AXIS);

        let mut x = GRAPH_X_MIN;
        while x <= GRAPH_X_MAX {
            lcd.set_pixel(x, 36, COL_GRID); // 75%
            lcd.set_pixel(x, 49, COL_GRID); // 50%
            lcd.set_pixel(x, 63, COL_GRID); // 25%
            x += 4;
        }
    }

    /// Replots all available historical samples from the circular buffer at the current scale tier.
    pub fn replot_history(&mut self, lcd: &mut Lcd, history: &WaveformHistory) {
        self.redraw_grid(lcd);

        let scale_max = CURRENT_TIERS[self.scale_tier].max_val;
        let count = history.count;
        let mut prev = GRAPH_Y_BASE;

        for i in 0..count {
            let x = GRAPH_X_MIN + (i as u16);
            if x > GRAPH_X_MAX {
                break;
            }
            let raw_sample = history.get_chronological(i);
            let is_sample_rev = raw_sample < 0;
            let sample_abs = raw_sample.unsigned_abs() as i32;
            let y_calc = (GRAPH_Y_BASE as i32) - ((sample_abs * GRAPH_HEIGHT as i32) / scale_max);
            let y_curr = if y_calc < GRAPH_Y_TOP as i32 {
                GRAPH_Y_TOP
            } else if y_calc > GRAPH_Y_BASE as i32 {
                GRAPH_Y_BASE
            } else {
                y_calc as u16
            };

            let mut col_buf = [0u16; 55];
            let y_min = core::cmp::min(prev, y_curr);
            let y_max = core::cmp::max(prev, y_curr);
            let trace_col = if is_sample_rev { COL_RED } else { COL_CYAN };
            let fill_h = core::cmp::max(1, (GRAPH_Y_BASE - y_max) as i32);

            for y in GRAPH_Y_TOP..=GRAPH_Y_BASE {
                let idx = (y - GRAPH_Y_TOP) as usize;
                if idx < col_buf.len() {
                    if y >= y_min && y <= y_max {
                        col_buf[idx] = trace_col;
                    } else if y > y_max && y < GRAPH_Y_BASE {
                        let dist = (y - y_max) as i32;
                        if is_sample_rev {
                            let r = 20 - (14 * dist / fill_h);
                            let g = 4 - (3 * dist / fill_h);
                            let b = 4 - (3 * dist / fill_h);
                            col_buf[idx] = ((r as u16 & 0x1F) << 11) | ((g as u16 & 0x3F) << 5) | (b as u16 & 0x1F);
                        } else {
                            let g = 46 - (36 * dist / fill_h);
                            let b = 14 - (11 * dist / fill_h);
                            col_buf[idx] = ((g as u16 & 0x3F) << 5) | (b as u16 & 0x1F);
                        }
                    } else if y == GRAPH_Y_BASE {
                        col_buf[idx] = COL_AXIS;
                    } else if (y == 36 || y == 49 || y == 63) && (x % 4 == 0) {
                        col_buf[idx] = COL_GRID;
                    } else {
                        col_buf[idx] = COL_GRAPH_BG;
                    }
                }
            }
            lcd.write_pixels(x, GRAPH_Y_TOP, 1, GRAPH_HEIGHT + 1, &col_buf);
            prev = y_curr;
        }

        self.sweep_x = GRAPH_X_MIN + (count as u16);
        if self.sweep_x > GRAPH_X_MAX {
            self.sweep_x = GRAPH_X_MIN;
        }
        self.prev_y = prev;
    }

    pub fn update(
        &mut self,
        lcd: &mut Lcd,
        reading: &InaReading,
        accum: &Accumulators,
        history: &WaveformHistory,
        ina_present: bool,
        sd_status: SdStatus,
        usb_configured: bool,
    ) {
        use core::fmt::Write;

        // --- 1. Top Header Line 1: V, I, P, Status Badge (Compact bounds to prevent overlap) ---
        let mut v_buf = [0u8; 12];
        let mut v_cur = BufferCursor::new(&mut v_buf);
        write!(v_cur, "{:>2}.{:02}V", reading.voltage_mv / 1000, (reading.voltage_mv % 1000) / 10).ok();
        if self.v_field.update(v_cur.as_str()) {
            lcd.draw_string(2, 1, v_cur.as_str(), &FONT_5X7, COL_MINT, COL_BG_TOP);
        }

        let mut c_buf = [0u8; 12];
        let mut c_cur = BufferCursor::new(&mut c_buf);
        let abs_c = reading.abs_current_tenth();
        if abs_c >= 10_000 {
            write!(c_cur, "{}{}.{:02}A", if reading.is_reverse { "-" } else { " " }, abs_c / 10_000, (abs_c % 10_000) / 100).ok();
        } else {
            write!(c_cur, "{}{:>3}mA", if reading.is_reverse { "-" } else { " " }, abs_c / 10).ok();
        }
        if self.i_field.update(c_cur.as_str()) {
            lcd.draw_string(40, 1, c_cur.as_str(), &FONT_5X7, COL_CYAN, COL_BG_TOP);
        }

        let mut p_buf = [0u8; 12];
        let mut p_cur = BufferCursor::new(&mut p_buf);
        if reading.power_tenth_mw < 100_000 {
            write!(p_cur, "{:>4}mW", reading.power_tenth_mw / 10).ok();
        } else {
            write!(p_cur, "{:>2}.{:02}W", (reading.power_tenth_mw / 10) / 1000, ((reading.power_tenth_mw / 10) % 1000) / 10).ok();
        }
        if self.p_field.update(p_cur.as_str()) {
            lcd.draw_string(84, 1, p_cur.as_str(), &FONT_5X7, COL_AMBER, COL_BG_TOP);
        }

        // Status badge strictly placed at x=124..158 without overlap
        let (badge_str, badge_col) = if !ina_present {
            ("[ERR] ", COL_RED)
        } else if reading.overflow {
            ("[OVF] ", COL_AMBER)
        } else if reading.is_reverse {
            ("[REV] ", COL_RED)
        } else {
            ("[LIVE]", COL_MINT)
        };
        if self.badge_field.update(badge_str) {
            lcd.draw_string(124, 1, badge_str, &FONT_5X7, badge_col, COL_BG_TOP);
        }

        // --- 2. Top Header Line 2: Energy, Charge, Tier Badge, USB/SD Status Icons ---
        let mut e_buf = [0u8; 16];
        let mut e_cur = BufferCursor::new(&mut e_buf);
        fmt_energy_auto(&mut e_cur, accum);
        if self.e_field.update(e_cur.as_str()) {
            lcd.draw_string(2, 10, e_cur.as_str(), &FONT_5X7, COL_AMBER, COL_BG_TOP);
        }

        let mut q_buf = [0u8; 16];
        let mut q_cur = BufferCursor::new(&mut q_buf);
        fmt_charge_auto(&mut q_cur, accum);
        if self.q_field.update(q_cur.as_str()) {
            lcd.draw_string(52, 10, q_cur.as_str(), &FONT_5X7, COL_MINT, COL_BG_TOP);
        }

        let badge = CURRENT_TIERS[self.scale_tier].badge;
        if self.scale_field.update(badge) {
            lcd.draw_string(96, 10, badge, &FONT_5X7, COL_WHITE, COL_BG_TOP);
        }

        if self.prev_usb_cfg != Some(usb_configured) {
            let usb_col = if usb_configured { COL_CYAN } else { rgb565(50, 60, 75) };
            lcd.draw_bitmap_8x8(136, 10, &ICON_USB, usb_col, COL_BG_TOP);
            self.prev_usb_cfg = Some(usb_configured);
        }

        if self.prev_sd_status != Some(sd_status) {
            match sd_status {
                SdStatus::Logging => {
                    lcd.draw_bitmap_8x8(148, 10, &ICON_REC, COL_RED, COL_BG_TOP);
                }
                SdStatus::Ready => {
                    lcd.draw_bitmap_8x8(148, 10, &ICON_SD, COL_MINT, COL_BG_TOP);
                }
                SdStatus::WriteError => {
                    lcd.draw_bitmap_8x8(148, 10, &ICON_SD, COL_AMBER, COL_BG_TOP);
                }
                SdStatus::NoCard => {
                    lcd.draw_bitmap_8x8(148, 10, &ICON_SD, rgb565(50, 60, 75), COL_BG_TOP);
                }
            }
            self.prev_sd_status = Some(sd_status);
        }

        // --- 3. Oscilloscope Waveform & Autoscale ---
        let plot_val: i32 = reading.abs_current_tenth() as i32;
        if plot_val > self.peak_in_sweep {
            self.peak_in_sweep = plot_val;
        }

        let num_tiers = CURRENT_TIERS.len();
        let current_max = CURRENT_TIERS[self.scale_tier].max_val;
        let mut scale_changed = false;

        // Upscale if sample exceeds tier ceiling
        if plot_val > current_max && self.scale_tier + 1 < num_tiers {
            while self.scale_tier + 1 < num_tiers {
                if plot_val > CURRENT_TIERS[self.scale_tier].max_val {
                    self.scale_tier += 1;
                } else {
                    break;
                }
            }
            scale_changed = true;
        }

        // Downscale on wrap if signal fits well in lower tier
        if self.sweep_x == GRAPH_X_MIN && !scale_changed {
            if self.scale_tier > 0 {
                let lower_ceiling = CURRENT_TIERS[self.scale_tier - 1].max_val;
                if self.peak_in_sweep < (lower_ceiling * 65 / 100) {
                    self.scale_tier -= 1;
                    scale_changed = true;
                }
            }
            self.peak_in_sweep = 0;
        }

        if scale_changed {
            // Replot full waveform history without wiping signal!
            self.replot_history(lcd, history);
            return;
        }

        // Normal Sweep Column Rendering
        let scale_max = CURRENT_TIERS[self.scale_tier].max_val;
        let y_calc = (GRAPH_Y_BASE as i32) - ((plot_val * GRAPH_HEIGHT as i32) / scale_max);
        let y_curr = if y_calc < GRAPH_Y_TOP as i32 {
            GRAPH_Y_TOP
        } else if y_calc > GRAPH_Y_BASE as i32 {
            GRAPH_Y_BASE
        } else {
            y_calc as u16
        };

        // Advance 2 columns ahead for trace erase bar
        let erase_x1 = if self.sweep_x + 1 > GRAPH_X_MAX { GRAPH_X_MIN } else { self.sweep_x + 1 };
        let erase_x2 = if self.sweep_x + 2 > GRAPH_X_MAX { GRAPH_X_MIN + 1 } else { self.sweep_x + 2 };

        for ex in [erase_x1, erase_x2] {
            lcd.fill_rect(ex, GRAPH_Y_TOP, 1, GRAPH_HEIGHT + 1, COL_GRAPH_BG);
            lcd.set_pixel(ex, GRAPH_Y_BASE, COL_AXIS);
            if ex % 4 == 0 {
                lcd.set_pixel(ex, 36, COL_GRID);
                lcd.set_pixel(ex, 49, COL_GRID);
                lcd.set_pixel(ex, 63, COL_GRID);
            }
        }

        let mut col_buf = [0u16; 55];
        let y_min = core::cmp::min(self.prev_y, y_curr);
        let y_max = core::cmp::max(self.prev_y, y_curr);
        let trace_col = if reading.is_reverse { COL_RED } else { COL_CYAN };
        let fill_h = core::cmp::max(1, (GRAPH_Y_BASE - y_max) as i32);

        for y in GRAPH_Y_TOP..=GRAPH_Y_BASE {
            let idx = (y - GRAPH_Y_TOP) as usize;
            if idx < col_buf.len() {
                if y >= y_min && y <= y_max {
                    col_buf[idx] = trace_col;
                } else if y > y_max && y < GRAPH_Y_BASE {
                    let dist = (y - y_max) as i32;
                    if reading.is_reverse {
                        let r = 20 - (14 * dist / fill_h);
                        let g = 4 - (3 * dist / fill_h);
                        let b = 4 - (3 * dist / fill_h);
                        col_buf[idx] = ((r as u16 & 0x1F) << 11) | ((g as u16 & 0x3F) << 5) | (b as u16 & 0x1F);
                    } else {
                        let g = 46 - (36 * dist / fill_h);
                        let b = 14 - (11 * dist / fill_h);
                        col_buf[idx] = ((g as u16 & 0x3F) << 5) | (b as u16 & 0x1F);
                    }
                } else if y == GRAPH_Y_BASE {
                    col_buf[idx] = COL_AXIS;
                } else if (y == 36 || y == 49 || y == 63) && (self.sweep_x % 4 == 0) {
                    col_buf[idx] = COL_GRID;
                } else {
                    col_buf[idx] = COL_GRAPH_BG;
                }
            }
        }

        lcd.write_pixels(self.sweep_x, GRAPH_Y_TOP, 1, GRAPH_HEIGHT + 1, &col_buf);

        self.prev_y = y_curr;
        self.sweep_x += 1;
        if self.sweep_x > GRAPH_X_MAX {
            self.sweep_x = GRAPH_X_MIN;
        }
    }
}
