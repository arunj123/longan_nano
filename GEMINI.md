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

## Flashing & In-Circuit Debugging
- **Programmer**: FT2232D Dual-Channel USB interface (`VID: 0x0403, PID: 0x6010`):
  - **Interface 0 (`MI_00`)**: JTAG debugger (`Dual RS232`, WinUSB driver via OpenOCD 0.12).
  - **Interface 1 (`MI_01`)**: Debug UART monitor (`USB Serial Converter B`, Virtual COM port `COM13` @ 115200 baud).
- **UART Monitor**: Use `tools/uart_monitor.py` for resilient, auto-reconnecting serial logging.
- **SRST-less OpenOCD Programming**:
  - The physical reset line (SRST) is not wired to the JTAG header.
  - Flashing commands must software-resume execution at the flash entry point:
    `program "<hex_path>" verify; halt; reg pc 0x08000000; resume; shutdown`

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
- **User Button**: `PA8` (active-low with internal pull-up). Always ensure `KeyButton::init()` is called to enable pull-up.
- **LCD**: 160x80 ST7735 SPI LCD on SPI0 (CS: PB2, DC: PB0, RST: PB1, SCK: PA5, MOSI: PA7).

## Interrupts & USB Architecture
- **ECLIC Controller**:
  - Base address `0xD2000000`. Each interrupt takes 4 bytes (IP, IE, ATTR, CTRL).
  - Always use direct array indexing `base[i]` for all 87 interrupts; avoid pointer increment multipliers.
  - Unhandled interrupts vector to `_unassigned_interrupts_handler()`, which prints `mcause` to UART0 and halts.
- **USB CDC-ACM Performance**:
  - Never use blocking delays (`delay_ms()`) in the main loop of USB-enabled applications.
  - `usb::poll()` must run continuously with zero latency. Use non-blocking `hal::time::Instant` and `Duration` for application-level task scheduling.

## Project Structure
- Active applications are prefixed with `prj_*` in the project root (`prj_usb_composite`, `prj_current_monitor`, `prj_usb_serial`, `prj_uart_test`, `prj_lcd_test`, `prj_sdcard_test`, `prj_sdcard_fs_test`).
- Modern C++23 zero-cost drivers and register abstractions reside in `hal/`, `bsp/`, and `drivers/`.
- Modern, low-footprint components reside in `lib/` (`lib/system`, `lib/fatfs`, `lib/gd32v_lcd`, `lib/debug_uart0`).
- Legacy vendor firmware library resides in `gd32/` and is compiled strictly as C.

## Resource & Peripheral Conflict Checking
- **Always Check Hardware Allocations**: Before modifying or creating drivers, verify that peripherals, DMA channels, timers, GPIO pins, and interrupt vectors do not collide:
  - **DMA0 Channels**:
    - CH0: ADC0
    - CH1: SPI0_RX (LCD Read)
    - CH2: SPI0_TX (LCD Blit/Draw)
    - CH3: SPI1_RX / USART0_TX
    - CH4: SPI1_TX / USART0_RX
    - CH5: I2C0_TX
    - CH6: I2C0_RX
  - **Timers**: Do not use hardware TIMER2 for simple delays (use core 64-bit `mtime` via `hal::time`).
  - **GPIO Pins**: Check all SPI, I2C, UART, Button, and LED mappings before assigning or reconfiguring pins.

