# Firmware Historical Archive: Phase 5

**Archive Reference**: Archived from [`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`](../FIRMWARE_REVIEW_AND_ROADMAP.md) per Roadmap Archival & Size Management policy.  
**Target Architecture**: Sipeed Longan Nano (GD32VF103CBT6, RISC-V RV32IMAC @ 96/108 MHz, 32 KB SRAM, 128 KB Flash).

---

## Historical Overview of Completed Phase 5

| Phase | Description | Key Deliverables & Verifications | Verified Builds |
|---|---|---|---|
| **Phase 5** | Dedicated USB Drive Mode & Production Hardening | Screen 6 USB Drive Mode (`Standby` <-> `Active`), dynamic USB soft-disconnect re-enumeration engine (`UsbMode::into_inner`), zero dual-master FAT32 corruption safety, hardware/software Write-Protect toggle & SCSI `MODE_SENSE_6`/`10` protection, BKP6 persistence, non-breaking stream cache lookup policy (CMD18 read throughput 846 kB/s), workspace-wide host unit test harness (`cargo test` passes 92 tests). | Build `00E1` (Drive Mode), Build `00E2` (Write Protect), Build `00E3` (Cache Policy) |

---

## Detailed Task Breakdown & Implementation Records

### Phase 5: Dedicated USB Drive Mode & Production Hardening (COMPLETED)
- [x] **Task 5.1**: Implement Dedicated USB Drive Mode on Screen 6 for `prj_current_monitor` (Verified in Build `00E1`):
  - **FAT32 Concurrency & Safety (Elimination of Two Masters)**:
    - Entering Screen 6 enters `Standby` (instructions displayed, logging remains safe, USB remains HID).
    - Double-click or long-press on PA8 toggles Drive Mode state (`Standby` <-> `Active`).
    - When transitioning to `Active`: flushes buffer (`datalogger.flush_buffer()`), safely closes FAT file handle, unmounts `embedded-sdmmc`, and hands exclusive raw sector access to `UsbMsc`.
    - When transitioning to `Standby` or exiting Screen 6 (single-click advancing to Hero, or UART switch): automatically detaches `UsbMsc` (`DCTL.SDIS = 1`), restores `UsbHid`, re-probes SD card, rotates to the next sequential file (`LOG_XXXX.CSV`), and resumes logging seamlessly.
  - **Dynamic USB Re-enumeration Engine**:
    - Added `into_inner(self) -> Usbfs` to `UsbHid`, `UsbMsc`, `UsbCdcAcm`, and `UsbComposite` in `crates/gd32vf103-hal/src/usb/` asserting soft-disconnect (`dctl.SDIS = 1`) and disabling global USB interrupts (`gahbcs.GINTEN = 0`).
    - Added `UsbMode` enum (`Hid(UsbHid)`, `Msc(UsbMsc)`, `None`) and `switch_to_msc()` / `switch_to_hid()` transition helpers in `CurrentMonitorBoard` with a 50 ms disconnect settle delay.
  - **UI/UX & Screen 6 Component**:
    - Extended `ScreenMode` with `ScreenMode::UsbDrive` (Screen 6).
    - Implemented `screen_usbdrive.rs` with clean status badges (`[STANDBY]` vs `[PC MOUNTED]`), active log file, MicroSD capacity, live read/write sector counters, and PA8 gesture prompts (`DBL-CLICK/HOLD: MOUNT`, `CLICK PA8: EXIT/RESUME`).
- [x] **Task 5.2**: Implement Hardware/Software Write-Protect Toggle & SCSI `MODE_SENSE_6`/`10` Protection Handling in `prj_usb_msc` (Verified in Build `00E2`):
  - Add Write-Protect (WP) bit reporting in SCSI `MODE_SENSE_6` (Byte 2, bit 7 `0x80`) and `MODE_SENSE_10` (Byte 3, bit 7 `0x80`).
  - Reject `WRITE_6` and `WRITE_10` with `SenseKey::DataProtect (0x07)`, `Asc::WriteProtected (0x27, 0x00)` when write-protect is active.
  - Add Write-Protect state toggle via UART command (`'w' | 'W'`) and PA8 double-click gesture in `prj_usb_msc`, persisting across reboots in Backup Domain register 6 (`BKP_DATA6`).
  - Display `[RO]` vs `[RW]` in the live LCD telemetry header badge.
