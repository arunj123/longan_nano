use longan_nano_bsp::{
    lcd::{FONT_28, FONT_5X7, FONT_8X16},
    Lcd,
};
use crate::fmt::{fmt_energy_auto, fmt_hero_current, fmt_power, fmt_voltage, BufferCursor, DirtyField};
use crate::model::{Accumulators, EmaFilter, InaReading};
use crate::ui::theme::*;
use crate::ui::SdStatus;


pub struct HeroScreen {
    cur_field: DirtyField<16>,
    unit_field: DirtyField<4>,
    volt_field: DirtyField<16>,
    pwr_field: DirtyField<16>,
    energy_field: DirtyField<16>,
    peak_bar_val: u16,
    peak_decay_counter: u8,
    ema: EmaFilter,
    prev_usb_cfg: Option<bool>,
    prev_sd_status: Option<SdStatus>,
    prev_badge: Option<u8>,
}

impl HeroScreen {
    pub const fn new() -> Self {
        Self {
            cur_field: DirtyField::new(),
            unit_field: DirtyField::new(),
            volt_field: DirtyField::new(),
            pwr_field: DirtyField::new(),
            energy_field: DirtyField::new(),
            peak_bar_val: 0,
            peak_decay_counter: 0,
            ema: EmaFilter::new(),
            prev_usb_cfg: None,
            prev_sd_status: None,
            prev_badge: None,
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // 1. Top Header Bar (Y: 0..13)
        lcd.fill_rect(0, 0, 160, 13, COL_BG_TOP);
        lcd.fill_rect(0, 13, 160, 1, COL_DIVIDER);

        // 2. Bottom Weather Widget Tiles (Y: 51..78)
        // Left Tile: VOLTAGE
        lcd.fill_rect(3, 51, 76, 27, COL_CARD_BG);
        lcd.rect(3, 51, 76, 27, COL_CARD_BORDER);
        lcd.draw_string(7, 53, "VOLTAGE", &FONT_5X7, COL_MINT, COL_CARD_BG);

        // Right Tile: POWER
        lcd.fill_rect(81, 51, 76, 27, COL_CARD_BG);
        lcd.rect(81, 51, 76, 27, COL_CARD_BORDER);
        lcd.draw_string(85, 53, "POWER", &FONT_5X7, COL_AMBER, COL_CARD_BG);

        self.cur_field.invalidate();
        self.unit_field.invalidate();
        self.volt_field.invalidate();
        self.pwr_field.invalidate();
        self.energy_field.invalidate();
        self.peak_bar_val = 0;
        self.ema.reset();
        self.prev_usb_cfg = None;
        self.prev_sd_status = None;
        self.prev_badge = None;
    }

    pub fn update(
        &mut self,
        lcd: &mut Lcd,
        reading: &InaReading,
        accum: &Accumulators,
        ina_present: bool,
        usb_configured: bool,
        sd_status: SdStatus,
    ) {
        // --- 1. Header Bar: Status Icons, Energy & Status Badge ---
        // 1a. USB Status Icon (X: 4, Y: 3)
        if self.prev_usb_cfg != Some(usb_configured) {
            let usb_col = if usb_configured { COL_CYAN } else { rgb565(50, 60, 75) };
            lcd.draw_bitmap_8x8(4, 3, &ICON_USB, usb_col, COL_BG_TOP);
            self.prev_usb_cfg = Some(usb_configured);
        }

        // 1b. SD Card Status Icon (X: 16, Y: 3)
        if self.prev_sd_status != Some(sd_status) {
            match sd_status {
                SdStatus::Logging => {
                    lcd.draw_bitmap_8x8(16, 3, &ICON_REC, COL_RED, COL_BG_TOP);
                }
                SdStatus::Ready => {
                    lcd.draw_bitmap_8x8(16, 3, &ICON_SD, COL_MINT, COL_BG_TOP);
                }
                SdStatus::WriteError => {
                    lcd.draw_bitmap_8x8(16, 3, &ICON_SD, COL_AMBER, COL_BG_TOP);
                }
                SdStatus::NoCard => {
                    lcd.draw_bitmap_8x8(16, 3, &ICON_SD, rgb565(50, 60, 75), COL_BG_TOP);
                }
            }
            self.prev_sd_status = Some(sd_status);
        }

        // 1c. Energy Accumulator (X: 30, Y: 3)
        let mut e_buf = [0u8; 16];
        let mut e_cur = BufferCursor::new(&mut e_buf);
        fmt_energy_auto(&mut e_cur, accum);
        if self.energy_field.update(e_cur.as_str()) {
            lcd.draw_string(30, 3, e_cur.as_str(), &FONT_5X7, COL_AMBER, COL_BG_TOP);
        }

        // 1d. Status Badge (Top-Right)
        let cur_badge = if !ina_present {
            0u8
        } else if reading.overflow {
            1u8
        } else if reading.is_reverse {
            2u8
        } else {
            3u8
        };

        if self.prev_badge != Some(cur_badge) {
            match cur_badge {
                0 => {
                    lcd.fill_rect(122, 2, 35, 10, rgb565(60, 0, 0));
                    lcd.rect(122, 2, 35, 10, COL_RED);
                    lcd.draw_string(126, 3, "ERR ", &FONT_5X7, COL_RED, rgb565(60, 0, 0));
                }
                1 => {
                    lcd.fill_rect(122, 2, 35, 10, rgb565(60, 40, 0));
                    lcd.rect(122, 2, 35, 10, COL_AMBER);
                    lcd.draw_string(126, 3, "OVF ", &FONT_5X7, COL_AMBER, rgb565(60, 40, 0));
                }
                2 => {
                    lcd.fill_rect(122, 2, 35, 10, rgb565(60, 0, 0));
                    lcd.rect(122, 2, 35, 10, COL_RED);
                    lcd.draw_string(126, 3, "REV ", &FONT_5X7, COL_RED, rgb565(60, 0, 0));
                }
                _ => {
                    lcd.fill_rect(122, 2, 35, 10, 0x0162);
                    lcd.rect(122, 2, 35, 10, 0x05E2);
                    lcd.draw_string(126, 3, "LIVE", &FONT_5X7, COL_MINT, 0x0162);
                }
            }
            self.prev_badge = Some(cur_badge);
        }

        // --- 2. Hero Current Display with EMA Smoothing ---
        let raw_c = reading.current_tenth_ma;
        let display_c = self.ema.update(raw_c);

        let mut c_buf = [0u8; 16];
        let mut c_cur = BufferCursor::new(&mut c_buf);
        let (num_str, unit_str) = fmt_hero_current(&mut c_cur, display_c);

        let curr_col = if reading.is_reverse {
            COL_RED
        } else {
            COL_CYAN
        };

        if self.cur_field.update(num_str) {
            let str_w = FONT_28.string_width(num_str);
            let x = if str_w < 136 { (136 - str_w) / 2 } else { 0 };

            // Clear margins
            if x > 0 {
                lcd.fill_rect(0, 14, x, 31, COL_BLACK);
            }
            if x + str_w < 138 {
                lcd.fill_rect(x + str_w, 14, 138 - (x + str_w), 31, COL_BLACK);
            }
            lcd.fill_rect(x, 14, str_w, 2, COL_BLACK);
            lcd.fill_rect(x, 44, str_w, 2, COL_BLACK);

            lcd.draw_string(x, 16, num_str, &FONT_28, curr_col, COL_BLACK);
        }

        if self.unit_field.update(unit_str) {
            lcd.fill_rect(138, 14, 22, 31, COL_BLACK);
            lcd.draw_string(140, 24, unit_str, &FONT_8X16, curr_col, COL_BLACK);
        }

        // --- 3. Dynamic Load Gauge Bar with Peak-Hold Marker (Y: 46..48) ---
        let bar_max: u32 = 10_000; // 1000.0 mA nominal full-scale for gauge
        let abs_c = reading.abs_current_tenth();
        let fill_w = core::cmp::min(130, (abs_c * 130) / bar_max) as u16;

        if fill_w > self.peak_bar_val {
            self.peak_bar_val = fill_w;
            self.peak_decay_counter = 20; // Hold for 2 seconds (20 ticks @ 10Hz)
        } else if self.peak_decay_counter > 0 {
            self.peak_decay_counter -= 1;
        } else if self.peak_bar_val > fill_w {
            self.peak_bar_val -= 1; // Smooth decay
        }

        // Draw gauge bar
        let gauge_x = 4;
        let gauge_y = 47;
        let gauge_w = 130;
        let bar_col = if reading.is_reverse {
            COL_RED
        } else if fill_w > 100 {
            COL_AMBER
        } else {
            COL_CYAN
        };

        if fill_w > 0 {
            lcd.fill_rect(gauge_x, gauge_y, fill_w, 2, bar_col);
        }
        if fill_w < gauge_w {
            lcd.fill_rect(gauge_x + fill_w, gauge_y, gauge_w - fill_w, 2, COL_GRAPH_BG);
        }
        // Peak-hold tick
        if self.peak_bar_val > 0 && self.peak_bar_val <= gauge_w {
            lcd.fill_rect(gauge_x + self.peak_bar_val - 1, gauge_y - 1, 2, 4, COL_WHITE);
        }

        // --- 4. Bottom Weather Tiles ---
        // Voltage
        let mut v_buf = [0u8; 16];
        let mut v_cur = BufferCursor::new(&mut v_buf);
        fmt_voltage(&mut v_cur, reading.voltage_mv);
        if self.volt_field.update(v_cur.as_str()) {
            lcd.draw_string(13, 62, v_cur.as_str(), &FONT_8X16, 0xD7FA, COL_CARD_BG);
        }

        // Power
        let mut p_buf = [0u8; 16];
        let mut p_cur = BufferCursor::new(&mut p_buf);
        fmt_power(&mut p_cur, reading.power_tenth_mw);
        if self.pwr_field.update(p_cur.as_str()) {
            lcd.draw_string(91, 62, p_cur.as_str(), &FONT_8X16, COL_AMBER, COL_CARD_BG);
        }
    }
}
