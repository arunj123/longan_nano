#pragma once

#include <cstdint>
#include <span>
#include <cstdio>
#include "hal/gpio.hpp"
#include "hal/spi.hpp"
#include "hal/rcu.hpp"
#include "hal/time.hpp"

namespace drivers::sdcard {

enum class CardType : uint8_t {
    Unknown = 0,
    MMC     = 1,
    SD1     = 2,
    SD2SC   = 3, // Standard Capacity (byte-addressed)
    SD2HC   = 4  // High/Extended Capacity (block-addressed, 512-byte blocks)
};

enum class SdResult : uint8_t {
    Success = 0,
    Timeout,
    NoResponse,
    Cmd0Fail,
    Cmd8Fail,
    Acmd41Timeout,
    Cmd58Fail,
    ReadTokenTimeout,
    WriteError,
    NotInitialized
};

[[nodiscard]] inline const char* result_to_string(SdResult res) noexcept {
    switch (res) {
        case SdResult::Success:          return "Success";
        case SdResult::Timeout:          return "Timeout";
        case SdResult::NoResponse:       return "No Response (MISO stuck High 0xFF)";
        case SdResult::Cmd0Fail:         return "CMD0 Failed (Card did not enter SPI idle state)";
        case SdResult::Cmd8Fail:         return "CMD8 Failed (Voltage pattern check mismatch)";
        case SdResult::Acmd41Timeout:    return "ACMD41 Timeout (Card failed to leave idle state)";
        case SdResult::Cmd58Fail:        return "CMD58 Failed (OCR read failed)";
        case SdResult::ReadTokenTimeout: return "Read Token Timeout (Card did not send 0xFE data token)";
        case SdResult::WriteError:       return "Write Error";
        case SdResult::NotInitialized:   return "Not Initialized";
        default:                         return "Unknown Error";
    }
}

[[nodiscard]] inline const char* card_type_to_string(CardType type) noexcept {
    switch (type) {
        case CardType::MMC:   return "MMC (MultiMediaCard)";
        case CardType::SD1:   return "SDSC v1.x (Standard Capacity)";
        case CardType::SD2SC: return "SDSC v2.0 (Standard Capacity, byte-addressed)";
        case CardType::SD2HC: return "SDHC/SDXC v2.0+ (High Capacity, block-addressed)";
        default:              return "Unknown / Unidentified Card";
    }
}

/**
 * @brief Modern C++23 Zero-overhead SD Card SPI Driver for GD32VF103.
 * Configured specifically for Sipeed Longan Nano onboard MicroSD slot (SPI1 on PB12-15).
 */
template <
    typename SpiPeriph = hal::spi::Spi1,
    hal::gpio::Port CsPort = hal::gpio::Port::B, uint8_t CsPinNum = 12,
    hal::gpio::Port SckPort = hal::gpio::Port::B, uint8_t SckPinNum = 13,
    hal::gpio::Port MisoPort = hal::gpio::Port::B, uint8_t MisoPinNum = 14,
    hal::gpio::Port MosiPort = hal::gpio::Port::B, uint8_t MosiPinNum = 15
>
class SdCard {
public:
    using CsPin   = hal::gpio::GpioPin<CsPort, CsPinNum>;
    using SckPin  = hal::gpio::GpioPin<SckPort, SckPinNum>;
    using MisoPin = hal::gpio::GpioPin<MisoPort, MisoPinNum>;
    using MosiPin = hal::gpio::GpioPin<MosiPort, MosiPinNum>;

    static inline CardType card_type{CardType::Unknown};
    static inline uint32_t ocr{0};
    static inline uint32_t sector_count{0};
    static inline bool is_initialized{false};

    static inline void cs_high() noexcept {
        CsPin::set();
    }

    static inline void cs_low() noexcept {
        CsPin::reset();
    }

    static inline uint8_t xchg(uint8_t val = 0xFF) noexcept {
        return SpiPeriph::transfer_8(val);
    }

    /// Wait until card is ready (MISO goes high 0xFF)
    static inline bool wait_ready(uint32_t timeout_ms = 500) noexcept {
        auto deadline = hal::time::Instant::now() + hal::time::Duration::from_ms(timeout_ms);
        do {
            if (xchg(0xFF) == 0xFF) return true;
        } while (hal::time::Instant::now() < deadline);
        return false;
    }

