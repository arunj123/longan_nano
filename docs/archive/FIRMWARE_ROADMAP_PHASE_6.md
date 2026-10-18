# Firmware Historical Archive: Phase 6 (Tasks 6.1–6.5)

**Archive Reference**: Archived from [`docs/FIRMWARE_REVIEW_AND_ROADMAP.md`](../FIRMWARE_REVIEW_AND_ROADMAP.md) per Roadmap Archival & Size Management policy.  
**Target Architecture**: Sipeed Longan Nano (GD32VF103CBT6, RISC-V RV32IMAC @ 96/108 MHz, 32 KB SRAM, 128 KB Flash).

---

## Historical Overview of Completed Phase 6 (Initial 5 Tasks)

| Phase | Description | Key Deliverables & Verifications | Verified Builds |
|---|---|---|---|
| **Phase 6 (Initial)** | Multi-Model Independent Review Hardening | Unified `AppCommand` dispatcher (-1.8 KB text), logger buffer preservation on flush failure, clamped session numbering at 9999, Read-Only default MSC on Screen 6, INA219 tare load sanity check (±50 mA), 3-strike I2C debounce, 33% I2C read reduction, shunt saturation overflow detection, phase-locked 10 Hz / 1 Hz schedulers, watchdog coverage in SD loops, dynamic mtime frequency calculation, graph history direct streaming, 4-byte MSC cache alignment. | Build `00E4`, Build `00E5`, Build `00E6`, Build `00E7`, Build `00E8` |

---

## Detailed Task Breakdown & Implementation Records

### Task 6.1: Storage & Filesystem Data Integrity (P0) (Verified in Build 00E4)
- In `logger.rs`, changed `flush_buffer(&mut self) -> Result<(), ()>` and added `push_row_with_flush` / `rotate_session_with_flush` with host unit tests.
- In `app.rs`, verified `flush_buffer().is_ok() && !has_pending_data()` before mounting MSC in `enter_usb_drive_active()`; aborts with `" SD FLUSH ERR: NO MOUNT"` if flush fails or `buf_len > 0` (`[MON-STOR-01]`).
- In `logger.rs`, prevented `rotate_session()` from advancing `session_file_num` when `buf_len > 0`; prevented silent buffer clearing on flush failure and tracked `committed_rows` / `dropped_rows` (`[MON-STOR-02]`).
- Clamped `session_file_num` at `9999` (`num.min(9999)`) to prevent 4-digit wrap collision with `LOG_0000.CSV` (`[MON-UI-02]`).
- In `crates/gd32vf103-hal/src/usb/msc/` (`scsi.rs`, `mod.rs`) and `app.rs`, defaulted Screen 6 MSC to Read-Only (`write_protected = true`), bounded `handle_reset()` FIFO flush spin-loops (`100_000` iterations), and tracked host eject state via SCSI `START_STOP_UNIT` (`decode_start_stop_eject`) (`[MON-STOR-03]`).
- *Empirical Verification (Build 00E4)*: 70/70 host unit tests pass (`33` in `gd32vf103-hal`, `37` in `prj_current_monitor`); firmware `text = 124,112` bytes, `bss = 32,768` bytes; 12/12 hardware checks passed on `192.168.0.63` via `test_usb_drive_mode.py` (Read-Only mount `/dev/sdb1`, `sg_raw` LBA 0 `55aa`, clean HID restore `28e9:1234`, session rotation `LOG_0117.CSV`).

### Task 6.2: Sensor DSP, Tare Calibration & Telemetry Correctness (P1) (Verified in Build 00E5)
- In `ina219.rs` and `app.rs`, replaced the 8-read tare loop with a single uncompensated `REG_SHUNTVOLTAGE` read (`read_uncompensated_tenth_ma()`, hardware 128-sample average over 68.1 ms); added load sanity check (`validate_tare_offset`) rejecting tare if `|offset| > 50.0 mA` with `" TARE REJECTED: LOAD ON "` banner (`[MON-SENS-01]`).
- In `main.rs`, added 3-strike I2C error debounce (`i2c_err_count >= 3`) before setting `ina_present = false`; skipped `histogram.record()` and `stats.update()` when sensor is offline (`[MON-SENS-02]`).
- In `ina219.rs`, flagged `overflow` when `math_overflow || raw_c_tenth.unsigned_abs() >= 32000` and saturated `power_mw` at `65535` (`[MON-SENS-03]`).
- In `main.rs`, evaluated over-current alert using current magnitude `reading.abs_current_tenth()` (`[MON-SENS-04]`).
- In `ina219.rs`, removed redundant `REG_POWER` read from `read_all()`, cutting 10 Hz I2C traffic by 33% (`[MON-SENS-05]`).
- In `telemetry.rs`, incremented sequence counter `seq` unconditionally on every 10 Hz sample; set overflow bit (`1 << 2`) when `reading.power_tenth_mw > 655_350`; added strict HID OUT command packet length validation (`decode_hid_command`) (`[MON-SENS-06]`).
- *Empirical Verification (Build 00E5)*: 81/81 host unit tests pass (`33` in `gd32vf103-hal`, `8` in `longan-nano-bsp`, `40` in `prj_current_monitor`); firmware `text = 124,346` bytes, `bss = 32,768` bytes; 12/12 hardware checks passed on `192.168.0.63` (`v_mv: 3320, seq: 93`, `LOG_0120.CSV -> LOG_0122.CSV`).

