# Longan Nano - Tools & Diagnostic Utilities

This directory contains host-side Python and system utilities for flashing, serial monitoring, regression testing, and telemetry for firmware running on the Sipeed Longan Nano (GD32VF103CBT6).

The toolkit is designed for **dual-environment operation**:
- **Local Windows Host**: When the board is connected directly via USB/JTAG to Windows.
- **Remote Linux Host**: When the board is connected to a dedicated remote testbed (`192.168.0.63`) over SSH.

---

## 1. Environment & Centralized Credentials (`ssh_util.py`)

All remote utilities share centralized authentication and host configuration in [`tools/ssh_util.py`](ssh_util.py):

| Environment Variable | Default | Purpose |
|---|---|---|
| `LONGAN_SSH_HOST` | `192.168.0.63` | Remote testbed IP or hostname |
| `LONGAN_SSH_USER` | `arun` | SSH login username |
| `LONGAN_SSH_PASS` | `arun` | SSH login password |
| `LONGAN_SSH_KEY` | *(None)* | Optional path to private SSH key |
| `LONGAN_SSH_STRICT_HOST_CHECK` | `0` | Set `1` to enforce strict host key verification |
| `LONGAN_UART_PORT` | `COM13` | Default local UART COM port on Windows |

---

## 2. Flashing & In-Circuit Debugging

- **`tools/remote_flash.py`**:
  Uploads a `.hex` binary to the remote Linux host via SFTP and programs the GD32VF103 over JTAG using OpenOCD with SRST-less execution resume.
  ```powershell
  python tools/remote_flash.py build/prj_current_monitor/firmware.hex
  ```

- **`tools/flash_and_monitor.py`**:
  Uploads `.hex`, flashes via OpenOCD, and immediately streams UART logs for a configurable duration (default 12 seconds).
  ```powershell
  python tools/flash_and_monitor.py build/prj_current_monitor/firmware.hex
  ```

- **Local Windows Flashing via `bldmgr`**:
  When connected directly to Windows, use the Python build manager:
  ```powershell
  $env:PYTHONUTF8=1; python bldmgr/build.py prj_current_monitor flash
  ```

---

## 3. UART Monitoring

- **`tools/uart_monitor.py` (Local Windows / Linux)**:
  Resilient serial monitor that automatically reconnects when the target device is reset or disconnected.
  - Automatically identifies FTDI FT2232D Interface 1 (`0403:6010`) on Windows.
  - Supports CLI port arguments: `python tools/uart_monitor.py COM13` (or `/dev/ttyUSB1`).
  ```powershell
  python tools/uart_monitor.py
  ```

- **`tools/remote_uart_monitor.py` (Remote Linux)**:
  Streams real-time serial output from `/dev/ttyUSB1` @ 115200 baud over SSH.
  ```powershell
  python tools/remote_uart_monitor.py
  ```

---

## 4. Remote Host Administration & Recovery

- **`tools/unstick_host.py`**:
  Recovers the remote Linux testbed when a USB device hangs or triggers an xHCI reset loop. Kills stuck `fdisk`/`dd`/`sg_raw` processes, cycles xHCI port disable/enable via sysfs, and dumps `dmesg`.
  ```powershell
  python tools/unstick_host.py
  ```

- **`tools/check_remote_state.py`**:
  Inspects kernel logs (`dmesg`), block devices (`lsblk`), and hung disk processes on the remote testbed.
  ```powershell
  python tools/check_remote_state.py
  ```

- **`tools/remote_exec.py`**:
  Runs arbitrary commands on the remote Linux testbed with exit code forwarding and optional `--sudo` elevation:
  ```powershell
  python tools/remote_exec.py --sudo "dmesg | tail -n 20"
  ```

---

## 5. Current Monitor Telemetry & Desktop Applications

- **`tools/run_monitor_gui.py`**:
  One-click launcher for the native 3D Rust desktop monitor (`current_monitor_gui`).
  - Auto-detects local Windows USB HID (`0x28E9:0x1234`).
  - If not local, automatically launches `hid_tcp_bridge.py` to the remote Linux testbed.
  - If no hardware is detected, launches in Simulation Mode.
  ```powershell
  python tools/run_monitor_gui.py
  ```