    /// Send a 6-byte SD command packet and wait for R1 response
    static inline uint8_t send_cmd(uint8_t cmd, uint32_t arg, uint8_t crc = 0x01) noexcept {
        // Send command token
        xchg(static_cast<uint8_t>(0x40 | (cmd & 0x3F)));
        xchg(static_cast<uint8_t>(arg >> 24));
        xchg(static_cast<uint8_t>(arg >> 16));
        xchg(static_cast<uint8_t>(arg >> 8));
        xchg(static_cast<uint8_t>(arg));
        xchg(crc);

        // Wait for R1 response (bit 7 must be 0)
        // Give the card up to 16 attempts
        uint8_t r1 = 0xFF;
        for (int i = 0; i < 16; ++i) {
            r1 = xchg(0xFF);
            if ((r1 & 0x80) == 0) {
                return r1;
            }
        }
        return r1;
    }

    /// Send an application-specific command (ACMD) preceded by CMD55
    static inline uint8_t send_acmd(uint8_t cmd, uint32_t arg) noexcept {
        cs_low();
        uint8_t r1 = send_cmd(55, 0, 0x01); // CMD55 (APP_CMD)
        cs_high();
        xchg(0xFF); // 8 clocks

        if (r1 > 0x01) {
            return r1;
        }

        cs_low();
        r1 = send_cmd(cmd, arg, 0x01);
        cs_high();
        xchg(0xFF); // 8 clocks
        return r1;
    }

