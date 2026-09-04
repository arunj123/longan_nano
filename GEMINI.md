# Longan Nano Bare-Metal Project Guidelines

## Build & Toolchain
- **Build System**: Standalone Python build manager (`bldmgr/build.py`).
- **Compiler**: xPack RISC-V GCC 14.2 (`riscv-none-elf-gcc` / `riscv-none-elf-g++`).
- **Language Standards**: C17 (`-std=gnu17`), C++23 (`-std=gnu++23`).
- **Optimization Flags**: `-Os -flto -fuse-linker-plugin -ffunction-sections -fdata-sections`.
- **C++ Embedded Flags**: `-fno-exceptions -fno-rtti -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-threadsafe-statics`.
- **UTF-8 Output**: Always set `$env:PYTHONUTF8=1` in PowerShell before invoking the script:
  ```powershell
  $env:PYTHONUTF8=1; python bldmgr/build.py <project_name> [command]
  ```
- **Supported Commands**: `build`, `rebuild`, `clean`, `flash`, `program`, `nucleus`, `dfu`, `debug`.

## Hardware & Architecture Reference
- **MCU**: GD32VF103CBT6 (RISC-V 32-bit RV32IMAC @ up to 108MHz, 32KB SRAM, 128KB Flash).
- **FPU Policy**: **No hardware FPU**. Strictly avoid software floating-point emulation (`float`/`double`). Use fixed-point integer arithmetic (e.g. mV, mA, mW) for all sensor and control processing.
- **Clock Configuration**:
  - **96 MHz** for applications using USB (`__SYSTEM_CLOCK_96M_PLL_HXTAL`, generates exact 48 MHz USB clock).
  - **108 MHz** for non-USB applications (`__SYSTEM_CLOCK_108M_PLL_HXTAL` for maximum CPU performance).
- **Timing & Timers**:
  - Hardware `mtime` timer runs at `SystemCoreClock / 4` (24 MHz at 96 MHz SYSCLK, 27 MHz at 108 MHz SYSCLK).
  - Always use typed `hal::time` utilities (`ms_to_ticks`, `us_to_ticks`, `Instant`, `Duration`) for delays and debouncing. Never compare raw ticks directly to millisecond values.
- **Onboard LEDs**:
  - Red: `PC13` (active-low)
  - Green: `PA1` (active-low via anode to 3.3V)
  - Blue: `PA2` (active-low via anode to 3.3V)
- **LCD**: 160x80 ST7735 SPI LCD on SPI0 (CS: PB2, DC: PB0, RST: PB1, SCK: PA5, MOSI: PA7).

## Project Structure
- Active applications are prefixed with `prj_*` in the project root (`prj_usb_composite`, `prj_current_monitor`, `prj_usb_serial`, `prj_uart_test`).
- Historical/abandoned projects reside in `archive/` (`archive/prj_example`, `archive/prj_lcd_test`, `archive/prj_sdcard_test`).
- Modern C++23 zero-cost drivers and register abstractions reside in `hal/`, `bsp/`, and `drivers/`.
- Legacy vendor firmware library resides in `gd32/` and is compiled strictly as C.
