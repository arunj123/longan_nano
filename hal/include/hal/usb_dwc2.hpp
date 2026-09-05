#pragma once

#include <cstdint>
#include "hal/register.hpp"

namespace hal::usb {

/**
 * @brief Synopsys DesignWare DWC2 OTG Full-Speed USB Controller Register Memory Map.
 * GD32VF103 Base Address: 0x50000000.
 * Dedicated FIFO Packet RAM: 1.25 KB (320 words).
 */
inline constexpr uintptr_t kUsbBase = 0x50000000;

// Global Control & Status Registers
namespace reg {
    inline constexpr uintptr_t GOTGCTL      = kUsbBase + 0x000;
    inline constexpr uintptr_t GOTGINT      = kUsbBase + 0x004;
    inline constexpr uintptr_t GAHBCFG      = kUsbBase + 0x008;
    inline constexpr uintptr_t GUSBCFG      = kUsbBase + 0x00C;
    inline constexpr uintptr_t GRSTCTL      = kUsbBase + 0x010;
    inline constexpr uintptr_t GINTSTS      = kUsbBase + 0x014;
    inline constexpr uintptr_t GINTMSK      = kUsbBase + 0x018;
    inline constexpr uintptr_t GRXSTSR      = kUsbBase + 0x01C;
    inline constexpr uintptr_t GRXSTSP      = kUsbBase + 0x020;
    inline constexpr uintptr_t GRXFSIZ      = kUsbBase + 0x024;
    inline constexpr uintptr_t DIEP0TFLEN   = kUsbBase + 0x028;
    inline constexpr uintptr_t HNPTFLEN     = kUsbBase + 0x028;
    inline constexpr uintptr_t GCCFG        = kUsbBase + 0x038;
    inline constexpr uintptr_t CID          = kUsbBase + 0x03C;

    constexpr uintptr_t dieptflen(uint8_t ep) noexcept {
        return kUsbBase + 0x100 + (ep * 4);
    }

    // Device Mode Registers
    inline constexpr uintptr_t DCFG         = kUsbBase + 0x800;
    inline constexpr uintptr_t DCTL         = kUsbBase + 0x804;
    inline constexpr uintptr_t DSTS         = kUsbBase + 0x808;
    inline constexpr uintptr_t DIEPMSK      = kUsbBase + 0x810;
    inline constexpr uintptr_t DOEPMSK      = kUsbBase + 0x814;
    inline constexpr uintptr_t DAINT        = kUsbBase + 0x818;
    inline constexpr uintptr_t DAINTMSK     = kUsbBase + 0x81C;
    inline constexpr uintptr_t DIEPEMPMSK   = kUsbBase + 0x834;

    // IN Endpoint Registers (ep: 0..3)
    constexpr uintptr_t diepctl(uint8_t ep) noexcept  { return kUsbBase + 0x900 + (ep * 0x20); }
    constexpr uintptr_t diepint(uint8_t ep) noexcept  { return kUsbBase + 0x908 + (ep * 0x20); }
    constexpr uintptr_t dieptsiz(uint8_t ep) noexcept { return kUsbBase + 0x910 + (ep * 0x20); }
    constexpr uintptr_t dtxfsts(uint8_t ep) noexcept  { return kUsbBase + 0x918 + (ep * 0x20); }

    // OUT Endpoint Registers (ep: 0..3)
    constexpr uintptr_t doepctl(uint8_t ep) noexcept  { return kUsbBase + 0xB00 + (ep * 0x20); }
    constexpr uintptr_t doepint(uint8_t ep) noexcept  { return kUsbBase + 0xB08 + (ep * 0x20); }
    constexpr uintptr_t doeptsiz(uint8_t ep) noexcept { return kUsbBase + 0xB10 + (ep * 0x20); }

    // Power and Clock Gating Control
    inline constexpr uintptr_t PCGCCTL      = kUsbBase + 0xE00;

