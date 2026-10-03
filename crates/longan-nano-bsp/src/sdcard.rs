use gd32vf103_hal::delay::Delay;
use gd32vf103_hal::gpio::{mode, Pin, PortB};
use gd32vf103_hal::spi::{Prescaler, Spi1};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CardType {
    Unknown,
    SD1,
    SD2SC, // Standard Capacity (byte-addressed)
    SD2HC, // High/Extended Capacity (block-addressed, 512B)
}

impl CardType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            CardType::Unknown => "Unknown",
            CardType::SD1 => "SDv1 / MMC",
            CardType::SD2SC => "SDv2 SC",
            CardType::SD2HC => "SDHC / SDXC",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SdError {
    Timeout,
    NoResponse,
    Cmd0Fail(u8),
    Cmd8Fail(u8),
    Acmd41Timeout,
    Cmd58Fail(u8),
    ReadTokenTimeout,
    WriteError,
    NotInitialized,
}

impl SdError {
    pub const fn as_str(&self) -> &'static str {
        match self {
            SdError::Timeout => "Timeout",
            SdError::NoResponse => "No Response",
            SdError::Cmd0Fail(_) => "CMD0 Failed",
            SdError::Cmd8Fail(_) => "CMD8 Failed",
            SdError::Acmd41Timeout => "ACMD41 Timeout",
            SdError::Cmd58Fail(_) => "CMD58 Failed",
            SdError::ReadTokenTimeout => "Read Token Timeout",
            SdError::WriteError => "Write Error",
            SdError::NotInitialized => "Not Initialized",
        }
    }
}

pub struct SdCard {
    spi: Spi1,
    cs: Pin<PortB, 12, mode::Output<mode::PushPull>>,
    pub card_type: CardType,
    pub sector_count: u32,
    pub is_initialized: bool,
}

impl SdCard {
    pub fn new(spi: Spi1, cs: Pin<PortB, 12, mode::Output<mode::PushPull>>) -> Self {
        Self {
            spi,
            cs,
            card_type: CardType::Unknown,
            sector_count: 0,
            is_initialized: false,
        }
    }

    #[inline(always)]
    fn cs_high(&mut self) {
        let _ = self.cs.set_high();
    }

    #[inline(always)]
    fn cs_low(&mut self) {
        let _ = self.cs.set_low();
    }

    #[inline(always)]
    fn release_bus(&mut self) {
        self.cs_high();
        self.spi.transfer_u8(0xFF);
        self.spi.transfer_u8(0xFF);
    }

    #[inline(always)]
    fn xchg(&mut self, val: u8) -> u8 {
        self.spi.transfer_u8(val)
    }

    fn send_cmd(&mut self, cmd: u8, arg: u32, crc: u8) -> u8 {
        self.xchg(0x40 | (cmd & 0x3F));
        self.xchg((arg >> 24) as u8);
        self.xchg((arg >> 16) as u8);
        self.xchg((arg >> 8) as u8);
        self.xchg(arg as u8);
        self.xchg(crc);

        for _ in 0..16 {
            let r1 = self.xchg(0xFF);
            if (r1 & 0x80) == 0 {
                return r1;
            }
        }
        0xFF
    }

    fn send_acmd(&mut self, cmd: u8, arg: u32) -> u8 {
        self.cs_low();
        let r1 = self.send_cmd(55, 0, 0x01); // CMD55 (APP_CMD)
        self.cs_high();
        self.xchg(0xFF);

        if r1 > 0x01 {
            return r1;
        }

        self.cs_low();
        let r1 = self.send_cmd(cmd, arg, 0x01);
        self.cs_high();
        self.xchg(0xFF);
        r1
    }

