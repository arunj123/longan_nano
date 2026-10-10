use core::marker::PhantomData;
use gd32vf103_pac::gpio::{GpioPort, Port};
use gd32vf103_pac::rcu::Rcu;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

pub mod mode {
    pub struct Input<MODE> {
        _marker: core::marker::PhantomData<MODE>,
    }
    pub struct Floating;
    pub struct PullUp;
    pub struct PullDown;

    pub struct Output<MODE> {
        _marker: core::marker::PhantomData<MODE>,
    }
    pub struct PushPull;
    pub struct OpenDrain;

    pub struct Alternate<MODE> {
        _marker: core::marker::PhantomData<MODE>,
    }
}

pub struct Pin<PORT, const PIN: u8, MODE> {
    _port: PhantomData<PORT>,
    _mode: PhantomData<MODE>,
}

pub struct PortA;
pub struct PortB;
pub struct PortC;

pub trait PortTrait {
    const PORT: Port;
}

impl PortTrait for PortA {
    const PORT: Port = Port::A;
}

impl PortTrait for PortB {
    const PORT: Port = Port::B;
}

impl PortTrait for PortC {
    const PORT: Port = Port::C;
}

// Helper to configure a pin mode in CTL0/CTL1
fn configure_pin(port: Port, pin: u8, config_bits: u32) {
    let port_regs = unsafe { GpioPort::steal(port) };
    let regs = port_regs.regs();
    let bit_offset = (pin % 8) * 4;
    let mask = 0xF << bit_offset;
    let val = config_bits << bit_offset;

    if pin < 8 {
        regs.ctl0.modify(|curr| (curr & !mask) | val);
    } else {
        regs.ctl1.modify(|curr| (curr & !mask) | val);
    }
}