    // FIFO Data Buffer Portal for Endpoint (ep: 0..3)
    constexpr uintptr_t fifo(uint8_t ep) noexcept {
        return kUsbBase + 0x1000 + (ep * 0x1000);
    }
}

// Global Interrupt Status Bits (GINTSTS / GINTMSK)
namespace gint {
    inline constexpr uint32_t CurMod        = 1U << 0;  // Current Mode (0=Device, 1=Host)
    inline constexpr uint32_t Sof           = 1U << 3;  // Start of Frame
    inline constexpr uint32_t RxFlvl        = 1U << 4;  // RxFIFO Non-Empty
    inline constexpr uint32_t GOutNakEff    = 1U << 6;  // Global OUT NAK Effective
    inline constexpr uint32_t GInNakEff     = 1U << 7;  // Global IN Non-periodic NAK Effective
    inline constexpr uint32_t ErlySusp      = 1U << 10; // Early Suspend
    inline constexpr uint32_t UsbSusp       = 1U << 11; // USB Suspend
    inline constexpr uint32_t UsbRst        = 1U << 12; // USB Reset
    inline constexpr uint32_t EnumDone      = 1U << 13; // Enumeration Done (Speed detected)
    inline constexpr uint32_t IsoOutDrop    = 1U << 14; // Isochronous OUT Packet Dropped
    inline constexpr uint32_t EopF          = 1U << 15; // End of Periodic Frame
    inline constexpr uint32_t IEpInt        = 1U << 18; // IN Endpoints Interrupt
    inline constexpr uint32_t OEpInt        = 1U << 19; // OUT Endpoints Interrupt
    inline constexpr uint32_t InCompIsoIn   = 1U << 20; // Incomplete Isochronous IN
    inline constexpr uint32_t InCompIsoOut  = 1U << 21; // Incomplete Isochronous OUT
    inline constexpr uint32_t WkUpInt       = 1U << 31; // Resume/Remote Wakeup Detected
}

// Device Control Bits (DCTL)
namespace dctl {
    inline constexpr uint32_t RWKUP         = 1U << 0;  // Remote Wakeup Signaling
    inline constexpr uint32_t SftDiscon     = 1U << 1;  // Soft Disconnect (1 = Disconnected from host)
    inline constexpr uint32_t CgInNak       = 1U << 8;  // Clear Global IN NAK
    inline constexpr uint32_t SgInNak       = 1U << 7;  // Set Global IN NAK
    inline constexpr uint32_t CgOutNak      = 1U << 10; // Clear Global OUT NAK
    inline constexpr uint32_t SgOutNak      = 1U << 9;  // Set Global OUT NAK
    inline constexpr uint32_t PwrOnPrgDone  = 1U << 11; // Power-On Programming Done
}

// Device Status Bits (DSTS)
namespace dsts {
    inline constexpr uint32_t SuspSts       = 1U << 0;  // Suspend Status
    inline constexpr uint32_t EnumSpdMask   = 3U << 1;  // Enumerated Speed (0b11 = Full Speed 48MHz PHY)
    inline constexpr uint32_t EnumSpdFull   = 3U << 1;
}

// Endpoint Control Bits (DIEPCTLx / DOEPCTLx)
namespace epctl {
    inline constexpr uint32_t EpEna         = 1U << 31; // Endpoint Enable
    inline constexpr uint32_t EpDis         = 1U << 30; // Endpoint Disable
    inline constexpr uint32_t SetD0Pid      = 1U << 28; // Set DATA0 PID
    inline constexpr uint32_t Snak          = 1U << 27; // Set NAK
    inline constexpr uint32_t Cnak          = 1U << 26; // Clear NAK
    inline constexpr uint32_t Stall         = 1U << 21; // Handshake STALL
    inline constexpr uint32_t UsbActEp      = 1U << 15; // USB Active Endpoint
    inline constexpr uint32_t EpTypeControl = 0U << 18;
    inline constexpr uint32_t EpTypeIso     = 1U << 18;
    inline constexpr uint32_t EpTypeBulk    = 2U << 18;
    inline constexpr uint32_t EpTypeInt     = 3U << 18;
}

// Endpoint Interrupt Bits (DIEPINTx / DOEPINTx)
namespace epint {
    inline constexpr uint32_t XferCompl     = 1U << 0;  // Transfer Completed
    inline constexpr uint32_t EpDisbld      = 1U << 1;  // Endpoint Disabled
    inline constexpr uint32_t AhbErr        = 1U << 2;  // AHB Error
    inline constexpr uint32_t TimeOut       = 1U << 3;  // Timeout Handshake
    inline constexpr uint32_t ItxEmp        = 1U << 4;  // IN Token Received when TxFIFO Empty
    inline constexpr uint32_t Setup         = 1U << 3;  // Setup Phase Done (DOEPINT)
    inline constexpr uint32_t Back2BackSet  = 1U << 6;  // Back-to-Back SETUP Received
}

} // namespace hal::usb
