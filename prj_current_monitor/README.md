# Longan Nano - INA219 DC Current & Power Monitor (Pure Rust)

This project transforms the Sipeed Longan Nano (GD32VF103 RISC-V RV32IMAC MCU @ 96 MHz) into a real-time DC current, power, and energy monitor with an on-device ST7735 color display, persistent MicroSD FAT32 CSV logging, and continuous high-frequency USB HID telemetry streaming.

The firmware is implemented in **100% vendor-free, pure embedded Rust** (`no_std`) with zero software float arithmetic, achieving high DSP throughput and low latency.

---

## Key Features

- **Multi-Screen On-Device UI (160x80 ST7735 LCD @ 24 MHz SPI):**
  - **Hero Screen:** Large 28px TrueType-rasterized current metric, live dynamic load gauge bar with peak-hold transient marker, mint voltage card, amber power card, and live status badges (`[LIVE]`, `[REV]`, `[OVF]`, `[ERR]`).
  - **Oscilloscope Screen:** 10 Hz real-time sweep with an integrated **137-sample circular buffer**. When autoscaling shifts scale tiers, the historical waveform is instantly replotted without erasing previous measurements. Includes shaded area fill, grid dots, and non-overlapping header telemetry.
  - **Stats Dashboard:** Live session run time, estimated load impedance (\(R = V/I\)), voltage min..max range, current min..max range, peak power, MicroSD log counter, and accumulated energy (\(\mu\text{Wh} \to \text{mWh} \to \text{Wh}\)) & charge (\(\mu\text{Ah} \to \text{mAh} \to \text{Ah}\)).
- **Precision Sensor DSP (INA219 @ 400 kHz Fast I2C):**
  - 128-sample hardware shunt ADC averaging (68.1 ms conversion time) covering ~68% of continuous real time.
  - 16-sample hardware bus ADC averaging.
  - Math overflow (`OVF`) and conversion ready (`CNVR`) monitoring.
  - Near-zero deadband filter to eliminate ADC quantization jitter.
  - Sub-sample millisecond delta integration with zero timing drift.
- **MicroSD FAT32 CSV Datalogging (`MONITOR.CSV`):**
  - Internal 512-byte sector buffer: eliminates costly per-second filesystem overhead and flash wear by writing full sector blocks.
  - Hot-plug recovery with exponential backoff (2.5s \(\to\) 5s \(\to\) 10s \(\to\) 30s) to guarantee zero UI or USB stutter when no card is inserted.
  - Logs true instantaneous measurements with exact decimal points and signed current.
- **Interactive Controls (User Button PA8):**
  - **Short Press (< 1.5s):** Cycle through screens: `Hero` \(\to\) `Graph` \(\to\) `Stats` \(\to\) `Hero`.
  - **Long Press (\(\ge\) 1.5s):** **Tare Zero Calibration** (stores active reading as zero-offset) + resets session accumulators and stats.
- **High-Frequency USB HID Telemetry Streaming:**
  - Streams 9-byte reports at 10 Hz over custom USB HID (VID: `0x28E9`, PID: `0x1234`).
  - Transmits bus voltage (mV), signed current with 0.1 mA resolution, power (mW), sequence counter, and diagnostic flags (`ONLINE`, `REV`, `OVF`, `SD-REC`).
  - Accompanied by a live Python terminal dashboard (`monitor.py`).

---

## Hardware Pinout & Wiring

Connect the INA219 sensor module to the Longan Nano:

| INA219 Pin | Longan Nano Pin | Physical Pin # | Function                       |
| :--------- | :-------------- | :------------- | :----------------------------- |
| `VCC`      | **3.3V**        |                | 3.3V System Power              |
| `GND`      | **GND**         |                | Common Ground                  |
| `SCL`      | **PB6**         | Pin 17         | I2C0 Clock (400 kHz Fast Mode) |
| `SDA`      | **PB7**         | Pin 18         | I2C0 Data (AF Open-Drain)      |

Other integrated on-board peripherals used:
- **LCD (SPI0):** PA5 (SCK), PA7 (MOSI), PB0 (DC), PB1 (RST), PB2 (CS) @ 24 MHz (`Prescaler::Div4`).
- **MicroSD (SPI1):** PB12 (CS), PB13 (SCK), PB14 (MISO), PB15 (MOSI).
- **USB:** PA11 (DM), PA12 (DP).
- **Button:** PA8 (Active-high with internal pull-down resistor; asserts HIGH on press).
- **LEDs:** PC13 (Red - INA219 error alert), PA1 (Green - Polite 35 ms / 2.5s heartbeat), PA2 (Blue - User input feedback), DS1 (PWR - Hardwired 3.3V indicator).

---

## Building and Flashing

Build the release binary using Cargo or the repository build manager:

```powershell
# Build Rust firmware and generate .hex / .bin
$env:PYTHONUTF8=1; python bldmgr/build.py prj_current_monitor build

# Flash via remote OpenOCD programmer
$env:PYTHONUTF8=1; python tools/remote_flash.py build/prj_current_monitor/rust/firmware_rust.hex
```

---

## Host PC Monitoring

Run the Python telemetry monitor to view the live ANSI dashboard:

```powershell
python prj_current_monitor/monitor.py
```
