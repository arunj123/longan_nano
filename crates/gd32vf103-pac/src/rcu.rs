use crate::VolatileCell;

pub const RCU_BASE: usize = 0x4002_1000;

#[repr(C)]
pub struct RcuRegisterBlock {
    pub ctl: VolatileCell<u32>,       // 0x00
    pub cfg0: VolatileCell<u32>,      // 0x04
    pub int: VolatileCell<u32>,       // 0x08
    pub apb2rst: VolatileCell<u32>,   // 0x0C
    pub apb1rst: VolatileCell<u32>,   // 0x10
    pub ahben: VolatileCell<u32>,     // 0x14
    pub apb2en: VolatileCell<u32>,    // 0x18
    pub apb1en: VolatileCell<u32>,    // 0x1C
    pub bdctl: VolatileCell<u32>,     // 0x20
    pub rstsck: VolatileCell<u32>,    // 0x24
    pub ahbrst: VolatileCell<u32>,    // 0x28
    pub cfg1: VolatileCell<u32>,      // 0x2C
    pub _reserved0: u32,             // 0x30
    pub dsv: VolatileCell<u32>,       // 0x34
}

// Bit definitions for RCU_CTL
pub mod ctl {
    pub const IRC8MEN: u32 = 1 << 0;
    pub const IRC8MSTB: u32 = 1 << 1;
    pub const HXTALEN: u32 = 1 << 16;
    pub const HXTALSTB: u32 = 1 << 17;
    pub const HXTALBPS: u32 = 1 << 18;
    pub const CKMEN: u32 = 1 << 19;
    pub const PLLEN: u32 = 1 << 24;
    pub const PLLSTB: u32 = 1 << 25;
    pub const PLL1EN: u32 = 1 << 26;
    pub const PLL1STB: u32 = 1 << 27;
    pub const PLL2EN: u32 = 1 << 28;
    pub const PLL2STB: u32 = 1 << 29;
}

// Bit definitions for RCU_CFG0
pub mod cfg0 {
    pub const SCS_MASK: u32 = 0x3;
    pub const SCS_IRC8M: u32 = 0x0;
    pub const SCS_HXTAL: u32 = 0x1;
    pub const SCS_PLL: u32 = 0x2;

    pub const SCSS_MASK: u32 = 0x3 << 2;
    pub const SCSS_IRC8M: u32 = 0x0 << 2;
    pub const SCSS_HXTAL: u32 = 0x1 << 2;
    pub const SCSS_PLL: u32 = 0x2 << 2;

    pub const AHB_DIV1: u32 = 0x0 << 4;
    pub const APB1_DIV2: u32 = 0x4 << 8;
    pub const APB2_DIV1: u32 = 0x0 << 11;
}

// Bit definitions for RCU_CFG1
pub mod cfg1 {
    pub const PREDV0_MASK: u32 = 0xF;
    pub const PREDV0_DIV1: u32 = 0x0;
    pub const PREDV0_DIV2: u32 = 0x1;
    pub const PREDV0SRC_HXTAL: u32 = 0 << 16;
    pub const PREDV0SRC_PLL1: u32 = 1 << 16;
}

// Bit definitions for RCU_APB2EN
pub mod apb2en {
    pub const AFEN: u32 = 1 << 0;
    pub const PAEN: u32 = 1 << 2;
    pub const PBEN: u32 = 1 << 3;
    pub const PCEN: u32 = 1 << 4;
    pub const PDEN: u32 = 1 << 5;
    pub const PEEN: u32 = 1 << 6;
    pub const ADC0EN: u32 = 1 << 9;
    pub const ADC1EN: u32 = 1 << 10;
    pub const TIMER0EN: u32 = 1 << 11;
    pub const SPI0EN: u32 = 1 << 12;
    pub const USART0EN: u32 = 1 << 14;
}

pub struct Rcu {
    ptr: *mut RcuRegisterBlock,
}

unsafe impl Send for Rcu {}
unsafe impl Sync for Rcu {}

impl Rcu {
    /// Creates an RCU instance pointing to the hardware register block.
    #[inline(always)]
    pub unsafe fn steal() -> Self {
        Self {
            ptr: RCU_BASE as *mut RcuRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &RcuRegisterBlock {
        unsafe { &*self.ptr }
    }
}
