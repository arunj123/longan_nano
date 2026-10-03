use crate::VolatileCell;

pub const SPI0_BASE: usize = 0x4001_3000;
pub const SPI1_BASE: usize = 0x4000_3800;
pub const SPI2_BASE: usize = 0x4000_3C00;

#[repr(C)]
pub struct SpiRegisterBlock {
    pub ctl0: VolatileCell<u32>,   // 0x00: Control register 0
    pub ctl1: VolatileCell<u32>,   // 0x04: Control register 1
    pub stat: VolatileCell<u32>,   // 0x08: Status register
    pub data: VolatileCell<u32>,   // 0x0C: Data register
    pub crpoly: VolatileCell<u32>, // 0x10: CRC polynomial register
    pub rxcrc: VolatileCell<u32>,  // 0x14: Rx CRC register
    pub txcrc: VolatileCell<u32>,  // 0x18: Tx CRC register
    pub i2sctl: VolatileCell<u32>, // 0x1C: I2S control register
    pub i2spsc: VolatileCell<u32>, // 0x20: I2S prescaler register
}

pub mod ctl0 {
    pub const CKPH: u32    = 1 << 0;  // Clock phase
    pub const CKPL: u32    = 1 << 1;  // Clock polarity
    pub const MSTMOD: u32  = 1 << 2;  // Master mode
    pub const PSC_DIV2: u32   = 0 << 3;
    pub const PSC_DIV4: u32   = 1 << 3;
    pub const PSC_DIV8: u32   = 2 << 3;
    pub const PSC_DIV16: u32  = 3 << 3;
    pub const PSC_DIV32: u32  = 4 << 3;
    pub const PSC_DIV64: u32  = 5 << 3;
    pub const PSC_DIV128: u32 = 6 << 3;
    pub const PSC_DIV256: u32 = 7 << 3;
    pub const SPIEN: u32   = 1 << 6;  // SPI enable
    pub const LF: u32      = 1 << 7;  // LSB first
    pub const SWNSS: u32   = 1 << 8;  // Software NSS
    pub const SWNSSEN: u32 = 1 << 9;  // Software NSS enable
    pub const RO: u32      = 1 << 10; // Receive only
    pub const FF16: u32    = 1 << 11; // Frame format (0: 8-bit, 1: 16-bit)
}

pub mod stat {
    pub const RBNE: u32  = 1 << 0; // Receive buffer not empty
    pub const TBE: u32   = 1 << 1; // Transmit buffer empty
    pub const I2SCH: u32 = 1 << 2; // I2S channel
    pub const TXURERR: u32 = 1 << 3; // Transmission underrun error
    pub const CRCERR: u32  = 1 << 4; // SPI CRC error
    pub const CONFERR: u32 = 1 << 5; // SPI configuration error
    pub const RXORERR: u32 = 1 << 6; // Reception overrun error
    pub const TRANS: u32   = 1 << 7; // Transmitting / busy
}

pub struct Spi {
    ptr: *mut SpiRegisterBlock,
}

unsafe impl Send for Spi {}
unsafe impl Sync for Spi {}

impl Spi {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut SpiRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &SpiRegisterBlock {
        unsafe { &*self.ptr }
    }
}
