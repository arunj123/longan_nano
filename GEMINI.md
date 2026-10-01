# Longan Nano Bare-Metal Project Guidelines

## CRITICAL MANDATORY INVARIANT: PER-BUILD GIT COMMITS
> [!CAUTION]
> **STRICT COMMIT-AFTER-EVERY-BUILD ENFORCEMENT**:
> 1. **Immediate Commit After Testing**: After testing ANY firmware build on the target hardware or Linux host, you MUST create a git commit immediately documenting the build number and test results before running further tests, before modifying code, and before ending the turn.
> 2. **Session Start Verification**: At the start of EVERY new session or task, run `git status`. If uncommitted changes exist from a previously tested build, commit them immediately before proceeding with any other actions.
> 3. **Strict Commit Message Structure**: Every build commit message must follow the standardized 5-part schema:
>    - **Header**: `build(<component>): Build <NNNN> - <Concise Headline>`
>    - **Summary of Changes**: Explicit, bulleted breakdown of code, register, timing, or configuration adjustments across modified files and the design rationale behind them.
>    - **Test Results**: Concrete empirical observations from hardware/host testing (e.g., pass/fail status of SCSI tags, sector counts, host `dmesg`, `usbmon` packet trace status, transfer throughput).
>    - **Root Cause & Diagnostics**: Detailed register states, hardware counter readings, or findings explaining any observed failures.
>    - **What Next**: Prioritized, actionable roadmap outlining the exact steps and hypotheses to investigate in the subsequent build iteration.
> 4. **No Batching Across Builds**: Never combine multiple build iterations into a single commit. Every single build iteration (e.g., Build 009B, Build 009C, etc.) requires its own distinct commit.


## Build & Toolchain
- **Build System**: Standalone Python build manager (`bldmgr/build.py`).
- **Compiler**: xPack RISC-V GCC 14.2 (`riscv-none-elf-gcc` / `riscv-none-elf-g++`).
- **Language Standards**: C17 (`-std=gnu17`), C++23 (`-std=gnu++23`).
- **Optimization Flags**: `-Os -flto -fuse-linker-plugin -ffunction-sections -fdata-sections`.
- **C++ Embedded Flags**: `-fno-exceptions -fno-rtti -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-threadsafe-statics`.
- **UTF-8 Output**: Always set `$env:PYTHONUTF8=1` in PowerShell before invoking the script:
  ```powershell
  $env:PYTHONUTF8=1; python bldmgr/build.py <project_name> [command]
  ```
- **Supported Commands**: `build`, `rebuild`, `clean`, `flash`, `program`, `nucleus`, `dfu`, `debug`.

## Flashing & In-Circuit Debugging
- **Programmer**: FT2232D Dual-Channel USB interface (`VID: 0x0403, PID: 0x6010`):
  - **Interface 0 (`MI_00`)**: JTAG debugger (`Dual RS232`, WinUSB driver via OpenOCD 0.12).
  - **Interface 1 (`MI_01`)**: Debug UART monitor (`USB Serial Converter B`, Virtual COM port `COM13` @ 115200 baud).
- **UART Monitor**: Use `tools/uart_monitor.py` for resilient, auto-reconnecting serial logging.
- **SRST-less OpenOCD Programming**:
  - The physical reset line (SRST) is not wired to the JTAG header.
  - Flashing commands must software-resume execution at the flash entry point:
    `program "<hex_path>" verify; halt; reg pc 0x08000000; resume; shutdown`

## Remote Linux Host Debugging & Flashing
- **Remote Host**: `192.168.0.63` (`user: arun`, `password: arun`).
- **Hardware Layout on Linux**:
  - FT2232D: Interface 0 (`ttyUSB0`) for JTAG OpenOCD; Interface 1 (`/dev/ttyUSB1`) for Debug UART @ 115200 baud.
  - GD32VF103 Device USB: `28e9:018a` (CDC-ACM `/dev/ttyACM0`).
- **Permissions / Udev**:
  - Rule at `/etc/udev/rules.d/99-ftdi.rules`: `SUBSYSTEM=="usb", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6010", MODE="0666", GROUP="plugdev", TAG+="uaccess"`.
  - User `arun` is in `dialout` and `plugdev`.
- **Remote Utilities**:
  - Remote Flash: `$env:PYTHONUTF8=1; python tools/remote_flash.py <hex_path>`
    - Uses SFTP to transfer binary to `/home/arun/longan_nano_tools/` and executes OpenOCD with SRST-less reset resume.
  - Remote UART Monitor: `python tools/remote_uart_monitor.py`
    - Streams live logs from `/dev/ttyUSB1` @ 115200 baud over SSH.
  - Remote xHCI Port Unsticking:
    - When Linux xHCI disables the root hub port after enumeration or timeout loops (`unable to enumerate USB device`), re-enable it via sysfs without host reboot:
      `echo 1 | sudo tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 1; echo 0 | sudo tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable`


