use crate::VolatileCell;

pub const RTC_BASE: usize = 0x4000_2800;

#[repr(C)]
pub struct RtcRegisterBlock {
    pub inten: VolatileCell<u32>, // 0x00
    pub ctl: VolatileCell<u32>,   // 0x04
    pub psch: VolatileCell<u32>,  // 0x08
    pub pscl: VolatileCell<u32>,  // 0x0C
    pub divh: VolatileCell<u32>,  // 0x10
    pub divl: VolatileCell<u32>,  // 0x14
    pub cnth: VolatileCell<u32>,  // 0x18
    pub cntl: VolatileCell<u32>,  // 0x1C
    pub alrmh: VolatileCell<u32>, // 0x20
    pub alrml: VolatileCell<u32>, // 0x24
}

pub mod ctl {
    pub const SCIF: u32   = 1 << 0; // Second interrupt flag
    pub const ALRMIF: u32 = 1 << 1; // Alarm interrupt flag
    pub const OVIF: u32   = 1 << 2; // Overflow interrupt flag
    pub const RSYNF: u32  = 1 << 3; // Registers synchronized flag
    pub const CMF: u32    = 1 << 4; // Configuration mode flag
    pub const LWOFF: u32  = 1 << 5; // Last write operation finished flag
}

pub struct Rtc {
    ptr: *mut RtcRegisterBlock,
}

unsafe impl Send for Rtc {}
unsafe impl Sync for Rtc {}

impl Rtc {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut RtcRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &RtcRegisterBlock {
        unsafe { &*self.ptr }
    }
}