- [x] **Task 5.3**: Implement Non-Breaking Stream Cache Lookup Policy ([MSC-PERF-04]) (Verified in Build `00E3`):
  - In `crates/gd32vf103-hal/src/usb/msc/mod.rs`, optimized read-ahead pipelining so that sequential cache hits during an active `CMD18` multi-block read do not invoke `disk.end_multi_read()`.
  - Maintained the active SPI streaming read pipeline across cache hits for large block transfers (> 8 blocks), only consulting cache when starting a new transfer or for small isolated reads ($\le 8$ blocks).
  - Added round-robin sector caching (`insert_lba`) across 16 cache slots for small transfers ($\le 8$ blocks), speeding up repeated partition table and FAT sector lookups.
- [x] **Task 5.4**: Fix Workspace-Wide Host Unit Test Harness & `panic_halt` Guarding:
  - Guarded `panic_halt`, `riscv_rt::entry`, and `main()` in auxiliary crates (`prj_blink`, `prj_uart_test`, `prj_lcd_test`, `prj_sdcard_test`, `prj_sdcard_fs_test`, `prj_usb_serial`, `prj_usb_composite`) with `#[cfg(not(test))]`.
  - Verified pure `cargo test --workspace --target x86_64-pc-windows-msvc` passes 100% cleanly across all workspace crates (89 passed tests, 3 doc-tests, 0 failed, 0 errors).

---

## Detailed Empirical Verification Logs: Phase 5

### 1. `prj_current_monitor` Dedicated USB Drive Mode on Screen 6 (Build 00E1)
- **Dynamic USB Peripheral Ownership & Soft-Disconnect (`crates/gd32vf103-hal/src/usb/`, `crates/longan-nano-bsp/src/lib.rs`)**:
  - Implemented `into_inner(self) -> Usbfs` across `UsbHid`, `UsbMsc`, `UsbCdcAcm`, and `UsbComposite`, asserting `dctl.sdis = 1` (soft-disconnect) and clearing `gahbcs.ginten = 0` to cleanly detach from the host root hub.
  - Added `UsbMode` enum (`Hid(UsbHid)`, `Msc(UsbMsc)`, `None`) and stored `rcu: hal::pac::rcu::Rcu` inside `CurrentMonitorBoard`, enabling `switch_to_msc()` and `switch_to_hid()` with a 50 ms bus disconnect settle delay before re-initializing the DWC2 core.
- **Zero Dual-Master FAT32 Corruption Architecture (`prj_current_monitor/src/app.rs`, `main.rs`)**:
  - Navigating to Screen 6 (`ScreenMode::UsbDrive`) starts in `UsbDriveState::Standby`, keeping USB HID active and datalogging undisturbed.
  - Toggling to `UsbDriveState::Active` (via PA8 double-click, PA8 1.5s hold, or UART `'u'`) flushes the pending 512-byte CSV sector buffer (`datalogger.flush_buffer()`), terminates any active SPI multi-block stream, suppresses the 1 Hz datalogger in `main.rs`, and transitions the USB peripheral to `UsbMsc` (`28e9:abd4`).
  - Exiting `Active` mode (via PA8 single-click, double-click, hold, rotary encoder turn, or UART screen switch) terminates any active MSC stream, switches USB back to `UsbHid` (`28e9:1234`), re-probes the MicroSD card, and rotates to the next sequential `LOG_XXXX.CSV` file.
- **Flash Optimization (`Cargo.toml`)**:
  - Switching release profile optimization from `opt-level = "s"` to `opt-level = "z"` reduced the combined `UsbHid` + `UsbMsc` + `embedded-sdmmc` firmware image from 131,172 bytes (100B overflow) down to **122,056 bytes** (`-9,116 bytes`), leaving **9,016 bytes** of free Flash headroom within the 128 KB GD32VF103 Flash.