## Hardware & Architecture Reference
- **MCU**: GD32VF103CBT6 (RISC-V 32-bit RV32IMAC @ up to 108MHz, 32KB SRAM, 128KB Flash).
- **FPU Policy**: **No hardware FPU**. Strictly avoid software floating-point emulation (`float`/`double`). Use fixed-point integer arithmetic (e.g. mV, mA, mW) for all sensor and control processing.
- **Memory Alignment**: RV32IMAC traps on unaligned multi-byte memory loads (`lw`). Sector and stream buffers must maintain 32-bit alignment, or use byte-level accessors (`ld_word`, `st_word`, `memcpy`) for unaligned memory structs.
- **Clock Configuration**:
  - **96 MHz** for applications using USB (`__SYSTEM_CLOCK_96M_PLL_HXTAL`, generates exact 48 MHz USB clock).
  - **108 MHz** for non-USB applications (`__SYSTEM_CLOCK_108M_PLL_HXTAL` for maximum CPU performance).
- **Timing & Timers**:
  - Hardware `mtime` timer runs at `SystemCoreClock / 4` (24 MHz at 96 MHz SYSCLK, 27 MHz at 108 MHz SYSCLK).
  - Always use typed `hal::time` utilities (`ms_to_ticks`, `us_to_ticks`, `Instant`, `Duration`) for delays and debouncing. Never compare raw ticks directly to millisecond values.
- **Onboard LEDs**:
  - Red: `PC13` (active-low)
  - Green: `PA1` (active-low via anode to 3.3V)
  - Blue: `PA2` (active-low via anode to 3.3V)
- **User Button**: `PA8` (active-low with internal pull-up). Always ensure `KeyButton::init()` is called to enable pull-up.
- **LCD**: 160x80 ST7735 SPI LCD on SPI0 (CS: PB2, DC: PB0, RST: PB1, SCK: PA5, MOSI: PA7).

## SD Card & SPI Interface Rules
- **Bus & Pinout**: Dedicated SPI1 peripheral (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI).
- **MISO Internal Pull-Up**: The MicroSD socket leaves MISO floating when unselected or uninitialized. PB14 (MISO) **must** be initialized with `InputPullUp` so idle reads yield `0xFF`.
- **Power-Up Sequence**: Send $\ge 80$ dummy clocks ($\ge 10$ bytes of `0xFF`) with CS held HIGH at $\le 400\text{ kHz}$ SPI clock before issuing `CMD0`.
- **Prescaler Switching**: Probe and initialize at $\le 400\text{ kHz}$ (SPI Mode 0: CPOL=0, CPHA=0, `BaudRatePrescaler::Div256`); switch dynamically to high speed (`BaudRatePrescaler::Div8`, ~6.75 MHz) only after operational initialization completes for sustained signal integrity over jumper wires.
- **Write Busy Polling**: Always poll card ready (`wait_ready()`) after writing sector data before releasing CS or initiating subsequent commands.

## FatFs Architecture on 32-Bit RISC-V
- **Version**: ChaN's FatFs R0.15 w/patch2 (`FFCONF_DEF 80286`) in `lib/fatfs/`.
- **SRAM Footprint (`FF_FS_TINY = 1`)**: Sector buffer is shared in `FATFS`, reducing `FIL` size to 36 bytes.
- **Native 32-Bit ALU (`FF_LBA64 = 0`)**: Sector addresses (`LBA_t`) use native 32-bit `DWORD`, avoiding 64-bit pair emulation overhead.
- **Flash Optimization**: Set `FF_USE_LFN = 0` (8.3 SFN) and `FF_CODE_PAGE = 437` to omit multi-megabyte unicode tables.
- **Forced FAT32 Small-Volume Detection**: When volumes are formatted with forced FAT32 (`mkfs.fat -F 32` or Windows), cluster count may fall below 65,526. Volume detection must check `ld_word(fs->win + BPB_FATSz16) == 0` to accurately classify FAT32.

## Interrupts & USB Architecture
- **ECLIC Controller**:
  - Base address `0xD2000000`. Each interrupt takes 4 bytes (IP, IE, ATTR, CTRL).
  - Always use direct array indexing `base[i]` for all 87 interrupts; avoid pointer increment multipliers.
  - Unhandled interrupts vector to `_unassigned_interrupts_handler()`, which prints `mcause` to UART0 and halts.