impl<PORT: PortTrait, const PIN: u8, MODE> Pin<PORT, PIN, MODE> {
    #[inline(always)]
    pub fn into_push_pull_output(self) -> Pin<PORT, PIN, mode::Output<mode::PushPull>> {
        configure_pin(PORT::PORT, PIN, 0b0011); // Output Push-Pull, 50 MHz
        Pin {
            _port: PhantomData,
            _mode: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_open_drain_output(self) -> Pin<PORT, PIN, mode::Output<mode::OpenDrain>> {
        configure_pin(PORT::PORT, PIN, 0b0111); // Output Open-Drain, 50 MHz
        Pin {
            _port: PhantomData,
            _mode: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_alternate_push_pull(self) -> Pin<PORT, PIN, mode::Alternate<mode::PushPull>> {
        configure_pin(PORT::PORT, PIN, 0b1011); // Alternate function Push-Pull, 50 MHz
        Pin {
            _port: PhantomData,
            _mode: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_alternate_open_drain(self) -> Pin<PORT, PIN, mode::Alternate<mode::OpenDrain>> {
        configure_pin(PORT::PORT, PIN, 0b1111); // Alternate function Open-Drain, 50 MHz
        Pin {
            _port: PhantomData,
            _mode: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_pull_up_input(self) -> Pin<PORT, PIN, mode::Input<mode::PullUp>> {
        configure_pin(PORT::PORT, PIN, 0b1000); // Input with pull-up/pull-down
        let port_regs = unsafe { GpioPort::steal(PORT::PORT) };
        port_regs.regs().bop.write(1 << PIN); // Set BOP bit for pull-up
        Pin {
            _port: PhantomData,
            _mode: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_pull_down_input(self) -> Pin<PORT, PIN, mode::Input<mode::PullDown>> {
        configure_pin(PORT::PORT, PIN, 0b1000); // Input with pull-up/pull-down
        let port_regs = unsafe { GpioPort::steal(PORT::PORT) };
        port_regs.regs().bc.write(1 << PIN); // Set BC bit for pull-down (OCTL=0)
        Pin {
            _port: PhantomData,
            _mode: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_floating_input(self) -> Pin<PORT, PIN, mode::Input<mode::Floating>> {
        configure_pin(PORT::PORT, PIN, 0b0100); // Input floating
        Pin {
            _port: PhantomData,
            _mode: PhantomData,
        }
    }
}

// Embedded-HAL digital trait implementations for Output pins
impl<PORT: PortTrait, const PIN: u8, MODE> ErrorType for Pin<PORT, PIN, mode::Output<MODE>> {
    type Error = core::convert::Infallible;
}

impl<PORT: PortTrait, const PIN: u8, MODE> OutputPin for Pin<PORT, PIN, mode::Output<MODE>> {
    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let port_regs = unsafe { GpioPort::steal(PORT::PORT) };
        port_regs.regs().bc.write(1 << PIN);
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let port_regs = unsafe { GpioPort::steal(PORT::PORT) };
        port_regs.regs().bop.write(1 << PIN);
        Ok(())
    }
}

impl<PORT: PortTrait, const PIN: u8, MODE> StatefulOutputPin for Pin<PORT, PIN, mode::Output<MODE>> {
    #[inline(always)]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        let port_regs = unsafe { GpioPort::steal(PORT::PORT) };
        Ok((port_regs.regs().octl.read() & (1 << PIN)) != 0)
    }

    #[inline(always)]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_set_high()?)
    }

    #[inline(always)]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        if self.is_set_high()? {
            self.set_low()
        } else {
            self.set_high()
        }
    }
}

// Embedded-HAL digital trait implementations for Input pins
impl<PORT: PortTrait, const PIN: u8, MODE> ErrorType for Pin<PORT, PIN, mode::Input<MODE>> {
    type Error = core::convert::Infallible;
}

impl<PORT: PortTrait, const PIN: u8, MODE> InputPin for Pin<PORT, PIN, mode::Input<MODE>> {
    #[inline(always)]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let port_regs = unsafe { GpioPort::steal(PORT::PORT) };
        Ok((port_regs.regs().istat.read() & (1 << PIN)) != 0)
    }

    #[inline(always)]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

pub struct PartsA {
    pub pa0: Pin<PortA, 0, mode::Input<mode::Floating>>,
    pub pa1: Pin<PortA, 1, mode::Input<mode::Floating>>,
    pub pa2: Pin<PortA, 2, mode::Input<mode::Floating>>,
    pub pa3: Pin<PortA, 3, mode::Input<mode::Floating>>,
    pub pa4: Pin<PortA, 4, mode::Input<mode::Floating>>,
    pub pa5: Pin<PortA, 5, mode::Input<mode::Floating>>,
    pub pa6: Pin<PortA, 6, mode::Input<mode::Floating>>,
    pub pa7: Pin<PortA, 7, mode::Input<mode::Floating>>,
    pub pa8: Pin<PortA, 8, mode::Input<mode::Floating>>,
    pub pa9: Pin<PortA, 9, mode::Input<mode::Floating>>,
    pub pa10: Pin<PortA, 10, mode::Input<mode::Floating>>,
    pub pa11: Pin<PortA, 11, mode::Input<mode::Floating>>,
    pub pa12: Pin<PortA, 12, mode::Input<mode::Floating>>,
    pub pa13: Pin<PortA, 13, mode::Input<mode::Floating>>,
    pub pa14: Pin<PortA, 14, mode::Input<mode::Floating>>,
    pub pa15: Pin<PortA, 15, mode::Input<mode::Floating>>,
}

pub struct PartsB {
    pub pb0: Pin<PortB, 0, mode::Input<mode::Floating>>,
    pub pb1: Pin<PortB, 1, mode::Input<mode::Floating>>,
    pub pb2: Pin<PortB, 2, mode::Input<mode::Floating>>,
    pub pb3: Pin<PortB, 3, mode::Input<mode::Floating>>,
    pub pb4: Pin<PortB, 4, mode::Input<mode::Floating>>,
    pub pb5: Pin<PortB, 5, mode::Input<mode::Floating>>,
    pub pb6: Pin<PortB, 6, mode::Input<mode::Floating>>,
    pub pb7: Pin<PortB, 7, mode::Input<mode::Floating>>,
    pub pb8: Pin<PortB, 8, mode::Input<mode::Floating>>,
    pub pb9: Pin<PortB, 9, mode::Input<mode::Floating>>,
    pub pb10: Pin<PortB, 10, mode::Input<mode::Floating>>,
    pub pb11: Pin<PortB, 11, mode::Input<mode::Floating>>,
    pub pb12: Pin<PortB, 12, mode::Input<mode::Floating>>,
    pub pb13: Pin<PortB, 13, mode::Input<mode::Floating>>,
    pub pb14: Pin<PortB, 14, mode::Input<mode::Floating>>,
    pub pb15: Pin<PortB, 15, mode::Input<mode::Floating>>,
}

pub struct PartsC {
    pub pc13: Pin<PortC, 13, mode::Input<mode::Floating>>,
}

pub trait GpioPortExt {
    fn split_a(self, rcu: &Rcu) -> PartsA;
    fn split_b(self, rcu: &Rcu) -> PartsB;
    fn split_c(self, rcu: &Rcu) -> PartsC;
}

impl GpioPortExt for GpioPort {
    fn split_a(self, rcu: &Rcu) -> PartsA {
        rcu.regs().apb2en.set_bits(Port::A.rcu_apb2_en_bit());
        PartsA {
            pa0: Pin { _port: PhantomData, _mode: PhantomData },
            pa1: Pin { _port: PhantomData, _mode: PhantomData },
            pa2: Pin { _port: PhantomData, _mode: PhantomData },
            pa3: Pin { _port: PhantomData, _mode: PhantomData },
            pa4: Pin { _port: PhantomData, _mode: PhantomData },
            pa5: Pin { _port: PhantomData, _mode: PhantomData },
            pa6: Pin { _port: PhantomData, _mode: PhantomData },
            pa7: Pin { _port: PhantomData, _mode: PhantomData },
            pa8: Pin { _port: PhantomData, _mode: PhantomData },
            pa9: Pin { _port: PhantomData, _mode: PhantomData },
            pa10: Pin { _port: PhantomData, _mode: PhantomData },
            pa11: Pin { _port: PhantomData, _mode: PhantomData },
            pa12: Pin { _port: PhantomData, _mode: PhantomData },
            pa13: Pin { _port: PhantomData, _mode: PhantomData },
            pa14: Pin { _port: PhantomData, _mode: PhantomData },
            pa15: Pin { _port: PhantomData, _mode: PhantomData },
        }
    }

    fn split_b(self, rcu: &Rcu) -> PartsB {
        rcu.regs().apb2en.set_bits(Port::B.rcu_apb2_en_bit());
        PartsB {
            pb0: Pin { _port: PhantomData, _mode: PhantomData },
            pb1: Pin { _port: PhantomData, _mode: PhantomData },
            pb2: Pin { _port: PhantomData, _mode: PhantomData },
            pb3: Pin { _port: PhantomData, _mode: PhantomData },
            pb4: Pin { _port: PhantomData, _mode: PhantomData },
            pb5: Pin { _port: PhantomData, _mode: PhantomData },
            pb6: Pin { _port: PhantomData, _mode: PhantomData },
            pb7: Pin { _port: PhantomData, _mode: PhantomData },
            pb8: Pin { _port: PhantomData, _mode: PhantomData },
            pb9: Pin { _port: PhantomData, _mode: PhantomData },
            pb10: Pin { _port: PhantomData, _mode: PhantomData },
            pb11: Pin { _port: PhantomData, _mode: PhantomData },
            pb12: Pin { _port: PhantomData, _mode: PhantomData },
            pb13: Pin { _port: PhantomData, _mode: PhantomData },
            pb14: Pin { _port: PhantomData, _mode: PhantomData },
            pb15: Pin { _port: PhantomData, _mode: PhantomData },
        }
    }

    fn split_c(self, rcu: &Rcu) -> PartsC {
        rcu.regs().apb2en.set_bits(Port::C.rcu_apb2_en_bit());
        PartsC {
            pc13: Pin { _port: PhantomData, _mode: PhantomData },
        }
    }
}
