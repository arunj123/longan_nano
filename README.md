# GD32VF103 Longan Nano Bare-Metal Project

This repository contains a collection of bare-metal firmware projects for the **Sipeed Longan Nano**, designed as a learning exercise for the GD32VF103 RISC-V microcontroller. It uses a custom Python-based build system (`bldmgr/build.py`) and avoids reliance on large IDEs or complex frameworks like PlatformIO or CMake.

## Featured Projects

This repository is structured to support multiple independent firmware applications. Each project resides in its own directory in the project root, and its name must be prefixed with `prj_`.

### `prj_uart_test`

This project uses the `debug_uart0` library to verify that the basic UART configuration and `printf` retargeting are working correctly.

### `prj_usb_serial`

This project configures the Longan Nano as a USB CDC (Virtual COM Port) device.

*   **USB CDC (Virtual COM Port):** The board enumerates as a USB serial device, allowing you to send and receive data from a host computer using a standard terminal emulator.
*   **UART0 Debug Output:** `printf` is retargeted to `USART0` (pins `PA9`/`PA10`) at 115200 baud, providing a convenient way to print debug messages.

### `prj_usb_composite`

This project demonstrates a composite USB device with multiple HID interfaces and an optional Mass Storage device. Its primary feature is a dynamic information display streamed from a host PC to the board's LCD. See the project's README for full details.

!Live Display UI

**Note:** The USB Mass Storage (MSC) feature in this composite device is currently a work-in-progress and is disabled by default.

## Hardware Setup

### Target Board: Sipeed Longan Nano

A compact development board featuring the GigaDevice GD32VF103CBT6 RISC-V microcontroller.
*   **Product Wiki:** wiki.sipeed.com/longan-nano

### Programmer/Debugger: Sipeed USB-JTAG/TTL

This project is configured to be flashed using the **Sipeed RV-Debugger** (or a compatible FT2232-based JTAG adapter). This device provides both JTAG programming and a USB-to-TTL serial converter for monitoring UART output.
*   **Product Wiki:** wiki.sipeed.com/rv-debugger

#### Connections

1.  **JTAG:** Connect the JTAG pins from the debugger to the corresponding pins on the Longan Nano (`JTAG_TMS`, `JTAG_TCK`, `JTAG_TDI`, `JTAG_TDO`, `GND`, `3V3`).
2.  **UART:** For debug output, connect the debugger's UART pins to the Longan Nano:
    *   Debugger `RX` -> Longan Nano `PA9` (TX)
    *   Debugger `TX` -> Longan Nano `PA10` (RX)

## Prerequisites

This build system is designed for **Windows** and requires **Python 3** to be installed and available in your system's `PATH`.

## Toolchain Setup

This project requires the RISC-V GCC toolchain and OpenOCD for flashing. The custom Python build script will automatically handle this for you.

When you run a build command for the first time, the script will check for the required tools. If they are not found in the `tools/` directory, it will automatically download and extract them. All you need is an internet connection.

## Building and Flashing

All build and flash operations are handled by the `bldmgr/build.py` script. You must specify which project you want to build.

Open a terminal in the project root and use the commands below.

### Available Commands

The command structure is `python bldmgr/build.py <project_name> [command]`.

```bash
python bldmgr/build.py <project_name> [command]

Arguments:
  project_name      The name of the project to build (e.g., prj_usb_serial).
  command           The action to perform. Defaults to 'build' if not specified.

Commands:
  build             (default) Incrementally builds the selected project.
  rebuild           Cleans and then builds the project from scratch.
  clean             Removes the project's build directory.
  flash             Builds and programs the device using the default OpenOCD.
  program           Programs an already-built binary using OpenOCD (no build).
  nucleus           Builds and programs using the Nuclei-specific OpenOCD config.
  dfu               Builds and programs using dfu-util.
  debug             Builds the project and starts an interactive GDB debug session.
```

### Examples

```bash
# Build the 'prj_usb_serial' project
python bldmgr/build.py prj_usb_serial

# Clean and rebuild the 'prj_usb_composite' project
python bldmgr/build.py prj_usb_composite rebuild

# Build and flash the 'prj_usb_serial' project
python bldmgr/build.py prj_usb_serial flash

# Build and start a command-line debug session for 'prj_usb_serial'
python bldmgr/build.py prj_usb_serial debug
```

## References

This project was inspired by and references code from:
*   [cjacker/opensource-toolchain-gd32vf103](https://github.com/cjacker/opensource-toolchain-gd32vf103)
*   GigaDevice GD32VF103 Firmware Library (V1.5.0), downloaded from the [official product page](https://www.gigadevice.com/microcontrollers/gd32vf103/).
*   [Appelsiini.net: Programming GD32V (Longan Nano)](https://www.appelsiini.net/2020/programming-gd32v-longan-nano/)
*   [Longan-Nano-Rainbow](https://github.com/joba-1/Longan-Nano-Rainbow/tree/main): An example of using Timer/PWM to control an RGB LED.
*   [GD32VF103_templates](https://github.com/WRansohoff/GD32VF103_templates/tree/master): A collection of templates for low-level peripheral usage, including Systick.
https://www.reddit.com/r/RISCV/comments/107407u/did_something_happen_to_sipeed_longan_nano/ : This chip has been discontinued and company is not much committed. So, better to go with CH32V103. Abandoning SD Card part. Will leave with USB Composite and LCD for Volume control etc.