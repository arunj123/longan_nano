#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SenseKey {
    NoSense = 0x00,
    RecoveredError = 0x01,
    NotReady = 0x02,
    MediumError = 0x03,
    HardwareError = 0x04,
    IllegalRequest = 0x05,
    UnitAttention = 0x06,
    DataProtect = 0x07,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Asc {
    NoAsc = 0x00,
    UnrecoveredReadError = 0x11,
    WriteFault = 0x03,
    InvalidCdb = 0x20,
    AddressOutOfRange = 0x21,
    InvalidFieldInCommand = 0x24,
    WriteProtected = 0x27,
    NotReadyToReadyTransition = 0x28,
    MediumNotPresent = 0x3A,
}

pub const MSC_INQUIRY_DATA: [u8; 36] = [
    0x00, // Direct access block device
    0x80, // Removable medium
    0x02, // ANSI SCSI-2
    0x02, // Response data format
    31,   // Additional length (36 - 5)
    0x00, 0x00, 0x00, // Reserved
    b'L', b'o', b'n', b'g', b'a', b'n', b' ', b' ', // Vendor: 8 bytes
    b'L', b'o', b'n', b'g', b'a', b'n', b' ', b'N', b'a', b'n', b'o', b' ', b'S', b'D', b' ',
    b' ', // Product: 16 bytes
    b'1', b'.', b'0', b'0', // Revision: 4 bytes
];

pub mod scsi_cmd {
    pub const TEST_UNIT_READY: u8 = 0x00;
    pub const REQUEST_SENSE: u8 = 0x03;
    pub const READ_6: u8 = 0x08;
    pub const WRITE_6: u8 = 0x0A;
    pub const INQUIRY: u8 = 0x12;
    pub const MODE_SELECT_6: u8 = 0x15;
    pub const MODE_SENSE_6: u8 = 0x1A;
    pub const START_STOP_UNIT: u8 = 0x1B;
    pub const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
    pub const READ_FORMAT_CAPACITIES: u8 = 0x23;
    pub const READ_CAPACITY_10: u8 = 0x25;
    pub const READ_10: u8 = 0x28;
    pub const WRITE_10: u8 = 0x2A;
    pub const SYNCHRONIZE_CACHE_10: u8 = 0x35;
    pub const MODE_SENSE_10: u8 = 0x5A;
    pub const SERVICE_ACTION_IN_16: u8 = 0x9E;
    pub const SECURITY_PROTOCOL_IN: u8 = 0xA2;
}
