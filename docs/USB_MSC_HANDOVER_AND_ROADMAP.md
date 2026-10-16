# USB Mass Storage (MSC) Handover & Implementation Roadmap

## 1. Executive Context & Decision

- **Project**: `prj_usb_msc` (Sipeed Longan Nano / GD32VF103CBT6 RISC-V RV32IMAC @ 96 MHz, 32 KB SRAM, 128 KB Flash).
- **Goal**: Make USB Mass Storage completely stable, high-throughput, and compliant with Linux/Windows hosts, supporting multi-sector reads, filesystem mounting (`FAT32`), and sustained data streaming without xHCI `-71 EPROTO` transaction errors or USB bus resets.
- **Language Decision**: **Pure Rust** (`prj_usb_msc/src/main.rs`, `crates/gd32vf103-hal/src/usb.rs`, `crates/longan-nano-bsp/src/sdcard.rs`).
  - **Single-Context Polling**: Eliminates the severe interrupt-to-main-loop race conditions present in the C++ version (`USBFS_IRQHandler` vs `poll()`).
  - **SRAM Efficiency**: Rust firmware uses only `3.2 KB` SRAM (leaving **>28 KB free**), allowing an 8 KB hot-LBA cache and pipelined double-buffering.
  - **Repository Consistency**: Aligns with the modern repository standard established in Builds `00B2`–`00D0`.

---

## 2. Hardware Capabilities & Timing Budget

1. **GD32VF103 Clocks**:
   - `SYSCLK = 96 MHz` (`HXTAL` 8 MHz crystal via PLL $\times 24$).
   - `APB1 = 48 MHz` (max 54 MHz).
   - `USBFS = 48 MHz` (`RCU_CFG0.USBFSPSC = 0b11` Div2).
   - `mtime = 24 MHz` ($1\ \mu\text{s} = 24\text{ ticks}$).
   - `SPI1 (SD Card)` on APB1: `Prescaler::Div2` gives **24 MHz SPI clock**.
2. **DWC2 USBFS FIFO Allocation (1.25 KB dedicated USB SRAM at `0x50002000`)**:
   - `RX FIFO`: 96 words (384 bytes).
   - `TX0 FIFO (EP0)`: 32 words (128 bytes).
   - `TX1 FIFO (EP1 IN)`: **192 words (768 bytes)** (`DIEPTFLEN[0] = (192 << 16) | 128`).
   - *Key Hardware Insight*: Calling `write_fifo(1, &buf[..512])` copies 512 bytes into `TX1 FIFO` in **~12 µs**. Once loaded and `CNAK` is asserted, DWC2 transmits all 8 $\times$ 64-byte packets over the 12 Mbps wire over **~426 µs completely in hardware**. During those 426 µs, the RAM buffer is **100% free** for reading the next sector from SPI1!
3. **SD Card Characteristics (480 MB SDSC, `OCR = 0x80FF8000`)**:
   - Probed as `CardType::SD2SC` (`CCS = 0`, byte addressing: `arg = sector * 512`).
   - Regular sectors: `CMD17` token wait is ~180 µs; total read time is ~400–510 µs at 24 MHz SPI.
   - **LBA 1024 AU Boundary**: Jumping cold from LBA 560 to LBA 1024 (`0x00080000`, 512 KB boundary) causes the SDSC card's internal flash controller to stall the `0xFE` start token by **~2.40 ms**.
4. **Host xHCI + High-Speed Hub TT Split-Transaction Limit**:
   - When connected via a High-Speed USB 2.0 Hub Transaction Translator (TT), continuous NAK responses on Full-Speed Bulk-IN exceeding **~2.0–2.4 ms** cause the TT / xHCI driver to abort with `COMP_USB_TRANSACTION_ERROR` (`DID_ERROR`, `stat = -71 EPROTO`) and trigger a bus reset.

---

## 3. The 6 Root Causes of Previous Failures

1. **Stop-and-Go Execution Wastes the 768-Byte Hardware FIFO**:
   - Both C++ and Rust drivers wait idly for `DIEPINTF.TF` (426 µs later) before setting `need_read = true` to read the next sector from SD. The 426 µs USB wire serialization time was completely wasted instead of overlapping with the next SPI read.