    /**
     * @brief Initialize hardware and probe SD card.
     * @param verbose Print step-by-step diagnostic information to stdout.
     */
    static SdResult init(bool verbose = true) noexcept {
        card_type = CardType::Unknown;
        ocr = 0;
        is_initialized = false;

        if (verbose) {
            printf("\n========================================\n");
            printf("--- Modern C++23 SD Card Probe (SPI1) ---\n");
            printf("========================================\n");
        }

        // 1. Enable peripheral clocks
        hal::rcu::enable(hal::rcu::Peripheral::GpioB);
        hal::rcu::enable(hal::rcu::Peripheral::Spi1);
        hal::rcu::enable(hal::rcu::Peripheral::Afio);

        // 2. Configure GPIO pins
        // PB12: CS (Output Push-Pull, default HIGH)
        CsPin::init(hal::gpio::Mode::OutputPushPull, hal::gpio::Speed::Speed50MHz);
        cs_high();

        // PB13: SCK (Alternate Function Push-Pull)
        SckPin::init(hal::gpio::Mode::AlternatePushPull, hal::gpio::Speed::Speed50MHz);

        // PB14: MISO (Input with pull-up so floating bus reads 0xFF)
        MisoPin::init(hal::gpio::Mode::InputPullUp);

        // PB15: MOSI (Alternate Function Push-Pull)
        MosiPin::init(hal::gpio::Mode::AlternatePushPull, hal::gpio::Speed::Speed50MHz);

        // 3. Initialize SPI1 at slow speed (Prescaler Div256 = ~200-400 kHz)
        // SD cards support SPI Mode 0 and Mode 3. We use Mode 3 (CPOL=1, CPHA=1)
        SpiPeriph::init_master(
            hal::spi::Prescaler::Div256,
            hal::spi::ClockPolarity::High,
            hal::spi::ClockPhase::Edge2,
            hal::spi::FrameFormat::Bits8
        );

        hal::time::delay_ms(10);

        // 4. Power-up sequence: Send >= 80 clock cycles with CS HIGH and MOSI HIGH
        if (verbose) {
            printf("[SD:Step 1] Power-up sync: Sending 80+ clock cycles with CS=HIGH...\n");
        }
        cs_high();
        for (int i = 0; i < 16; ++i) {
            xchg(0xFF);
        }

        // 5. Send CMD0 (GO_IDLE_STATE) with CRC=0x95
        if (verbose) {
            printf("[SD:Step 2] Entering SPI mode: Sending CMD0 (GO_IDLE_STATE)...\n");
        }

        uint8_t r1 = 0xFF;
        // Some cards take a few retries after power-up
        for (int retry = 0; retry < 10; ++retry) {
            cs_low();
            r1 = send_cmd(0, 0, 0x95);
            cs_high();
            xchg(0xFF); // 8 dummy clocks

            if (r1 == 0x01) {
                break;
            }
            hal::time::delay_ms(5);
        }

        if (verbose) {
            printf("             CMD0 Response: 0x%02X (Expected: 0x01)\n", r1);
        }

        if (r1 != 0x01) {
            if (verbose) {
                if (r1 == 0xFF) {
                    printf("             [DIAGNOSIS] Received 0xFF (No Response).\n");
                    printf("             Possible causes:\n");
                    printf("             1. No SD card inserted or not fully seated.\n");
                    printf("             2. Loose physical contact inside the metal TF cage.\n");
                    printf("             3. Card pins need gentle push / re-insertion.\n");
                } else {
                    printf("             [DIAGNOSIS] Card replied with unexpected status 0x%02X.\n", r1);
                }
            }
            return SdResult::Cmd0Fail;
        }

        // 6. Send CMD8 (SEND_IF_COND) to verify SD version and voltage range
        if (verbose) {
            printf("[SD:Step 3] Sending CMD8 (SEND_IF_COND, arg=0x000001AA, crc=0x87)...\n");
        }

        cs_low();
        r1 = send_cmd(8, 0x000001AA, 0x87);
        uint8_t r7[4] = {0};
        if (r1 == 0x01) {
            for (int i = 0; i < 4; ++i) {
                r7[i] = xchg(0xFF);
            }
        }
        cs_high();
        xchg(0xFF);

        if (verbose) {
            printf("             CMD8 Response: R1=0x%02X, Data: [0x%02X, 0x%02X, 0x%02X, 0x%02X]\n",
                   r1, r7[0], r7[1], r7[2], r7[3]);
        }

        bool is_v2 = false;
        if (r1 == 0x01 && r7[2] == 0x01 && r7[3] == 0xAA) {
            is_v2 = true;
            if (verbose) {
                printf("             --> Confirmed: SD Card v2.0+ (2.7V - 3.6V valid, pattern 0x1AA OK)\n");
            }
        } else if (r1 & 0x04) {
            // Illegal command -> SDv1 or MMC
            if (verbose) {
                printf("             --> CMD8 rejected as illegal command: Legacy SDv1 / MMC card.\n");
            }
        }

        // 7. Send ACMD41 loop until card exits idle state (R1 == 0x00)
        if (verbose) {
            printf("[SD:Step 4] Initializing card operating conditions via ACMD41...\n");
        }

        uint32_t acmd41_arg = is_v2 ? (1UL << 30) : 0; // Set HCS (High Capacity Support) bit 30
        auto deadline = hal::time::Instant::now() + hal::time::Duration::from_ms(1500);
        bool ready = false;
        int attempts = 0;

        while (hal::time::Instant::now() < deadline) {
            ++attempts;
            r1 = send_acmd(41, acmd41_arg);
            if (r1 == 0x00) {
                ready = true;
                break;
            }
            hal::time::delay_ms(10);
        }

        if (verbose) {
            printf("             ACMD41 Attempts: %d, Final R1: 0x%02X\n", attempts, r1);
        }

        if (!ready) {
            if (verbose) {
                printf("             [ERROR] ACMD41 timed out waiting for card ready state!\n");
            }
            return SdResult::Acmd41Timeout;
        }

        // 8. If SDv2, send CMD58 (READ_OCR) to check CCS (Card Capacity Status bit 30)
        if (is_v2) {
            if (verbose) {
                printf("[SD:Step 5] Reading OCR register via CMD58...\n");
            }

            cs_low();
            r1 = send_cmd(58, 0, 0x01);
            uint8_t ocr_bytes[4] = {0};
            if (r1 == 0x00) {
                for (int i = 0; i < 4; ++i) {
                    ocr_bytes[i] = xchg(0xFF);
                }
            }
            cs_high();
            xchg(0xFF);

            ocr = (static_cast<uint32_t>(ocr_bytes[0]) << 24) |
                  (static_cast<uint32_t>(ocr_bytes[1]) << 16) |
                  (static_cast<uint32_t>(ocr_bytes[2]) << 8)  |
                  static_cast<uint32_t>(ocr_bytes[3]);

            if (verbose) {
                printf("             CMD58 Response: R1=0x%02X, OCR: 0x%08lX\n", r1, static_cast<unsigned long>(ocr));
            }

            if (r1 != 0x00) {
                return SdResult::Cmd58Fail;
            }

            // Bit 30 of OCR is CCS (Card Capacity Status)
            if (ocr & (1UL << 30)) {
                card_type = CardType::SD2HC;
            } else {
                card_type = CardType::SD2SC;
            }
        } else {
            card_type = CardType::SD1;
        }

        // 9. If standard capacity card (byte-addressed), force block size to 512 bytes with CMD16
        if (card_type != CardType::SD2HC) {
            if (verbose) {
                printf("[SD:Step 5b] Setting block length to 512 bytes via CMD16...\n");
            }
            cs_low();
            r1 = send_cmd(16, 512, 0x01);
            cs_high();
            xchg(0xFF);
            if (r1 != 0x00) {
                printf("             [WARNING] CMD16 returned R1=0x%02X\n", r1);
            }
        }

        // 10. Read CSD register (CMD9) to determine card capacity
        cs_low();
        r1 = send_cmd(9, 0, 0x01);
        if (r1 == 0x00) {
            auto csd_deadline = hal::time::Instant::now() + hal::time::Duration::from_ms(300);
            uint8_t csd_token = 0xFF;
            do {
                csd_token = xchg(0xFF);
                if (csd_token != 0xFF) break;
            } while (hal::time::Instant::now() < csd_deadline);

            if (csd_token == 0xFE) {
                uint8_t csd[16];
                for (size_t i = 0; i < 16; ++i) {
                    csd[i] = xchg(0xFF);
                }
                (void)xchg(0xFF); // CRC
                (void)xchg(0xFF);

                if ((csd[0] >> 6) == 1) { // CSD v2.0 (SDHC/SDXC)
                    uint32_t csize = static_cast<uint32_t>(csd[9]) +
                                     (static_cast<uint32_t>(csd[8]) << 8) +
                                     (static_cast<uint32_t>(csd[7] & 0x3F) << 16) + 1;
                    sector_count = csize << 10;
                } else { // CSD v1.0 (SDSC/MMC)
                    uint32_t n = static_cast<uint32_t>(csd[5] & 15) +
                                 static_cast<uint32_t>((csd[10] & 128) >> 7) +
                                 static_cast<uint32_t>((csd[9] & 3) << 1) + 2U;
                    uint32_t csize = (csd[8] >> 6) + (static_cast<uint32_t>(csd[7]) << 2) + (static_cast<uint32_t>(csd[6] & 3) << 10) + 1;
                    sector_count = csize << (n - 9);
                }
                if (verbose) {
                    printf("             Capacity: %lu sectors (~%lu MB)\n",
                           static_cast<unsigned long>(sector_count),
                           static_cast<unsigned long>(sector_count / 2048));
                }
            }
        }
        cs_high();
        xchg(0xFF);

        // 11. Switch SPI1 to high-speed transfer mode (Prescaler Div4 -> ~13.5 MHz)
        if (verbose) {
            printf("[SD:Step 6] Switching SPI1 to High-Speed mode (Prescaler Div4 = ~13.5 MHz)...\n");
        }
        SpiPeriph::set_prescaler(hal::spi::Prescaler::Div4);

        is_initialized = true;
        if (verbose) {
            printf("========================================\n");
            printf(">>> SD Card INITIALIZATION COMPLETE! <<<\n");
            printf("    Card Type : %s\n", card_type_to_string(card_type));
            printf("    Addressing: %s\n", (card_type == CardType::SD2HC) ? "Block-addressed (512B)" : "Byte-addressed");
            printf("========================================\n\n");
        }

        return SdResult::Success;
    }

