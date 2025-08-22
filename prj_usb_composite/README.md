# Longan Nano - Composite USB HID & MSC Device

This project transforms the Sipeed Longan Nano (GD32VF103) board into a powerful, multi-function composite USB device. It serves as an excellent example of advanced USB development, interrupt handling, and hardware interfacing on the RISC-V platform.

The device enumerates as three distinct USB interfaces simultaneously:
1.  **Standard HID:** Provides Keyboard, Mouse, and Consumer Control (media keys) functionality.
2.  **Custom HID:** A generic interface for custom communication with a host application.
3.  **Mass Storage Class (MSC):** (Optional) Allows the device to act as a USB flash drive using an attached SD card.

## Features

- **Composite USB Device:** Combines multiple device classes into a single USB connection without needing a hub.
- **Rotary Encoder Control:**
    - Rotating the knob sends **Volume Up/Down** commands to the host computer.
    - Pressing the knob sends a **Mute** command.
- **Onboard Button Input:** The onboard `USER` button (PA8) is configured to type "Hi!" to demonstrate keyboard functionality.
- **Custom HID Communication:**
    - The device can send custom data packets to a host application (e.g., when the `USER` button is pressed).
    - A host application can send data packets to the device to control the onboard RGB LED.
- **Optional SD Card Mass Storage:** When enabled, the device appears as a USB drive, allowing direct access to the SD card from the host OS.
- **Robust Interrupt-Driven Logic:**
    - All user inputs (rotary encoder, buttons) are handled via interrupts for maximum efficiency.
    - Includes robust software debouncing for both button presses and encoder rotation to prevent crashes and false inputs.
- **Resilient Host Application:** A companion Python script is provided to demonstrate communication with the Custom HID interface. It's designed to automatically handle device disconnections and reconnections.

## Hardware Required

- **Sipeed Longan Nano** board.
- **Rotary Encoder Module** (e.g., KY-040) with 5 pins (S1, S2, KEY, +, GND).
- Jumper wires.
- (Optional) MicroSD card and adapter for the Mass Storage feature.

## Wiring

Connect the rotary encoder to the Longan Nano using the following conflict-free pins. These pins do not interfere with the onboard LEDs or other components.

| Rotary Encoder Pin | Longan Nano Pin | Physical Pin # |
| :----------------- | :-------------- | :------------- |
| `S1` (or CLK)      | **PB10**        | 21             |
| `S2` (or DT)       | **PB11**        | 22             |
| `KEY` (or SW)      | **PB12**        | 23             |
| `+` (or 5V)        | **3.3V**        |                |
| `GND`              | **GND**         |                |

## Getting Started

### Prerequisites

1.  For the host script: Python 3 and the `hidapi` library.
2.  A RISC-V development toolchain (will be automatically setup on Windows by python build scripts).

### Building the Firmware

1.  **Clone the repository:**
    ```sh
    git clone <your_repository_url>
    cd <your_repository_name>
    ```
2.  **(Optional) Enable Mass Storage:**
    To include the SD card functionality, open `main.cpp` and ensure the `USE_SD_CARD_MSC` macro is defined and set to `1`.
    Note: This feature is not working now.
    ```cpp
    #define USE_SD_CARD_MSC 1
    ```
    If you don't have an SD card, set this to `0` or comment it out. The USB stack will dynamically adjust.

3.  **Build and Upload:**
    Using a tool like PlatformIO, the commands are straightforward:
    ```sh
    # Build the project (from root directory of repository)
    python .\bldmgr\build.py prj_usb_composite

    # Upload the firmware to the Longan Nano
    python .\bldmgr\build.py prj_usb_composite flash
    ```

## How to Use

### Device Functionality

Once the firmware is flashed and the device is connected to your computer, it will immediately start working as a standard HID device:

- **Rotate the knob:** Adjusts the system volume on your PC.
- **Press the knob:** Toggles mute for your PC's audio.
- **Press the onboard `USER` button (on PA8):** The device will type `Hi!` into any active text editor.

### Host-Side Python Script

The Python script (`test_custom_hid.py`) allows you to interact with the Custom HID interface.

1.  **Install the required library:**
    ```sh
    pip install hidapi
    ```
2.  **Run the script:**
    ```sh
    python test_custom_hid.py
    ```
3.  **Interact:**
    - The script will automatically find and connect to the device.
    - It will begin sending commands to cycle the onboard RGB LED (Red -> Green -> Blue).
    - When you press the onboard `USER` button, you will see a "Received data" message printed in the terminal, showing the counter value sent from the device.
    - If you unplug or reset the device, the script will detect the disconnection and automatically attempt to reconnect.

## Code Structure

- `src/main.cpp`: The main application entry point. Handles initialization and the main loop, which polls the encoder and button states and sends USB reports.
- `src/usb_device.cpp / .h`: The core of the USB abstraction layer. Implements the `UsbDevice` singleton, manages the composite device descriptors, and dispatches USB events to the correct handlers (HID, MSC).
- `src/rotary_encoder.cpp / .h`: A self-contained driver for the rotary encoder. It uses interrupts and software debouncing for reliable, efficient input processing.
- `src/usb_composite/gd32vf103_it.cpp`: Contains the interrupt service routines (ISRs) that link hardware events (like USB interrupts and GPIO pin changes) to the C++ handler functions.
- `../tools/test_custom_hid.py`: The Python script for communicating with the Custom HID interface.

## Technical Deep Dive: Key Concepts

This project solves several common challenges in embedded USB development:

- **Composite Device Management:** The code demonstrates how to define a composite descriptor set and route requests based on the target interface number (`wIndex`). This is crucial for creating multi-function devices.
- **Interrupt Stability:** The final rotary encoder driver is a case study in stable interrupt design. By triggering on a single edge and using a timer for debouncing, it solves the critical problem of **stack overflow** that can occur when interrupts fire too frequently from a noisy mechanical source.
- **Race Condition Prevention:** The standard HID and Custom HID send functions use a `prev_transfer_complete` flag. This acts as a semaphore to prevent the main loop from trying to write to a USB endpoint while it's still busy transmitting the previous packet, which would otherwise corrupt the USB driver's state and cause a crash.
- **Dynamic Interface Configuration:** The USB MSC interface is included or excluded at runtime based on the successful initialization of the SD card. The USB descriptors are dynamically modified *before* the USB core is initialized if the SD card is not present.