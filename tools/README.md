# Longan Nano - Tools & Diagnostic Utilities

This directory contains host-side Python and PowerShell utilities for programming, monitoring, and regression testing firmware projects on the Sipeed Longan Nano.

---

## 1. Flashing & In-Circuit Debugging

- **`tools/remote_flash.py`**:
  Uploads a `.hex` binary to the remote Linux host (`192.168.0.63`) via SFTP and programs the GD32VF103 over JTAG using OpenOCD with SRST-less execution resume.
  ```powershell
  python tools/remote_flash.py build/prj_usb_msc/firmware.hex
  ```

- **`tools/uart_monitor.py`**:
  Local serial monitor on Windows (Virtual COM port `COM13` @ 115200 baud). Automatically reconnects on USB disconnect/reconnect.
  ```powershell
  python tools/uart_monitor.py
  ```

- **`tools/remote_uart_monitor.py`**:
  Streams live serial output from `/dev/ttyUSB1` @ 115200 baud over SSH from the remote Linux host.
  ```powershell
  python tools/remote_uart_monitor.py
  ```

---

## 2. USB Mass Storage (MSC) Regression Testing & Diagnostics

These utilities are essential for verifying that refactoring in shared common code (`hal/`, `bsp/`, `drivers/`) does not cause regressions in USB Mass Storage functionality:

- **`tools/unstick_host.py`**:
  Recovers the remote Linux testbed when a device hangs or enters an xHCI error reset loop. Kills stuck `fdisk`/`dd`/`sg_raw` processes, cycles xHCI port disable/enable via sysfs, and prints the resulting `dmesg`.
  ```powershell
  python tools/unstick_host.py
  ```

- **`tools/test_reads_direct.py`**:
  Direct SCSI sector test using `sg_raw`. Tests single sectors (LBA 0, 32, 64, 128, 256, 496, 528, 560), 8-sector multi-sector reads (4096 bytes), and benchmark timings.
  ```powershell
  python tools/test_reads_direct.py
  ```

- **`tools/test_lba_sweep.py`**:
  Automated sector sweep using direct I/O (`dd iflag=direct`) across sectors 0 to 560. Halts immediately on any unexpected error.
  ```powershell
  python tools/test_lba_sweep.py
  ```

- **`tools/dump_stats.py`**:
  Connects via OpenOCD JTAG to read `g_msc_stats` from target SRAM (`0x20000f20`). Outputs sectors read, sectors written, last SD result, and last error LBA.
  ```powershell
  python tools/dump_stats.py
  ```

- **`tools/dump_msc_trace.py`**:
  Dumps the 64-entry in-memory circular trace buffer from target SRAM (`0x20000200`). Decodes `CBW_RECV`, `SCSI_RES`, `CSW_SENT`, CDB opcodes, and residue.
  ```powershell
  python tools/dump_msc_trace.py
  ```

- **`tools/dump_ep1_debug.py`**:
  Dumps EP1 hardware registers and state transition timestamps (`0x20000f50`), allowing microsecond-level analysis of NAK/CNAK arming and FIFO writes.
  ```powershell
  python tools/dump_ep1_debug.py
  ```

- **`tools/check_status.py`**:
  Dumps the active BOT context struct (`0x20000690`) over JTAG to inspect CBW, CSW, and transfer state.
  ```powershell
  python tools/check_status.py
  ```

---

## 3. HID & Display Diagnostics

- **`tools/test_custom_hid.py`**:
  Sends test packets or raw display frames to `prj_usb_composite` over Custom HID.
  ```powershell
  python tools/test_custom_hid.py
  ```

- **`tools/display_manager/`**:
  Full Python application streaming live clock, weather, and system stats to the ST7735 LCD over USB HID.
