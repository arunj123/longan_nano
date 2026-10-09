#![no_std]

pub use gd32vf103_hal as hal;

use hal::pac::Peripherals;
use hal::rcu::{Clocks, RcuConfig, RcuExt};
use hal::gpio::{mode, GpioPortExt, Pin, PortA, PortB, PortC};
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

pub use lcd_font;
pub mod lcd;
pub use lcd::{color as lcd_color, Lcd, LCD_HEIGHT, LCD_WIDTH};

pub mod sdcard;
pub use sdcard::{CardType, SdCard, SdError};

pub mod ina219;
pub use ina219::{Ina219, Ina219Data};

pub mod rotary_encoder;
pub use rotary_encoder::RotaryEncoder;

pub use hal::{MscBlockDevice, MscStats, UsbCdcAcm, UsbComposite, UsbHid, UsbMsc};
pub use hal::i2c::{I2c, I2cError};
use hal::spi::{Prescaler, Spi0, Spi1};

pub type BoardRotaryEncoder = RotaryEncoder<
    Pin<PortB, 10, mode::Input<mode::PullUp>>,
    Pin<PortB, 11, mode::Input<mode::PullUp>>,
    Pin<PortB, 12, mode::Input<mode::PullUp>>,
>;

/// Longan Nano Board peripherals container
pub struct Board {
    pub led_red: LedRed,
    pub led_green: LedGreen,
    pub led_blue: LedBlue,
    pub button: KeyButton,
    pub uart0: Uart0,
    pub lcd: Lcd,
    pub sdcard: SdCard,
    pub usb: Option<UsbCdcAcm>,
    pub delay: Delay,
    pub clocks: Clocks,
}

/// Board container specialized for INA219 Current Monitor application
pub struct CurrentMonitorBoard {
    pub led_red: LedRed,
    pub led_green: LedGreen,
    pub led_blue: LedBlue,
    pub button: KeyButton,
    pub uart0: Uart0,
    pub lcd: Lcd,
    pub ina219: Ina219,
    pub usb_hid: UsbHid,
    pub sdcard: SdCard,
    pub delay: Delay,
    pub clocks: Clocks,
}

/// Board container specialized for USB Composite device application
pub struct CompositeBoard {
    pub led_red: LedRed,
    pub led_green: LedGreen,
    pub led_blue: LedBlue,
    pub button: KeyButton,
    pub uart0: Uart0,
    pub lcd: Lcd,
    pub encoder: BoardRotaryEncoder,
    pub usb_composite: UsbComposite,
    pub delay: Delay,
    pub clocks: Clocks,
}

/// Board container specialized for USB Mass Storage Class (MSC) application
pub struct MscBoard {
    pub led_red: LedRed,
    pub led_green: LedGreen,
    pub led_blue: LedBlue,
    pub button: KeyButton,
    pub uart0: Uart0,
    pub lcd: Lcd,
    pub sdcard: SdCard,
    pub usb_msc: UsbMsc,
    pub delay: Delay,
    pub clocks: Clocks,
}

impl hal::MscBlockDevice for SdCard {
    fn is_ready(&self) -> bool {
        self.is_initialized
    }
    fn capacity(&self) -> (u32, u32) {
        (self.sector_count, 512)
    }
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) -> bool {
        self.read_sector(lba, buf).is_ok()
    }
    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) -> bool {
        self.write_sector(lba, buf).is_ok()
    }
}

impl Board {
    /// Initializes board clocks (108 MHz via HXTAL PLL), LEDs, button, and UART0 @ 115200 baud.
    pub fn take() -> Option<Self> {
        Self::take_with_baud(115200)
    }

    /// Initializes board with a custom UART0 baud rate at 108 MHz.
    pub fn take_with_baud(baud: u32) -> Option<Self> {
        Self::take_internal(RcuConfig::default(), baud, false)
    }

    /// Initializes board for USB operation (96 MHz via HXTAL PLL, 48 MHz USB clock),
    /// LEDs, button, UART0 @ 115200 baud, and USB CDC-ACM peripheral.
    pub fn take_usb() -> Option<Self> {
        Self::take_usb_with_baud(115200)
    }

    /// Initializes board for USB with custom UART0 baud rate.
    pub fn take_usb_with_baud(baud: u32) -> Option<Self> {
        let rcu_cfg = RcuConfig {
            use_hxtal: true,
            target_sysclk: 96_000_000,
        };
        Self::take_internal(rcu_cfg, baud, true)
    }

