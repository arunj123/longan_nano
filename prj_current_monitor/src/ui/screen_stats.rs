use core::fmt::Write;
use longan_nano_bsp::{
    lcd::FONT_5X7,
    Lcd,
};
use crate::fmt::{fmt_charge_auto, fmt_energy_auto, fmt_power, fmt_resistance, fmt_time, BufferCursor, DirtyField};
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
    logs_field: DirtyField<12>,
    e_field: DirtyField<16>,
    q_field: DirtyField<16>,
    sd_badge_field: DirtyField<12>,
}

impl StatsScreen {
    pub const fn new() -> Self {
        Self {
            time_field: DirtyField::new(),
            load_field: DirtyField::new(),
            vrng_field: DirtyField::new(),
            irng_field: DirtyField::new(),
            pmax_field: DirtyField::new(),
            logs_field: DirtyField::new(),
            e_field: DirtyField::new(),
            q_field: DirtyField::new(),
            sd_badge_field: DirtyField::new(),
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // Header Bar (Y: 0..12)
        lcd.fill_rect(0, 0, 160, 12, COL_BG_TOP);
        lcd.fill_rect(0, 12, 160, 1, COL_DIVIDER);
        lcd.draw_string(4, 2, "SYSTEM STATS", &FONT_5X7, COL_WHITE, COL_BG_TOP);

        // Row Labels (Y = 16, 29, 42, 55, 68)
        lcd.draw_string(4, 16, "TIME:", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);
        lcd.draw_string(88, 16, "LOAD:", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);

        lcd.draw_string(4, 29, "V-RNG:", &FONT_5X7, COL_MINT, COL_BLACK);
        lcd.draw_string(4, 42, "I-RNG:", &FONT_5X7, COL_CYAN, COL_BLACK);

        lcd.draw_string(4, 55, "P-MAX:", &FONT_5X7, COL_AMBER, COL_BLACK);
        lcd.draw_string(88, 55, "LOGS:", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);

        lcd.draw_string(4, 68, "E:", &FONT_5X7, COL_AMBER, COL_BLACK);
        lcd.draw_string(88, 68, "Q:", &FONT_5X7, COL_MINT, COL_BLACK);

        self.time_field.invalidate();
        self.load_field.invalidate();
        self.vrng_field.invalidate();
        self.irng_field.invalidate();
        self.pmax_field.invalidate();
        self.logs_field.invalidate();
        self.e_field.invalidate();
        self.q_field.invalidate();
        self.sd_badge_field.invalidate();
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
    ) {
        // SD Status Badge on Top Header (X: 110..158)
        let (sd_text, sd_col, sd_bg) = match sd_status {
            SdStatus::Logging => ("[REC] ", COL_MINT, 0x0162),
            SdStatus::Ready => ("[IDLE]", COL_AMBER, 0x0162),
            SdStatus::NoCard => ("[NO-SD]", COL_TEXT_MUTED, 0x2104),
            SdStatus::WriteError => ("[SD-ERR]", COL_RED, rgb565(80, 0, 0)),
        };
        if self.sd_badge_field.update(sd_text) {
            lcd.fill_rect(110, 1, 48, 10, sd_bg);
            lcd.draw_string(112, 2, sd_text, &FONT_5X7, sd_col, sd_bg);
        }

        // Row 1: Session Elapsed Time & Load Resistance
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

        // Row 2: Voltage Min..Max Range
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

        // Row 3: Current Min..Max Range
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

        // Row 4: Peak Power & Total SD Logs Count
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

        let mut lg_buf = [0u8; 12];
        let mut lg_cur = BufferCursor::new(&mut lg_buf);
        write!(lg_cur, "{:>6}", sd_log_count).ok();
        if self.logs_field.update(lg_cur.as_str()) {
            lcd.draw_string(120, 55, lg_cur.as_str(), &FONT_5X7, COL_AMBER, COL_BLACK);
        }

        // Row 5: Total Energy & Charge
        let mut es_buf = [0u8; 16];
        let mut es_cur = BufferCursor::new(&mut es_buf);
        fmt_energy_auto(&mut es_cur, accum);
        if self.e_field.update(es_cur.as_str()) {
            // Strip the leading "E:" if already on screen
            let s = if es_cur.as_str().starts_with("E:") {
                &es_cur.as_str()[2..]
            } else {
                es_cur.as_str()
            };
            lcd.draw_string(18, 68, s, &FONT_5X7, COL_AMBER, COL_BLACK);
        }

        let mut qs_buf = [0u8; 16];
        let mut qs_cur = BufferCursor::new(&mut qs_buf);
        fmt_charge_auto(&mut qs_cur, accum);
        if self.q_field.update(qs_cur.as_str()) {
            let s = if qs_cur.as_str().starts_with("Q:") {
                &qs_cur.as_str()[2..]
            } else {
                qs_cur.as_str()
            };
            lcd.draw_string(102, 68, s, &FONT_5X7, COL_MINT, COL_BLACK);
        }
    }
}
