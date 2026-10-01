use crate::VolatileCell;

pub const MTIME_BASE: usize = 0xD100_0000;

#[repr(C)]
pub struct MtimeRegisterBlock {
    pub mtime_lo: VolatileCell<u32>,     // 0x00
    pub mtime_hi: VolatileCell<u32>,     // 0x04
    pub mtimecmp_lo: VolatileCell<u32>,  // 0x08
    pub mtimecmp_hi: VolatileCell<u32>,  // 0x0C
}

pub struct Mtime {
    ptr: *mut MtimeRegisterBlock,
}

unsafe impl Send for Mtime {}
unsafe impl Sync for Mtime {}

impl Mtime {
    #[inline(always)]
    pub unsafe fn steal() -> Self {
        Self {
            ptr: MTIME_BASE as *mut MtimeRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &MtimeRegisterBlock {
        unsafe { &*self.ptr }
    }

    /// Atomically reads the 64-bit hardware mtime counter without rollover races.
    #[inline]
    pub fn read_ticks(&self) -> u64 {
        let regs = self.regs();
        loop {
            let hi = regs.mtime_hi.read();
            let lo = regs.mtime_lo.read();
            if hi == regs.mtime_hi.read() {
                return ((hi as u64) << 32) | (lo as u64);
            }
        }
    }
}
