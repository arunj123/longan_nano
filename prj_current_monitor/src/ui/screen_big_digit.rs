use longan_nano_bsp::{
    lcd::{Font, FONT_28, FONT_5X7, FONT_8X16},
    Lcd,
};
use crate::fmt::{fmt_voltage, BufferCursor, DirtyField};
use crate::model::{Accumulators, BatteryState, EmaFilter, InaReading};
use crate::ui::theme::*;
use crate::ui::SdStatus;

#[derive(Copy, Clone)]
struct SlotConfig {
    x: u16,
    w: u16,
}

// 5 pre-allocated, fixed-width character slots for mA mode:
// Total width: 36 + 36 + 36 + 16 + 36 = 160 pixels (X: 0..159).
// Slot 0: [0..36) (Hundreds / '-' / ' ')
// Slot 1: [36..72) (Tens / ' ')
// Slot 2: [72..108) (Ones)
// Slot 3: [108..124) (Decimal point '.')
// Slot 4: [124..160) (Tenths)
const SLOTS_MA: [SlotConfig; 5] = [
    SlotConfig { x: 0, w: 36 },
    SlotConfig { x: 36, w: 36 },
    SlotConfig { x: 72, w: 36 },
    SlotConfig { x: 108, w: 16 },
    SlotConfig { x: 124, w: 36 },
];

// 5 pre-allocated, fixed-width character slots for Amps mode:
// Total width: 36 + 16 + 36 + 36 + 36 = 160 pixels (X: 0..159).
// Slot 0: [0..36) (Ones / '-')
// Slot 1: [36..52) (Decimal point '.')
// Slot 2: [52..88) (Tenths)
// Slot 3: [88..124) (Hundredths)
// Slot 4: [124..160) (Thousandths)
const SLOTS_A: [SlotConfig; 5] = [
    SlotConfig { x: 0, w: 36 },
    SlotConfig { x: 36, w: 16 },
    SlotConfig { x: 52, w: 36 },
    SlotConfig { x: 88, w: 36 },
    SlotConfig { x: 124, w: 36 },
];

/// Formats current into exactly 5 fixed-slot ASCII characters:
/// - mA mode: `{:>3}.{:01}` (e.g. `"  0.0"`, `" 99.8"`, `"100.1"`, `"999.9"`, `"-99.8"`)
/// - A mode: `{:>1}.{:03}` (e.g. `"1.000"`, `"2.450"`, `"3.200"`)
/// Returns `(slot_bytes, unit_str, is_amps)`.
fn fmt_big_digits(c_tenth: i16) -> ([u8; 5], &'static str, bool) {
    let is_rev = c_tenth < 0;
    let abs_c = c_tenth.unsigned_abs() as u32;

    if abs_c >= 10_000 {
        // >= 1.000 A: format as "{:>1}.{:03} A"
        let whole_a = (abs_c / 10_000) as u8;
        let frac_a = (abs_c % 10_000) / 10; // 3 decimal digits (0..999)
        let d0 = if is_rev { b'-' } else { b'0' + whole_a };
        let d1 = b'.';
        let d2 = b'0' + ((frac_a / 100) as u8);
        let d3 = b'0' + (((frac_a / 10) % 10) as u8);
        let d4 = b'0' + ((frac_a % 10) as u8);
        ([d0, d1, d2, d3, d4], " A", true)
    } else {
        // < 1000.0 mA: format as "{:>3}.{:01} mA"
        let whole_ma = (abs_c / 10) as u16;
        let frac_ma = (abs_c % 10) as u8;

        let (d0, d1, d2) = if !is_rev {
            let d0 = if whole_ma >= 100 { b'0' + ((whole_ma / 100) as u8) } else { b' ' };
            let d1 = if whole_ma >= 10 { b'0' + (((whole_ma / 10) % 10) as u8) } else { b' ' };
            let d2 = b'0' + ((whole_ma % 10) as u8);
            (d0, d1, d2)
        } else {
            // Negative current
            if whole_ma >= 100 {
                (b'-', b'0' + (((whole_ma / 10) % 10) as u8), b'0' + ((whole_ma % 10) as u8))
            } else if whole_ma >= 10 {
                (b'-', b'0' + ((whole_ma / 10) as u8), b'0' + ((whole_ma % 10) as u8))
            } else {
                (b' ', b'-', b'0' + (whole_ma as u8))
            }
        };

        let d3 = b'.';
        let d4 = b'0' + frac_ma;
        ([d0, d1, d2, d3, d4], "mA", false)
    }
}

