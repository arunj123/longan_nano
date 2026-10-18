# Longan Nano AI Maintainer Guide & Architectural Reference

**Document Status**: Authoritative Engineering Manual & AI Agent Guidelines  
**Target Hardware**: Sipeed Longan Nano (GD32VF103CBT6, RISC-V RV32IMAC @ 96/108 MHz, 32 KB SRAM, 128 KB Flash)  
**Primary Projects**: `prj_usb_msc` (Pure Rust USB MSC) & `prj_current_monitor` (Pure Rust Telemetry/Monitor)  
**Author / Audience**: Autonomous AI Agents & Embedded Firmware Engineers  

---

## 1. Architectural Philosophy & Repository Structure

The `longan_nano` repository provides vendor-library-free, bare-metal firmware for the GigaDevice GD32VF103 RISC-V microcontroller. The production codebase is implemented in **100% Pure Safe/Embedded Rust** (Edition 2021) utilizing zero-cost hardware abstractions:

```
longan_nano/
├── crates/
│   ├── gd32vf103-pac/        # Auto-generated & hand-curated peripheral access crate
│   ├── gd32vf103-hal/        # Zero-cost embedded-hal drivers (RCU, GPIO, SPI, I2C, USB, DMA, ECLIC)
│   ├── longan-nano-bsp/      # Board support package (LEDs, LCD ST7735, PA8 button, INA219, RTC, BKP)
│   ├── lcd_font/             # Bitmapped embedded font subset engine
│   └── embedded-sdmmc/       # Pure-Rust SPI SD card block device driver
├── prj_current_monitor/      # Flagship 10 Hz current/voltage/power/energy monitor & USB Drive
├── prj_usb_msc/              # Flagship high-speed USB Mass Storage Class (BOT / SCSI) device
├── prj_usb_composite/        # Legacy reference C++/Rust composite HID device
├── tools/                    # Automated hardware verification, telemetry, and flashing scripts
│   ├── ssh_util.py           # Centralized remote SSH execution & credential manager
│   ├── remote_flash.py       # OpenOCD JTAG remote flasher over SSH
│   ├── remote_uart_monitor.py# Resilient remote UART monitor
│   ├── test_usb_drive_mode.py# Automated USB drive mode hardware test suite
│   └── stream_remote_monitor.py # 10 Hz live telemetry test client
├── docs/                     # Specifications, datasheets, and master roadmap
│   ├── FIRMWARE_REVIEW_AND_ROADMAP.md # Master tracking document (MANDATORY)
│   └── AI_MAINTAINER.md      # This document
└── bldmgr/                   # Python build manager & size analyzer
```

---

## 2. Hardware Specifications & Invariants

### 2.1 MCU & Architecture (GD32VF103CBT6)
- **Core**: Nuclei Bumblebee N205 (RV32IMAC: Base 32-bit Integer, Hardware Integer Multiply/Divide, Atomic, Compressed 16-bit instructions).
- **FPU Policy**: **No hardware FPU**. Strictly avoid software floating-point emulation (`f32`/`f64`). All DSP, telemetry, and display routines use fixed-point integer math (tenth of mA, tenth of °C, mV, mW, mWh, mAh).
- **Memory Alignment Rule**: RV32IMAC traps on unaligned multi-byte memory loads (`lw`). Sector and stream buffers must maintain 32-bit alignment via `#[repr(C, align(4))]`, or use byte-level accessors (`ld_word`, `st_word`, `core::ptr::read_unaligned`).

### 2.2 Memory Map & Linker Headroom ([MON-MEM-02])
The GD32VF103CBT6 features **32 KB (32,768 bytes) of physical SRAM** and **128 KB of Flash**:
- Real System V ELF section breakdown (`riscv-none-elf-size -A`):
  - `.text`: ~91.9 KB (Flash)
  - `.rodata`: ~31.4 KB (Flash, total Flash text = ~123.3 KB / 94.1%)
  - `.data`: 0 bytes (Initialized RAM)
  - `.bss`: 8,196 bytes (Static RAM: contains the 8 KB LBA sector cache block)
  - `.stack`: 24,572 bytes (Allocated to runtime stack by `riscv-rt`)
- **Important**: Berkeley `size` lumps `.stack` into the `bss` column ($8,196 + 24,572 = 32,768$), which misleadingly appears as 100% RAM usage. Real static RAM consumption is only **25.0%**, leaving **24.0 KB (75.0%)** of runtime stack headroom.