- **Host Unit Test Suite (`cargo test`)**:
  - **77/77 host unit tests pass** across the workspace (27 in `gd32vf103-hal`, 6 in `longan-nano-bsp`, 34 in `prj_current_monitor`, 10 in `prj_usb_msc`), including 3 new unit tests in `ui::screen_usbdrive::tests`.
- **Target Hardware Empirical Verification (`tools/test_usb_drive_mode.py`)**:
  - Flashed to GD32VF103 over JTAG OpenOCD on remote Linux host `192.168.0.63`.
  - **12/12 automated hardware verification checks passed**:
    - Screen 6 `Standby` entry via UART `'6'`, preserving `28e9:1234` HID enumeration.
    - Simulated PA8 double-click (`'K'`) transitioned Screen 6 to `Active`, detached `28e9:1234`, and re-enumerated on Linux xHCI as `28e9:abd4` (`Longan Nano SD Reader`, `/dev/sdb`, `/dev/sg1`).
    - Direct SCSI `READ_10` of LBA 0 via `sg_raw` succeeded (valid MBR signature `0x55aa`).
    - Host OS mounted `/dev/sdb1` FAT32 partition cleanly and verified 77 `LOG_*.CSV` files on disk (`LOG_0109.CSV`).
    - Simulated PA8 single-click (`'k'`) automatically unmounted MSC, restored `28e9:1234` HID, advanced to Screen 1 (`HERO`), re-probed the SD card, rotated the active session file (`LOG_0109.CSV -> LOG_0111.CSV`), and resumed 10 Hz USB HID telemetry (`v_mv: 3320, seq: 7`) on `/dev/hidraw1`.

### 2. `prj_usb_msc` Hardware & Software Write-Protect Support & BKP6 Persistence (Build 00E2)
- **SCSI Write-Protect Compliance (`scsi_cmd::MODE_SENSE_6` / `MODE_SENSE_10` & `WRITE_6` / `WRITE_10`)**:
  - In `crates/gd32vf103-hal/src/usb/msc/mod.rs`, added `pub write_protected: bool` with `is_write_protected()` and `set_write_protected()`.
  - In `MODE_SENSE_6`, Byte 2 (Device-Specific Parameter) bit 7 (WP) dynamically reports `0x80` when write-protected and `0x00` in read-write mode.
  - In `MODE_SENSE_10`, Byte 3 bit 7 (WP) dynamically reports `0x80` when write-protected and `0x00` in read-write mode.
  - On receiving `WRITE_6` or `WRITE_10` while `self.write_protected` is active: immediately sets `SenseKey::DataProtect (0x07)` and `Asc::WriteProtected (0x27)`, invokes `abort_transfer(disk)` (stalling OUT EP1 and pending CSW), cleanly rejecting write attempts without card damage or silent corruption.
  - Bumped USB serial string descriptor and SCSI VPD page 0x80 serial to `LNMSC00000E2`.
- **Backup Domain Register 6 (`BKP_DATA6`) Persistence & PA8 Gestures (`prj_usb_msc/src/main.rs`, `bsp/lcd.rs`)**:
  - Defined `BKP_REG_WRITE_PROTECT: usize = 6`, `BKP_WRITE_PROTECT_RW: u16 = 0x5700` (`'W'`, 0), `BKP_WRITE_PROTECT_RO: u16 = 0x5701` (`'W'`, 1) in `longan-nano-bsp`.
  - On boot, restores Write-Protect state directly from `BKP_DATA6`.
  - Added PA8 button gesture discrimination:
    - Single-Click: toggles display rotation (saved to `BKP_DATA5`).
    - Double-Click (second tap within 280 ms): toggles Write-Protect mode (`RW` <-> `RO`, saved to `BKP_DATA6`).
  - Added UART interactive commands: `'w' | 'W'` toggles Write-Protect; `'k'` simulates single-click; `'K'` simulates double-click; `'s'` queries rotation (`BKP5`) and WP (`BKP6`); `'?'` prints command help.
