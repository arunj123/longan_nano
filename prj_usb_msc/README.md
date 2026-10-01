# Longan Nano - USB Mass Storage Class (MSC) Device

This project implements a standalone USB Mass Storage Bulk-Only Transport (BOT / BBB) device for the Sipeed Longan Nano (GD32VF103CBT6 RISC-V MCU), presenting an attached MicroSD card over SPI1 as a USB flash drive to host operating systems (Linux, Windows, macOS).

---

## 1. Project Status & Milestone Overview

- **Current Status**: **PAUSED** at **Build 00A3** (Commit `3b5a996`).
- **USB Device ID**: `VID: 0x28E9`, `PID: 0xABA3`
- **USB Strings**:
  - Manufacturer: `Sipeed`
  - Product: `Longan Nano SD Reader`
  - Serial Number: `LNMSC00000A3`
- **SCSI Inquiry**: Vendor `Sipeed`, Product `Longan Nano SD`, Revision `1.00`
- **Medium**: MicroSD card on SPI1 (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI).

### What Is Working
1. **Host Enumeration**:
   - Clean Full-Speed (12 Mbps) enumeration on Linux xHCI root hub (`usb 1-1`) without descriptor validation errors or host timeouts.
   - Identified by the Linux SCSI subsystem as `/dev/sdb` (e.g., 503 MB / 480 MiB, 983,040 sectors).
2. **Partition Table & MBR Mount**:
   - The Linux kernel successfully scans the partition table:
     ```
     sdb: sdb1
     sd 2:0:0:0: [sdb] Attached SCSI removable disk
     ```
   - Partition `/dev/sdb1` (`W95 FAT32 (LBA)`) attaches automatically.
3. **Single-Sector Direct Reads**:
   - `fdisk -l /dev/sdb` and `dd if=/dev/sdb bs=512 count=1 iflag=direct` complete cleanly at 1.2 MB/s with valid MBR `0x55AA` signature.
4. **SCSI Command Compliance**:
   - Implements `INQUIRY` (Standard + EVPD 0x00, 0x80, 0x83), `READ_CAPACITY_10`, `READ_CAPACITY_16` (0x9E), `MODE_SENSE_6` (0x1A), `REQUEST_SENSE` (0x03), `TEST_UNIT_READY` (0x00), `READ_FORMAT_CAPACITIES` (0x23), `SECURITY_PROTOCOL_IN` (0xA2, BitLocker Silo check), `SYNCHRONIZE_CACHE_10` (0x35), `READ_6` (0x08), and `READ_10` (0x28).
5. **Multi-Sector Hardware Pipelining**:
   - Firmware trace buffer confirms successful consecutive 8-sector (4096-byte) reads (LBA 0..7, 496..503, 528..535, 560..567) completing with status `0x00` and `residue = 0`.

---

## 2. Root Cause Analysis & Known Limitations

### Flash Allocation Unit (AU) Boundary Latency at LBA 1024
- **Symptom**: During sustained filesystem mounting or sequential host sweeps, Linux reports:
  ```
  sd 2:0:0:0: [sdb] tag#0 FAILED Result: hostbyte=DID_ERROR driverbyte=DRIVER_OK cmd_age=1s
  sd 2:0:0:0: [sdb] tag#0 CDB: Read(10) 28 00 00 00 04 00 00 00 08 00
  I/O error, dev sdb, sector 1024 op 0x0:(READ) flags 0x0 phys_seg 1 prio class 2
  ```
- **Root Cause**:
  - LBA 1024 sits on a 512 KB flash Allocation Unit (AU) boundary in the SD card. When reading across this boundary, the SD card internal controller performs internal house-keeping / block switching, delaying the `0xFE` data start token by ~3.5 ms.
  - In the current **single-buffered architecture**, reading sector $N+1$ from the SD card only starts after sector $N$ has finished transmitting over USB.
  - While the SD read is executing over SPI1, the USB IN endpoint is held in NAK mode.
  - If NAK holds for $\ge 3.0\text{ ms}$, the host xHCI Transaction Translator (TT) split-transaction retry threshold expires, triggering `COMP_USB_TRANSACTION_ERROR` (`EPROTO -71`, Linux `DID_ERROR`) and initiating a bus reset.

---

## 3. Prioritized Roadmap for Resuming USB MSC

When development on this project resumes, follow this execution roadmap:

1. **Ping-Pong Double Buffering ($2 \times 512\text{B}$)**:
   - Allocate two sector buffers: `media_buffer[0]` and `media_buffer[1]`.
   - While the DWC2 USB core transmits sector $N$ from buffer 0, the main loop / interrupt fetches sector $N+1$ from the SD card into buffer 1.
   - Completely eliminates the 1.1 ms SPI dead-time between sectors on the USB bus, preventing USB FIFO starvation and bringing AU boundary latency within tolerance.
2. **Pipelined Sector Prefetching in `poll()`**:
   - Overlap SD SPI block reads with USB IN packet transmission so that the next sector is already in RAM before the host issues the next IN token.
3. **CSW Framing Guard Verification**:
   - Verify that the CSW framing guard only inserts timing separation before the final CSW packet, without adding inter-sector latency.
4. **SPI1 Clock Prescaler Tuning**:
   - Currently operating at `BaudRatePrescaler::Div4` (~12 MHz). Evaluate `Div2` (~24 MHz) if signal integrity allows.
5. **Automated Stress Testing**:
   - Run `tools/test_lba_sweep.py` and `tools/test_reads_direct.py` across full disk ranges (LBA 0 to 65536).

---

## 4. Building and Flashing

### Build
From the repository root:
```powershell
$env:PYTHONUTF8=1; python bldmgr/build.py prj_usb_msc build
```

### Rebuild (Clean Build)
```powershell
$env:PYTHONUTF8=1; python bldmgr/build.py prj_usb_msc rebuild
```

### Flash (Target Hardware via OpenOCD JTAG)
```powershell
# Local JTAG:
$env:PYTHONUTF8=1; python bldmgr/build.py prj_usb_msc flash

# Remote Linux Host (192.168.0.63):
$env:PYTHONUTF8=1; python tools/remote_flash.py build/prj_usb_msc/firmware.hex
```

---

## 5. Diagnostic & Test Scripts

All test and diagnostic tools are maintained in `tools/`:

| Script | Purpose |
|---|---|
| `tools/unstick_host.py` | Kills stuck host disk processes and cycles xHCI port disable/enable to recover from USB error loops. |
| `tools/test_reads_direct.py` | Issues SCSI `READ_10` commands via `sg_raw` to benchmark single-sector and 8-sector multi-sector reads. |
| `tools/test_lba_sweep.py` | Sweeps LBAs using direct I/O (`dd iflag=direct`) across sectors 0 to 560. |
| `tools/dump_msc_trace.py` | Halts target via OpenOCD JTAG and dumps the in-memory circular trace buffer (`CBW_RECV`, `SCSI_RES`, `CSW_SENT`). |
| `tools/dump_stats.py` | Inspects live `g_msc_stats` (sectors read/written, last SD result code, error LBA). |
| `tools/dump_ep1_debug.py` | Dumps hardware IN endpoint state transitions and timestamp log from target SRAM. |
| `tools/check_status.py` | Dumps the active BOT context struct (`ctx.state`, `ctx.csw`, LBA, remaining bytes). |