    /**
     * @brief Read a 512-byte sector from the SD card.
     * @param sector 32-bit sector index (LBA).
     * @param buffer Output buffer of at least 512 bytes.
     */
    static SdResult read_sector(uint32_t sector, std::span<uint8_t> buffer) noexcept {
        if (!is_initialized) {
            return SdResult::NotInitialized;
        }
        if (buffer.size() < 512) {
            return SdResult::WriteError;
        }

        // Calculate argument:
        // SDHC/SDXC uses block address directly.
        // SDSC uses byte address (sector * 512).
        uint32_t arg = (card_type == CardType::SD2HC) ? sector : (sector * 512);

        cs_low();

        // Wait for card to be ready
        if (!wait_ready(500)) {
            cs_high();
            xchg(0xFF);
            return SdResult::Timeout;
        }

        // Send CMD17 (READ_SINGLE_BLOCK)
        uint8_t r1 = send_cmd(17, arg, 0x01);
        if (r1 != 0x00) {
            cs_high();
            xchg(0xFF);
            return SdResult::NoResponse;
        }

        // Wait for data start token (0xFE)
        auto deadline = hal::time::Instant::now() + hal::time::Duration::from_ms(300);
        uint8_t token = 0xFF;
        do {
            token = xchg(0xFF);
            if (token != 0xFF) break;
        } while (hal::time::Instant::now() < deadline);

        if (token != 0xFE) {
            cs_high();
            xchg(0xFF);
            return SdResult::ReadTokenTimeout;
        }

        // Read 512 data bytes
        for (size_t i = 0; i < 512; ++i) {
            buffer[i] = xchg(0xFF);
        }

        // Read 2-byte CRC16 (ignored in SPI mode)
        (void)xchg(0xFF);
        (void)xchg(0xFF);

        cs_high();
        xchg(0xFF); // 8 dummy clocks after transfer

        return SdResult::Success;
    }