- **Live LCD Telemetry Dashboard Header Badge (`telemetry.rs`)**:
  - Added `fmt_wp_badge(write_protected: bool) -> (&'static str, u16)` returning `[RO]` (Yellow) vs `[RW]` (Green).
  - Renders WP badge at X: 90, Y: 2 alongside header status badge `[DISC]` / `[IDLE]` / `[BUSY]` at X: 118, Y: 2.
  - Repositioned header bar title to `"USB MSC"` (X: 4, Y: 2) providing clean pixel layout.
- **Host Unit Test Suite (`cargo test`)**:
  - **82/82 host unit tests pass** across the workspace (30 in `gd32vf103-hal`, 7 in `longan-nano-bsp`, 34 in `prj_current_monitor`, 11 in `prj_usb_msc`), with 5 new tests verifying `MODE_SENSE_6`/`10` WP bytes, `SenseKey::DataProtect` & `Asc::WriteProtected` constants, `write_protected` flag operations, `BKP_REG_WRITE_PROTECT` constants, and `fmt_wp_badge` rendering.
- **Target Hardware Empirical Verification (`tools/verify_build_00e2.py`)**:
  - Flashed to GD32VF103 over JTAG OpenOCD on remote Linux host `192.168.0.63`.
  - Clean USB enumeration as `28e9:abd4` `Longan Nano SD Reader`, serial `LNMSC00000E2` (480 MiB / 983,040 blocks).
  - All UART commands and simulated gestures verified: help menu (`?`), status (`s`), UART WP toggle (`w`), PA8 simulated double-click (`K`), PA8 simulated single-click (`k`).
  - Verified `MODE_SENSE_6` byte 2 = `0x00` and `MODE_SENSE_10` byte 3 = `0x00` in RW mode.
  - Switched to RO mode: verified `MODE_SENSE_6` byte 2 = `0x80` and `MODE_SENSE_10` byte 3 = `0x80`.
  - Verified `WRITE_10` in RO mode was rejected with exit code 7 (`SCSI Status: Check Condition`, `Sense key: Data Protect`, `Additional sense: Write protected`).
  - Switched back to RW mode: all 7 isolated `sg_raw` reads passed, sustained 4.0 MiB read streaming achieved **838 kB/s**.
  - Partition 1 (`/dev/sdb1`) mounted read-write cleanly, single-file write verified, and multi-block 128 KiB CMD25 write streaming verified with 100% cryptographic MD5 match (`51d30322d34685b9a8417e116bee0a3c`).
  - Tested BKP persistence across reboot: set WP to `RO` (`0x5701`), re-triggered hardware reset via OpenOCD, verified boot status retained `Write-Protect: RO (BKP6: 0x5701)`, restored to `RW`.
  - Clean unmount with zero USB resets or bus dropouts during testing.

### 3. `prj_usb_msc` Non-Breaking Stream Cache Lookup Policy & Round-Robin Cache Population (Build 00E3)
- **Stream Preservation Architecture (`crates/gd32vf103-hal/src/usb/msc/`)**:
  - Added `total_blocks_requested: u32` tracking to `UsbMsc`, initialized from CDB byte counts in `READ_6` and `READ_10` (`to_transfer / 512`).
  - Implemented intelligent cache lookup gating:
    `let check_cache = !disk.is_streaming() || self.total_blocks_requested <= 8 || self.remaining_bytes <= 512;`
  - When streaming large multi-block reads (> 8 blocks, e.g. 64 blocks = 32 KiB), the hardware `CMD18` SPI streaming pipeline remains open and unbroken across sector boundaries, eliminating artificial pipeline teardown and re-issuance latency.
  - Cache lookups are executed unconditionally when starting a new transfer (`!disk.is_streaming()`), for small isolated reads ($\le 8$ blocks, such as MBR, BPB, FAT, and root directory sectors), and on the final sector of any transfer (`remaining_bytes <= 512`).