2. **SD Single-Block (`CMD17`) Teardown & LBA 1024 AU Boundary Stall**:
   - Reading 1 sector at a time with `CMD17` forces the SDSC card to close its internal NAND page after every 512 bytes. A cold jump to LBA 1024 stalls for 2.40 ms, exceeding the TT split-transaction retry budget.
   - In `sdcard.rs`, `wait_token_fast(60_000)` at 24 MHz SPI only waits ~27 ms instead of the 100 ms SD $N_{AC}$ timeout.
3. **Premature `DEPCTL_EPEN = 1` on Empty FIFO & `DIEPLEN` Overwrite**:
   - `ep_nak_arm()` sets `EPEN = 1` while `TX1 FIFO` is empty. In DWC2 hardware, setting `EPEN = 1` on an empty FIFO risks underrun/ZLP generation. Furthermore, modifying `DIEPLEN` while `EPEN == 1` violates the DWC2 specification and corrupts `PCNT`.
   - Hardware NAK during SD read only requires `EPACT = 1` and `depctl::SNAK` (polling `NAKS == 1`) with **`EPEN = 0`**.
4. **Artificial 2.5 ms Delay (`kCswFramingGuard`) in `StatusPending`**:
   - Waiting 2.5 ms (60,000 ticks) after `DIEPINTF.TF` before sending the CSW forces `EP1 IN` to NAK the CSW for 2,500 µs on every single command, pushing every CSW right onto the ~2.4 ms xHCI TT NAK timeout cliff.
5. **Main Loop Blocks for ~10 ms on `SET_CONFIGURATION`**:
   - `main.rs` called `lcd.draw_string()` and `writeln!(board.uart0)` immediately when `is_configured()` became true, starving `poll()` for ~10 ms right when the host sends `GET_MAX_LUN` (`a1 fe`) and the initial SCSI CBW.
6. **Rust `usb.rs` Port Omissions**:
   - 100 ms disconnect instead of 200 ms GPIO SE0 + 800 ms `SDIS`.
   - Missing `BbbState::SendZlp` on short transfers that are exact multiples of 64 bytes.
   - Missing `depctl::EPD` polling loop in `REQ_BBB_RESET`.
   - Missing watchdog timeout in `StatusPending`.

---

## 4. Step-by-Step Implementation Blueprint

### Step 1: Upgrade `crates/longan-nano-bsp/src/sdcard.rs`
1. **Calibrate fast-polling iteration counts for 24 MHz SPI**:
   - Update `wait_ready_fast` multiplier to `timeout_ms * 2500`.
   - Update `wait_token_fast` in `read_sector()` to `250_000` (~110 ms, meeting SD $N_{AC}$ specification).
2. **Implement Continuous Multi-Block Read Streaming (`CMD18` / `CMD12`)**:
   - Add state: `streaming_read: bool`, `stream_next_lba: u32`.
   - `begin_multi_read(start_sector: u32) -> Result<(), SdError>`: Asserts CS low, syncs 8 clocks, waits ready, issues `CMD18`, sets `streaming_read = true`, `stream_next_lba = start_sector`.
   - `read_next_stream_sector(buf: &mut [u8; 512]) -> Result<(), SdError>`: While CS remains low, calls `wait_token_fast(250_000)` for `0xFE`, reads 512 bytes + 2 CRC bytes, increments `stream_next_lba`. (Leaves CS LOW; SD card prefetches next sector in hardware).
   - `end_multi_read()`: Sends `CMD12` (`STOP_TRANSMISSION`), waits ready, deasserts CS (`release_bus()`), clears `streaming_read = false`.
   - In `read_sector(sector, buf)`: if `streaming_read && sector == self.stream_next_lba`, fetch directly via `read_next_stream_sector(buf)`.

### Step 2: Implement 8 KB Hot-LBA Cache in `crates/gd32vf103-hal/src/usb.rs`
1. Allocate an 8 KB (16-sector) cache in `UsbMsc` (or a dedicated struct):
   - Pre-warm at boot in `main.rs` (before USB connects):
     - **Slot 0..7 (4 KB)**: LBAs `0..7` (MBR and partition table).
     - **Slot 8..15 (4 KB)**: LBAs `1024..1031` (Partition 1 AU boundary probed by Linux `udev`/`blkid`).
2. On `read_sector(lba)`:
   - If `lba` hits in cache, `memcpy` in **~3 µs** (0 µs SPI latency, 0 µs AU stall).
   - If cache miss, read from SD.
3. On `write_sector(lba, data)`:
   - Write-through to SD and update/invalidate cache entry.