- **`tools/stream_remote_monitor.py`**:
  Streams live 10 Hz telemetry (voltage, current, power, energy, flags, screen mode) directly to the console.
  ```powershell
  python tools/stream_remote_monitor.py
  ```

- **`tools/test_remote_current_monitor.py`**:
  Automated test suite verifying 30 consecutive 10 Hz HID telemetry packets with sanity bounds checking.
  ```powershell
  python tools/test_remote_current_monitor.py
  ```

- **`tools/test_remote_ui_cycling.py`**:
  Automated test suite verifying LCD screen cycling across all 6 screens, PA8 gesture simulation, and CLI console commands.
  ```powershell
  python tools/test_remote_ui_cycling.py
  ```

- **`tools/hid_tcp_bridge.py`**:
  Bidirectional TCP bridge linking remote `/dev/hidraw0` over SSH to local `localhost:5055` for Windows GUI tools.

---

## 6. USB Mass Storage (MSC) Regression Testing

- **`tools/test_usb_drive_mode.py`**:
  Production regression suite testing Screen 6 USB Drive Mode:
  - Enters USB Drive Mode via simulated PA8 double-click or CLI command.
  - Verifies dynamic USB re-enumeration (`28e9:abd4`).
  - Tests SCSI mount, FAT32 directory listing, and CSV row reading.
  - Verifies session log rotation (e.g., `LOG_0152.CSV` -> `LOG_0153.CSV`).
  - Exits USB Drive Mode and verifies return to Standby.
  ```powershell
  python tools/test_usb_drive_mode.py
  ```

- **`tools/test_reads_direct.py`**:
  Direct SCSI sector benchmark using `sg_raw`. Tests single sectors (LBA 0, 32, 64, 128, 256, 496, 528, 560), 8-sector multi-sector reads (4096 bytes), and flash AU boundary behavior.
  ```powershell
  python tools/test_reads_direct.py
  ```

- **`tools/test_lba_sweep.py`**:
  Sector sweep using direct I/O (`dd iflag=direct`) across sectors 0 to 560.
  ```powershell
  python tools/test_lba_sweep.py
  ```

- **`tools/pcap_capture.py` & `tools/parse_pcap.py`**:
  Captures and analyzes low-level USB BOT/SCSI transactions via `usbmon` and `tcpdump`:
  ```powershell
  python tools/pcap_capture.py
  python tools/parse_pcap.py build/prj_usb_msc/msc.pcap
  ```

- **`tools/test_virtual_msc.py`**:
  Automated USB/IP virtual mass storage simulator for host-side regression testing without physical hardware.

---

## 7. USB CDC-ACM Serial & Custom HID Testing

- **`tools/testserial.py` (Local Windows)**:
  Scans COM ports on Windows, connects to GD32VF103 CDC-ACM, sends test message, and verifies echo.
  ```powershell
  python tools/testserial.py
  ```

- **`tools/test_extended_usb_serial.py` (Remote Linux)**:
  Tests CDC-ACM echo across multiple payload sizes on the remote Linux host.
  ```powershell
  python tools/test_extended_usb_serial.py
  ```

- **`tools/test_custom_hid.py` (Local Windows)**:
  Sends custom HID frames (`0x28E9:0xABE7`) for testing `prj_usb_composite`.

- **`tools/test_remote_composite.py` (Remote Linux)**:
  Sends custom HID drawing commands to `/dev/hidraw1` over SSH.

- **`tools/display_manager/`**:
  Full desktop Python application streaming live clock, weather, and system stats to the ST7735 LCD over USB HID.

---

## 8. Asset & Font Generators

- **`tools/gen_font28.py`**: Generates 28px Bahnschrift TrueType proportional font glyphs for Rust `crates/lcd_font`.
- **`tools/gen_font48.py`**: Generates 48px Bahnschrift TrueType proportional font glyphs for Rust `crates/lcd_font`.
- **`tools/image_to_bin.py`**: Converts image files into raw RGB565 binary bitmaps for embedded ST7735 LCD rendering.
- **`tools/generate_font_library.py`**: Generates C++23 font library headers for legacy `lib/gd32v_lcd`.
