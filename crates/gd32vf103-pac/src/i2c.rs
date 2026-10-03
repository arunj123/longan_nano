use crate::VolatileCell;

pub const I2C0_BASE: usize = 0x4000_5400;
pub const I2C1_BASE: usize = 0x4000_5800;

#[repr(C)]
pub struct I2cRegisterBlock {
    pub ctl0: VolatileCell<u32>,       // 0x00
    pub ctl1: VolatileCell<u32>,       // 0x04
    pub saddr0: VolatileCell<u32>,     // 0x08
    pub saddr1: VolatileCell<u32>,     // 0x0C
    pub data: VolatileCell<u32>,       // 0x10
    pub stat0: VolatileCell<u32>,      // 0x14
    pub stat1: VolatileCell<u32>,      // 0x18
    pub ckcfg: VolatileCell<u32>,      // 0x1C
    pub rt: VolatileCell<u32>,         // 0x20
    pub _reserved0: [u32; 27],        // 0x24 - 0x8C
    pub fmpcfg: VolatileCell<u32>,     // 0x90
}

pub mod ctl0 {
    pub const I2CEN: u32 = 1 << 0;
    pub const SMBEN: u32 = 1 << 1;
    pub const SMBSEL: u32 = 1 << 3;
    pub const ARPEN: u32 = 1 << 4;
    pub const PECEN: u32 = 1 << 5;
    pub const GCEN: u32 = 1 << 6;
    pub const SS: u32 = 1 << 7;
    pub const START: u32 = 1 << 8;
    pub const STOP: u32 = 1 << 9;
    pub const ACKEN: u32 = 1 << 10;
    pub const POAP: u32 = 1 << 11;
    pub const PECTRANS: u32 = 1 << 12;
    pub const SALT: u32 = 1 << 13;
    pub const SRESET: u32 = 1 << 15;
}

pub mod ctl1 {
    pub const I2CCLK_MASK: u32 = 0x3F;
    pub const ERRIE: u32 = 1 << 8;
    pub const EVIE: u32 = 1 << 9;
    pub const BUFIE: u32 = 1 << 10;
    pub const DMAON: u32 = 1 << 11;
    pub const DMALST: u32 = 1 << 12;
}

pub mod stat0 {
    pub const SBSEND: u32 = 1 << 0;
    pub const ADDSEND: u32 = 1 << 1;
    pub const BTC: u32 = 1 << 2;
    pub const ADD10SEND: u32 = 1 << 3;
    pub const STPDET: u32 = 1 << 4;
    pub const RBNE: u32 = 1 << 6;
    pub const TBE: u32 = 1 << 7;
    pub const BERR: u32 = 1 << 8;
    pub const LOSTARB: u32 = 1 << 9;
    pub const AERR: u32 = 1 << 10;
    pub const OUERR: u32 = 1 << 11;
    pub const PECERR: u32 = 1 << 12;
    pub const SMBTO: u32 = 1 << 14;
    pub const SMBALT: u32 = 1 << 15;
}

pub mod stat1 {
    pub const MASTER: u32 = 1 << 0;
    pub const I2CBSY: u32 = 1 << 1;
    pub const TR: u32 = 1 << 2;
    pub const RXGC: u32 = 1 << 4;
    pub const DEFSMB: u32 = 1 << 5;
    pub const HSTSMB: u32 = 1 << 6;
    pub const DUMODF: u32 = 1 << 7;
}

pub mod ckcfg {
    pub const CLKC_MASK: u32 = 0xFFF;
    pub const DTCY: u32 = 1 << 14;
    pub const FAST: u32 = 1 << 15;
}

pub struct I2c {
    ptr: *mut I2cRegisterBlock,
}

unsafe impl Send for I2c {}
unsafe impl Sync for I2c {}

impl I2c {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut I2cRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &I2cRegisterBlock {
        unsafe { &*self.ptr }
    }
}
