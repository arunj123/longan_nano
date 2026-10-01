use crate::VolatileCell;

pub const GPIO_BASE: usize = 0x4001_0800;
pub const PORT_STRIDE: usize = 0x0000_0400;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Port {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
}

impl Port {
    #[inline(always)]
    pub const fn base_addr(self) -> usize {
        GPIO_BASE + (self as usize) * PORT_STRIDE
    }

    #[inline(always)]
    pub const fn rcu_apb2_en_bit(self) -> u32 {
        1 << (2 + (self as u32))
    }
}

#[repr(C)]
pub struct GpioRegisterBlock {
    pub ctl0: VolatileCell<u32>,   // 0x00: Pin 0..7 configuration
    pub ctl1: VolatileCell<u32>,   // 0x04: Pin 8..15 configuration
    pub istat: VolatileCell<u32>,  // 0x08: Port input status
    pub octl: VolatileCell<u32>,   // 0x0C: Port output control
    pub bop: VolatileCell<u32>,    // 0x10: Port bit operate (set/reset)
    pub bc: VolatileCell<u32>,     // 0x14: Port bit clear
    pub lock: VolatileCell<u32>,   // 0x18: Port lock
}

pub struct GpioPort {
    port: Port,
    ptr: *mut GpioRegisterBlock,
}

unsafe impl Send for GpioPort {}
unsafe impl Sync for GpioPort {}

impl GpioPort {
    #[inline(always)]
    pub unsafe fn steal(port: Port) -> Self {
        Self {
            port,
            ptr: port.base_addr() as *mut GpioRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn port(&self) -> Port {
        self.port
    }

    #[inline(always)]
    pub fn regs(&self) -> &GpioRegisterBlock {
        unsafe { &*self.ptr }
    }
}

pub struct Gpio;
