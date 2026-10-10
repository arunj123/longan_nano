use crate::VolatileCell;

pub const ADC0_BASE: usize = 0x4001_2400;
pub const ADC1_BASE: usize = 0x4001_2800;

#[repr(C)]
pub struct AdcRegisterBlock {
    pub stat: VolatileCell<u32>,       // 0x00
    pub ctl0: VolatileCell<u32>,       // 0x04
    pub ctl1: VolatileCell<u32>,       // 0x08
    pub sampt0: VolatileCell<u32>,     // 0x0C
    pub sampt1: VolatileCell<u32>,     // 0x10
    pub ioff0: VolatileCell<u32>,      // 0x14
    pub ioff1: VolatileCell<u32>,      // 0x18
    pub ioff2: VolatileCell<u32>,      // 0x1C
    pub ioff3: VolatileCell<u32>,      // 0x20
    pub wdht: VolatileCell<u32>,       // 0x24
    pub wdlt: VolatileCell<u32>,       // 0x28
    pub rsq0: VolatileCell<u32>,       // 0x2C
    pub rsq1: VolatileCell<u32>,       // 0x30
    pub rsq2: VolatileCell<u32>,       // 0x34
    pub isq: VolatileCell<u32>,        // 0x38
    pub idata0: VolatileCell<u32>,     // 0x3C
    pub idata1: VolatileCell<u32>,     // 0x40
    pub idata2: VolatileCell<u32>,     // 0x44
    pub idata3: VolatileCell<u32>,     // 0x48
    pub rdata: VolatileCell<u32>,      // 0x4C
    pub _reserved0: [u32; 12],         // 0x50..0x7C
    pub ovscr: VolatileCell<u32>,      // 0x80
}

pub mod stat {
    pub const WDE: u32 = 1 << 0;
    pub const EOC: u32 = 1 << 1;
    pub const EOIC: u32 = 1 << 2;
    pub const STIC: u32 = 1 << 3;
    pub const STRC: u32 = 1 << 4;
}

pub mod ctl1 {
    pub const ADCON: u32 = 1 << 0;
    pub const CTN: u32 = 1 << 1;
    pub const CLB: u32 = 1 << 2;
    pub const RSTCLB: u32 = 1 << 3;
    pub const DMA: u32 = 1 << 8;
    pub const DAL: u32 = 1 << 11;
    pub const ETSRC_NONE: u32 = 7 << 17;
    pub const ETERC: u32 = 1 << 20;
    pub const SWICST: u32 = 1 << 21;
    pub const SWRCST: u32 = 1 << 22;
    pub const TSVREN: u32 = 1 << 23;
}

pub struct Adc {
    ptr: *mut AdcRegisterBlock,
}

unsafe impl Send for Adc {}
unsafe impl Sync for Adc {}

impl Adc {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut AdcRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &AdcRegisterBlock {
        unsafe { &*self.ptr }
    }
}
