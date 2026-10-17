pub mod cdc_acm;
pub mod composite;
pub mod hid;
pub mod msc;

pub use cdc_acm::UsbCdcAcm;
pub use composite::UsbComposite;
pub use hid::UsbHid;
pub use msc::{MscBlockDevice, MscStats, SectorBuf, UsbMsc, MSC_CONFIG_DESC};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ControlState {
    Idle,
    DataIn,
    LastDataIn,
    StatusIn,
    StatusOut,
    DataOut,
    Stall,
}

#[derive(Copy, Clone, Default)]
pub struct SetupPacket {
    pub bm_request_type: u8,
    pub b_request: u8,
    pub w_value: u16,
    pub w_index: u16,
    pub w_length: u16,
}

pub const STR_LANG_ID: [u8; 4] = [4, 0x03, 0x09, 0x04]; // English (0x0409)

pub const STR_MANUFACTURER: [u8; 24] = [
    24, 0x03,
    b'L', 0, b'o', 0, b'n', 0, b'g', 0, b'a', 0, b'n', 0,
    b' ', 0, b'N', 0, b'a', 0, b'n', 0, b'o', 0,
];

pub const STR_SERIAL: [u8; 26] = [
    26, 0x03,
    b'3', 0, b'9', 0, b'4', 0, b'1', 0, b'3', 0, b'1', 0,
    b'1', 0, b'9', 0, b'0', 0, b'1', 0, b'0', 0, b'3', 0,
];
