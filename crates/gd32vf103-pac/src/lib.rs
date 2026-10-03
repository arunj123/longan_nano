#![no_std]

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub mod rcu;
pub mod gpio;
pub mod mtime;
pub mod usart;

pub use rcu::Rcu;
pub use gpio::{Gpio, GpioPort};
pub use mtime::Mtime;
pub use usart::Usart;

/// GD32VF103 Peripherals singleton
pub struct Peripherals {
    pub rcu: Rcu,
    pub gpioa: GpioPort,
    pub gpiob: GpioPort,
    pub gpioc: GpioPort,
    pub mtime: Mtime,
    pub usart0: Usart,
}

static TAKEN: AtomicBool = AtomicBool::new(false);

impl Peripherals {
    /// Takes the peripheral singleton once. Returns None if already taken.
    #[inline]
    pub fn take() -> Option<Self> {
        if TAKEN.swap(true, Ordering::SeqCst) {
            None
        } else {
            Some(unsafe { Self::steal() })
        }
    }

    /// Unsafely steals the peripheral singleton.
    ///
    /// # Safety
    /// Caller must guarantee that concurrent access does not cause race conditions.
    #[inline]
    pub unsafe fn steal() -> Self {
        Self {
            rcu: Rcu::steal(),
            gpioa: GpioPort::steal(gpio::Port::A),
            gpiob: GpioPort::steal(gpio::Port::B),
            gpioc: GpioPort::steal(gpio::Port::C),
            mtime: Mtime::steal(),
            usart0: Usart::steal(usart::USART0_BASE),
        }
    }
}

/// Generic register wrapper for memory-mapped volatile I/O.
#[repr(transparent)]
pub struct VolatileCell<T> {
    value: UnsafeCell<T>,
}

impl<T> VolatileCell<T> {
    #[inline(always)]
    pub const fn new(val: T) -> Self {
        Self {
            value: UnsafeCell::new(val),
        }
    }

    #[inline(always)]
    pub fn read(&self) -> T
    where
        T: Copy,
    {
        unsafe { core::ptr::read_volatile(self.value.get()) }
    }

    #[inline(always)]
    pub fn write(&self, val: T)
    where
        T: Copy,
    {
        unsafe { core::ptr::write_volatile(self.value.get(), val) }
    }

    #[inline(always)]
    pub fn modify<F>(&self, f: F)
    where
        T: Copy,
        F: FnOnce(T) -> T,
    {
        let val = self.read();
        self.write(f(val));
    }
}

impl VolatileCell<u32> {
    #[inline(always)]
    pub fn set_bits(&self, mask: u32) {
        self.modify(|val| val | mask);
    }

    #[inline(always)]
    pub fn clear_bits(&self, mask: u32) {
        self.modify(|val| val & !mask);
    }
}
