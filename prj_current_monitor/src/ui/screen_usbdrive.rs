use core::fmt::Write;
use longan_nano_bsp::{
    lcd::FONT_5X7,
    Lcd,
    MscStats,
};
use crate::fmt::{BufferCursor, DirtyField};
use crate::ui::theme::*;
use crate::ui::SdStatus;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UsbDriveState {
    Standby,
    Active,
}

impl UsbDriveState {
    pub const fn name(&self) -> &'static str {
        match self {
            UsbDriveState::Standby => "STANDBY",
            UsbDriveState::Active => "ACTIVE",
        }
    }
}

pub struct UsbDriveScreen {
    pub state: UsbDriveState,
    state_field: DirtyField<24>,
    card_field: DirtyField<24>,
    file_field: DirtyField<24>,
    action_field: DirtyField<24>,
    prompt_field: DirtyField<24>,
    badge_field: DirtyField<16>,
}

impl UsbDriveScreen {
    pub const fn new() -> Self {
        Self {
            state: UsbDriveState::Standby,
            state_field: DirtyField::new(),
            card_field: DirtyField::new(),
            file_field: DirtyField::new(),
            action_field: DirtyField::new(),
            prompt_field: DirtyField::new(),
            badge_field: DirtyField::new(),
        }
    }

    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(COL_BLACK);

        // Header bar (Y: 0..12)
        lcd.fill_rect(0, 0, 160, 12, COL_BG_TOP);
        lcd.fill_rect(0, 12, 160, 1, COL_DIVIDER);
        lcd.draw_string(4, 2, "USB DRIVE", &FONT_5X7, COL_WHITE, COL_BG_TOP);

        // Static Row Labels (Y = 16, 28, 40, 52, 64)
        lcd.draw_string(4, 16, "STATE:", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);
        lcd.draw_string(4, 28, "CARD :", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);
        lcd.draw_string(4, 40, "FILE :", &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);

