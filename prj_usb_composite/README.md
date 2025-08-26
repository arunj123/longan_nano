# Longan Nano - Composite USB HID & MSC Device

This project transforms the Sipeed Longan Nano board into a powerful, multi-function composite USB device. It serves as an excellent example of advanced USB development, interrupt handling, and hardware interfacing on the GD32VF103 RISC-V platform.

The device enumerates as three distinct USB interfaces simultaneously:
1.  **Standard HID:** Provides Keyboard, Mouse, and Consumer Control (media keys) functionality.
2.  **Custom HID:** A generic interface for custom communication with a host application.
3.  **Mass Storage Class (MSC):** (Optional) Allows the device to act as a USB flash drive using an attached SD card.

![Live Display UI](display.png)

## Features

- **Composite USB Device:** Combines multiple device classes into a single USB connection without needing a hub.
- **Rotary Encoder Control:**
  - Rotating the knob sends **Volume Up/Down** commands to the host computer.
  - Pressing the knob sends a **Mute** command.
- **Onboard Button Input:** The onboard `USER` button (PA8) is configured to type "Hi!" to demonstrate keyboard functionality.
- **Custom HID Communication:** A generic HID interface allows for bidirectional communication with host applications, used for both simple commands (like controlling the RGB LED) and complex data streaming.
- **Dynamic LCD Display:** A Python tool (`tools/display_manager`) streams a dynamically generated UI to the device's LCD via Custom HID, showing time, date, and live weather data.
- **Optional SD Card Mass Storage:** When enabled, the device appears as a USB drive, allowing direct access to the SD card from the host OS.
- **Robust Interrupt-Driven Logic:**
  - All user inputs (rotary encoder, buttons) are handled via interrupts for maximum efficiency.
  - Includes robust software debouncing for both button presses and encoder rotation to prevent crashes and false inputs.
- **Resilient Host Applications:** Companion Python scripts are provided to demonstrate communication with the Custom HID interface. They are designed to automatically handle device disconnections and reconnections.

## Hardware Required

- **Sipeed Longan Nano** board.
- **Rotary Encoder Module** (e.g., KY-040) with 5 pins (S1, S2, KEY, +, GND).
- Jumper wires.
- (Optional) MicroSD card and adapter for the Mass Storage feature.

## Wiring

Connect the rotary encoder to the Longan Nano using the following conflict-free pins. These pins do not interfere with the onboard LEDs or other components.

| Rotary Encoder Pin | Longan Nano Pin | Physical Pin # |
| :----------------- | :-------------- | :------------- |
| `S1` (or CLK)      | **PB10**        | Pin 21         |
| `S2` (or DT)       | **PB11**        | Pin 22         |
| `KEY` (or SW)      | **PB12**        | Pin 23         |
| `+` (or 5V)        | **3.3V**        |                |
| `GND`              | **GND**         |                |

## Getting Started

### Prerequisites

1.  For the host script: Python 3 and the `hidapi` library.
2.  A RISC-V development toolchain (will be automatically setup on Windows by python build scripts).
3.  Refer to the main README.md for build system and toolchain setup instructions.

### Building the Firmware

1.  **(Optional) Enable Mass Storage:**
    To include the SD card functionality, open `prj_usb_composite/config.py` and change the `USE_SD_CARD_MSC` definition.
    **Note:** This feature is a work-in-progress and is disabled by default. Set to `1` to enable.
    ```cpp
    GLOBAL_C_DEFINES = [
        # ... other defines
        "-DUSE_SD_CARD_MSC=0", # Change to 1 to enable
    ]
    ```
    If you don't have an SD card, leave this set to `0`. The USB stack will dynamically adjust and hide the MSC interface from the host.

2.  **Build and Upload:**
    Using the custom build script from the repository root, run the following commands:
    ```bash
    # To build the project:
    python .\bldmgr\build.py prj_usb_composite

    # Upload the firmware to the Longan Nano
    python .\bldmgr\build.py prj_usb_composite flash
    ```

## How to Use

Once the firmware is flashed and the device is connected to your computer, it will immediately start working as a standard HID device:

- **Rotate the knob:** Adjusts the system volume on your PC.
- **Press the knob:** Toggles mute for your PC's audio.
- **Press the onboard `USER` button (on PA8):** The device will type `Hi!` into any active text editor.

### Dynamic Display Manager (Host Application)

This is the primary host application that turns the Longan Nano into a mini information display. It generates a UI with the current time, date, and weather, and streams it to the device's LCD.

1.  **Install the required library:**
    ```bash
    pip install hidapi Pillow requests
    ```
2.  **(Optional) Configure Location for Weather:**
    Edit `tools/display_manager/config.py` and set your latitude and longitude to get local weather.
3.  **Run the script:**
    Navigate to the `tools/display_manager` directory and run the script.
    ```sh
    python display_manager.py
    ```
4.  **Interact:**
    - The script will automatically find and connect to the device.
    - It performs an initial full-screen update and then sends only partial updates for changed areas (e.g., the clock changing every minute).
    - Press the `USER` button on the Longan Nano to cycle through different color themes on the display.

### Basic Image Transfer Test

This is a simpler script for testing the raw image data transfer capability over Custom HID. It reads a pre-compiled binary image file and sends it to the device.

To run it, navigate to the `tools` directory and execute:
```bash
python test_custom_hid.py
```

## Code Structure

- `src/main.cpp`: The main application entry point. Handles initialization and the main loop, which polls the encoder and button states and sends USB reports.
- `src/usb_device.cpp / .h`: The core of the USB abstraction layer. Implements the `UsbDevice` singleton, manages the composite device descriptors, and dispatches USB events to the correct handlers (HID, MSC).
- `src/rotary_encoder.cpp / .h`: A self-contained driver for the rotary encoder. It uses interrupts and software debouncing for reliable, efficient input processing.
- `src/usb_composite/gd32vf103_it.cpp`: Contains the interrupt service routines (ISRs) that link hardware events (like USB interrupts and GPIO pin changes) to the C++ handler functions.
- `tools/display_manager/`: The Python application for the dynamic LCD display.

## Technical Deep Dive: Key Concepts

This project solves several common challenges in embedded USB development:

- **Composite Device Management:** The `usb_device.cpp` code shows how to define a composite descriptor set and how to route class-specific requests based on the target interface number (`wIndex` in the USB setup packet). This is the key to creating multi-function devices that work correctly on all major operating systems.

- **Interrupt Stability:** The rotary encoder driver is a case study in stable interrupt design. By triggering on a single GPIO edge and using a timer for debouncing, it solves the critical problem of **stack overflow** that can occur when interrupts fire too frequently from a noisy mechanical source.

- **Race Condition Prevention:** The HID `send_*_report` functions use a `prev_transfer_complete` flag. This acts as a simple software semaphore to prevent the main loop from trying to write to a USB endpoint while it's still busy transmitting the previous packet. Attempting to do so would corrupt the USB driver's internal state and cause a hard-to-debug crash.

- **Dynamic Interface Configuration:** The USB MSC interface is included or excluded at runtime based on the successful initialization of the SD card. The USB descriptors are dynamically modified *before* the USB core is initialized, ensuring the host OS only sees the interfaces that are actually available.

## Fonts Used

The Python-based `display_manager` tool uses open-source fonts, which are located in the `tools/display_manager/fonts` directory. For more details on the specific fonts and their licenses, please see the `README.md` file within that folder.