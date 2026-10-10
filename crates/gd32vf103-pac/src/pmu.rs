use crate::VolatileCell;

pub const PMU_BASE: usize = 0x4000_7000;

#[repr(C)]
pub struct PmuRegisterBlock {
    pub ctl: VolatileCell<u32>, // 0x00
    pub cs: VolatileCell<u32>,  // 0x04
}

pub mod ctl {
    pub const LDOLP: u32  = 1 << 0;
    pub const STBMOD: u32 = 1 << 1;
    pub const WURST: u32  = 1 << 2;
    pub const STBRST: u32 = 1 << 3;
    pub const LVDEN: u32  = 1 << 4;
    pub const BKPWEN: u32 = 1 << 8; // Backup domain write enable
}

pub struct Pmu {
    ptr: *mut PmuRegisterBlock,
}

unsafe impl Send for Pmu {}
unsafe impl Sync for Pmu {}

impl Pmu {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut PmuRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &PmuRegisterBlock {
        unsafe { &*self.ptr }
    }
}