    fn take_internal(rcu_cfg: RcuConfig, baud: u32, init_usb: bool) -> Option<Self> {
        let dp = Peripherals::take()?;
        let (rcu, clocks) = dp.rcu.freeze(rcu_cfg);

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

        let mut delay = Delay::new(dp.mtime, clocks);

        let usb = if init_usb {
            Some(UsbCdcAcm::new(dp.usbfs, &rcu, &mut delay))
        } else {
            None
        };

        Some(Self {
            led_red,
            led_green,
            led_blue,
            button,
            uart0,
            lcd,
            sdcard,
            usb,
            delay,
            clocks,
        })
    }

    /// Initializes board for INA219 Current Monitor (96 MHz PLL, 48 MHz USBFS, LCD, I2C0, UsbHid).
    pub fn take_current_monitor() -> Option<CurrentMonitorBoard> {
        let dp = Peripherals::take()?;
        let rcu_cfg = RcuConfig {
            use_hxtal: true,
            target_sysclk: 96_000_000,
        };
        let (rcu, clocks) = dp.rcu.freeze(rcu_cfg);

        let gpioa = dp.gpioa.split_a(&rcu);
        let gpiob = dp.gpiob.split_b(&rcu);
        let gpioc = dp.gpioc.split_c(&rcu);

        let mut led_red = Led::new(gpioc.pc13.into_push_pull_output());
        let mut led_green = Led::new(gpioa.pa1.into_push_pull_output());
        let mut led_blue = Led::new(gpioa.pa2.into_push_pull_output());
        led_red.off();
        led_green.off();
        led_blue.off();

        let button = Button::new(gpioa.pa8.into_pull_up_input());

        let tx = gpioa.pa9.into_alternate_push_pull();
        let rx = gpioa.pa10.into_floating_input();
        let uart0 = Uart0::new(dp.usart0, tx, rx, 115200, &clocks, &rcu);

        let cs = gpiob.pb2.into_push_pull_output();
        let dc = gpiob.pb0.into_push_pull_output();
        let rst = gpiob.pb1.into_push_pull_output();
        let sck = gpioa.pa5.into_alternate_push_pull();
        let mosi = gpioa.pa7.into_alternate_push_pull();
        let spi0 = Spi0::new_master(dp.spi0, sck, mosi, Prescaler::Div4, &rcu);
        let lcd = Lcd::new(spi0, cs, dc, rst);

        // PB6 (SCL) and PB7 (SDA) as AF Open-Drain 50MHz, Fast Mode 400 kHz
        let scl = gpiob.pb6.into_alternate_open_drain();
        let sda = gpiob.pb7.into_alternate_open_drain();
        let i2c0 = I2c::new(dp.i2c0, scl, sda, 400_000, &clocks, &rcu);
        let ina219 = Ina219::new(i2c0, ina219::INA219_ADDR_DEFAULT);

        let mut delay = Delay::new(dp.mtime, clocks);
        let usb_hid = UsbHid::new(dp.usbfs, &rcu, &mut delay);

        // Configure SPI1 and SD card control pins (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI)
        let mut sd_cs = gpiob.pb12.into_push_pull_output();
        let _ = sd_cs.set_high(); // Deselect SD card initially
        let sd_sck = gpiob.pb13.into_alternate_push_pull();
        let sd_miso = gpiob.pb14.into_pull_up_input();
        let sd_mosi = gpiob.pb15.into_alternate_push_pull();
        let spi1 = Spi1::new_master(dp.spi1, sd_sck, sd_miso, sd_mosi, Prescaler::Div256, &rcu);
        let sdcard = SdCard::new(spi1, sd_cs);

        Some(CurrentMonitorBoard {
            led_red,
            led_green,
            led_blue,
            button,
            uart0,
            lcd,
            ina219,
            usb_hid,
            sdcard,
            delay,
            clocks,
        })
    }

