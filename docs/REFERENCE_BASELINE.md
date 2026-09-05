# GD32VF103 Reference Baselines & Modern Architecture

## 1. Hardware & Silicon Reference
- **Microcontroller**: GigaDevice GD32VF103CBT6 (Status: Discontinued / End of Life).
- **Core**: Nuclei Bumblebee N200 RISC-V 32-bit RV32IMAC @ up to 108 MHz.
  - RV32I: Base integer instruction set.
  - M: Hardware integer multiplication and division.
  - A: Atomic memory instructions (`lr.w`, `sc.w`, `amoswap.w`, etc.).
  - C: Compressed 16-bit instructions.
  - _zicsr: Control and Status Register instructions (`csrr`, `csrw`, `csrs`, `csrc`).
- **Memory**:
  - 128 KB On-chip Flash memory (`0x08000000 - 0x0801FFFF`).
  - 32 KB SRAM (`0x20000000 - 0x20007FFF`).
  - 1.25 KB Dedicated USB RAM / FIFO (`0x50000000`).
- **Datasheet**: Preserved in `docs/GD32VF103_Datasheet_Rev2.1.pdf`.
- **Vendor Firmware Baseline**: GigaDevice GD32VF103 Firmware Library **V1.5.0** (dated 2025-02-10). The vendor change log is preserved in `docs/GD32VF103_Firmware_Library_V1.5.0_changelog.txt`.

---

## 2. Architecture Transition: Vendor C to Modern C++23

Prior to this refactoring, the project depended on legacy C vendor files:
1. `gd32_std_peripheral_lib`: Heavy vendor header files (`gd32vf103.h`, `gd32vf103_*.h`).
2. `riscv_drivers`: Vendor RISC-V headers (`riscv_encoding.h`, `n200_func.h`).
3. `GD32VF103_usbfs_library`: Vendor USB FS device stack (~7 C files, macros, `usbd_core.c`, `drv_usb_core.c`, etc.).

### Modern C++23 Replacement Layout

All vendor code has been completely replaced with zero-cost modern C++23 abstractions:

| Subsystem | Legacy Vendor Location | Modern C++23 Location | Key Features |
|---|---|---|---|
| **RISC-V CSR & Core** | `gd32/Firmware/RISCV/drivers/` | `hal/include/hal/csr.hpp`<br>`hal/include/hal/riscv_asm.h` | Type-safe templated CSR read/write/set/clear, atomic CSR operations, standard inline assembly |
| **Interrupt Controller (ECLIC)** | `gd32/Firmware/RISCV/drivers/n200_eclic.h` | `hal/include/hal/eclic.hpp` | Strongly typed enum `hal::eclic::Irq`, safe base address indexing, zero raw array offset pitfalls |
| **Hardware Registers** | `gd32/Firmware/GD32VF103_standard_peripheral/Include/` | `hal/include/hal/registers.hpp`<br>`hal/include/hal/*.hpp` | Base addresses, memory-mapped I/O peripherals (`RCU`, `GPIO`, `AFIO`, `TIMER`, `USART`, `SPI`, `I2C`, `DMA`) |
| **USB DWC2 Peripheral** | `gd32/Firmware/GD32VF103_usbfs_library/driver/` | `hal/include/hal/usb_dwc2.hpp` | Complete Synopsys DWC2 OTG full-speed core register mapping (`0x50000000`), strongly typed bitfields |
| **USB Chapter 9 & Descriptors** | `gd32/Firmware/GD32VF103_usbfs_library/device/core/` | `drivers/include/drivers/usb/usb_ch9.hpp` | Packed standard descriptors, CDC functional descriptors, request type masks, string helper macros |
| **USB Device Engine & ISR** | `gd32/Firmware/GD32VF103_usbfs_library/` | `drivers/include/drivers/usb/usb_core.hpp`<br>`drivers/src/usb/usb_core.cpp` | Robust DWC2 control endpoint engine, dynamic FIFO management, RV32IMAC 32-bit aligned memory safety via `memcpy` packet chunks, VBUSIG automatic host-connect |
| **USB CDC-ACM Class Driver** | `gd32/Firmware/GD32VF103_usbfs_library/device/class/cdc/` | `drivers/include/drivers/usb/cdc_acm.hpp`<br>`drivers/src/usb/cdc_acm.cpp` | Modern C++23 non-blocking serial CDC implementation, circular buffer RX/TX, line coding control |

---

## 3. Hardware Nuances & Traps

1. **Hardware VBUS Pin Disconnect**:
   On the Sipeed Longan Nano, pin PA9 (VBUS sense) is not connected to the Type-C receptacle. The DWC2 core must be forced into session-valid mode via `GCCFG |= (1U << 21)` (`VBUSIG`) to allow USB enumeration.
2. **Unaligned Memory Traps (RV32IMAC)**:
   RV32IMAC raises hardware exceptions when 32-bit loads (`lw`) are executed on addresses not aligned to 4-byte boundaries. In `drivers/src/usb/usb_core.cpp`, all data FIFO write/read transfers transfer bytes to/from arbitrary host buffers using safely aligned word buffers with `std::memcpy`.
3. **Dedicated Target**:
   All peripheral drivers are written strictly and exclusively for the GD32VF103CBT6 (Sipeed Longan Nano). Multi-target vendor abstraction boilerplate has been completely excised.
