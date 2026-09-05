#pragma once

#include <cstdint>
#include "hal/register.hpp"

namespace hal::reg {

// =============================================================================
// GD32VF103CBT6 Physical Memory Map & Peripheral Base Addresses
// RV32IMAC @ up to 108 MHz, 32 KB SRAM, 128 KB Flash (LQFP48 Package)
// =============================================================================

inline constexpr uintptr_t kFlashBase       = 0x08000000;
inline constexpr uintptr_t kSramBase        = 0x20000000;
inline constexpr uintptr_t kPeriphBase      = 0x40000000;

// Bus Bases
inline constexpr uintptr_t kApb1Base        = kPeriphBase + 0x00000;
inline constexpr uintptr_t kApb2Base        = kPeriphBase + 0x10000;
inline constexpr uintptr_t kAhbBase         = kPeriphBase + 0x20000;

// APB1 Peripherals
inline constexpr uintptr_t kTimer1Base      = kApb1Base + 0x0000;
inline constexpr uintptr_t kTimer2Base      = kApb1Base + 0x0400;
inline constexpr uintptr_t kTimer3Base      = kApb1Base + 0x0800;
inline constexpr uintptr_t kTimer4Base      = kApb1Base + 0x0C00;
inline constexpr uintptr_t kRtcBase         = kApb1Base + 0x2800;
inline constexpr uintptr_t kWwdgtBase       = kApb1Base + 0x2C00;
inline constexpr uintptr_t kFwdgtBase       = kApb1Base + 0x3000;
inline constexpr uintptr_t kSpi1Base        = kApb1Base + 0x3800;
inline constexpr uintptr_t kUsart1Base      = kApb1Base + 0x4400;
inline constexpr uintptr_t kUsart2Base      = kApb1Base + 0x4800;
inline constexpr uintptr_t kI2c0Base        = kApb1Base + 0x5400;
inline constexpr uintptr_t kI2c1Base        = kApb1Base + 0x5800;
inline constexpr uintptr_t kBkpBase         = kApb1Base + 0x6C00;
inline constexpr uintptr_t kPmuBase         = kApb1Base + 0x7000;

// APB2 Peripherals
inline constexpr uintptr_t kAfioBase        = kApb2Base + 0x0000;
inline constexpr uintptr_t kExtiBase        = kApb2Base + 0x0400;
inline constexpr uintptr_t kGpioABase       = kApb2Base + 0x0800;
inline constexpr uintptr_t kGpioBBase       = kApb2Base + 0x0C00;
inline constexpr uintptr_t kGpioCBase       = kApb2Base + 0x1000;
inline constexpr uintptr_t kAdc0Base        = kApb2Base + 0x2400;
inline constexpr uintptr_t kAdc1Base        = kApb2Base + 0x2800;
inline constexpr uintptr_t kTimer0Base      = kApb2Base + 0x2C00;
inline constexpr uintptr_t kSpi0Base        = kApb2Base + 0x3000;
inline constexpr uintptr_t kUsart0Base      = kApb2Base + 0x3800;

// AHB Peripherals
inline constexpr uintptr_t kDma0Base        = kAhbBase + 0x0000;
inline constexpr uintptr_t kDma1Base        = kAhbBase + 0x0400;
inline constexpr uintptr_t kRcuBase         = kAhbBase + 0x1000;
inline constexpr uintptr_t kFmcBase         = kAhbBase + 0x2000;
inline constexpr uintptr_t kCrcBase         = kAhbBase + 0x3000;

// USB Full-Speed OTG Core (Synopsys DWC2)
inline constexpr uintptr_t kUsbfsBase       = 0x50000000;

// Nuclei Bumblebee N200 Core Peripherals
inline constexpr uintptr_t kCoreTimerBase   = 0xD1000000; // 64-bit mtime/mtimecmp
inline constexpr uintptr_t kEclicBase       = 0xD2000000; // Enhanced Core-Local Interrupt Controller

// =============================================================================
// RCU Clock & Reset Registers & Bitfields
// =============================================================================
namespace rcu {
    inline constexpr uintptr_t CTL          = kRcuBase + 0x00;
    inline constexpr uintptr_t CFG0         = kRcuBase + 0x04;
    inline constexpr uintptr_t INT          = kRcuBase + 0x08;
    inline constexpr uintptr_t APB2RST      = kRcuBase + 0x0C;
    inline constexpr uintptr_t APB1RST      = kRcuBase + 0x10;
    inline constexpr uintptr_t AHBEN        = kRcuBase + 0x14;
    inline constexpr uintptr_t APB2EN       = kRcuBase + 0x18;
    inline constexpr uintptr_t APB1EN       = kRcuBase + 0x1C;
    inline constexpr uintptr_t BDCTL        = kRcuBase + 0x20;
    inline constexpr uintptr_t RSTSCK       = kRcuBase + 0x24;
    inline constexpr uintptr_t AHBRST       = kRcuBase + 0x28;
    inline constexpr uintptr_t CFG1         = kRcuBase + 0x2C;

