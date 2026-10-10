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

    pub const USBFS_PSC_MASK: u32 = 0x3 << 22;
    pub const USBFS_PSC_DIV1_5: u32 = 0x0 << 22;
    pub const USBFS_PSC_DIV1: u32 = 0x1 << 22;
    pub const USBFS_PSC_DIV2_5: u32 = 0x2 << 22;
    pub const USBFS_PSC_DIV2: u32 = 0x3 << 22;
}

// Bit definitions for RCU_AHBEN
pub mod ahben {
    pub const DMA0EN: u32 = 1 << 0;
    pub const DMA1EN: u32 = 1 << 1;
    pub const SRAMSPEN: u32 = 1 << 2;
    pub const FMCSPEN: u32 = 1 << 4;
    pub const CRCEN: u32 = 1 << 6;
    pub const EXMCEN: u32 = 1 << 8;
    pub const USBFSEN: u32 = 1 << 12;
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
}

// Bit definitions for RCU_APB1RST
pub mod apb1rst {
    pub const I2C0RST: u32 = 1 << 21;
    pub const I2C1RST: u32 = 1 << 22;
}

// Bit definitions for RCU_APB1EN
pub mod apb1en {
    pub const TIMER1EN: u32 = 1 << 0;
    pub const TIMER2EN: u32 = 1 << 1;
    pub const TIMER3EN: u32 = 1 << 2;
    pub const TIMER4EN: u32 = 1 << 3;
    pub const TIMER5EN: u32 = 1 << 4;
    pub const TIMER6EN: u32 = 1 << 5;
    pub const WWDGTEN: u32 = 1 << 11;
    pub const SPI1EN: u32 = 1 << 14;
    pub const SPI2EN: u32 = 1 << 15;
    pub const USART1EN: u32 = 1 << 17;
    pub const USART2EN: u32 = 1 << 18;
    pub const UART3EN: u32 = 1 << 19;
    pub const UART4EN: u32 = 1 << 20;
    pub const I2C0EN: u32 = 1 << 21;
    pub const I2C1EN: u32 = 1 << 22;
    pub const CAN0EN: u32 = 1 << 25;
    pub const CAN1EN: u32 = 1 << 26;
    pub const BKPIEN: u32 = 1 << 27;
    pub const PMUEN: u32 = 1 << 28;
    pub const DACEN: u32 = 1 << 29;
}

// Bit definitions for RCU_BDCTL
pub mod bdctl {
    pub const LXTALEN: u32 = 1 << 0;
    pub const LXTALSTB: u32 = 1 << 1;
    pub const LXTALBPS: u32 = 1 << 2;
    pub const RTCSRC_MASK: u32 = 3 << 8;
    pub const RTCSRC_NONE: u32 = 0 << 8;
    pub const RTCSRC_LXTAL: u32 = 1 << 8;
    pub const RTCSRC_IRC40K: u32 = 2 << 8;
    pub const RTCSRC_HXTAL_DIV128: u32 = 3 << 8;
    pub const RTCEN: u32 = 1 << 15;
    pub const BKPRST: u32 = 1 << 16;
}

// Bit definitions for RCU_RSTSCK
pub mod rstsck {
    pub const IRC40KEN: u32 = 1 << 0;
    pub const IRC40KSTB: u32 = 1 << 1;
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