    /**
     * @brief Write a 512-byte sector to the SD card.
     * @param sector 32-bit sector index (LBA).
     * @param buffer Input buffer of 512 bytes.
     */
    static SdResult write_sector(uint32_t sector, std::span<const uint8_t> buffer) noexcept {
        if (!is_initialized) {
            return SdResult::NotInitialized;
        }
        if (buffer.size() < 512) {
            return SdResult::WriteError;
        }

        uint32_t arg = (card_type == CardType::SD2HC) ? sector : (sector * 512);

        cs_low();

        if (!wait_ready(500)) {
            cs_high();
            xchg(0xFF);
            return SdResult::Timeout;
        }

        // Send CMD24 (WRITE_BLOCK)
        uint8_t r1 = send_cmd(24, arg, 0x01);
        if (r1 != 0x00) {
            cs_high();
            xchg(0xFF);
            return SdResult::WriteError;
        }

        // Send at least 1 dummy clock before data token
        xchg(0xFF);

        // Send Data Start Token (0xFE)
        xchg(0xFE);

        // Transmit 512 data bytes
        for (size_t i = 0; i < 512; ++i) {
            xchg(buffer[i]);
        }

        // Send dummy CRC16
        xchg(0xFF);
        xchg(0xFF);

        // Read Data Response token (xxx00101b = 0x05 -> data accepted)
        uint8_t resp = xchg(0xFF);
        if ((resp & 0x1F) != 0x05) {
            cs_high();
            xchg(0xFF);
            return SdResult::WriteError;
        }

        // Wait while card programs flash (MISO held LOW until programming completes)
        if (!wait_ready(1000)) {
            cs_high();
            xchg(0xFF);
            return SdResult::Timeout;
        }

        cs_high();
        xchg(0xFF); // 8 dummy clocks after transfer

        return SdResult::Success;
    }

    /**
     * @brief Multi-sector read helper.
     * @param sector 32-bit starting sector index (LBA).
     * @param buff Destination buffer pointer.
     * @param count Number of 512-byte sectors to read.
     */
    static SdResult read_sectors(uint32_t sector, uint8_t* buff, uint32_t count) noexcept {
        if (!is_initialized) return SdResult::NotInitialized;
        for (uint32_t i = 0; i < count; ++i) {
            std::span<uint8_t, 512> block_span{buff + (i * 512), 512};
            auto res = read_sector(sector + i, block_span);
            if (res != SdResult::Success) return res;
        }
        return SdResult::Success;
    }

    /**
     * @brief Multi-sector write helper.
     * @param sector 32-bit starting sector index (LBA).
     * @param buff Source buffer pointer.
     * @param count Number of 512-byte sectors to write.
     */
    static SdResult write_sectors(uint32_t sector, const uint8_t* buff, uint32_t count) noexcept {
        if (!is_initialized) return SdResult::NotInitialized;
        for (uint32_t i = 0; i < count; ++i) {
            std::span<const uint8_t, 512> block_span{buff + (i * 512), 512};
            auto res = write_sector(sector + i, block_span);
            if (res != SdResult::Success) return res;
        }
        return SdResult::Success;
    }
};

} // namespace drivers::sdcard