- **USB CDC-ACM Performance**:
  - Never use blocking delays (`delay_ms()`) in the main loop of USB-enabled applications.
  - `usb::poll()` must run continuously with zero latency. Use non-blocking `hal::time::Instant` and `Duration` for application-level task scheduling.
- **USB Enumeration & Descriptor Invariants**:
  - **DWC2 EP0 Control State Machine Sequencing**:
    - Never invoke `usb_ctlep_startout(udev)` inside `usbd_ctl_status_recev()` prior to receiving the host's 0-length Status OUT packet. Doing so corrupts the DWC2 OUT transfer register state, causing a 5-second `DATA_STAGE_TIMEOUT` bus freeze.
    - EP0 OUT must only be re-armed (`usb_ctlep_startout`) inside `usbd_out_transc()` under `case USB_CTL_STATUS_OUT:`, which cleanly transitions `ctl_state = USB_CTL_IDLE`. Similarly, `usbd_in_transc()` must handle `case USB_CTL_STATUS_IN:` to reset `ctl_state = USB_CTL_IDLE`.
  - **DWC2 IN Endpoint FIFO Loading & EP0 Invariants**:
    - In Synopsys DWC2 / GD32VF103 USBFS slave mode, Tx FIFO writes must strictly occur **after** the endpoint is enabled: `udev->regs.er_in[ep_num]->DIEPCTL = epctl | DEPCTL_CNAK | DEPCTL_EPEN;` must precede `usbd_emptytxfifo_write(udev, ep_num);`. Writing data while `EPEN=0` causes the DWC2 core to reject packet routing, failing enumeration.
    - Always mask write-only strobe bits prior to writing `DIEPCTL`: `epctl &= ~(DEPCTL_SD0PID | DEPCTL_SD1PID);`.
    - Control Endpoint 0 lacks a dedicated `DIEPTFSTAT` register (register `0x918` is reserved on EP0 and reads 0 words remaining). `usbd_emptytxfifo_write()` must account for EP0 using `GNPTXFSTS` rather than `DIEPTFSTAT`.
  - **usb_devcore_init Inactive Endpoint Zeroing Invariant**:
    - Inactive endpoints must be cleared to zero on core initialization: `udev->regs.er_in[i]->DIEPCTL = 0U;` and `udev->regs.er_out[i]->DOEPCTL = 0U;`. Retaining bits or OR-ing `DEPCTL_SNAK` on EP0 prevents SETUP packet reception, resulting in persistent 5-second `-110` host timeouts on `GET_DESCRIPTOR`.
  - **Windows xHCI 8-Byte Initial Probe Invariant**:
    - When Windows connects to a Full-Speed device on an xHCI root hub, it issues `GET_DESCRIPTOR(Device, wLength=64)` solely to read `bMaxPacketSize0` (the 8th byte) and immediately resets the bus.
    - The firmware must enforce `if (64U == req->wLength) { transc->remain_len = 8U; }` in `usbd_enum.cpp`. Returning all 18 bytes causes Windows xHCI to fail immediately with `USB\VID_0000&PID_0002` ("Device Descriptor Request Failed").
  - **HID Endpoint Packet Sizing Invariant**:
    - An endpoint's `wMaxPacketSize` must be strictly $\ge$ the size of the largest report transmitted on that endpoint (including Report ID).
    - For composite HID containing Keyboard reports (`[Report ID] + [Modifier] + [Reserved] + [6 Keycodes] = 9 bytes`), `wMaxPacketSize` must be at least 16 bytes (never 8 bytes), or host enumeration fails with descriptor validation errors.
  - **Descriptor Struct Packing & xHCI Port Error Caching**:
    - All configuration descriptor sets (such as `usb_composite_desc_config_set`) must be guarded with `#pragma pack(push, 1)` and `#pragma pack(pop)` to prevent GCC from inserting alignment padding between descriptor structs.
    - When debugging enumeration, note that Windows xHCI caches failed port states (`PID_0006` / `PID_0002`). Changing PID or executing a soft disconnect of $\ge 1000\text{ ms}$ (`usbd_disconnect()`) is required to force Windows to clear its port state.
  - **String Descriptor Bounds**: String queries must bounds-check `desc_index < USB_STRING_COUNT`. Windows unconditionally queries index `0xEE` (Microsoft OS descriptor); out-of-bounds indices must immediately return `REQ_NOTSUPP` (STALL) to prevent memory corruption and Code 10 failures.
  - **Strict Control Request Dispatching**: HID request handlers must explicitly support `DESC_TYPE_HID` (`0x21`) alongside `DESC_TYPE_REPORT` (`0x22`). Any unhandled or unsupported control request (`GET_REPORT`, `SET_REPORT`) must return `USBD_FAIL` (STALL). Handlers must never return `USBD_OK` without setting buffer pointers and lengths.
  - **Windows HID Sizing Rule**:
    - When HID reports are 64 bytes without explicit Report IDs, Windows requires a 65-byte packet (`[0x00 Report ID] + 64 data bytes`). Passing 64 bytes causes `0x000003E5` Overlapped I/O timeouts.