    /// Initializes board for USB Composite device (96 MHz PLL, 48 MHz USBFS, LCD, Rotary Encoder, UsbComposite).
    pub fn take_composite() -> Option<CompositeBoard> {
        let dp = Peripherals::take()?;
        let rcu_cfg = RcuConfig {
            use_hxtal: true,
            target_sysclk: 96_000_000,
        };
        let (rcu, clocks) = dp.rcu.freeze(rcu_cfg);

        let gpioa = dp.gpioa.split_a(&rcu);
        let gpiob = dp.gpiob.split_b(&rcu);
        let gpioc = dp.gpioc.split_c(&rcu);

        let mut led_red = Led::new(gpioc.pc13.into_push_pull_output());
        let mut led_green = Led::new(gpioa.pa1.into_push_pull_output());
        let mut led_blue = Led::new(gpioa.pa2.into_push_pull_output());
        led_red.off();
        led_green.off();
        led_blue.off();

        let button = Button::new(gpioa.pa8.into_pull_up_input());

        let tx = gpioa.pa9.into_alternate_push_pull();
        let rx = gpioa.pa10.into_floating_input();
        let uart0 = Uart0::new(dp.usart0, tx, rx, 115200, &clocks, &rcu);

        let cs = gpiob.pb2.into_push_pull_output();
        let dc = gpiob.pb0.into_push_pull_output();
        let rst = gpiob.pb1.into_push_pull_output();
        let sck = gpioa.pa5.into_alternate_push_pull();
        let mosi = gpioa.pa7.into_alternate_push_pull();
        let spi0 = Spi0::new_master(dp.spi0, sck, mosi, Prescaler::Div8, &rcu);
        let lcd = Lcd::new(spi0, cs, dc, rst);

        // Rotary encoder on PB10 (CLK), PB11 (DT), PB12 (SW)
        let clk = gpiob.pb10.into_pull_up_input();
        let dt = gpiob.pb11.into_pull_up_input();
        let sw = gpiob.pb12.into_pull_up_input();
        let encoder = RotaryEncoder::new(clk, dt, sw);

        let mut delay = Delay::new(dp.mtime, clocks);
        let usb_composite = UsbComposite::new(dp.usbfs, &rcu, &mut delay);

        Some(CompositeBoard {
            led_red,
            led_green,
            led_blue,
            button,
            uart0,
            lcd,
            encoder,
            usb_composite,
            delay,
            clocks,
        })
    }

    /// Initializes board for USB Mass Storage Class (96 MHz PLL, 48 MHz USBFS, LCD, MicroSD SPI1, UsbMsc).
    pub fn take_msc() -> Option<MscBoard> {
        let dp = Peripherals::take()?;
        let rcu_cfg = RcuConfig {
            use_hxtal: true,
            target_sysclk: 96_000_000,
        };
        let (rcu, clocks) = dp.rcu.freeze(rcu_cfg);

        let gpioa = dp.gpioa.split_a(&rcu);
        let gpiob = dp.gpiob.split_b(&rcu);
        let gpioc = dp.gpioc.split_c(&rcu);

        let mut led_red = Led::new(gpioc.pc13.into_push_pull_output());
        let mut led_green = Led::new(gpioa.pa1.into_push_pull_output());
        let mut led_blue = Led::new(gpioa.pa2.into_push_pull_output());
        led_red.off();
        led_green.off();
        led_blue.off();

        let button = Button::new(gpioa.pa8.into_pull_up_input());

        let tx = gpioa.pa9.into_alternate_push_pull();
        let rx = gpioa.pa10.into_floating_input();
        let uart0 = Uart0::new(dp.usart0, tx, rx, 115200, &clocks, &rcu);

        let cs = gpiob.pb2.into_push_pull_output();
        let dc = gpiob.pb0.into_push_pull_output();
        let rst = gpiob.pb1.into_push_pull_output();
        let sck = gpioa.pa5.into_alternate_push_pull();
        let mosi = gpioa.pa7.into_alternate_push_pull();
        let spi0 = Spi0::new_master(dp.spi0, sck, mosi, Prescaler::Div8, &rcu);
        let lcd = Lcd::new(spi0, cs, dc, rst);

        // Configure SPI1 and SD card control pins (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI)
        let mut sd_cs = gpiob.pb12.into_push_pull_output();
        let _ = sd_cs.set_high();
        let sd_sck = gpiob.pb13.into_alternate_push_pull();
        let sd_miso = gpiob.pb14.into_pull_up_input();
        let sd_mosi = gpiob.pb15.into_alternate_push_pull();
        let spi1 = Spi1::new_master(dp.spi1, sd_sck, sd_miso, sd_mosi, Prescaler::Div256, &rcu);
        let sdcard = SdCard::new(spi1, sd_cs);

        let mut delay = Delay::new(dp.mtime, clocks);
        let usb_msc = UsbMsc::new(dp.usbfs, &rcu, &mut delay);

        Some(MscBoard {
            led_red,
            led_green,
            led_blue,
            button,
            uart0,
            lcd,
            sdcard,
            usb_msc,
            delay,
            clocks,
        })
    }
}