### Step 3: Refactor DWC2 Bulk-IN State Machine in `crates/gd32vf103-hal/src/usb.rs`
1. **Fix NAK Pre-Arming**:
   - In `set_ep_snak(1)`: Assert `depctl::SNAK` and poll `(diepctl & depctl::NAKS) != 0`. **Leave `EPEN = 0` during SD read!**
2. **Atomic FIFO Loading & Transmission**:
   - In `ep_send_media(len)`:
     - Verify `(diepctl & depctl::NAKS) != 0` and `(diepctl & depctl::EPEN) == 0`.
     - Clear `diepintf = TF | EPDIS | TXFUD`.
     - Set `dieplen = (pkt_cnt << 19) | len`.
     - Set `diepctl |= depctl::EPEN`.
     - Write all 128 words (`512B`) into `DFIFO[1]` via `write_fifo(1, ...)` (~12 µs).
     - Set `diepctl |= depctl::EPEN | depctl::CNAK` to release NAK and serialize onto wire.
3. **Pipelined Read-Ahead (Overlap SPI Read with USB Transmission)**:
   - The moment `write_fifo` and `CNAK` finish for Sector $k$, `self.media_buf` in SRAM is free!
   - Immediately read Sector $k+1$ into `self.media_buf` (or ping-pong buffer) while DWC2 transmits Sector $k$ on the USB wire (~426 µs).
   - When `DIEPINTF.TF` fires for Sector $k$, Sector $k+1$ is already in SRAM and is pushed into `DFIFO[1]` in **15 µs**!
4. **Remove `kCswFramingGuard` (2.5 ms Delay)**:
   - In `BbbState::StatusPending`, once `hw_done && fifo_empty && ep_naking && no_pending` is true, immediately send the 13-byte CSW (or max 50 µs guard).
5. **Fix Remaining Rust Bugs**:
   - In `UsbMsc::new()`: Drive PA11/PA12 push-pull LOW for 200 ms, then assert `dctl::SDIS` for 800 ms (total $\ge 1000\text{ ms}$ disconnect).
   - Add `BbbState::SendZlp` in `send_data()` for exact multiples of 64 bytes with residue.
   - In `handle_setup()` for `REQ_BBB_RESET`: write `depctl::EPD | depctl::SNAK` and poll `EPEN == 0`.
   - Add a 2,000 ms watchdog in `StatusPending` to recover if a host transfer is aborted.

### Step 4: Refactor UI / Telemetry in `prj_usb_msc/src/main.rs`
1. Make LCD drawing and UART logging strictly non-blocking.
2. Defer `board.lcd.draw_string("USB: CONFIGURED OK")` and `writeln!(board.uart0)` until the USB bus is idle for $\ge 250\text{ ms}$.

---

## 5. Build, Flash, and Verification Protocol

```powershell
# 1. Clean build
$env:PYTHONUTF8=1; python bldmgr/build.py prj_usb_msc rebuild

# 2. Remote flash to target hardware
$env:PYTHONUTF8=1; python tools/remote_flash.py build/prj_usb_msc/rust/firmware_rust.hex

# 3. Monitor live UART telemetry
python tools/remote_uart_monitor.py
```

### Remote Linux Host Verification Commands (`192.168.0.63`)
```bash
# Check clean enumeration without DID_ERROR
dmesg | tail -n 30

# Verify direct SCSI read across LBA 0, 528, 560, and 1024
python3 /home/arun/longan_nano_tools/test_sg_read.py

# Multi-sector sweep and throughput test
dd if=/dev/sdb of=/dev/null bs=64k count=64 iflag=direct

# Mount filesystem and verify read/write
sudo mount /dev/sdb1 /mnt
ls -la /mnt
sudo touch /mnt/test.txt && sudo sync
sudo umount /mnt
```

---

## 6. Mandatory Commit Protocol for Build 00D1

Immediately upon completing testing on hardware, commit using the standardized 5-part schema:
- **Header**: `build(msc): Build 00D1 - Pure Rust USB MSC with CMD18 streaming and hot LBA cache`
- **Summary of Changes**: Explicit bulleted breakdown across `sdcard.rs`, `usb.rs`, and `main.rs`.
- **Test Results**: Empirical pass/fail results for LBA 1024, `dd` transfer throughput, and `mount` status.
- **Root Cause & Diagnostics**: Confirmation of register states, FIFO timing, and latency measurements.
- **What Next**: Subsequent tuning (e.g. write optimization or DMA-accelerated SPI1).