        self.state_field.invalidate();
        self.card_field.invalidate();
        self.file_field.invalidate();
        self.action_field.invalidate();
        self.prompt_field.invalidate();
        self.badge_field.invalidate();
    }

    pub fn update(
        &mut self,
        lcd: &mut Lcd,
        sd_status: SdStatus,
        sector_count: u32,
        current_file: &str,
        msc_stats: Option<&MscStats>,
    ) {
        // 1. Header Badge: [STANDBY] vs [PC MOUNTED]
        let mut b_buf = [0u8; 16];
        let mut b_cur = BufferCursor::new(&mut b_buf);
        let (badge_col, is_active) = match self.state {
            UsbDriveState::Standby => {
                write!(b_cur, "[STANDBY]").ok();
                (COL_AMBER, false)
            }
            UsbDriveState::Active => {
                write!(b_cur, "[PC MOUNTED]").ok();
                (COL_MINT, true)
            }
        };
        let b_str = b_cur.as_str();
        if self.badge_field.update(b_str) {
            lcd.fill_rect(80, 2, 76, 8, COL_BG_TOP);
            lcd.draw_string(84, 2, b_str, &FONT_5X7, badge_col, COL_BG_TOP);
        }

        // 2. Line 1: State
        let mut s_buf = [0u8; 24];
        let mut s_cur = BufferCursor::new(&mut s_buf);
        let s_col = if is_active {
            write!(s_cur, "ACTIVE (MSC)").ok();
            COL_MINT
        } else {
            write!(s_cur, "STANDBY (HID)").ok();
            COL_AMBER
        };
        let s_str = s_cur.as_str();
        if self.state_field.update(s_str) {
            lcd.fill_rect(44, 16, 114, 8, COL_BLACK);
            lcd.draw_string(44, 16, s_str, &FONT_5X7, s_col, COL_BLACK);
        }

        // 3. Line 2: Card Capacity
        let mut c_buf = [0u8; 24];
        let mut c_cur = BufferCursor::new(&mut c_buf);
        let c_col = match sd_status {
            SdStatus::NoCard => {
                write!(c_cur, "NO CARD").ok();
                COL_RED
            }
            SdStatus::WriteError => {
                write!(c_cur, "CARD ERROR").ok();
                COL_RED
            }
            _ => {
                let cap_mb = sector_count / 2048;
                if cap_mb >= 1024 {
                    let gb = cap_mb / 1024;
                    let rem = (cap_mb % 1024) / 102;
                    write!(c_cur, "{}.{} GB (SDHC)", gb, rem).ok();
                } else {
                    write!(c_cur, "{} MB (SDHC)", cap_mb).ok();
                }
                COL_CYAN
            }
        };
        let c_str = c_cur.as_str();
        if self.card_field.update(c_str) {
            lcd.fill_rect(44, 28, 114, 8, COL_BLACK);
            lcd.draw_string(44, 28, c_str, &FONT_5X7, c_col, COL_BLACK);
        }

        // 4. Line 3: Active File Name
        let mut f_buf = [0u8; 24];
        let mut f_cur = BufferCursor::new(&mut f_buf);
        write!(f_cur, "{}", current_file).ok();
        let f_str = f_cur.as_str();
        if self.file_field.update(f_str) {
            lcd.fill_rect(44, 40, 114, 8, COL_BLACK);
            lcd.draw_string(44, 40, f_str, &FONT_5X7, COL_WHITE, COL_BLACK);
        }

        // 5. Line 4: Action / Transfer Stats
        let mut a_buf = [0u8; 24];
        let mut a_cur = BufferCursor::new(&mut a_buf);
        let a_col = if is_active {
            if let Some(stats) = msc_stats {
                write!(a_cur, "RD:{} WR:{}", stats.sectors_read, stats.sectors_written).ok();
            } else {
                write!(a_cur, "HOST ACCESS EXCLUSIVE").ok();
            }
            COL_MINT
        } else {
            write!(a_cur, "DBL-CLICK/HOLD: MOUNT").ok();
            COL_CYAN
        };
        let a_str = a_cur.as_str();
        if self.action_field.update(a_str) {
            lcd.fill_rect(4, 52, 154, 8, COL_BLACK);
            lcd.draw_string(4, 52, a_str, &FONT_5X7, a_col, COL_BLACK);
        }

        // 6. Line 5: Navigation Prompt
        let mut p_buf = [0u8; 24];
        let mut p_cur = BufferCursor::new(&mut p_buf);
        if is_active {
            write!(p_cur, "CLICK PA8: EXIT/RESUME").ok();
        } else {
            write!(p_cur, "CLICK PA8: ADVANCE").ok();
        }
        let p_str = p_cur.as_str();
        if self.prompt_field.update(p_str) {
            lcd.fill_rect(4, 64, 154, 8, COL_BLACK);
            lcd.draw_string(4, 64, p_str, &FONT_5X7, COL_TEXT_MUTED, COL_BLACK);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_usb_drive_state_default_and_names() {
        let screen = UsbDriveScreen::new();
        assert_eq!(screen.state, UsbDriveState::Standby);
        assert_eq!(UsbDriveState::Standby.name(), "STANDBY");
        assert_eq!(UsbDriveState::Active.name(), "ACTIVE");
    }

    #[test]
    fn test_format_card_capacity() {
        // 480 MB card = 983,040 sectors
        let sector_count = 983_040;
        let cap_mb = sector_count / 2048;
        assert_eq!(cap_mb, 480);

        // 16 GB card = 33,554,432 sectors
        let sector_count_16gb = 33_554_432;
        let cap_mb_16gb = sector_count_16gb / 2048;
        assert_eq!(cap_mb_16gb, 16384);
        let gb = cap_mb_16gb / 1024;
        let rem = (cap_mb_16gb % 1024) / 102;
        assert_eq!(gb, 16);
        assert_eq!(rem, 0);
    }

    #[test]
    fn test_dirty_fields_invalidation_and_update() {
        let mut screen = UsbDriveScreen::new();
        assert!(screen.state_field.update("STANDBY (HID)"));
        assert!(!screen.state_field.update("STANDBY (HID)"));
        assert!(screen.state_field.update("ACTIVE (MSC)"));

        screen.state_field.invalidate();
        assert!(screen.state_field.update("ACTIVE (MSC)"));
    }
}
