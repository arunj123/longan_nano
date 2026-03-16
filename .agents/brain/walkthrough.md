# INA219 Current Monitor Walkthrough

I have successfully implemented the INA219 current monitor project for the Longan Nano. The project now uses hardware I2C for efficient sensor communication and streams data via USB HID.

## Changes Made

### 1. Hardware I2C Driver
- Implemented a robust hardware I2C driver for I2C0 (on pins PB6 and PB7) in [i2c_hw.c](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/i2c_hw.c) and [i2c_hw.h](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/i2c_hw.h).
- Replaced the initial software bit-banging approach at your request for better performance and efficiency.

### 2. INA219 Sensor Integration
- Created a driver for the INA219 DC current monitor in [ina219.c](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/ina219.c).
- Configured the sensor for 32V range and 0.1 ohm shunt resistor support.
- Implemented real-time reading of Voltage (mV), Current (mA), and Power (mW).

### 3. LCD Display
- Enhanced the [DisplayManager](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/display_manager.cpp) to show real-time sensor data.
- Implemented a custom text drawing library (`lcd_draw_string`) as the base LCD library lacked these functions.
- The LCD now displays:
    - Current Voltage
    - Current Milliamps
    - Current Power

### 4. USB HID Streaming
- Simplified the USB stack to provide a single Custom HID interface.
- Modified HID descriptors in [usbd_descriptors.cpp](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/usb_hid/usbd_descriptors.cpp) to support sending 9-byte reports containing sensor data.
- The main loop in [main.cpp](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/main.cpp) polls the sensor and streams data via USB HID 10 times per second.

## Verification Results

### Build Status
- The project builds successfully using the custom build script.
- Binary files created:
    - [current_monitor.elf](file:///c:/Users/arunj/projects/longan_nano/build/prj_current_monitor/current_monitor.elf)
    - [current_monitor.bin](file:///c:/Users/arunj/projects/longan_nano/build/prj_current_monitor/current_monitor.bin)

### Code Structure
- **Drivers:** [i2c_hw.c](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/i2c_hw.c), [ina219.c](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/ina219.c).
- **Display Layer:** [display_manager.cpp](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/display_manager.cpp).
- **USB Layer:** [usb_device.cpp](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/usb_hid/usb_device.cpp).
- **App Entry:** [main.cpp](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/main.cpp).

## How to Test on Your Hardware

1.  **Connections:**
    - Connect INA219 SDA to PB7 and SCL to PB6 on the Longan Nano.
    - Power VCC/GND as required.
2.  **Flash:**
    - Run: `python bldmgr/build.py prj_current_monitor flash`
3.  **Observation:**
    - You should see the terminal and LCD update with current values.
    - Connect the device to a PC and use a HID monitoring tool to receive the 10Hz data stream.
