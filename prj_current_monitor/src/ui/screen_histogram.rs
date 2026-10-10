use core::fmt::Write;
use longan_nano_bsp::{
    lcd::FONT_5X7,
    Lcd,
};
use crate::fmt::{BufferCursor, DirtyField};
use crate::model::HistogramData;
use crate::ui::theme::*;

struct BinConfig {
    lbl: &'static str,
    color: u16,
}

static BINS: [BinConfig; 7] = [
    BinConfig { lbl: "< 1mA", color: COL_MINT },
    BinConfig { lbl: " 1-5m", color: COL_MINT },
    BinConfig { lbl: " 5-20", color: COL_CYAN },
    BinConfig { lbl: "20-50", color: COL_CYAN },
    BinConfig { lbl: "50-150", color: COL_AMBER },
    BinConfig { lbl: "150-500", color: COL_AMBER },
    BinConfig { lbl: ">500m", color: COL_RED },
];

const BAR_X: u16 = 44;
const BAR_W: u16 = 78;
const BAR_H: u16 = 6;

pub struct HistogramScreen {
    pct_fields: [DirtyField<6>; 7],
    count_field: DirtyField<12>,
    prev_pct: [u8; 7],
}

impl HistogramScreen {
    pub const fn new() -> Self {
        Self {
            pct_fields: [
                DirtyField::new(),
                DirtyField::new(),
                DirtyField::new(),
                DirtyField::new(),
                DirtyField::new(),
                DirtyField::new(),
                DirtyField::new(),
            ],
            count_field: DirtyField::new(),
            prev_pct: [255; 7],
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // 1. Header Bar (Y: 0..12)
        lcd.fill_rect(0, 0, 160, 12, COL_BG_TOP);
        lcd.fill_rect(0, 12, 160, 1, COL_DIVIDER);
        lcd.draw_string(4, 2, "CURRENT PROFILE", &FONT_5X7, COL_WHITE, COL_BG_TOP);

        // 2. Draw Bin Labels and Empty Bar Tracks
        for i in 0..7 {
            let y = 15 + (i as u16) * 9;
            lcd.draw_string(2, y, BINS[i].lbl, &FONT_5X7, BINS[i].color, COL_BLACK);
            lcd.fill_rect(BAR_X, y + 1, BAR_W, BAR_H, COL_GRAPH_BG);
            self.pct_fields[i].invalidate();
            self.prev_pct[i] = 255;
        }

        self.count_field.invalidate();
    }

    pub fn update(&mut self, lcd: &mut Lcd, histogram: &HistogramData) {
        // Update sample count in header
        let mut cnt_buf = [0u8; 12];
        let mut cnt_cur = BufferCursor::new(&mut cnt_buf);
        write!(cnt_cur, "N:{:>5}", histogram.total).ok();
        if self.count_field.update(cnt_cur.as_str()) {
            lcd.draw_string(112, 2, cnt_cur.as_str(), &FONT_5X7, COL_TEXT_MUTED, COL_BG_TOP);
        }

        for i in 0..7 {
            let pct = histogram.pct(i);
            let y = 15 + (i as u16) * 9;

            // Redraw bar fill if percentage changed
            if self.prev_pct[i] != pct {
                let fill_w = ((pct as u16) * BAR_W) / 100;
                if fill_w > 0 {
                    lcd.fill_rect(BAR_X, y + 1, fill_w, BAR_H, BINS[i].color);
                }
                if fill_w < BAR_W {
                    lcd.fill_rect(BAR_X + fill_w, y + 1, BAR_W - fill_w, BAR_H, COL_GRAPH_BG);
                }
                self.prev_pct[i] = pct;
            }

            // Percentage text label on the right
            let mut p_buf = [0u8; 6];
            let mut p_cur = BufferCursor::new(&mut p_buf);
            write!(p_cur, "{:>3}%", pct).ok();
            if self.pct_fields[i].update(p_cur.as_str()) {
                lcd.draw_string(126, y, p_cur.as_str(), &FONT_5X7, BINS[i].color, COL_BLACK);
            }
        }
    }
}