- **Round-Robin Cache Population (`cache.rs`)**:
  - Added `next_insert_slot: usize` and `insert_lba(&mut self, lba: u32, buf: &[u8; 512])` to `LbaCache`.
  - Automatically populates the 16-slot LBA cache on reads when `total_blocks_requested <= 8`, accelerating repeated filesystem lookups while preventing large streaming reads from flushing active directory metadata.
  - Bumped USB serial string descriptor and SCSI VPD page 0x80 serial to `LNMSC00000E3`.
- **Host Unit Test Suite (`cargo test`)**:
  - **84/84 host unit tests pass** across the workspace (32 in `gd32vf103-hal`, 7 in `longan-nano-bsp`, 34 in `prj_current_monitor`, 11 in `prj_usb_msc`).
  - Added `test_lba_cache_insert_lba_round_robin` in `cache.rs` verifying full 16-slot filling, lookups, and round-robin eviction.
  - Added `test_non_breaking_stream_cache_lookup_policy` in `msc/mod.rs` verifying all 4 policy decision cases.
- **Target Hardware Empirical Verification (`tools/verify_build_00e3.py`)**:
  - Flashed to GD32VF103 over JTAG OpenOCD on remote Linux host `192.168.0.63`.
  - Clean USB enumeration as `28e9:abd4` `Longan Nano SD Reader`, serial `LNMSC00000E3` (480 MiB / 983,040 blocks).
  - Verified all UART commands and PA8 simulation hooks: Help (`?`), Status (`s`), WP toggle (`w`), PA8 double-click (`K`), PA8 single-click (`k`).
  - Verified SCSI `TEST_UNIT_READY`, `MODE_SENSE_6`/`10` in RW and RO modes, and `WRITE_10` rejection in RO mode with Check Condition (`Data Protect: Write protected`).
  - Switched to RW mode: all 7 isolated `sg_raw` sector queries passed (LBA 0 1/8 blks, LBA 528 8 blks, LBA 560 8 blks, LBA 1024 1/8 blks, OOB LBA cleanly rejected).
  - Sustained 4.0 MiB read streaming (`dd bs=64k count=64`) achieved **846 kB/s** (up from 838 kB/s in Build 00E2) with uninterrupted continuous CMD18 streaming.
  - Partition 1 (`/dev/sdb1`) mounted read-write cleanly (`mount code=0`).
  - Single-file write & verification round-trip match verified on `/mnt/sdcard/build_00e3.txt`.
  - Multi-block 128 KiB CMD25 write streaming with double buffering verified with 100% cryptographic match (`MD5: 9ec54e9020276f923d7a70ac1e28d5a6`).
  - Tested BKP persistence across reboot: set WP to `RO` (`0x5701`), re-triggered hardware reset via OpenOCD, verified boot status retained `Write-Protect: RO (BKP6: 0x5701)`, restored to `RW`.
  - Clean unmount with zero kernel errors or USB bus resets.

### 4. Workspace-Wide Host Unit Test Harness & `panic_halt` Guarding (Task 5.4)
- **`panic_halt` & `#![no_std]` Guarding Across Auxiliary Crates**:
  - In `prj_blink`, `prj_uart_test`, `prj_lcd_test`, `prj_sdcard_test`, `prj_sdcard_fs_test`, `prj_usb_serial`, and `prj_usb_composite`:
    - Changed crate attributes to `#![cfg_attr(not(test), no_std)]` and `#![cfg_attr(not(test), no_main)]`.
    - Guarded `use panic_halt as _;` and `use riscv_rt::entry;` with `#[cfg(not(test))]`.
    - Guarded firmware `#[entry] fn main() -> !` with `#[cfg(not(test))]`.
- **Verification Results**:
  - `cargo test --workspace --target x86_64-pc-windows-msvc` passes 100% cleanly across all 13 workspace members (89 unit tests + 3 doc tests = 92 tests passing, 0 failures, 0 errors).
  - Target RV32 embedded compilation verified: both `prj_usb_msc` and `prj_current_monitor` build release binaries without errors.
