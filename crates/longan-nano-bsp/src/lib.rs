#![no_std]

pub use gd32vf103_hal as hal;

use hal::pac::Peripherals;
use hal::rcu::{Clocks, RcuConfig, RcuExt};
use hal::gpio::{mode, GpioPortExt, Pin, PortA, PortC};
use hal::delay::Delay;
use hal::Uart0;
use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};

/// Active-low LED abstraction
pub struct Led<P> {
    pin: P,
}

impl<P: OutputPin + StatefulOutputPin> Led<P> {
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    #[inline(always)]
    pub fn on(&mut self) {
        let _ = self.pin.set_low(); // Active low: Low turns LED ON
    }

    #[inline(always)]
    pub fn off(&mut self) {
        let _ = self.pin.set_high(); // High turns LED OFF
    }

    #[inline(always)]
    pub fn toggle(&mut self) {
        let _ = self.pin.toggle();
    }
}

pub type LedRed = Led<Pin<PortC, 13, mode::Output<mode::PushPull>>>;
pub type LedGreen = Led<Pin<PortA, 1, mode::Output<mode::PushPull>>>;
pub type LedBlue = Led<Pin<PortA, 2, mode::Output<mode::PushPull>>>;

/// Active-low User Button abstraction (PA8)
pub struct Button<P> {
    pin: P,
}

impl<P: InputPin> Button<P> {
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    #[inline(always)]
    pub fn is_pressed(&mut self) -> bool {
        // Active-low: Low means button is pressed
        self.pin.is_low().unwrap_or(false)
    }

    #[inline(always)]
    pub fn is_released(&mut self) -> bool {
        self.pin.is_high().unwrap_or(true)
    }
}

pub type KeyButton = Button<Pin<PortA, 8, mode::Input<mode::PullUp>>>;

/// Longan Nano Board peripherals container
pub struct Board {
    pub led_red: LedRed,
    pub led_green: LedGreen,
    pub led_blue: LedBlue,
    pub button: KeyButton,
    pub uart0: Uart0,
    pub delay: Delay,
    pub clocks: Clocks,
}

impl Board {
    /// Initializes board clocks (108 MHz via HXTAL PLL), LEDs, button, and UART0 @ 115200 baud.
    pub fn take() -> Option<Self> {
        Self::take_with_baud(115200)
    }

    /// Initializes board with a custom UART0 baud rate.
    pub fn take_with_baud(baud: u32) -> Option<Self> {
        let dp = Peripherals::take()?;
        let (rcu, clocks) = dp.rcu.freeze(RcuConfig::default());

        let gpioa = dp.gpioa.split_a(&rcu);
        let gpioc = dp.gpioc.split_c(&rcu);

        let mut led_red = Led::new(gpioc.pc13.into_push_pull_output());
        let mut led_green = Led::new(gpioa.pa1.into_push_pull_output());
        let mut led_blue = Led::new(gpioa.pa2.into_push_pull_output());

        // Initial state: all LEDs off
        led_red.off();
        led_green.off();
        led_blue.off();

        let button = Button::new(gpioa.pa8.into_pull_up_input());

        // Configure PA9 (TX) and PA10 (RX) for USART0
        let tx = gpioa.pa9.into_alternate_push_pull();
        let rx = gpioa.pa10.into_floating_input();
        let uart0 = Uart0::new(dp.usart0, tx, rx, baud, &clocks, &rcu);

        let delay = Delay::new(dp.mtime, clocks);

        Some(Self {
            led_red,
            led_green,
            led_blue,
            button,
            uart0,
            delay,
            clocks,
        })
    }
}
