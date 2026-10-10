use crate::VolatileCell;

pub const FWDGT_BASE: usize = 0x4000_3000;

#[repr(C)]
pub struct FwdgtRegisterBlock {
    pub ctl: VolatileCell<u32>,   // 0x00
    pub psc: VolatileCell<u32>,   // 0x04
    pub rld: VolatileCell<u32>,   // 0x08
    pub stat: VolatileCell<u32>,  // 0x0C
}

pub mod ctl {
    pub const CMD_WRITE_ENABLE: u32 = 0x5555;
    pub const CMD_RELOAD: u32 = 0xAAAA;
    pub const CMD_ENABLE: u32 = 0xCCCC;
}

pub mod psc {
    pub const DIV4: u32 = 0;
    pub const DIV8: u32 = 1;
    pub const DIV16: u32 = 2;
    pub const DIV32: u32 = 3;
    pub const DIV64: u32 = 4;
    pub const DIV128: u32 = 5;
    pub const DIV256: u32 = 6;
}

pub mod stat {
    pub const PUD: u32 = 1 << 0;
    pub const RUD: u32 = 1 << 1;
}

pub struct Fwdgt {
    ptr: *mut FwdgtRegisterBlock,
}

unsafe impl Send for Fwdgt {}
unsafe impl Sync for Fwdgt {}

impl Fwdgt {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut FwdgtRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &FwdgtRegisterBlock {
        unsafe { &*self.ptr }
    }
}