- **USB Mass Storage (MSC) Bulk-Only Transport (BOT) & SCSI Invariants**:
  - **SCSI READ_10 / WRITE_10 0-Length Ambiguity**:
    - In SCSI `READ_10` / `WRITE_10`, Windows hosts may send CDB Transfer Length bytes 7–8 = `0x0000` while specifying a non-zero byte count in the BOT CBW (`dCBWDataTransferLength`, e.g. 65,536 bytes).
    - Firmware must never treat bytes 7–8 being 0 as an invalid or 0-block transfer if `dCBWDataTransferLength > 0`; it must compute `blocks = dCBWDataTransferLength / block_size` (e.g. `65536 / 512 = 128`).
  - **BOT CSW Data Residue Compliance (Case 8 / Case 4)**:
    - On any stalled or aborted transfer where fewer bytes than requested were transferred, `dCSWDataResidue` in the CSW must report the exact un-transferred byte count (`dCBWDataTransferLength - transferred_bytes`).
    - Reporting residue as 0 when no data was transferred violates BOT Case 8, corrupting `USBSTOR.SYS` and causing Windows storage driver IRP deadlocks that freeze Windows Explorer.
  - **Essential SCSI Queries in Windows**:
    - **`0x9E` (`SERVICE_ACTION_IN_16` / `READ_CAPACITY_16`)**: Windows queries action `0x10` to check for 64-bit LBA. Returning the 32-byte capacity descriptor avoids fallback stalls.
    - **`0xA2` (`SECURITY_PROTOCOL_IN`)**: Windows queries security protocol `0x00` to probe for IEEE 1667 Encrypted Drives (BitLocker Silo). Returning an 8-byte payload with Supported Security Protocol List Length = 0 cleanly signals that no encryption protocols are supported, preventing BitLocker error-recovery loops.
    - **`0x35` (`SYNCHRONIZE_CACHE_10`)**: Sent by Windows during volume mount/flush; must return `CMD_PASSED`.
  - **DWC2 Empty-FIFO ZLP Race Elimination**:
    - In Synopsys DWC2 USBFS slave mode, if `EPEN=1` is asserted while `NAKS=0` and the TX FIFO is empty, incoming host IN tokens trigger an empty-FIFO Zero-Length Packet (ZLP), prematurely decrementing the hardware packet counter `PCNT` (`DIEPLEN`).
    - Writing `DEPCTL_EPEN | DEPCTL_SNAK` in a single register write does NOT prevent this race because `SNAK` is a write-only strobe requiring 1-2 USB clock cycles to latch `NAKS=1`.
    - Firmware must enforce a two-phase arming protocol:
      1. Write `DEPCTL_SNAK` (with strobe bits masked).
      2. Poll `while (!(DIEPCTL & DEPCTL_NAKS) && --timeout) {}` to guarantee hardware confirmation of NAK status.
      3. Program `DIEPLEN`.
      4. Only then assert `DEPCTL_EPEN`.
  - **Immediate Inter-Phase NAK Pre-Arming (Elimination of xHCI EPROTO -71)**:
    - Host controllers (e.g. xHCI) submit the Bulk-IN data phase URB immediately upon receiving the CBW ACK (often within $\le 35\ \mu\text{s}$). If endpoint arming is deferred to the main application loop (`poll()`), the IN endpoint remains disabled (`EPEN=0`).
    - An in-flight IN token arriving while the endpoint is disabled triggers an xHCI transaction error (`COMP_USB_TRANSACTION_ERROR` / `stat=-71 EPROTO`), causing the host to issue a bus reset.
    - Pre-arming via `usbd_ep_nak_arm()` must be invoked immediately inside `process_scsi()` for `READ_6` and `READ_10` before exiting the CBW receive ISR.
    - `usbd_ep_nak_arm()` must be idempotent: if `(DIEPCTL & (DEPCTL_EPEN | DEPCTL_NAKS)) == (DEPCTL_EPEN | DEPCTL_NAKS)`, it must return immediately to avoid corrupting active transfers when re-invoked in `poll()`.
  - **BOT Inter-Command NAK Guarding & Toggle Continuity**:
    - On CSW completion (`case BbbState::SEND_CSW:`), the IN endpoint must be returned to NAK state (`DIEPCTL |= DEPCTL_SNAK`) with write-sensitive strobe bits masked.
    - Data toggle bits must never be reset between normal commands; reset only on BOT reset or CLEAR_FEATURE.
  - **SD Card AU Boundary Latency (~3.5 ms at LBA 1024) & Double-Buffering Invariant**:
    - At flash Allocation Unit (AU) erase block boundaries (such as LBA 1024), the SD card internal controller delays the data start token `0xFE` by ~3.5 ms.
    - In single-buffered transfers, this dead time causes the USB IN endpoint to NAK for longer than the host xHCI TT split-transaction retry threshold, causing host-side transaction timeouts (`DID_ERROR`, `EPROTO -71`).
    - Sustained multi-sector streaming requires a 2-slot ping-pong double buffer (2 $\times$ 512B) so SD SPI reading for sector $N+1$ overlaps with USB packet transmission of sector $N$.
