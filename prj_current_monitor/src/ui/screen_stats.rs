use core::fmt::Write;
use longan_nano_bsp::{
    lcd::FONT_5X7,
    Lcd,
};
use crate::fmt::{fmt_energy_auto, fmt_power, fmt_resistance, fmt_time, BufferCursor, DirtyField};
use crate::model::{Accumulators, InaReading, SessionStats};
use crate::ui::theme::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SdStatus {
    NoCard,
    Ready,
    Logging,
    WriteError,
}

pub struct StatsScreen {
    time_field: DirtyField<12>,
    load_field: DirtyField<12>,
    vrng_field: DirtyField<24>,
    irng_field: DirtyField<28>,
    pmax_field: DirtyField<16>,
    iavg_field: DirtyField<16>,
    e_field: DirtyField<16>,
    bat_field: DirtyField<16>,
    logs_field: DirtyField<12>,
    prev_usb_cfg: Option<bool>,
    prev_sd_status: Option<SdStatus>,
}

impl StatsScreen {
    pub const fn new() -> Self {
        Self {
            time_field: DirtyField::new(),
            load_field: DirtyField::new(),
            vrng_field: DirtyField::new(),
            irng_field: DirtyField::new(),
            pmax_field: DirtyField::new(),
            iavg_field: DirtyField::new(),
            e_field: DirtyField::new(),
            bat_field: DirtyField::new(),
            logs_field: DirtyField::new(),
            prev_usb_cfg: None,
            prev_sd_status: None,
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // Header Bar (Y: 0..12)
        lcd.fill_rect(0, 0, 160, 12, COL_BG_TOP);
        lcd.fill_rect(0, 12, 160, 1, COL_DIVIDER);
        lcd.draw_string(4, 2, "SESSION STATS", &FONT_5X7, COL_WHITE, COL_BG_TOP);

        // Row Labels (Y = 16, 29, 42, 55, 68)
        lcd.draw_string(4, 16, "TIME:", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);
        lcd.draw_string(88, 16, "LOAD:", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);

        lcd.draw_string(4, 29, "V-RNG:", &FONT_5X7, COL_MINT, COL_BLACK);
        lcd.draw_string(4, 42, "I-RNG:", &FONT_5X7, COL_CYAN, COL_BLACK);

        lcd.draw_string(4, 55, "P-MAX:", &FONT_5X7, COL_AMBER, COL_BLACK);
        lcd.draw_string(88, 55, "I-AVG:", &FONT_5X7, COL_CYAN, COL_BLACK);

        lcd.draw_string(4, 68, "E:", &FONT_5X7, COL_AMBER, COL_BLACK);
        lcd.draw_string(88, 68, "BAT:", &FONT_5X7, COL_MINT, COL_BLACK);

        self.time_field.invalidate();
        self.load_field.invalidate();
        self.vrng_field.invalidate();
        self.irng_field.invalidate();
        self.pmax_field.invalidate();
        self.iavg_field.invalidate();
        self.e_field.invalidate();
        self.bat_field.invalidate();
        self.logs_field.invalidate();
        self.prev_usb_cfg = None;
        self.prev_sd_status = None;
    }

    pub fn update(
        &mut self,
        lcd: &mut Lcd,
        reading: &InaReading,
        stats: &SessionStats,
        accum: &Accumulators,
        sd_status: SdStatus,
        sd_log_count: u32,
        now_ms: u32,
        usb_configured: bool,
    ) {
        // --- 1. Header Bar: Log Counter & Status Icons ---
        let mut lg_buf = [0u8; 12];
        let mut lg_cur = BufferCursor::new(&mut lg_buf);
        write!(lg_cur, "#{:04}", sd_log_count % 10000).ok();
        if self.logs_field.update(lg_cur.as_str()) {
            lcd.draw_string(96, 2, lg_cur.as_str(), &FONT_5X7, COL_AMBER, COL_BG_TOP);
        }

        if self.prev_usb_cfg != Some(usb_configured) {
            let usb_col = if usb_configured { COL_CYAN } else { rgb565(50, 60, 75) };
            lcd.draw_bitmap_8x8(136, 2, &ICON_USB, usb_col, COL_BG_TOP);
            self.prev_usb_cfg = Some(usb_configured);
        }

        if self.prev_sd_status != Some(sd_status) {
            match sd_status {
                SdStatus::Logging => {
                    lcd.draw_bitmap_8x8(148, 2, &ICON_REC, COL_RED, COL_BG_TOP);
                }
                SdStatus::Ready => {
                    lcd.draw_bitmap_8x8(148, 2, &ICON_SD, COL_MINT, COL_BG_TOP);
                }
                SdStatus::WriteError => {
                    lcd.draw_bitmap_8x8(148, 2, &ICON_SD, COL_AMBER, COL_BG_TOP);
                }
                SdStatus::NoCard => {
                    lcd.draw_bitmap_8x8(148, 2, &ICON_SD, rgb565(50, 60, 75), COL_BG_TOP);
                }
            }
            self.prev_sd_status = Some(sd_status);
        }

        // --- 2. Row 1: Session Elapsed Time & Load Resistance ---
        let elapsed_sec = (now_ms.wrapping_sub(stats.start_time_ms)) / 1000;
        let mut t_buf = [0u8; 12];
        let mut t_cur = BufferCursor::new(&mut t_buf);
        fmt_time(&mut t_cur, elapsed_sec);
        if self.time_field.update(t_cur.as_str()) {
            lcd.draw_string(36, 16, t_cur.as_str(), &FONT_5X7, COL_WHITE, COL_BLACK);
        }

        let r_load = stats.load_resistance_tenth_ohms(reading.voltage_mv, reading.current_tenth_ma);
        let mut r_buf = [0u8; 12];
        let mut r_cur = BufferCursor::new(&mut r_buf);
        fmt_resistance(&mut r_cur, r_load);
        if self.load_field.update(r_cur.as_str()) {
            lcd.draw_string(120, 16, r_cur.as_str(), &FONT_5X7, COL_MINT, COL_BLACK);
        }

        // --- 3. Row 2: Voltage Min..Max Range ---
        let mut vr_buf = [0u8; 24];
        let mut vr_cur = BufferCursor::new(&mut vr_buf);
        if stats.has_samples {
            write!(
                vr_cur,
                "{:>2}.{:02}V - {:>2}.{:02}V",
                stats.v_min / 1000,
                (stats.v_min % 1000) / 10,
                stats.v_max / 1000,
                (stats.v_max % 1000) / 10
            )
            .ok();
        } else {
            write!(vr_cur, " --.--V - --.--V").ok();
        }
        if self.vrng_field.update(vr_cur.as_str()) {
            lcd.draw_string(42, 29, vr_cur.as_str(), &FONT_5X7, COL_MINT, COL_BLACK);
        }

        // --- 4. Row 3: Current Min..Max Range ---
        let mut ir_buf = [0u8; 28];
        let mut ir_cur = BufferCursor::new(&mut ir_buf);
        if stats.has_samples {
            let c_min_abs = stats.c_min.unsigned_abs();
            let c_max_abs = stats.c_max.unsigned_abs();
            write!(
                ir_cur,
                "{}{}.{} - {}{}.{}mA",
                if stats.c_min < 0 { "-" } else { "" },
                c_min_abs / 10, c_min_abs % 10,
                if stats.c_max < 0 { "-" } else { "" },
                c_max_abs / 10, c_max_abs % 10
            )
            .ok();
        } else {
            write!(ir_cur, " ---.- - ---.- mA").ok();
        }
        if self.irng_field.update(ir_cur.as_str()) {
            lcd.draw_string(42, 42, ir_cur.as_str(), &FONT_5X7, COL_CYAN, COL_BLACK);
        }

        // --- 5. Row 4: Peak Power & Average Current ---
        let mut pm_buf = [0u8; 16];
        let mut pm_cur = BufferCursor::new(&mut pm_buf);
        if stats.has_samples {
            fmt_power(&mut pm_cur, stats.p_max);
        } else {
            write!(pm_cur, " ---.- mW").ok();
        }
        if self.pmax_field.update(pm_cur.as_str()) {
            lcd.draw_string(42, 55, pm_cur.as_str(), &FONT_5X7, COL_AMBER, COL_BLACK);
        }

        let avg_c = stats.avg_current_tenth_ma();
        let mut ia_buf = [0u8; 16];
        let mut ia_cur = BufferCursor::new(&mut ia_buf);
        if stats.has_samples {
            write!(ia_cur, "{:>4}.{}mA", avg_c / 10, avg_c % 10).ok();
        } else {
            write!(ia_cur, " ---.-mA").ok();
        }
        if self.iavg_field.update(ia_cur.as_str()) {
            lcd.draw_string(122, 55, ia_cur.as_str(), &FONT_5X7, COL_CYAN, COL_BLACK);
        }

        // --- 6. Row 5: Total Energy & Battery Runtime Estimator ---
        let mut es_buf = [0u8; 16];
        let mut es_cur = BufferCursor::new(&mut es_buf);
        fmt_energy_auto(&mut es_cur, accum);
        if self.e_field.update(es_cur.as_str()) {
            let s = if es_cur.as_str().starts_with("E:") {
                &es_cur.as_str()[2..]
            } else {
                es_cur.as_str()
            };
            lcd.draw_string(18, 68, s, &FONT_5X7, COL_AMBER, COL_BLACK);
        }

        let mut bat_buf = [0u8; 16];
        let mut bat_cur = BufferCursor::new(&mut bat_buf);
        if avg_c > 0 {
            // Projected runtime for nominal 500 mAh battery (5000 in 0.1 mA-hours)
            let total_mins = (5000u64 * 60) / (avg_c as u64);
            if total_mins < 60 {
                write!(bat_cur, "~{}m", total_mins).ok();
            } else if total_mins < 60 * 1000 {
                let hours = total_mins / 60;
                let frac = (total_mins % 60) / 6;
                write!(bat_cur, "~{}.{}h", hours, frac).ok();
            } else {
                write!(bat_cur, ">999h").ok();
            }
        } else {
            write!(bat_cur, " --- ").ok();
        }
        if self.bat_field.update(bat_cur.as_str()) {
            lcd.draw_string(114, 68, bat_cur.as_str(), &FONT_5X7, COL_MINT, COL_BLACK);
        }
    }
}