pub struct BigDigitScreen {
    slot_chars: [u8; 5],
    slot_colors: [u16; 5],
    prev_is_amps: bool,
    unit_field: DirtyField<4>,
    volt_field: DirtyField<16>,
    batt_field: DirtyField<16>,
    ema: EmaFilter,
    prev_usb_cfg: Option<bool>,
    prev_sd_status: Option<SdStatus>,
    prev_over_current: Option<bool>,
    prev_ina_present: Option<bool>,
    // Analog Segmented Bar Meter
    prev_lit_segments: u8,
    peak_bar_seg: u8,
    peak_decay_counter: u8,
    prev_is_reverse: bool,
}

impl BigDigitScreen {
    pub const fn new() -> Self {
        Self {
            slot_chars: [0; 5],
            slot_colors: [0; 5],
            prev_is_amps: false,
            unit_field: DirtyField::new(),
            volt_field: DirtyField::new(),
            batt_field: DirtyField::new(),
            ema: EmaFilter::new(),
            prev_usb_cfg: None,
            prev_sd_status: None,
            prev_over_current: None,
            prev_ina_present: None,
            prev_lit_segments: 255,
            peak_bar_seg: 0,
            peak_decay_counter: 0,
            prev_is_reverse: false,
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // 1. Slim Top Status Bar (Y: 0..12)
        lcd.fill_rect(0, 0, 160, 12, COL_BG_TOP);
        lcd.fill_rect(0, 12, 160, 1, COL_DIVIDER);

        // 2. Bottom Segmented Bar Meter Divider (Y: 71)
        lcd.fill_rect(0, 71, 160, 1, COL_DIVIDER);

        // Invalidate all cached field states
        self.slot_chars = [0; 5];
        self.slot_colors = [0; 5];
        self.prev_is_amps = false;
        self.unit_field.invalidate();
        self.volt_field.invalidate();
        self.batt_field.invalidate();
        self.ema.reset();
        self.prev_usb_cfg = None;
        self.prev_sd_status = None;
        self.prev_over_current = None;
        self.prev_ina_present = None;
        self.prev_lit_segments = 255;
        self.peak_bar_seg = 0;
        self.peak_decay_counter = 0;
        self.prev_is_reverse = false;
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

        // --- 2. Sensor Disconnection Handling ---
        if !ina_present {
            if self.prev_ina_present != Some(false) {
                lcd.fill_rect(0, 14, 160, 57, COL_BLACK);
                lcd.fill_rect(0, 72, 160, 8, COL_BLACK);
                lcd.draw_string(32, 38, "DISCONNECTED", &FONT_8X16, COL_RED, COL_BLACK);
                self.slot_chars = [0; 5];
                self.slot_colors = [0; 5];
                self.prev_ina_present = Some(false);
            }
            return;
        }

        if self.prev_ina_present == Some(false) {
            lcd.fill_rect(0, 14, 160, 57, COL_BLACK);
            self.slot_chars = [0; 5];
            self.slot_colors = [0; 5];
        }
        self.prev_ina_present = Some(true);

        // --- 3. Bench-Meter Fixed-Width Big Digits (Y: 15..70) ---
        let current = self.ema.update(reading.current_tenth_ma);

        let curr_col = if over_current {
            COL_AMBER
        } else if reading.is_reverse {
            COL_RED
        } else if current == 0 {
            COL_WHITE
        } else {
            COL_CYAN
        };

        self.prev_over_current = Some(over_current);

        let (chars, unit_str, is_amps) = fmt_big_digits(current);

        // If unit mode transitioned between mA and A, invalidate all slots and clear digit area
        if is_amps != self.prev_is_amps {
            lcd.fill_rect(0, 14, 160, 57, COL_BLACK);
            self.slot_chars = [0; 5];
            self.slot_colors = [0; 5];
            self.prev_is_amps = is_amps;
        }

        let slot_cfgs = if is_amps { &SLOTS_A } else { &SLOTS_MA };

        // Render only slots whose character or color has changed
        for i in 0..5 {
            let ch = chars[i];
            let dirty = ch != self.slot_chars[i] || curr_col != self.slot_colors[i];
            if dirty {
                let cfg = slot_cfgs[i];
                if ch == b' ' {
                    lcd.fill_rect(cfg.x, 15, cfg.w, 56, COL_BLACK);
                } else {
                    let glyph_w = (FONT_28.char_width(ch as char) as u16) * 2;
                    let char_x = cfg.x + (cfg.w.saturating_sub(glyph_w)) / 2;

                    // Clear left padding within slot if any
                    if char_x > cfg.x {
                        lcd.fill_rect(cfg.x, 15, char_x - cfg.x, 56, COL_BLACK);
                    }
                    // Clear right padding within slot if any
                    let right = char_x + glyph_w;
                    if right < cfg.x + cfg.w {
                        lcd.fill_rect(right, 15, (cfg.x + cfg.w) - right, 56, COL_BLACK);
                    }
                    // Draw anti-aliased 56px character
                    lcd.draw_char_scaled(char_x, 15, ch as char, &FONT_28, curr_col, COL_BLACK, 2);
                }
                self.slot_chars[i] = ch;
                self.slot_colors[i] = curr_col;
            }
        }

        // --- 4. Analog-Style Segmented Bar Meter & Unit (Y: 72..78) ---
        // Right unit badge (X: 138, Y: 72 in FONT_5X7)
        if self.unit_field.update(unit_str) {
            lcd.fill_rect(134, 72, 26, 7, COL_BLACK);
            lcd.draw_string(138, 72, unit_str, &FONT_5X7, curr_col, COL_BLACK);
        }

        // 25-segment bargraph (X: 4..128)
        let bar_max: u32 = if is_amps { 32_000 } else { 10_000 };
        let abs_c = reading.abs_current_tenth();
        let lit_count = core::cmp::min(25, (abs_c * 25) / bar_max) as u8;

        // Peak-hold transient marker logic
        let prev_peak_seg = self.peak_bar_seg;
        if lit_count > self.peak_bar_seg {
            self.peak_bar_seg = lit_count;
            self.peak_decay_counter = 20; // 2.0s hold @ 10Hz
        } else if self.peak_decay_counter > 0 {
            self.peak_decay_counter -= 1;
        } else if self.peak_bar_seg > lit_count {
            self.peak_bar_seg -= 1;
        }

        let bar_dirty = lit_count != self.prev_lit_segments
            || self.peak_bar_seg != prev_peak_seg
            || reading.is_reverse != self.prev_is_reverse;

        if bar_dirty {
            for i in 0..25u8 {
                let seg_x = 4 + (i as u16) * 5;
                let seg_col = if reading.is_reverse && i < lit_count {
                    COL_RED
                } else if i < lit_count {
                    if i < 16 {
                        COL_MINT
                    } else if i < 21 {
                        COL_AMBER
                    } else {
                        COL_RED
                    }
                } else if i + 1 == self.peak_bar_seg && self.peak_bar_seg > 0 {
                    COL_WHITE
                } else {
                    rgb565(25, 30, 40) // Unlit dark segment background
                };
                lcd.fill_rect(seg_x, 73, 4, 5, seg_col);
            }
            self.prev_lit_segments = lit_count;
            self.prev_is_reverse = reading.is_reverse;
        }
    }
}
