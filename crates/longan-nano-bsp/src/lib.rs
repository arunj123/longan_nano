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

pub mod lcd;
pub use lcd::{color as lcd_color, Lcd, LCD_HEIGHT, LCD_WIDTH};

pub mod sdcard;
pub use sdcard::{CardType, SdCard, SdError};

use hal::spi::{Prescaler, Spi0, Spi1};

/// Longan Nano Board peripherals container
pub struct Board {
    pub led_red: LedRed,
    pub led_green: LedGreen,
    pub led_blue: LedBlue,
    pub button: KeyButton,
    pub uart0: Uart0,
    pub lcd: Lcd,
    pub sdcard: SdCard,
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
        let gpiob = dp.gpiob.split_b(&rcu);
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

        // Configure SPI0 and LCD control pins (PB2 CS, PB0 DC, PB1 RST, PA5 SCK, PA7 MOSI)
        let cs = gpiob.pb2.into_push_pull_output();
        let dc = gpiob.pb0.into_push_pull_output();
        let rst = gpiob.pb1.into_push_pull_output();
        let sck = gpioa.pa5.into_alternate_push_pull();
        let mosi = gpioa.pa7.into_alternate_push_pull();
        let spi0 = Spi0::new_master(dp.spi0, sck, mosi, Prescaler::Div8, &rcu);
        let lcd = Lcd::new(spi0, cs, dc, rst);

        // Configure SPI1 and SD card control pins (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI)
        let mut sd_cs = gpiob.pb12.into_push_pull_output();
        let _ = sd_cs.set_high(); // Deselect SD card initially
        let sd_sck = gpiob.pb13.into_alternate_push_pull();
        let sd_miso = gpiob.pb14.into_pull_up_input();
        let sd_mosi = gpiob.pb15.into_alternate_push_pull();
        let spi1 = Spi1::new_master(dp.spi1, sd_sck, sd_miso, sd_mosi, Prescaler::Div256, &rcu);
        let sdcard = SdCard::new(spi1, sd_cs);

        let delay = Delay::new(dp.mtime, clocks);

        Some(Self {
            led_red,
            led_green,
            led_blue,
            button,
            uart0,
            lcd,
            sdcard,
            delay,
            clocks,
        })
    }
}
