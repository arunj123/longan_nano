# INA219 Current Monitor Project

This project creates a current monitor using the Sipeed Longan Nano and an INA219 sensor. It displays real-time current values on the LCD and streams them to a host PC via USB HID.

## Proposed Changes

### [Component: prj_current_monitor]

#### [NEW] [config.py](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/config.py)
Project configuration based on `prj_usb_composite`, enabling necessary drivers and defining source files.

#### [NEW] [src/main.cpp](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/main.cpp)
Main application logic: initializing I2C, INA219, LCD, and USB; periodic sensor reading and display/USB update.

#### [NEW] [src/ina219.h](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/ina219.h) / [src/ina219.c](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/ina219.c)
Driver for the INA219 DC current monitor, using hardware I2C (I2C0 on PB6/PB7).

#### [NEW] [src/i2c_hw.h](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/i2c_hw.h) / [src/i2c_hw.c](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/i2c_hw.c)
Hardware I2C driver for GD32VF103.

#### [NEW] [src/usb_hid_report.h](file:///c:/Users/arunj/projects/longan_nano/prj_current_monitor/src/usb_hid_report.h)
Definition of the HID report structure for streaming current data.

### [Component: Shared Libraries]

#### [MODIFY] [gd32/components.py](file:///c:/Users/arunj/projects/longan_nano/gd32/components.py)
Ensure the I2C driver is correctly defined as a component.

## Verification Plan

### Automated Tests
- None (Bare-metal firmware)

### Manual Verification
1. **Build:** Run `python bldmgr/build.py prj_current_monitor` to ensure code compiles.
2. **Flash:** Run `python bldmgr/build.py prj_current_monitor flash` to program the device.
3. **LCD Output:** Observe the LCD to see real-time Voltage/Current/Power values.
4. **USB HID:** Use a tool (e.g., `hidapi` or a simple Python script) on the host PC to verify that HID reports are being received and contain valid data.
5. **I2C Check:** Verify INA219 communication (e.g., reading the manufacturer ID or a known register).
