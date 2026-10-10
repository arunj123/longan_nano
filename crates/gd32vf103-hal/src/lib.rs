#![no_std]

pub use gd32vf103_pac as pac;

pub mod rcu;
pub mod gpio;
pub mod delay;
pub mod uart;
pub mod spi;
pub mod usb;
pub mod i2c;
pub mod fwdgt;
pub mod adc;

pub use rcu::{Clocks, RcuConfig, RcuExt};
pub use gpio::{GpioPortExt, Pin, PortA, PortB, PortC};
pub use delay::Delay;
pub use uart::Uart0;
pub use spi::{Prescaler, Spi0, Spi1};
pub use usb::{MscBlockDevice, MscStats, UsbCdcAcm, UsbComposite, UsbHid, UsbMsc};
pub use i2c::{DutyCycle, I2c, I2cError};
pub use fwdgt::Fwdgt;
pub use adc::Adc0TempSensor;
