#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BbbState {
    Idle,
    DataOut,
    DataIn,
    SendData,
    SendZlp,
    StatusPending,
    SendCsw,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CswStatus {
    CmdPassed = 0x00,
    CmdFailed = 0x01,
    PhaseError = 0x02,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct BbbCbw {
    pub d_cbw_signature: u32,
    pub d_cbw_tag: u32,
    pub d_cbw_data_transfer_length: u32,
    pub bm_cbw_flags: u8,
    pub b_cbw_lun: u8,
    pub b_cbw_cb_length: u8,
    pub cbwcb: [u8; 16],
}

impl BbbCbw {
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 31 {
            return None;
        }
        let sig = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let tag = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let len = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let flags = buf[12];
        let lun = buf[13];
        let cb_len = buf[14];
        let mut cbwcb = [0u8; 16];
        cbwcb.copy_from_slice(&buf[15..31]);
        Some(Self {
            d_cbw_signature: sig,
            d_cbw_tag: tag,
            d_cbw_data_transfer_length: len,
            bm_cbw_flags: flags,
            b_cbw_lun: lun,
            b_cbw_cb_length: cb_len,
            cbwcb,
        })
    }

    #[inline(always)]
    pub fn is_data_in(&self) -> bool {
        (self.bm_cbw_flags & 0x80) != 0
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.d_cbw_signature == 0x43425355
            && (self.bm_cbw_flags & 0x7F) == 0
            && (self.b_cbw_lun & 0xF0) == 0
            && self.b_cbw_cb_length >= 1
            && self.b_cbw_cb_length <= 16
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BbbCsw {
    pub d_csw_signature: u32,
    pub d_csw_tag: u32,
    pub d_csw_data_residue: u32,
    pub b_csw_status: u8,
}

impl BbbCsw {
    pub fn write_to_slice(&self, buf: &mut [u8]) {
        buf[0..4].copy_from_slice(&self.d_csw_signature.to_le_bytes());
        buf[4..8].copy_from_slice(&self.d_csw_tag.to_le_bytes());
        buf[8..12].copy_from_slice(&self.d_csw_data_residue.to_le_bytes());
        buf[12] = self.b_csw_status;
    }
}

/// USB BOT §6.7 The 13 Transfer Cases
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BotCase {
    Case1_Hn_Dn,
    Case2_Hn_Di,
    Case3_Hn_Do,
    Case4_Hi_Dn,
    Case5_Hi_Di,
    Case6_Hi_Di,
    Case7_Hi_Di,
    Case8_Hi_Do,
    Case9_Ho_Dn,
    Case10_Ho_Di,
    Case11_Ho_Do,
    Case12_Ho_Do,
    Case13_Ho_Do,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BotValidation {
    pub case: BotCase,
    pub to_transfer: u32,
    pub residue: u32,
    pub stall_in: bool,
    pub stall_out: bool,
    pub status: CswStatus,
}

/// Validates host direction and transfer length against device command requirements
/// strictly per USB Mass Storage Bulk-Only Transport §6.7 (The 13 Transfer Cases).
pub fn validate_bot_case(
    host_len: u32,
    host_in: bool,
    dev_len: u32,
    dev_in: bool,
) -> BotValidation {
    if host_len == 0 {
        // Hn
        if dev_len == 0 {
            // Case 1: Hn == Dn
            BotValidation {
                case: BotCase::Case1_Hn_Dn,
                to_transfer: 0,
                residue: 0,
                stall_in: false,
                stall_out: false,
                status: CswStatus::CmdPassed,
            }
        } else if dev_in {
            // Case 2: Hn < Di
            BotValidation {
                case: BotCase::Case2_Hn_Di,
                to_transfer: 0,
                residue: 0,
                stall_in: false,
                stall_out: false,
                status: CswStatus::PhaseError,
            }
        } else {
            // Case 3: Hn < Do
            BotValidation {
                case: BotCase::Case3_Hn_Do,
                to_transfer: 0,
                residue: 0,
                stall_in: false,
                stall_out: false,
                status: CswStatus::PhaseError,
            }
        }
    } else if host_in {
        // Hi
        if dev_len == 0 {
            // Case 4: Hi > Dn
            BotValidation {
                case: BotCase::Case4_Hi_Dn,
                to_transfer: 0,
                residue: host_len,
                stall_in: true,
                stall_out: false,
                status: CswStatus::CmdPassed,
            }
        } else if !dev_in {
            // Case 8: Hi <> Do (Direction mismatch)
            BotValidation {
                case: BotCase::Case8_Hi_Do,
                to_transfer: 0,
                residue: host_len,
                stall_in: true,
                stall_out: false,
                status: CswStatus::PhaseError,
            }
        } else if host_len > dev_len {
            // Case 5: Hi > Di
            BotValidation {
                case: BotCase::Case5_Hi_Di,
                to_transfer: dev_len,
                residue: host_len - dev_len,
                stall_in: true,
                stall_out: false,
                status: CswStatus::CmdPassed,
            }
        } else if host_len == dev_len {
            // Case 6: Hi == Di
            BotValidation {
                case: BotCase::Case6_Hi_Di,
                to_transfer: dev_len,
                residue: 0,
                stall_in: false,
                stall_out: false,
                status: CswStatus::CmdPassed,
            }
        } else {
            // Case 7: Hi < Di
            BotValidation {
                case: BotCase::Case7_Hi_Di,
                to_transfer: host_len,
                residue: 0,
                stall_in: false,
                stall_out: false,
                status: CswStatus::PhaseError,
            }
        }
    } else {
        // Ho
        if dev_len == 0 {
            // Case 9: Ho > Dn
            BotValidation {
                case: BotCase::Case9_Ho_Dn,
                to_transfer: 0,
                residue: host_len,
                stall_in: false,
                stall_out: true,
                status: CswStatus::CmdPassed,
            }
        } else if dev_in {
            // Case 10: Ho <> Di (Direction mismatch)
            BotValidation {
                case: BotCase::Case10_Ho_Di,
                to_transfer: 0,
                residue: host_len,
                stall_in: false,
                stall_out: true,
                status: CswStatus::PhaseError,
            }
        } else if host_len > dev_len {
            // Case 11: Ho > Do
            BotValidation {
                case: BotCase::Case11_Ho_Do,
                to_transfer: dev_len,
                residue: host_len - dev_len,
                stall_in: false,
                stall_out: true,
                status: CswStatus::CmdPassed,
            }
        } else if host_len == dev_len {
            // Case 12: Ho == Do
            BotValidation {
                case: BotCase::Case12_Ho_Do,
                to_transfer: dev_len,
                residue: 0,
                stall_in: false,
                stall_out: false,
                status: CswStatus::CmdPassed,
            }
        } else {
            // Case 13: Ho < Do
            BotValidation {
                case: BotCase::Case13_Ho_Do,
                to_transfer: host_len,
                residue: 0,
                stall_in: false,
                stall_out: true,
                status: CswStatus::PhaseError,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_cbw_csw_serialization() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&0x43425355u32.to_le_bytes()); // "USBC"
        raw[4..8].copy_from_slice(&0x12345678u32.to_le_bytes()); // Tag
        raw[8..12].copy_from_slice(&512u32.to_le_bytes());       // 512 bytes
        raw[12] = 0x80;                                         // Direction IN
        raw[13] = 0;                                            // LUN 0
        raw[14] = 10;                                           // CDB length 10
        raw[15] = 0x28;                                         // READ_10

        let cbw = BbbCbw::from_bytes(&raw).expect("valid CBW");
        assert_eq!(cbw.d_cbw_signature, 0x43425355);
        assert_eq!(cbw.d_cbw_tag, 0x12345678);
        assert_eq!(cbw.d_cbw_data_transfer_length, 512);
        assert!(cbw.is_data_in());
        assert_eq!(cbw.cbwcb[0], 0x28);

        let csw = BbbCsw {
            d_csw_signature: 0x53425355,
            d_csw_tag: cbw.d_cbw_tag,
            d_csw_data_residue: 0,
            b_csw_status: CswStatus::CmdPassed as u8,
        };
        let mut csw_bytes = [0u8; 13];
        csw.write_to_slice(&mut csw_bytes);
        assert_eq!(&csw_bytes[0..4], &0x53425355u32.to_le_bytes());
        assert_eq!(&csw_bytes[4..8], &0x12345678u32.to_le_bytes());
        assert_eq!(&csw_bytes[8..12], &[0, 0, 0, 0]);
        assert_eq!(csw_bytes[12], 0x00);
    }

    #[test]
    fn test_cbw_validity_check() {
        let mut raw = [0u8; 31];
        raw[0..4].copy_from_slice(&0x43425355u32.to_le_bytes()); // "USBC"
        raw[4..8].copy_from_slice(&1u32.to_le_bytes());
        raw[8..12].copy_from_slice(&512u32.to_le_bytes());
        raw[12] = 0x80; // IN
        raw[13] = 0;    // LUN 0
        raw[14] = 6;    // CDB len 6
        raw[15] = 0x00; // TEST_UNIT_READY

        let cbw = BbbCbw::from_bytes(&raw).unwrap();
        assert!(cbw.is_valid());

        // Invalid signature
        let mut bad_sig = raw;
        bad_sig[0] = 0xFF;
        assert!(!BbbCbw::from_bytes(&bad_sig).unwrap().is_valid());

        // Reserved flags set
        let mut bad_flags = raw;
        bad_flags[12] = 0x81;
        assert!(!BbbCbw::from_bytes(&bad_flags).unwrap().is_valid());

        // LUN out of range (> 15)
        let mut bad_lun = raw;
        bad_lun[13] = 0x10;
        assert!(!BbbCbw::from_bytes(&bad_lun).unwrap().is_valid());

        // Invalid CDB length (0 or > 16)
        let mut bad_len0 = raw;
        bad_len0[14] = 0;
        assert!(!BbbCbw::from_bytes(&bad_len0).unwrap().is_valid());

        let mut bad_len17 = raw;
        bad_len17[14] = 17;
        assert!(!BbbCbw::from_bytes(&bad_len17).unwrap().is_valid());
    }

    #[test]
    fn test_case1_hn_dn() {
        let v = validate_bot_case(0, false, 0, false);
        assert_eq!(v.case, BotCase::Case1_Hn_Dn);
        assert_eq!(v.status, CswStatus::CmdPassed);
        assert_eq!(v.residue, 0);
        assert!(!v.stall_in && !v.stall_out);
    }

    #[test]
    fn test_case2_hn_di() {
        let v = validate_bot_case(0, false, 512, true);
        assert_eq!(v.case, BotCase::Case2_Hn_Di);
        assert_eq!(v.status, CswStatus::PhaseError);
    }

    #[test]
    fn test_case3_hn_do() {
        let v = validate_bot_case(0, false, 512, false);
        assert_eq!(v.case, BotCase::Case3_Hn_Do);
        assert_eq!(v.status, CswStatus::PhaseError);
    }

    #[test]
    fn test_case4_hi_dn() {
        let v = validate_bot_case(512, true, 0, false);
        assert_eq!(v.case, BotCase::Case4_Hi_Dn);
        assert!(v.stall_in);
        assert_eq!(v.residue, 512);
    }

    #[test]
    fn test_case5_hi_di() {
        let v = validate_bot_case(512, true, 36, true);
        assert_eq!(v.case, BotCase::Case5_Hi_Di);
        assert_eq!(v.to_transfer, 36);
        assert_eq!(v.residue, 512 - 36);
        assert!(v.stall_in);
    }

    #[test]
    fn test_case6_hi_di() {
        let v = validate_bot_case(4096, true, 4096, true);
        assert_eq!(v.case, BotCase::Case6_Hi_Di);
        assert_eq!(v.to_transfer, 4096);
        assert_eq!(v.residue, 0);
        assert!(!v.stall_in);
        assert_eq!(v.status, CswStatus::CmdPassed);
    }

    #[test]
    fn test_case7_hi_di() {
        let v = validate_bot_case(256, true, 512, true);
        assert_eq!(v.case, BotCase::Case7_Hi_Di);
        assert_eq!(v.status, CswStatus::PhaseError);
    }

    #[test]
    fn test_case8_hi_do() {
        let v = validate_bot_case(512, true, 512, false);
        assert_eq!(v.case, BotCase::Case8_Hi_Do);
        assert_eq!(v.status, CswStatus::PhaseError);
        assert!(v.stall_in);
    }

    #[test]
    fn test_case9_ho_dn() {
        let v = validate_bot_case(512, false, 0, false);
        assert_eq!(v.case, BotCase::Case9_Ho_Dn);
        assert!(v.stall_out);
        assert_eq!(v.residue, 512);
    }

    #[test]
    fn test_case10_ho_di() {
        let v = validate_bot_case(512, false, 512, true);
        assert_eq!(v.case, BotCase::Case10_Ho_Di);
        assert_eq!(v.status, CswStatus::PhaseError);
        assert!(v.stall_out);
    }

    #[test]
    fn test_case11_ho_do() {
        let v = validate_bot_case(1024, false, 512, false);
        assert_eq!(v.case, BotCase::Case11_Ho_Do);
        assert_eq!(v.to_transfer, 512);
        assert_eq!(v.residue, 512);
        assert!(v.stall_out);
    }

    #[test]
    fn test_case12_ho_do() {
        let v = validate_bot_case(512, false, 512, false);
        assert_eq!(v.case, BotCase::Case12_Ho_Do);
        assert_eq!(v.to_transfer, 512);
        assert_eq!(v.residue, 0);
        assert!(!v.stall_out);
        assert_eq!(v.status, CswStatus::CmdPassed);
    }

    #[test]
    fn test_case13_ho_do() {
        let v = validate_bot_case(256, false, 512, false);
        assert_eq!(v.case, BotCase::Case13_Ho_Do);
        assert_eq!(v.status, CswStatus::PhaseError);
        assert!(v.stall_out);
    }
}
