# Longan Nano Current Monitor — High-Performance Native 3D Suite

A high-performance, cross-platform native desktop application implemented in **Rust** using hardware-accelerated immediate-mode GUI (`eframe`/`egui`) and an integrated interactive **3D Graphics Engine**.

It interfaces with the Sipeed Longan Nano (GD32VF103 RISC-V) INA219 current monitoring firmware to deliver real-time telemetry, 2D oscilloscope plotting, bidirectional hardware control, and live 3D visualization.

---

## 🌟 Key Features

### 1. Interactive 3D Graphics Viewport
- **3D Digital Twin (Hardware Model & Live Display)**:
  - Procedurally generated 3D model of the Sipeed Longan Nano MCU board and INA219 sensor module.
  - Realistic gold pin headers, metal USB-C receptacle, and GD32VF103 package.
  - **Live Virtual Screen Replica**: ST7735 color display surface rendered in 3D, mirroring the live telemetry readings and load bar gauge.
  - **Dynamic Emissive 3D LEDs**:
    * Green Heartbeat LED (`PA1`) pulses with live periodic rhythm.
    * Red Alert LED (`PC13`) flashes on overcurrent and math overflow events.
    * Cyan USB LED (`PA2`) glows solid when connected.
- **3D V-I-t Phase Space Trajectory Ribbon**:
  - Sweeps a continuous 3D dynamic ribbon in phase space (\(t\), \(V(t)\), \(I(t)\)).
  - Real-time vertex color gradient indicating load severity (Cyan \(\to\) Mint \(\to\) Amber \(\to\) Scarlet Red).
  - Highlighting transient current inrush loops, switching regulator ripples, and impedance hysteresis.
- **Full Camera Controls**:
  - **Left Mouse Drag**: Orbit rotation (Yaw and Pitch).
  - **Right Mouse Drag**: Pan camera in 3D.
  - **Scroll Wheel**: Smooth zoom in/out.
  - **Turntable Auto-Rotate**: Smooth 360° demonstration rotation.
  - **Wireframe CAD Mode**: Toggles high-tech engineering wireframe overlay.
  - **Reset Camera**: One-click viewport normalization.

### 2. High-Speed 2D Waveform Oscilloscope (`egui_plot`)
- Multi-channel real-time plotting (Voltage in green, Current in cyan, Power in gold).
- Selectable time windows: 5s, 10s, 15s, 30s, 60s, 120s.
- **Freeze / Pause Sweep**: Freezes the display so engineers can inspect spikes without interrupting data logging.
- Sub-sample precision with interactive pan, zoom, and cursor measurements.

### 3. Hero Bench-Meter Cards
- 34pt large digits for Current (mA / A) with dynamic color coding.
- Bus Voltage (V) with Min/Max bounds.
- Power (mW / W) with Peak-Hold indicator.
- Dynamic Load Impedance (\(R = V / I\)) with automatic \(\Omega \leftrightarrow \text{k}\Omega \leftrightarrow \text{M}\Omega\) unit scaling.
- Coulomb Counter & Energy Accumulators (\(\text{mWh}\) & \(\text{mAh}\)).
- Live Diagnostic Badges: `[ONLINE]`, `[REV]`, `[OVF]`, `[SD-LOG]`, `[ALERT]`.

### 4. Bidirectional Hardware Controls (USB HID OUT Reports)
- **Zero-Tare Button**: Sends `0x02` to stack zero-offset and calibrate shunt baseline in hardware.
- **One-Click RTC Epoch Sync**: Takes local system UTC epoch seconds and sends `0x07` (`SetEpoch`) to synchronize on-board RTC.
- **Battery Profile Selector**: Sets active profile on device (`None`, `LiPo 500mAh`, `LiPo 1200mAh`, `Li-Ion 2500mAh`, `Alkaline 1000mAh`).
- **Overcurrent Alert Limit Slider**: Configures hardware alert threshold (50 to 3200 mA) with quick presets (`250mA`, `500mA`, `1000mA`, `2000mA`, `3000mA`).
- **On-Device Screen Mode Switcher**: Toggles Longan Nano display mode (`Hero`, `Graph`, `Stats`, `Histogram`, `BigDigit`, `Cycle`).
- **MicroSD Remote Actions**: Flush sector buffer, rotate session file, dump UART summary.

### 5. Multi-Source Transport Support
- **Direct USB HID**: Connects directly via native OS USB HID APIs (`hidapi`, VID `0x28E9`, PID `0x1234`).
- **Remote TCP Bridge**: Connects seamlessly across the local network to remote Linux testbeds (`192.168.0.63:5055` or `127.0.0.1:5055`).
- **Simulation Mode**: Generates realistic DC load profiles (quiescent sleep, periodic RF bursts, overcurrent spikes) for testing without hardware.

### 6. CSV Data Recording & Export
- Live disk recording with microsecond timestamps, relative seconds, voltage, current, power, flags, sequence numbers.
- One-click "Export Buffer Snapshot" to save current buffer.

---

## 🚀 Quick Start

### 1. One-Click Launcher (Auto-Detects Hardware)
```powershell
python tools/run_monitor_gui.py
```
This script automatically detects whether the Longan Nano is connected to the local USB port, on the remote testbed (`192.168.0.63`), or offline, launching the native application with the appropriate transport.

### 2. Manual CLI Options
```powershell
# Run with Direct USB HID
.\tools\current_monitor_gui\target\x86_64-pc-windows-msvc\release\current_monitor_gui.exe --hid

# Run with Local TCP Bridge
.\tools\current_monitor_gui\target\x86_64-pc-windows-msvc\release\current_monitor_gui.exe --tcp 127.0.0.1:5055

# Run in High-Performance Simulation Mode
.\tools\current_monitor_gui\target\x86_64-pc-windows-msvc\release\current_monitor_gui.exe --sim
```

### 3. Building from Source
```powershell
cd tools/current_monitor_gui
cargo build --release
```
The optimized release binary is located at `tools/current_monitor_gui/target/x86_64-pc-windows-msvc/release/current_monitor_gui.exe`.