- **Display Buffer RAM Policy (Ping-Pong Double Buffering)**:
  - Never allocate full-screen framebuffers (25.6 KB) or large static quad-buffers (16 KB) on GD32VF103 (32 KB total SRAM).
  - Always use a 2-slot ping-pong buffer (e.g. 2 $\times$ 6.4 KB or 2 $\times$ 3.2 KB) synchronized with DMA0 Channel 2 completion interrupts (`dma_interrupt_enable(DMA0, DMA_CH2, DMA_INT_FTF)`).

## Project Structure
- Active applications are prefixed with `prj_*` in the project root (`prj_usb_msc`, `prj_usb_composite`, `prj_current_monitor`, `prj_usb_serial`, `prj_uart_test`, `prj_lcd_test`, `prj_sdcard_test`, `prj_sdcard_fs_test`).
- Modern C++23 zero-cost drivers and register abstractions reside in `hal/`, `bsp/`, and `drivers/` (100% vendor-free bare-metal implementation).
- Modern, low-footprint components reside in `lib/` (`lib/system`, `lib/fatfs`, `lib/gd32v_lcd`, `lib/debug_uart0`).
- Hardware reference datasheets and baseline version documentation reside in `docs/` (`docs/REFERENCE_BASELINE.md`, `docs/GD32VF103_Datasheet_Rev2.1.pdf`).

## Resource & Peripheral Conflict Checking
- **Always Check Hardware Allocations**: Before modifying or creating drivers, verify that peripherals, DMA channels, timers, GPIO pins, and interrupt vectors do not collide:
  - **DMA0 Channels**:
    - CH0: ADC0
    - CH1: SPI0_RX (LCD Read)
    - CH2: SPI0_TX (LCD Blit/Draw)
    - CH3: SPI1_RX / USART0_TX
    - CH4: SPI1_TX / USART0_RX
    - CH5: I2C0_TX
    - CH6: I2C0_RX
  - **Timers**: Do not use hardware TIMER2 for simple delays (use core 64-bit `mtime` via `hal::time`).
  - **GPIO Pins**: Check all SPI, I2C, UART, Button, and LED mappings before assigning or reconfiguring pins.

## Build Iteration & Debug Commit Protocol
- **Mandatory Per-Build Commits**: For EVERY build and debug iteration (e.g., Build 0084, Build 0085, etc.), a dedicated git commit MUST be created immediately upon completing testing before proceeding to the next iteration or ending the session.
- **Strict Commit Message Structure**: Every build commit message must follow this standardized schema:
  - **Header**: `build(<component>): Build <NNNN> - <Concise Headline>` (e.g., `build(msc): Build 0085 - Fix Tag 0x39 data toggle tracking`)
  - **Summary of Changes**: Explicit, bulleted breakdown of code, register, timing, or configuration adjustments across modified files and the design rationale behind them.
  - **Test Results**: Concrete empirical observations from hardware/host testing (e.g., pass/fail status of SCSI tags, sector counts, host `dmesg`, `usbmon` packet trace status, transfer throughput).
  - **Root Cause & Diagnostics**: Detailed register states, hardware counter readings, or findings explaining any observed failures.
  - **What Next**: Prioritized, actionable roadmap outlining the exact steps and hypotheses to investigate in the subsequent build iteration.
- **Debug Artifacts & Tools**: Commit all associated test scripts, diagnostic parsers, or trace analysis tools (e.g., in `tools/` or `scratch/`) alongside the firmware changes so the exact verification environment is preserved in git history.