### 2.3 Clock Tree & Multiplier Encoding
- **USB Applications (`96 MHz SYSCLK`)**:
  - Driven by `RCU_PLL_MUL24` from an 8 MHz external crystal (HXTAL).
  - USBFS requires an exact 48 MHz clock: $96\text{ MHz} / 2 = 48.0\text{ MHz}$.
  - **Critical Bit Encoding**: In GD32VF103 `RCU_CFG0`, `PLLMF[4:0] = 13` encodes $\times 6.5$, and `PLLMF = 14` encodes $\times 16$. Multipliers $16\times..30\times$ use `pllmf = multiplier - 1`. Therefore `MUL24` requires `(0x7 << 18) | (1 << 29)` (`PLLMF = 23`). Setting `PLLMF = 22` results in 92 MHz (46 MHz USB, $-4.17\%$ clock error that triggers NRZI CRC failures `EPROTO -71`).
- **Non-USB Applications (`108 MHz SYSCLK`)**:
  - Uses `RCU_PLL_MUL27` (`(0xA << 18) | (1 << 29)`).
- **Timer Frequency**:
  - RISC-V hardware `mtime` runs at `SystemCoreClock / 4` (24 MHz at 96 MHz SYSCLK, 27 MHz at 108 MHz SYSCLK). Always compute delay loops using dynamic `mtime` frequency or typed `hal::time` utilities.

---

## 3. Peripheral Drivers & Subsystem Architecture

### 3.1 USB Full-Speed Device Subsystem (Synopsys DWC2)
1. **Empty-FIFO ZLP Race Elimination**:
   - If `EPEN=1` is asserted on an IN endpoint while `NAKS=0` and the TX FIFO is empty, incoming host IN tokens trigger an empty-FIFO Zero-Length Packet (ZLP), corrupting packet counter `DIEPLEN.PCNT`.
   - Firmware enforces a two-phase arming protocol:
     1. Write `DEPCTL_SNAK` (with strobe bits masked).
     2. Poll `while (!(DIEPCTL & DEPCTL_NAKS) && --timeout) {}` to guarantee hardware confirmation of NAK status.
     3. Program `DIEPLEN`.
     4. Assert `DEPCTL_EPEN`.
2. **Immediate Inter-Phase NAK Pre-Arming**:
   - Host controllers (e.g., Linux/Windows xHCI) issue Bulk-IN data URBs within $\le 35\ \mu\text{s}$ of CBW ACK. Endpoint arming must occur immediately inside `process_scsi()` before exiting the receive ISR (`usbd_ep_nak_arm()`), preventing `stat=-71 EPROTO` bus resets.
3. **Bounded FIFO Polling ([MON-HAL-01])**:
   - All `GRSTCTL.TXFF` and `RXFF` FIFO flush polling loops are strictly bounded by `timeout = 100_000u32` across all USB classes (`UsbHid`, `UsbCdcAcm`, `UsbComposite`, `UsbMsc`).

### 3.2 Storage & USB Drive Mode Lifecycle
1. **Read-Only Default MSC Protection ([MON-STOR-07])**:
   - To eliminate filesystem corruption between host OS writes and embedded FAT32 append logging, USB Drive Mode defaults to **Read-Only** (`msc_write_protect: true`).
   - Host write commands return SCSI sense code `0x07` (`DATA_PROTECT`, `WRITE_PROTECTED`). A documented override command (`w`/`W` in CLI) allows temporary read-write access.
2. **Double File-Number Increment Elimination ([MON-STOR-04])**:
   - On exiting USB Drive Mode, `check_probe()` scans the directory and sets `session_file_num`. If disk scanning already advanced the session number, `rotate_session()` is skipped, preventing gaps in CSV logging sequence (`LOG_0136 -> LOG_0137`).
3. **Tare & Session Reset Safety ([MON-STOR-06])**:
   - `tare_and_reset()` flushes buffered rows before zeroing session accumulators. If SD flush fails, session reset aborts with `"SD FLUSH ERR: ABORTED"`, preserving disk-RAM synchronization.
4. **Sample Integration Decoupling ([MON-MATH-01])**:
   - Physical energy and charge accumulation are integrated via `dt_sample_ms = now.wrapping_sub(app.last_sample_ms)`. This decouples integration from the 10 Hz trigger schedule, eliminating double-counting during loop stalls.

