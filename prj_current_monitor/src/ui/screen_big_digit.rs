use longan_nano_bsp::{
    lcd::{FONT_28, FONT_5X7, FONT_8X16},
    Lcd,
};
use crate::fmt::{fmt_hero_current, fmt_power, fmt_voltage, BufferCursor, DirtyField};
use crate::model::{Accumulators, BatteryState, InaReading};
use crate::ui::theme::*;
use crate::ui::SdStatus;

pub struct BigDigitScreen {
    cur_field: DirtyField<16>,
    volt_field: DirtyField<16>,
    pwr_field: DirtyField<16>,
    batt_field: DirtyField<16>,
    smoothed_current: i16,
    ema_initialized: bool,
    prev_usb_cfg: Option<bool>,
    prev_sd_status: Option<SdStatus>,
}

impl BigDigitScreen {
    pub const fn new() -> Self {
        Self {
            cur_field: DirtyField::new(),
            volt_field: DirtyField::new(),
            pwr_field: DirtyField::new(),
            batt_field: DirtyField::new(),
            smoothed_current: 0,
            ema_initialized: false,
            prev_usb_cfg: None,
            prev_sd_status: None,
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // 1. Top Header Bar (Y: 0..12)
        lcd.fill_rect(0, 0, 160, 12, COL_BG_TOP);
        lcd.fill_rect(0, 12, 160, 1, COL_DIVIDER);
        lcd.draw_string(28, 2, "BIG DIGIT METER", &FONT_5X7, COL_WHITE, COL_BG_TOP);

        // 2. Middle Divider (Y: 48)
        lcd.fill_rect(0, 48, 160, 1, COL_DIVIDER);

        // 3. Bottom Row Tiles (Y: 51..78)
        // Left Tile: VOLTAGE
        lcd.fill_rect(3, 51, 75, 27, COL_CARD_BG);
        lcd.rect(3, 51, 75, 27, COL_CARD_BORDER);
        lcd.draw_string(7, 53, "VOLTAGE", &FONT_5X7, COL_MINT, COL_CARD_BG);

        // Right Tile: POWER
        lcd.fill_rect(82, 51, 75, 27, COL_CARD_BG);
        lcd.rect(82, 51, 75, 27, COL_CARD_BORDER);
        lcd.draw_string(86, 53, "POWER", &FONT_5X7, COL_AMBER, COL_CARD_BG);

        self.cur_field.invalidate();
        self.volt_field.invalidate();
        self.pwr_field.invalidate();
        self.batt_field.invalidate();
        self.ema_initialized = false;
        self.prev_usb_cfg = None;
        self.prev_sd_status = None;
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
    ) {
        // --- 1. Top Bar Status Icons ---
        if self.prev_usb_cfg != Some(usb_configured) {
            let usb_col = if usb_configured { COL_CYAN } else { rgb565(50, 60, 75) };
            lcd.draw_bitmap_8x8(4, 2, &ICON_USB, usb_col, COL_BG_TOP);
            self.prev_usb_cfg = Some(usb_configured);
        }

        if self.prev_sd_status != Some(sd_status) {
            match sd_status {
                SdStatus::Logging => {
                    lcd.draw_bitmap_8x8(16, 2, &ICON_REC, COL_RED, COL_BG_TOP);
                }
                SdStatus::Ready => {
                    lcd.draw_bitmap_8x8(16, 2, &ICON_SD, COL_MINT, COL_BG_TOP);
                }
                SdStatus::WriteError => {
                    lcd.draw_bitmap_8x8(16, 2, &ICON_SD, COL_AMBER, COL_BG_TOP);
                }
                SdStatus::NoCard => {
                    lcd.draw_bitmap_8x8(16, 2, &ICON_SD, rgb565(50, 60, 75), COL_BG_TOP);
                }
            }
            self.prev_sd_status = Some(sd_status);
        }

        // Header right: Battery badge if active
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
        }

        // --- 2. Big Digit Current Reading (Y: 16..44) ---
        let current = if ina_present {
            if !self.ema_initialized {
                self.smoothed_current = reading.current_tenth_ma;
                self.ema_initialized = true;
            } else {
                let diff = (reading.current_tenth_ma as i32) - (self.smoothed_current as i32);
                self.smoothed_current = (self.smoothed_current as i32 + diff / 4) as i16;
            }
            self.smoothed_current
        } else {
            0
        };

        let mut cur_buf = [0u8; 16];
        let mut cur_cur = BufferCursor::new(&mut cur_buf);
        if ina_present {
            fmt_hero_current(&mut cur_cur, current);
        } else {
            use core::fmt::Write;
            write!(cur_cur, "DISCONNECTED").ok();
        }

        let cur_str = cur_cur.as_str();
        if self.cur_field.update(cur_str) {
            // Clear current area
            lcd.fill_rect(0, 14, 160, 33, COL_BLACK);

            if ina_present {
                let text_w = FONT_28.string_width(cur_str);
                let x = if text_w < 160 { (160 - text_w) / 2 } else { 0 };
                let col = if current < 0 {
                    COL_RED // Reverse flow: Coral Red
                } else if current > 0 {
                    COL_CYAN // Forward: Neon Cyan
                } else {
                    COL_WHITE
                };
                lcd.draw_string(x, 16, cur_str, &FONT_28, col, COL_BLACK);
            } else {
                lcd.draw_string(30, 24, cur_str, &FONT_8X16, COL_RED, COL_BLACK);
            }
        }

        // --- 3. Bottom Tiles: Voltage & Power ---
        // Voltage
        let mut v_buf = [0u8; 16];
        let mut v_cur = BufferCursor::new(&mut v_buf);
        if ina_present {
            fmt_voltage(&mut v_cur, reading.voltage_mv);
        } else {
            use core::fmt::Write;
            write!(v_cur, "---.- V").ok();
        }
        if self.volt_field.update(v_cur.as_str()) {
            lcd.fill_rect(7, 62, 68, 14, COL_CARD_BG);
            lcd.draw_string(7, 62, v_cur.as_str(), &FONT_8X16, COL_MINT, COL_CARD_BG);
        }

        // Power
        let mut p_buf = [0u8; 16];
        let mut p_cur = BufferCursor::new(&mut p_buf);
        if ina_present {
            fmt_power(&mut p_cur, reading.power_tenth_mw);
        } else {
            use core::fmt::Write;
            write!(p_cur, "---.- mW").ok();
        }
        if self.pwr_field.update(p_cur.as_str()) {
            lcd.fill_rect(86, 62, 68, 14, COL_CARD_BG);
            lcd.draw_string(86, 62, p_cur.as_str(), &FONT_8X16, COL_AMBER, COL_CARD_BG);
        }
    }
}
