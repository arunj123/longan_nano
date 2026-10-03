use crate::VolatileCell;

pub const USART0_BASE: usize = 0x4001_3800;

#[repr(C)]
pub struct UsartRegisterBlock {
    pub stat: VolatileCell<u32>,   // 0x00: Status register
    pub data: VolatileCell<u32>,   // 0x04: Data register
    pub baud: VolatileCell<u32>,   // 0x08: Baud rate register
    pub ctl0: VolatileCell<u32>,   // 0x0C: Control register 0
    pub ctl1: VolatileCell<u32>,   // 0x10: Control register 1
    pub ctl2: VolatileCell<u32>,   // 0x14: Control register 2
    pub gp: VolatileCell<u32>,     // 0x18: Guard time and prescaler register
}

pub mod stat {
    pub const TBE: u32  = 1 << 7;  // Transmit data buffer empty
    pub const TC: u32   = 1 << 6;  // Transmission complete
    pub const RBNE: u32 = 1 << 5;  // Read data buffer not empty
    pub const IDLE: u32 = 1 << 4;  // IDLE frame detected
    pub const ORERR: u32 = 1 << 3; // Overrun error
    pub const NERR: u32  = 1 << 2; // Noise error
    pub const FERR: u32  = 1 << 1; // Framing error
    pub const PERR: u32  = 1 << 0; // Parity error
}

pub mod ctl0 {
    pub const UEN: u32    = 1 << 13; // USART enable
    pub const WL: u32     = 1 << 12; // Word length (0: 8 bits, 1: 9 bits)
    pub const WM: u32     = 1 << 11; // Wakeup method
    pub const PCEN: u32   = 1 << 10; // Parity control enable
    pub const PM: u32     = 1 << 9;  // Parity selection
    pub const TBEIE: u32  = 1 << 7;  // Transmit data buffer empty interrupt enable
    pub const TCIE: u32   = 1 << 6;  // Transmission complete interrupt enable
    pub const RBNEIE: u32 = 1 << 5;  // Read data buffer not empty interrupt enable
    pub const TEN: u32    = 1 << 3;  // Transmitter enable
    pub const REN: u32    = 1 << 2;  // Receiver enable
    pub const RWU: u32    = 1 << 1;  // Receiver wakeup
    pub const SBKCMD: u32 = 1 << 0;  // Send break command
}

pub struct Usart {
    ptr: *mut UsartRegisterBlock,
}

unsafe impl Send for Usart {}
unsafe impl Sync for Usart {}

impl Usart {
    #[inline(always)]
    pub unsafe fn steal(base: usize) -> Self {
        Self {
            ptr: base as *mut UsartRegisterBlock,
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> &UsartRegisterBlock {
        unsafe { &*self.ptr }
    }
}