### 3.3 UI & Screen Registry ([MON-AI-04], [MON-UI-03])
1. **Central Screen Registry (`ScreenMode`)**:
   - Screens are centrally managed via `prj_current_monitor/src/ui/mod.rs`:
     - `ScreenMode::COUNT = 6`
     - `ScreenMode::ALL = [Hero, Graph, Stats, Histogram, BigDigit, UsbDrive]`
     - `mode.index() -> usize`
     - `ScreenMode::from_index(idx) -> Option<ScreenMode>`
     - `mode.next() -> ScreenMode`
     - `mode.prev() -> ScreenMode`
     - `mode.name() -> &'static str`
     - `mode.short_name() -> &'static str`
2. **Display Drawing Ergonomics**:
   - In `GraphScreen::draw_layout()`, `redraw_grid` is skipped unless `trigger_mode == TriggerMode::Single`, avoiding 105 redundant `set_pixel` SPI transactions immediately overwritten by `replot_history`.
   - Initial MCU die temperature is sampled once at boot via `board.mcu_temp.read_temperature_tenth_c()`, preventing uncalibrated 25.0 °C telemetry reports during the first second.

---

## 4. Developer Workflows & Verification

### 4.1 Host Unit Testing
Run workspace unit tests directly on the host development machine without flashing hardware:
```powershell
cargo test --workspace --target x86_64-pc-windows-msvc
```
*Current Baseline: 146 tests passing across workspace (61 in `prj_current_monitor`, 11 in `prj_usb_msc`).*

### 4.2 Building Firmware
Build release binaries using Cargo or the standalone build manager:
```powershell
# Build prj_current_monitor
cargo build --release -p prj_current_monitor

# Build via build manager (automatically prints ELF sections & size breakdown)
$env:PYTHONUTF8=1; python bldmgr/build.py prj_current_monitor build
```

### 4.3 Flashing Target Hardware
Flash the compiled binary to the remote testbed (`192.168.0.63`) using OpenOCD JTAG over SSH:
```powershell
python tools/remote_flash.py target/riscv32imac-unknown-none-elf/release/prj_current_monitor.hex
```

### 4.4 Automated Remote Testing Suites
Execute non-interactive regression test suites against the live target:
```powershell
# Test USB Drive Mode transitions, SCSI read/eject lifecycle, and file rotation
python tools/test_usb_drive_mode.py

# Test 10 Hz live USB HID telemetry stream and INA219 sensor reads
python tools/stream_remote_monitor.py

# Comprehensive 30-packet telemetry verification suite
python tools/test_remote_current_monitor.py
```

### 4.5 Remote SSH Credentials Configuration ([MON-SEC-01])
All test and flashing scripts in `tools/*.py` utilize `tools/ssh_util.py`. Credentials can be configured via environment variables:
- `LONGAN_SSH_HOST` (default: `192.168.0.63`)
- `LONGAN_SSH_USER` (default: `arun`)
- `LONGAN_SSH_PASS` (fallback if keys not configured)
- `LONGAN_SSH_KEY` (path to private key)
- `LONGAN_SSH_STRICT_HOST_CHECK` (`1` to enforce strict host key checking)

---

## 5. Mandatory AI Session Invariants

> [!IMPORTANT]
> Autonomous AI agents working in this repository MUST strictly follow these three invariants:

1. **Master Roadmap Tracking (`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`)**:
   - Check `git status` and read the active phase and next steps in `docs/FIRMWARE_REVIEW_AND_ROADMAP.md` at the start of every session.
   - Update task checkboxes (`[x]`) and record empirical results at the completion of every milestone.
2. **Strict Per-Build Git Commits**:
   - Every tested build or discrete code iteration requires an immediate git commit following the standardized 5-part schema:
     - **Header**: `build(<component>): Build <NNNN> - <Concise Headline>`
     - **Summary of Changes**: Bulleted breakdown of code, register, timing, or configuration adjustments.
     - **Test Results**: Concrete empirical observations from hardware/host testing.
     - **Root Cause & Diagnostics**: Detailed register states, hardware counter readings, or findings explaining any observed failures. (Include compiler/debug logs if failed).
     - **What Next**: Prioritized, actionable roadmap outlining the exact steps for the next iteration.
   - Never batch multiple builds into a single commit.
3. **Economic AI Session Handoff**:
   - When a roadmap milestone is completed or conversation context grows long, ensure all changes are tested, documented in the roadmap, and committed to git.
   - Prompt the user to start a new chat session, providing the exact continuation prompt.