    /// Initializes hardware and probes the SD card.
    pub fn init(&mut self, delay: &mut Delay) -> Result<CardType, SdError> {
        self.card_type = CardType::Unknown;
        self.sector_count = 0;
        self.is_initialized = false;

        // Start at slow speed (Prescaler Div256 = ~200-400 kHz)
        self.spi.set_prescaler(Prescaler::Div256);
        self.cs_high();

        delay.delay_ms(10);

        // Power-up sync: Send >= 80 clock cycles with CS=HIGH
        for _ in 0..16 {
            self.xchg(0xFF);
        }

        // Send CMD0 (GO_IDLE_STATE) with CRC=0x95
        let mut r1 = 0xFF;
        for _ in 0..10 {
            self.cs_low();
            r1 = self.send_cmd(0, 0, 0x95);
            self.cs_high();
            self.xchg(0xFF);
            if r1 == 0x01 {
                break;
            }
            delay.delay_ms(5);
        }

        if r1 != 0x01 {
            return Err(if r1 == 0xFF { SdError::NoResponse } else { SdError::Cmd0Fail(r1) });
        }

        // Send CMD8 (SEND_IF_COND) to verify voltage range and SDv2
        self.cs_low();
        r1 = self.send_cmd(8, 0x0000_01AA, 0x87);
        let mut r7 = [0u8; 4];
        if r1 == 0x01 {
            for b in &mut r7 {
                *b = self.xchg(0xFF);
            }
        }
        self.cs_high();
        self.xchg(0xFF);

        let is_v2 = r1 == 0x01 && r7[2] == 0x01 && r7[3] == 0xAA;

        // ACMD41 initialization loop until card exits idle state (R1 == 0x00)
        let acmd41_arg = if is_v2 { 1u32 << 30 } else { 0 }; // HCS bit
        let mut ready = false;
        for _ in 0..150 {
            r1 = self.send_acmd(41, acmd41_arg);
            if r1 == 0x00 {
                ready = true;
                break;
            }
            delay.delay_ms(10);
        }

        if !ready {
            return Err(SdError::Acmd41Timeout);
        }

        // If SDv2, read OCR via CMD58 to check CCS (Card Capacity Status bit 30)
        if is_v2 {
            self.cs_low();
            r1 = self.send_cmd(58, 0, 0x01);
            let mut ocr_bytes = [0u8; 4];
            if r1 == 0x00 {
                for b in &mut ocr_bytes {
                    *b = self.xchg(0xFF);
                }
            }
            self.cs_high();
            self.xchg(0xFF);

            if r1 != 0x00 {
                return Err(SdError::Cmd58Fail(r1));
            }

            let ocr = ((ocr_bytes[0] as u32) << 24)
                | ((ocr_bytes[1] as u32) << 16)
                | ((ocr_bytes[2] as u32) << 8)
                | (ocr_bytes[3] as u32);

            if (ocr & (1 << 30)) != 0 {
                self.card_type = CardType::SD2HC;
            } else {
                self.card_type = CardType::SD2SC;
            }
        } else {
            self.card_type = CardType::SD1;
        }

        // Force block length to 512 bytes for byte-addressed cards via CMD16
        if self.card_type != CardType::SD2HC {
            self.cs_low();
            self.send_cmd(16, 512, 0x01);
            self.cs_high();
            self.xchg(0xFF);
        }

        // Read CSD register (CMD9) to determine capacity
        self.cs_low();
        r1 = self.send_cmd(9, 0, 0x01);
        if r1 == 0x00 {
            let mut csd_token = 0xFF;
            for _ in 0..3000 {
                csd_token = self.xchg(0xFF);
                if csd_token != 0xFF {
                    break;
                }
            }
            if csd_token == 0xFE {
                let mut csd = [0u8; 16];
                for b in &mut csd {
                    *b = self.xchg(0xFF);
                }
                self.xchg(0xFF); // CRC
                self.xchg(0xFF);

                if (csd[0] >> 6) == 1 {
                    // CSD v2.0
                    let csize = (csd[9] as u32)
                        + ((csd[8] as u32) << 8)
                        + (((csd[7] & 0x3F) as u32) << 16)
                        + 1;
                    self.sector_count = csize << 10;
                } else {
                    // CSD v1.0
                    let n = ((csd[5] & 15) as u32)
                        + (((csd[10] & 128) >> 7) as u32)
                        + (((csd[9] & 3) << 1) as u32)
                        + 2;
                    let csize = ((csd[8] >> 6) as u32)
                        + (((csd[7] as u32) << 2))
                        + (((csd[6] & 3) as u32) << 10)
                        + 1;
                    self.sector_count = csize << (n - 9);
                }
            }
        }
        self.cs_high();
        self.xchg(0xFF);

        // Switch to High-Speed SPI mode (Prescaler Div2 = ~24-27 MHz)
        self.spi.set_prescaler(Prescaler::Div2);

        self.is_initialized = true;
        Ok(self.card_type)
    }

    /// Read a 512-byte sector from the SD card.
    pub fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) -> Result<(), SdError> {
        if !self.is_initialized {
            return Err(SdError::NotInitialized);
        }

        let arg = if self.card_type == CardType::SD2HC {
            lba
        } else {
            lba * 512
        };

        self.cs_low();
        self.xchg(0xFF); // 8 clocks sync

        if !self.spi.wait_ready_fast(100 * 1500) {
            self.release_bus();
            return Err(SdError::Timeout);
        }

        // Send CMD17 (READ_SINGLE_BLOCK)
        let r1 = self.send_cmd(17, arg, 0x01);
        if r1 != 0x00 {
            self.release_bus();
            return Err(SdError::NoResponse);
        }

        // Wait for data start token (0xFE)
        let token = self.spi.wait_token_fast(60000);
        if token != 0xFE {
            self.release_bus();
            return Err(SdError::ReadTokenTimeout);
        }

        // Fast block read using pipelined SPI
        self.spi.read_block_fast(buf);

        // Read 2-byte CRC16
        self.xchg(0xFF);
        self.xchg(0xFF);

        self.release_bus();
        Ok(())
    }

    /// Write a 512-byte sector to the SD card.
    pub fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) -> Result<(), SdError> {
        if !self.is_initialized {
            return Err(SdError::NotInitialized);
        }

        let arg = if self.card_type == CardType::SD2HC {
            lba
        } else {
            lba * 512
        };

        self.cs_low();

        if !self.spi.wait_ready_fast(500 * 1500) {
            self.release_bus();
            return Err(SdError::Timeout);
        }

        // Send CMD24 (WRITE_BLOCK)
        let r1 = self.send_cmd(24, arg, 0x01);
        if r1 != 0x00 {
            self.release_bus();
            return Err(SdError::WriteError);
        }

        // Send at least 1 dummy clock before data token
        self.xchg(0xFF);

        // Send Data Start Token (0xFE)
        self.xchg(0xFE);

        // Transmit 512 data bytes
        self.spi.write_block_fast(buf);

        // Send dummy CRC16
        self.xchg(0xFF);
        self.xchg(0xFF);

        // Read Data Response token (xxx00101b = 0x05 -> data accepted)
        let resp = self.xchg(0xFF);
        if (resp & 0x1F) != 0x05 {
            self.release_bus();
            return Err(SdError::WriteError);
        }

        // Wait while card programs flash (MISO held LOW until programming completes)
        if !self.spi.wait_ready_fast(1000 * 1500) {
            self.release_bus();
            return Err(SdError::Timeout);
        }

        self.release_bus();
        Ok(())
    }
}
