use crate::VolatileCell;

pub const BKP_BASE: usize = 0x4000_6C00;

#[repr(C)]
pub struct BkpRegisterBlock {
    pub _reserved0: u32,           // 0x00
    pub data0: VolatileCell<u32>,  // 0x04 (low 16 bits user data)
    pub data1: VolatileCell<u32>,  // 0x08
    pub data2: VolatileCell<u32>,  // 0x0C
    pub data3: VolatileCell<u32>,  // 0x10
    pub data4: VolatileCell<u32>,  // 0x14
    pub data5: VolatileCell<u32>,  // 0x18
    pub data6: VolatileCell<u32>,  // 0x1C
    pub data7: VolatileCell<u32>,  // 0x20
    pub data8: VolatileCell<u32>,  // 0x24
    pub data9: VolatileCell<u32>,  // 0x28
    pub oct_ctl: VolatileCell<u32>,// 0x2C
    pub tp_ctl: VolatileCell<u32>, // 0x30
    pub tp_cs: VolatileCell<u32>,  // 0x34
}

pub struct Bkp {
    ptr: *mut BkpRegisterBlock,
}

unsafe impl Send for Bkp {}
unsafe impl Sync for Bkp {}

impl Bkp {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut BkpRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &BkpRegisterBlock {
        unsafe { &*self.ptr }
    }
}