### Task 6.3: AI Maintainability & Unified Command Dispatcher (P1) (Verified in Build 00E6)
- In `crates/longan-nano-bsp/src/bkp.rs`, defined authoritative `#[repr(usize)] pub enum BkpSlot` covering `BKP_DATA0..9` with unit tests for uniqueness and slot bounds (`[MON-MEM-01]`).
- In `crates/longan-nano-bsp/src/lcd.rs` and `prj_current_monitor/src/app.rs`, unified all `BKP_REG_*` constants to reference `BkpSlot`.
- In `prj_current_monitor/src/app.rs`, defined `pub enum AppCommand` covering 22 discrete system actions, added `toggle_usb_drive()`, and implemented centralized `App::execute_command(&mut self, board, cmd, now)` (`[MON-AI-01]`).
- In `prj_current_monitor/src/main.rs`, refactored Rotary Encoder polling, UART0 interactive CLI, bidirectional USB HID OUT command polling, and PA8 ButtonController events into `AppCommand` dispatching, shrinking `main.rs` by 216 lines and eliminating ~1,820 bytes of flash text duplication.
- *Empirical Verification (Build 00E6)*: 87/87 host tests pass (`33` in `gd32vf103-hal`, `9` in `longan-nano-bsp`, `42` in `prj_current_monitor`, `3` doctests); firmware `text = 122,526` bytes (reduced by 1,820 bytes), `bss = 32,768` bytes; 12/12 hardware checks passed on `192.168.0.63` via `test_usb_drive_mode.py` (UART screen switching, simulated double-click mount, MSC block read, RO mount, single-click dismount, session rotation `LOG_0126.CSV -> LOG_0128.CSV`, clean HID restoration `28e9:1234`).

### Task 6.4: Real-Time Scheduling, Latency & Watchdog Safety (P1) (Verified in Build 00E7)
- In `prj_current_monitor/src/main.rs`, locked 10 Hz and 1 Hz scheduler phases deterministically (`wrapping_add(100)` / `wrapping_add(1000)`) (`[MON-SCHED-01]`).
- In `prj_current_monitor/src/main.rs`, silenced 1 Hz UART heartbeat during MSC Active mode to eliminate host xHCI split-transaction NAK timeout stalls (`[MON-SCHED-02]`).
- In `crates/gd32vf103-hal/src/fwdgt.rs`, implemented static `Fwdgt::feed_raw()`; added watchdog feeds in `SdCard::init_internal()` (CMD0 & ACMD41 loops), `iterate_dir()` in `logger.rs`, and `flush_buffer()` in `logger.rs` (`[MON-SCHED-03]`).
- In `prj_current_monitor/src/main.rs`, relocated internal MCU die temperature ADC conversion from the 10 Hz loop to the 1 Hz periodic block (`[MON-SCHED-04]`).
- In `crates/longan-nano-bsp/src/sdcard.rs` and `lib.rs`, stored `ticks_per_ms` dynamically from `clocks.mtime_freq / 1000`, eliminating hardcoded 24,000 ticks/ms clock assumption (`[MON-SCHED-05]`).
- *Empirical Verification (Build 00E7)*: 87/87 host tests pass (`33` in `gd32vf103-hal`, `9` in `longan-nano-bsp`, `42` in `prj_current_monitor`, `3` doctests); firmware `text = 121,872` bytes (reduced by 654 bytes from Build 00E6's 122,526 bytes), `bss = 32,768` bytes; 12/12 hardware checks passed on `192.168.0.63` via `test_usb_drive_mode.py` (UART screen switching, simulated double-click mount, MSC block read, RO mount, single-click dismount, session rotation `LOG_0133.CSV -> LOG_0135.CSV`, clean HID restoration `28e9:1234`).

### Task 6.5: UI Performance, Memory Alignment & Documentation Integrity (P2) (Verified in Build 00E8)
- In `prj_current_monitor/src/ui/screen_graph.rs`, factored out `redraw_labels_and_axis(&mut self, lcd)`; in `replot_history`, stream historical columns directly via `write_pixels` without redundant full-screen clears and 105 separate `set_pixel` SPI transactions, eliminating autoscale screen flicker (`[MON-UI-01]`).
- In `crates/gd32vf103-hal/src/usb/msc/cache.rs`, wrapped sector buffers in `#[repr(C, align(4))] pub struct CacheBlock(pub [u8; 512])` with alignment unit tests ensuring 32-bit alignment for RISC-V USB FIFO word loads/stores (`[MSC-MEM-01]`).
- Cleaned up documentation across `GEMINI.md`, `AGENTS.md`, `README.md`, `prj_current_monitor/README.md`, and archive roadmaps: replaced absolute `file:///` URLs with repository-relative paths and removed plaintext SSH credentials (`[MON-AI-02]`).
- Added comprehensive host unit tests in `prj_current_monitor/src/app.rs` and `crates/gd32vf103-hal/src/usb/msc/cache.rs` covering all 22 `AppCommand` variants, forward and backward `ScreenMode` transitions, tare load sanity boundary conditions (`-500`, `+500`, `-501`, `+501`), NVRAM config packing, and `CacheBlock` alignment (`[MON-AI-03]`).
- *Empirical Verification (Build 00E8)*: 91 host tests pass (`34` in `gd32vf103-hal`, `9` in `longan-nano-bsp`, `46` in `prj_current_monitor`, `5` in `lcd_font`, `11` in `prj_usb_msc`, `23` in `embedded-sdmmc`, `3` doctests; 131 total); firmware `text = 122,040` bytes, `bss = 32,768` bytes; 12/12 hardware checks passed on `192.168.0.63` via `test_usb_drive_mode.py` (UART screen switching, simulated double-click mount, MSC block read, RO mount, single-click dismount, session rotation `LOG_0136.CSV -> LOG_0138.CSV`, clean HID restoration `28e9:1234`).