    // CTL Register Bits
    inline constexpr uint32_t CTL_IRC8MEN   = 1U << 0;
    inline constexpr uint32_t CTL_IRC8MSTB  = 1U << 1;
    inline constexpr uint32_t CTL_HXTALEN   = 1U << 16;
    inline constexpr uint32_t CTL_HXTALSTB  = 1U << 17;
    inline constexpr uint32_t CTL_HXTALBPS  = 1U << 18;
    inline constexpr uint32_t CTL_CKMEN     = 1U << 19;
    inline constexpr uint32_t CTL_PLLEN     = 1U << 24;
    inline constexpr uint32_t CTL_PLLSTB    = 1U << 25;
    inline constexpr uint32_t CTL_PLL1EN    = 1U << 26;
    inline constexpr uint32_t CTL_PLL2EN    = 1U << 28;

    // CFG0 Register Bits
    inline constexpr uint32_t CFG0_SCS_IRC8M    = 0x0U;
    inline constexpr uint32_t CFG0_SCS_HXTAL    = 0x1U;
    inline constexpr uint32_t CFG0_SCS_PLL      = 0x2U;
    inline constexpr uint32_t CFG0_SCS_MASK     = 0x3U;
    inline constexpr uint32_t CFG0_SCSS_PLL     = 0x2U << 2;

    inline constexpr uint32_t CFG0_AHB_DIV1     = 0x0U << 4;
    inline constexpr uint32_t CFG0_APB1_DIV2    = 0x4U << 8;  // Max 54 MHz
    inline constexpr uint32_t CFG0_APB2_DIV1    = 0x0U << 11; // Max 108 MHz

    inline constexpr uint32_t CFG0_PLLSEL_HXTAL = 1U << 16;
    inline constexpr uint32_t CFG0_PLLMF_MASK   = 0xFU << 18;
    inline constexpr uint32_t CFG0_PLLMF4       = 1U << 29;

    inline constexpr uint32_t CFG0_USB_DIV1_5   = 0x0U << 22; // PLL / 1.5 (for 72 MHz)
    inline constexpr uint32_t CFG0_USB_DIV1     = 0x1U << 22; // PLL / 1   (for 48 MHz)
    inline constexpr uint32_t CFG0_USB_DIV2     = 0x2U << 22; // PLL / 2   (for 96 MHz)
    inline constexpr uint32_t CFG0_USB_DIV2_5   = 0x3U << 22; // PLL / 2.5 (for 120 MHz)

    // CFG1 Register Bits (Clock Pre-dividers)
    inline constexpr uint32_t CFG1_PREDV0_DIV2  = 0x1U;
    inline constexpr uint32_t CFG1_PREDV0SRC_HXTAL = 0x0U;
}

// =============================================================================
// RTC Registers
// =============================================================================
namespace rtc {
    inline constexpr uintptr_t INTEN        = kRtcBase + 0x00;
    inline constexpr uintptr_t CTL          = kRtcBase + 0x04;
    inline constexpr uintptr_t PSCH         = kRtcBase + 0x08;
    inline constexpr uintptr_t PSCL         = kRtcBase + 0x0C;
    inline constexpr uintptr_t DIVH         = kRtcBase + 0x10;
    inline constexpr uintptr_t DIVL         = kRtcBase + 0x14;
    inline constexpr uintptr_t CNTH         = kRtcBase + 0x18;
    inline constexpr uintptr_t CNTL         = kRtcBase + 0x1C;
    inline constexpr uintptr_t ALRMH        = kRtcBase + 0x20;
    inline constexpr uintptr_t ALRML        = kRtcBase + 0x24;

    inline constexpr uint32_t CTL_SCIF      = 1U << 0; // Second interrupt flag
    inline constexpr uint32_t CTL_ALRMIF    = 1U << 1; // Alarm interrupt flag
    inline constexpr uint32_t CTL_OVIF      = 1U << 2; // Overflow interrupt flag
    inline constexpr uint32_t CTL_RSYNF     = 1U << 3; // Registers synchronized flag
    inline constexpr uint32_t CTL_CMF       = 1U << 4; // Configuration mode flag
    inline constexpr uint32_t CTL_LWOFF     = 1U << 5; // Last write operation finished flag
}

} // namespace hal::reg
