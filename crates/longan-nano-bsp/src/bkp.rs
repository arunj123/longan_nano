//! Authoritative registry of GD32VF103 Backup Domain Data Registers (`BKP_DATA0..BKP_DATA9`).
//!
//! Centralizing all register indices in a single `#[repr(usize)]` enum prevents cross-crate
//! register collisions when new persistent settings are introduced.

/// Authoritative registry of all GD32VF103 Backup Domain Data Registers (`BKP_DATA0..BKP_DATA9`).
#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BkpSlot {
    RtcInitMagic = 0,     // 0xA5A5 when RTC IRC40K prescaler is initialized
    MonitorConfig = 1,    // [15..8] screen_mode, [7..0] battery_profile
    TareOffset = 2,       // tare_offset_tenth (i16)
    CurrentLimit = 3,     // current_limit_ma (u16)
    MonitorMagic = 4,     // BKP_MAGIC_KEY (0x5A5A)
    LcdRotation = 5,      // 0x5200 (Normal) or 0x5201 (Rotated180)
    MscWriteProtect = 6,  // 0x5700 (RW) or 0x5701 (RO)
    Reserved7 = 7,
    Reserved8 = 8,
    Reserved9 = 9,
}

impl BkpSlot {
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bkp_slot_indices_and_uniqueness() {
        assert_eq!(BkpSlot::RtcInitMagic.index(), 0);
        assert_eq!(BkpSlot::MonitorConfig.index(), 1);
        assert_eq!(BkpSlot::TareOffset.index(), 2);
        assert_eq!(BkpSlot::CurrentLimit.index(), 3);
        assert_eq!(BkpSlot::MonitorMagic.index(), 4);
        assert_eq!(BkpSlot::LcdRotation.index(), 5);
        assert_eq!(BkpSlot::MscWriteProtect.index(), 6);
        assert_eq!(BkpSlot::Reserved7.index(), 7);
        assert_eq!(BkpSlot::Reserved8.index(), 8);
        assert_eq!(BkpSlot::Reserved9.index(), 9);

        let slots = [
            BkpSlot::RtcInitMagic,
            BkpSlot::MonitorConfig,
            BkpSlot::TareOffset,
            BkpSlot::CurrentLimit,
            BkpSlot::MonitorMagic,
            BkpSlot::LcdRotation,
            BkpSlot::MscWriteProtect,
            BkpSlot::Reserved7,
            BkpSlot::Reserved8,
            BkpSlot::Reserved9,
        ];
        for i in 0..10 {
            assert_eq!(slots[i] as usize, i);
        }
    }
}
