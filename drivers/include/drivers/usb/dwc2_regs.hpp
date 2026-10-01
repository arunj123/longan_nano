#pragma once

#include <cstdint>
#include <cstddef>

/**
 * @file dwc2_regs.hpp
 * @brief Zero-overhead, strongly typed MMIO register layout for Synopsys DWC2 OTG FS
 *        on GD32VF103 (Longan Nano).
 *
 * Provides direct compile-time address calculations with zero RAM footprint
 * and zero pointer indirection overhead.
 */

namespace drivers::usb::dwc2 {

inline constexpr uintptr_t USBFS_BASE = 0x50000000UL;

// -----------------------------------------------------------------------------
// DWC2 Core Global Registers (0x50000000 - 0x500003FF)
// -----------------------------------------------------------------------------
struct CoreGlobalRegisters {
    volatile uint32_t GOTGCS;               // 0x000 OTG Control and Status
    volatile uint32_t GOTGINTF;             // 0x004 OTG Interrupt Flag
    volatile uint32_t GAHBCS;               // 0x008 Core AHB Configuration
    volatile uint32_t GUSBCS;               // 0x00C Core USB Configuration
    volatile uint32_t GRSTCTL;              // 0x010 Core Reset Control
    volatile uint32_t GINTF;                // 0x014 Core Interrupt Flag
    volatile uint32_t GINTEN;               // 0x018 Core Interrupt Mask/Enable
    volatile uint32_t GRSTATR;              // 0x01C Receive Status Debug Read
    volatile uint32_t GRSTATP;              // 0x020 Receive Status Pop
    volatile uint32_t GRFLEN;               // 0x024 Receive FIFO Size
    volatile uint32_t DIEP0TFLEN_HNPTFLEN;  // 0x028 EP0 / Non-Periodic TX FIFO Size
    volatile uint32_t HNPTFQSTAT;           // 0x02C Non-Periodic TX FIFO Status
    uint32_t          reserved0[2];         // 0x030 - 0x034
    volatile uint32_t GCCFG;                // 0x038 Global Core Configuration
    volatile uint32_t CID;                  // 0x03C Core ID
    uint32_t          reserved1[48];        // 0x040 - 0x0FF
    volatile uint32_t HPTFLEN;              // 0x100 Host Periodic TX FIFO Size
    volatile uint32_t DIEPTFLEN[15];        // 0x104 - 0x13C Device IN EP TX FIFO Sizes (index 0 = EP1)
};

// -----------------------------------------------------------------------------
// DWC2 Device Global Registers (0x50000800 - 0x500008FF)
// -----------------------------------------------------------------------------
struct DeviceRegisters {
    volatile uint32_t DCFG;                 // 0x800 Device Configuration
    volatile uint32_t DCTL;                 // 0x804 Device Control
    volatile uint32_t DSTAT;                // 0x808 Device Status
    uint32_t          reserved0;            // 0x80C
    volatile uint32_t DIEPINTEN;            // 0x810 Device IN EP Common Interrupt Mask
    volatile uint32_t DOEPINTEN;            // 0x814 Device OUT EP Common Interrupt Mask
    volatile uint32_t DAEPINT;              // 0x818 Device All Endpoints Interrupt
    volatile uint32_t DAEPINTEN;            // 0x81C Device All Endpoints Interrupt Mask
    uint32_t          reserved1[2];         // 0x820 - 0x824
    volatile uint32_t DVBUSDT;              // 0x828 Device VBUS Discharge Time
    volatile uint32_t DVBUSPT;              // 0x82C Device VBUS Pulsing Time
    volatile uint32_t DTHRCTL;              // 0x830 Device Threshold Control
    volatile uint32_t DIEPFEINTEN;          // 0x834 Device IN EP FIFO Empty Interrupt Mask
    volatile uint32_t DEP1INT;              // 0x838 Device Dedicated EP1 Interrupt
    volatile uint32_t DEP1INTEN;            // 0x83C Device Dedicated EP1 Interrupt Mask
    uint32_t          reserved2;            // 0x840
    volatile uint32_t DIEP1INTEN;           // 0x844 Device IN EP1 Interrupt Mask
    uint32_t          reserved3[15];        // 0x848 - 0x880
    volatile uint32_t DOEP1INTEN;           // 0x884 Device OUT EP1 Interrupt Mask
};

// -----------------------------------------------------------------------------
// DWC2 Device IN Endpoint Registers (0x50000900 + EpNum * 0x20)
// -----------------------------------------------------------------------------
struct InEndpointRegisters {
    volatile uint32_t DIEPCTL;              // +0x00 IN EP Control
    uint32_t          reserved0;            // +0x04
    volatile uint32_t DIEPINTF;             // +0x08 IN EP Interrupt Flag
    uint32_t          reserved1;            // +0x0C
    volatile uint32_t DIEPLEN;              // +0x10 IN EP Transfer Length
    uint32_t          reserved2;            // +0x14
    volatile uint32_t DIEPTFSTAT;           // +0x18 IN EP TX FIFO Space Remaining (words)
    uint32_t          reserved3;            // +0x1C
};

// -----------------------------------------------------------------------------
// DWC2 Device OUT Endpoint Registers (0x50000B00 + EpNum * 0x20)
// -----------------------------------------------------------------------------
struct OutEndpointRegisters {
    volatile uint32_t DOEPCTL;              // +0x00 OUT EP Control
    uint32_t          reserved0;            // +0x04
    volatile uint32_t DOEPINTF;             // +0x08 OUT EP Interrupt Flag
    uint32_t          reserved1;            // +0x0C
    volatile uint32_t DOEPLEN;              // +0x10 OUT EP Transfer Length
    uint32_t          reserved2;            // +0x14
    uint32_t          reserved3[2];         // +0x18 - +0x1C
};

// -----------------------------------------------------------------------------
// Register Bit Definitions & Masks
// -----------------------------------------------------------------------------

// Core Global (GAHBCS / GUSBCS / GRSTCTL / GCCFG)
inline constexpr uint32_t GAHBCS_GINTEN       = 1U << 0;

inline constexpr uint32_t GUSBCS_FDM          = 1U << 30;
inline constexpr uint32_t GUSBCS_FHM          = 1U << 29;
inline constexpr uint32_t GUSBCS_EMBPHY       = 1U << 6;
inline constexpr uint32_t GUSBCS_UTT          = 0xFU << 10;

inline constexpr uint32_t GRSTCTL_CSRST       = 1U << 0;
inline constexpr uint32_t GRSTCTL_TXFF        = 1U << 5;
inline constexpr uint32_t GRSTCTL_RXFF        = 1U << 4;

inline constexpr uint32_t GCCFG_PWRON         = 1U << 16;
inline constexpr uint32_t GCCFG_VBUSACEN      = 1U << 18;
inline constexpr uint32_t GCCFG_VBUSBCEN      = 1U << 19;
inline constexpr uint32_t GCCFG_SOFOEN        = 1U << 20;
inline constexpr uint32_t GCCFG_VBUSIG        = 1U << 21;

// Global Interrupt Flags & Enables (GINTF / GINTEN)
inline constexpr uint32_t GINTF_COPM          = 1U << 0;
inline constexpr uint32_t GINTF_SOF           = 1U << 3;
inline constexpr uint32_t GINTF_RXFNEIF       = 1U << 4;
inline constexpr uint32_t GINTF_SP            = 1U << 11;
inline constexpr uint32_t GINTF_RST           = 1U << 12;
inline constexpr uint32_t GINTF_ENUMFIF       = 1U << 13;
inline constexpr uint32_t GINTF_IEPIF         = 1U << 18;
inline constexpr uint32_t GINTF_OEPIF         = 1U << 19;
inline constexpr uint32_t GINTF_ISOINCIF      = 1U << 20;
inline constexpr uint32_t GINTF_ISOONCIF      = 1U << 21;
inline constexpr uint32_t GINTF_WKUPIF        = 1U << 31;

inline constexpr uint32_t GINTEN_SOFIE        = 1U << 3;
inline constexpr uint32_t GINTEN_RXFNEIE      = 1U << 4;
inline constexpr uint32_t GINTEN_SPIE         = 1U << 11;
inline constexpr uint32_t GINTEN_RSTIE        = 1U << 12;
inline constexpr uint32_t GINTEN_ENUMFIE      = 1U << 13;
inline constexpr uint32_t GINTEN_IEPIE        = 1U << 18;
inline constexpr uint32_t GINTEN_OEPIE        = 1U << 19;
inline constexpr uint32_t GINTEN_ISOINCIE     = 1U << 20;
inline constexpr uint32_t GINTEN_ISOONCIE     = 1U << 21;
inline constexpr uint32_t GINTEN_WKUPIE       = 1U << 31;

// Receive Status Pop (GRSTATP)
inline constexpr uint32_t GRSTATRP_EPNUM      = 0xFU << 0;
inline constexpr uint32_t GRSTATRP_BCOUNT     = 0x7FFU << 4;
inline constexpr uint32_t GRSTATRP_DPID       = 0x3U << 15;
inline constexpr uint32_t GRSTATRP_RPCKST     = 0xFU << 17;

inline constexpr uint32_t RSTAT_GOUT_NAK      = 1U;
inline constexpr uint32_t RSTAT_DATA_UPDT     = 2U;
inline constexpr uint32_t RSTAT_XFER_COMP     = 3U;
inline constexpr uint32_t RSTAT_SETUP_COMP    = 4U;
inline constexpr uint32_t RSTAT_SETUP_UPDT    = 6U;

// Device Configuration & Status (DCFG / DCTL / DSTAT)
inline constexpr uint32_t DCFG_DS             = 0x3U << 0;
inline constexpr uint32_t DCFG_DAR            = 0x7FU << 4;
inline constexpr uint32_t DCFG_EOPFT          = 0x3U << 11;

inline constexpr uint32_t DCTL_RWKUP          = 1U << 0;
inline constexpr uint32_t DCTL_SDIS           = 1U << 1;
inline constexpr uint32_t DCTL_CGINAK         = 1U << 8;

inline constexpr uint32_t DSTAT_SPST          = 1U << 0;
inline constexpr uint32_t DSTAT_ES            = 0x3U << 1;
inline constexpr uint32_t DSTAT_FNRSOF        = 0x3FFFU << 8;

inline constexpr uint32_t DIEPINTEN_TFEN      = 1U << 0;
inline constexpr uint32_t DOEPINTEN_TFEN      = 1U << 0;
inline constexpr uint32_t DOEPINTEN_STPFEN    = 1U << 3;

// Endpoint Control (DIEPCTL / DOEPCTL)
inline constexpr uint32_t DEPCTL_MPL          = 0x7FFU << 0;
inline constexpr uint32_t DEPCTL_EPACT        = 1U << 15;
inline constexpr uint32_t DEPCTL_NAKS         = 1U << 17;
inline constexpr uint32_t DEPCTL_EPTYPE       = 0x3U << 18;
inline constexpr uint32_t DEPCTL_STALL        = 1U << 21;
inline constexpr uint32_t DEPCTL_TXFNUM       = 0xFU << 22;
inline constexpr uint32_t DEPCTL_CNAK         = 1U << 26;
inline constexpr uint32_t DEPCTL_SNAK         = 1U << 27;
inline constexpr uint32_t DEPCTL_SD0PID       = 1U << 28;
inline constexpr uint32_t DEPCTL_SEVNFRM      = 1U << 28;
inline constexpr uint32_t DEPCTL_SD1PID       = 1U << 29;
inline constexpr uint32_t DEPCTL_SODDFRM      = 1U << 29;
inline constexpr uint32_t DEPCTL_EPD          = 1U << 30;
inline constexpr uint32_t DEPCTL_EPEN         = 1U << 31;

// Endpoint Interrupt Flags (DIEPINTF / DOEPINTF)
inline constexpr uint32_t DIEPINTF_TF         = 1U << 0;
inline constexpr uint32_t DIEPINTF_EPDIS      = 1U << 1;
inline constexpr uint32_t DIEPINTF_TXFUD      = 1U << 4;
inline constexpr uint32_t DIEPINTF_TXFE       = 1U << 7;

inline constexpr uint32_t DOEPINTF_TF         = 1U << 0;
inline constexpr uint32_t DOEPINTF_EPDIS      = 1U << 1;
inline constexpr uint32_t DOEPINTF_STPF       = 1U << 3;

// Endpoint Transfer Length (DIEPLEN / DOEPLEN)
inline constexpr uint32_t DEPLEN_TLEN         = 0x7FFFFU << 0;
inline constexpr uint32_t DEPLEN_PCNT         = 0x3FFU << 19;
inline constexpr uint32_t DIEPLEN_MCNT        = 0x3U << 29;
inline constexpr uint32_t DIEPTFSTAT_IEPTFS   = 0xFFFFU << 0;

// Power & Clock Control (PWRCLKCTL at 0x50000E00)
inline constexpr uint32_t PWRCLKCTL_SUCLK     = 1U << 0;
inline constexpr uint32_t PWRCLKCTL_SHCLK     = 1U << 1;

// -----------------------------------------------------------------------------
// Compile-Time Zero-Overhead Accessors
// -----------------------------------------------------------------------------
struct GlobalRegsAccessor {
    constexpr CoreGlobalRegisters* operator->() const noexcept {
        return reinterpret_cast<CoreGlobalRegisters*>(USBFS_BASE);
    }
    constexpr operator CoreGlobalRegisters*() const noexcept {
        return reinterpret_cast<CoreGlobalRegisters*>(USBFS_BASE);
    }
};

struct DeviceRegsAccessor {
    constexpr DeviceRegisters* operator->() const noexcept {
        return reinterpret_cast<DeviceRegisters*>(USBFS_BASE + 0x0800UL);
    }
    constexpr operator DeviceRegisters*() const noexcept {
        return reinterpret_cast<DeviceRegisters*>(USBFS_BASE + 0x0800UL);
    }
};

struct InEndpointsAccessor {
    constexpr InEndpointRegisters* operator[](size_t ep_num) const noexcept {
        return reinterpret_cast<InEndpointRegisters*>(USBFS_BASE + 0x0900UL + (ep_num * 0x20UL));
    }
};

struct OutEndpointsAccessor {
    constexpr OutEndpointRegisters* operator[](size_t ep_num) const noexcept {
        return reinterpret_cast<OutEndpointRegisters*>(USBFS_BASE + 0x0B00UL + (ep_num * 0x20UL));
    }
};

struct PwrclkctlAccessor {
    constexpr volatile uint32_t& operator*() const noexcept {
        return *reinterpret_cast<volatile uint32_t*>(USBFS_BASE + 0x0E00UL);
    }
    constexpr volatile uint32_t* operator->() const noexcept {
        return reinterpret_cast<volatile uint32_t*>(USBFS_BASE + 0x0E00UL);
    }
    constexpr operator volatile uint32_t*() const noexcept {
        return reinterpret_cast<volatile uint32_t*>(USBFS_BASE + 0x0E00UL);
    }
};

// Data FIFO memory access port (0x50001000 + ep_num * 0x1000)
[[nodiscard]] inline volatile uint32_t* fifo_address(size_t ep_num) noexcept {
    return reinterpret_cast<volatile uint32_t*>(USBFS_BASE + 0x1000UL + (ep_num * 0x1000UL));
}

} // namespace drivers::usb::dwc2

// Type aliases for seamless zero-overhead integration
using usb_gr = drivers::usb::dwc2::CoreGlobalRegisters;
using usb_dr = drivers::usb::dwc2::DeviceRegisters;
using usb_erin = drivers::usb::dwc2::InEndpointRegisters;
using usb_erout = drivers::usb::dwc2::OutEndpointRegisters;

// Export bit constants to global namespace for backwards compatibility
using namespace drivers::usb::dwc2;
