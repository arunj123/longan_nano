use longan_nano_bsp::{
    lcd::{FONT_28, FONT_5X7, FONT_8X16},
    Lcd,
};
use crate::fmt::{fmt_hero_current, fmt_voltage, BufferCursor, DirtyField};
use crate::model::{Accumulators, BatteryState, EmaFilter, InaReading};
use crate::ui::theme::*;
use crate::ui::SdStatus;

pub struct BigDigitScreen {
    cur_field: DirtyField<16>,
    unit_field: DirtyField<4>,
    volt_field: DirtyField<16>,
    batt_field: DirtyField<16>,
    prev_x: u16,
    prev_w: u16,
    prev_scale: u8,
    ema: EmaFilter,
    prev_usb_cfg: Option<bool>,
    prev_sd_status: Option<SdStatus>,
    prev_over_current: Option<bool>,
}

impl BigDigitScreen {
    pub const fn new() -> Self {
        Self {
            cur_field: DirtyField::new(),
            unit_field: DirtyField::new(),
            volt_field: DirtyField::new(),
            batt_field: DirtyField::new(),
            prev_x: 0,
            prev_w: 0,
            prev_scale: 0,
            ema: EmaFilter::new(),
            prev_usb_cfg: None,
            prev_sd_status: None,
            prev_over_current: None,
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // 1. Slim Top Status Bar (Y: 0..12)
        lcd.fill_rect(0, 0, 160, 12, COL_BG_TOP);
        lcd.fill_rect(0, 12, 160, 1, COL_DIVIDER);

        self.cur_field.invalidate();
        self.unit_field.invalidate();
        self.volt_field.invalidate();
        self.batt_field.invalidate();
        self.prev_x = 0;
        self.prev_w = 0;
        self.prev_scale = 0;
        self.ema.reset();
        self.prev_usb_cfg = None;
        self.prev_sd_status = None;
        self.prev_over_current = None;
    }

    pub fn update(
        &mut self,
        lcd: &mut Lcd,
        reading: &InaReading,
        _accum: &Accumulators,
        battery: &BatteryState,
        ina_present: bool,
        usb_configured: bool,
        sd_status: SdStatus,
        over_current: bool,
    ) {
        // --- 1. Top Bar Status Icons & Telemetry ---
        if self.prev_usb_cfg != Some(usb_configured) {
            let usb_col = if usb_configured { COL_CYAN } else { rgb565(50, 60, 75) };
            lcd.draw_bitmap_8x8(3, 2, &ICON_USB, usb_col, COL_BG_TOP);
            self.prev_usb_cfg = Some(usb_configured);
        }

        if self.prev_sd_status != Some(sd_status) {
            match sd_status {
                SdStatus::Logging => {
                    lcd.draw_bitmap_8x8(14, 2, &ICON_REC, COL_RED, COL_BG_TOP);
                }
                SdStatus::Ready => {
                    lcd.draw_bitmap_8x8(14, 2, &ICON_SD, COL_MINT, COL_BG_TOP);
                }
                SdStatus::WriteError => {
                    lcd.draw_bitmap_8x8(14, 2, &ICON_SD, COL_AMBER, COL_BG_TOP);
                }
                SdStatus::NoCard => {
                    lcd.draw_bitmap_8x8(14, 2, &ICON_SD, rgb565(50, 60, 75), COL_BG_TOP);
                }
            }
            self.prev_sd_status = Some(sd_status);
        }

        // Active Bus Voltage in Top Bar (X: 26, Y: 2)
        let mut v_buf = [0u8; 12];
        let mut v_cur = BufferCursor::new(&mut v_buf);
        if ina_present {
            fmt_voltage(&mut v_cur, reading.voltage_mv);
        } else {
            use core::fmt::Write;
            write!(v_cur, "---.- V").ok();
        }
        if self.volt_field.update(v_cur.as_str()) {
            lcd.draw_string(26, 2, v_cur.as_str(), &FONT_5X7, COL_MINT, COL_BG_TOP);
        }

        // Over-Current Alert Indicator or Clean Space in Center of Top Bar
        if self.prev_over_current != Some(over_current) {
            if over_current {
                lcd.draw_string(72, 2, "!ALERT!", &FONT_5X7, COL_AMBER, COL_BG_TOP);
            } else {
                lcd.fill_rect(72, 2, 42, 9, COL_BG_TOP);
            }
        }

        // Top Right: Battery Fuel Gauge or Mode Label
        let cap = battery.profile.capacity_mah();
        if cap > 0 {
            let mut bat_buf = [0u8; 12];
            let mut bat_cur = BufferCursor::new(&mut bat_buf);
            use core::fmt::Write;
            write!(bat_cur, "{}%{:>4}", battery.soc_pct, battery.profile.name()).ok();
            if self.batt_field.update(bat_cur.as_str()) {
                let col = if battery.is_low_voltage { COL_RED } else { COL_MINT };
                lcd.draw_string(114, 2, bat_cur.as_str(), &FONT_5X7, col, COL_BG_TOP);
            }
        } else if self.batt_field.update("BIG") {
            lcd.draw_string(138, 2, "BIG", &FONT_5X7, COL_TEXT_MUTED, COL_BG_TOP);
        }

        // --- 2. Giant Scaled Current Display (Y: 13..79) ---
        let current = if ina_present {
            self.ema.update(reading.current_tenth_ma)
        } else {
            0
        };

        let mut cur_buf = [0u8; 16];
        let mut cur_cur = BufferCursor::new(&mut cur_buf);
        let (num_str, raw_unit) = if ina_present {
            fmt_hero_current(&mut cur_cur, current)
        } else {
            use core::fmt::Write;
            write!(cur_cur, "DISCONNECTED").ok();
            (cur_cur.as_str(), "")
        };
        let unit_str = raw_unit.trim();

        let curr_col = if !ina_present {
            COL_RED
        } else if over_current {
            COL_AMBER
        } else if reading.is_reverse {
            COL_RED
        } else if current == 0 {
            COL_WHITE
        } else {
            COL_CYAN
        };

        let color_changed = self.prev_over_current != Some(over_current);
        self.prev_over_current = Some(over_current);

        if !ina_present {
            if self.cur_field.update(num_str) || color_changed {
                lcd.fill_rect(0, 14, 160, 66, COL_BLACK);
                let text_w = (num_str.len() as u16) * 8;
                let x = if text_w < 160 { (160 - text_w) / 2 } else { 0 };
                lcd.draw_string(x, 38, num_str, &FONT_8X16, COL_RED, COL_BLACK);
                self.unit_field.invalidate();
                self.prev_x = 0;
                self.prev_w = 0;
                self.prev_scale = 0;
            }
            return;
        }

        let unscaled_w = FONT_28.string_width(num_str);
        let scale2_w = unscaled_w * 2;
        let cur_changed = self.cur_field.update(num_str);

        if cur_changed || color_changed {
            let scale: u8 = if scale2_w <= 158 { 2 } else { 1 };
            let (x, y, w) = if scale == 2 {
                let x = if scale2_w <= 138 {
                    if scale2_w < 136 { (136 - scale2_w) / 2 } else { 1 }
                } else {
                    (160 - scale2_w) / 2
                };
                (x, 18u16, scale2_w)
            } else {
                let x = if unscaled_w < 134 { (134 - unscaled_w) / 2 } else { 4 };
                (x, 30u16, unscaled_w)
            };

            // If bounding box or scale changed, clear old digit area cleanly
            if x != self.prev_x || w != self.prev_w || scale != self.prev_scale {
                lcd.fill_rect(0, 14, 160, 66, COL_BLACK);
                self.prev_x = x;
                self.prev_w = w;
                self.prev_scale = scale;
                self.unit_field.invalidate();
            }

            if scale == 2 {
                lcd.draw_string_scaled(x, y, num_str, &FONT_28, curr_col, COL_BLACK, 2);
            } else {
                lcd.draw_string(x, y, num_str, &FONT_28, curr_col, COL_BLACK);
            }
        }

        // Render unit indicator in dedicated corner position
        if self.unit_field.update(unit_str) || cur_changed || color_changed {
            if !unit_str.is_empty() {
                if scale2_w <= 138 {
                    // Ample space: place unit in lower-right corner at scale 1
                    lcd.fill_rect(138, 50, 22, 28, COL_BLACK);
                    lcd.draw_string(140, 56, unit_str, &FONT_8X16, curr_col, COL_BLACK);
                } else if scale2_w <= 158 {
                    // Wide 5-digit number filling width: place unit in upper-right corner
                    lcd.fill_rect(138, 14, 22, 16, COL_BLACK);
                    lcd.draw_string(144, 16, unit_str, &FONT_5X7, curr_col, COL_BLACK);
                } else {
                    // Fallback scale 1: place unit next to centered digits
                    lcd.fill_rect(138, 30, 22, 24, COL_BLACK);
                    lcd.draw_string(140, 36, unit_str, &FONT_8X16, curr_col, COL_BLACK);
                }
            }
        }
    }
}
