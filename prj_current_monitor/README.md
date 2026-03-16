# Longan Nano - INA219 DC Current Monitor

This project turns the Sipeed Longan Nano board into a real-time DC current and power monitor. It interfaces with an INA219 sensor via hardware I2C, displays live readings on the integrated LCD, and streams high-frequency monitoring data to a host PC via a custom USB HID interface.

![Project Overview](../current_display.png)

## Features

- **Real-Time Visualization:** Displays Voltage (mV), Current (mA), and Power (mW) directly on the Longan Nano's 160x80 LCD.
- **High-Frequency Streaming:** Streams sensor data over USB HID at 10Hz for logging and analysis on a host PC.
- **Hardware I2C:** Utilizes the GD32VF103's hardware I2C peripheral (I2C0) for efficient, non-blocking sensor communication.
- **Custom Graphics:** Includes a lightweight embedded font library for high-performance text rendering on the LCD.
- **Agent Enhanced:** Includes pre-defined agent workflows and rules for automated building, flashing, and maintenance.

## Hardware Required

- **Sipeed Longan Nano** board.
- **INA219 DC Current Monitor** module.
- Jumper wires.

## Wiring

Connect the INA219 sensor to the Longan Nano using the hardware I2C0 pins:

| INA219 Pin | Longan Nano Pin | Physical Pin # | Function   |
| :--------- | :-------------- | :------------- | :--------- |
| `VCC`      | **3.3V**        |                | Power      |
| `GND`      | **GND**         |                | Ground     |
| `SCL`      | **PB6**         | Pin 17         | I2C0 Clock |
| `SDA`      | **PB7**         | Pin 18         | I2C0 Data  |

## Getting Started

### Prerequisites

1.  A RISC-V development toolchain (automatically managed by the provided build scripts on Windows).
2.  Refer to the repository's root `README.md` for general build system setup.

### Building and Flashing

You can use the custom build manager to compile and program the device:

```powershell
# To build the project:
python bldmgr/build.py prj_current_monitor build

# To flash the firmware to the device:
python bldmgr/build.py prj_current_monitor flash
```

Alternatively, if you are using an agentic IDE, you can use the built-in workflow:
`/build_and_flash`

## USB HID Data Format

The device enumerates as a Custom HID device and sends a 9-byte data packet at 10Hz.

**Report ID:** `0x01`

| Byte Index | Field       | Type     | Unites | Description         |
| :--------- | :---------- | :------- | :----- | :------------------ |
| 0          | Report ID   | uint8    | -      | Fixed at 0x01       |
| 1-2        | Voltage     | uint16le | mV     | Bus Voltage         |
| 3-4        | Current     | int16le  | mA     | Shunt Current       |
| 5-6        | Power       | uint16le | mW     | Calculated Power    |
| 7-8        | Padding     | -        | -      | Reserved for future |

## Code Structure

- `src/main.cpp`: Main application loop, sensors polling, and USB reporting logic.
- `src/i2c_hw.c / .h`: Robust hardware I2C driver for the GD32VF103.
- `src/ina219.c / .h`: Driver for the INA219 sensor, configured for high resolution.
- `src/display_manager.cpp / .h`: Manages the LCD UI, text rendering, and screen updates.
- `src/usb_hid/`: Simplified USB stack for custom HID communication.
- `config.py`: Project-specific build configuration and source listing.

## Agent Support

This project is "Agent-Ready" and includes:
- **.agentrules**: Context and constraints for AI coding assistants.
- **.agents/workflows/**: Automated steps for building and deployment.
- **.agents/brain/**: Preserved design documents and technical history.
