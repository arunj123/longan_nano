#![no_std]
#![no_main]

use core::cell::RefCell;
use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use embedded_sdmmc::{
    Block, BlockCount, BlockDevice, BlockIdx, DirEntry, Mode, TimeSource, Timestamp, VolumeIdx,
    VolumeManager,
};
use longan_nano_bsp::lcd_color;
use longan_nano_bsp::{Board, SdCard, SdError, LCD_HEIGHT, LCD_WIDTH};
use panic_halt as _;
use riscv_rt::entry;

// Stack-allocated string buffer for formatting without heap
struct StrBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StrBuf<N> {
    fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const N: usize> Write for StrBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = N - self.len;
        let to_copy = bytes.len().min(remaining);
        self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
        Ok(())
    }
}

/// BlockDevice adapter connecting longan_nano_bsp::SdCard to embedded_sdmmc
struct SdBlockDevice<'a> {
    sd: RefCell<&'a mut SdCard>,
}

impl<'a> BlockDevice for SdBlockDevice<'a> {
    type Error = SdError;

    fn read(&self, blocks: &mut [Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut sd = self.sd.borrow_mut();
        for (i, block) in blocks.iter_mut().enumerate() {
            let lba = start_block_idx.0 + i as u32;
            sd.read_sector(lba, &mut block.contents)?;
        }
        Ok(())
    }

    fn write(&self, blocks: &[Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut sd = self.sd.borrow_mut();
        for (i, block) in blocks.iter().enumerate() {
            let lba = start_block_idx.0 + i as u32;
            sd.write_sector(lba, &block.contents)?;
        }
        Ok(())
    }

    fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        let sd = self.sd.borrow();
        Ok(BlockCount(sd.sector_count))
    }
}

/// Hardware-backed TimeSource for FAT file timestamps
struct HardwareTimeSource;

impl TimeSource for HardwareTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56, // 2026
            zero_indexed_month: 9,  // October
            zero_indexed_day: 3,    // 4th
            hours: 22,
            minutes: 50,
            seconds: 0,
        }
    }
}

fn run_filesystem_test(board: &mut Board, run_counter: u32) {
    let _ = writeln!(board.uart0, "\r\n=======================================================");
    let _ = writeln!(
        board.uart0,
        ">>> RUNNING SD FAT32 FILESYSTEM TEST (RUN #{}) <<<",
        run_counter
    );
    let _ = writeln!(board.uart0, "=======================================================");

    board.led_red.off();
    board.led_green.off();
    board.led_blue.on(); // Blue = Testing

    // Clear display area below header
    board.lcd.fill_rect(0, 16, LCD_WIDTH, LCD_HEIGHT - 16, lcd_color::BLACK);
    board.lcd.draw_string(4, 18, "Mounting FAT32...", lcd_color::YELLOW, lcd_color::BLACK);

    // 1. Initialize SD Card SPI protocol
    if let Err(err) = board.sdcard.init(&mut board.delay) {
        let _ = writeln!(board.uart0, "[ERROR] SD Card Probe Failed: {}", err.as_str());
        board.led_blue.off();
        board.led_red.on();
        board.lcd.draw_string(4, 18, "SD Init: FAILED! ", lcd_color::RED, lcd_color::BLACK);
        board.lcd.draw_string(4, 30, err.as_str(), lcd_color::RED, lcd_color::BLACK);
        return;
    }

    // 2. Initialize VolumeManager and mount Volume 0
    let block_dev = SdBlockDevice {
        sd: RefCell::new(&mut board.sdcard),
    };
    let vol_mgr = VolumeManager::new(block_dev, HardwareTimeSource);

    let volume = match vol_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(err) => {
            let _ = writeln!(board.uart0, "[ERROR] Failed to mount FAT volume: {:?}", err);
            board.led_blue.off();
            board.led_red.on();
            board.lcd.draw_string(4, 18, "Mount: FAILED!   ", lcd_color::RED, lcd_color::BLACK);
            return;
        }
    };

    let _ = writeln!(board.uart0, "[FS] FAT Volume mounted successfully!");
    board.lcd.draw_string(4, 18, "Volume: MOUNT OK", lcd_color::GREEN, lcd_color::BLACK);

    let root_dir = match volume.open_root_dir() {
        Ok(dir) => dir,
        Err(err) => {
            let _ = writeln!(board.uart0, "[ERROR] Failed to open root directory: {:?}", err);
            board.led_blue.off();
            board.led_red.on();
            board.lcd.draw_string(4, 28, "RootDir: FAILED! ", lcd_color::RED, lcd_color::BLACK);
            return;
        }
    };

    // Inspect existing entries in root directory
    let _ = writeln!(board.uart0, "\r\n--- Initial Root Directory Listing (/) ---");
    let mut dir_count = 0u32;
    let _ = root_dir.iterate_dir(|entry: &DirEntry| {
        dir_count += 1;
        let dir_flag = if entry.attributes.is_directory() { "  <DIR>" } else { "" };
        let _ = writeln!(
            board.uart0,
            "  [{:02}] {:<12}  {:6} bytes{}",
            dir_count, entry.name, entry.size, dir_flag
        );
    });
    let _ = writeln!(board.uart0, "--- Total Entries: {} ---\r\n", dir_count);

    // 3. Generate dynamic 8.3 filename based on ticks and run_counter
    let ticks = board.delay.get_raw_ticks() as u32;
    let mut fname_buf = StrBuf::<16>::new();
    let _ = write!(fname_buf, "R{:04X}{:02X}.TXT", ticks & 0xFFFF, run_counter & 0xFF);
    let filename = fname_buf.as_str();

    let _ = writeln!(board.uart0, "[FS] Target Filename: /{}", filename);

    // 4. Prepare test payload
    let mut payload_buf = StrBuf::<384>::new();
    let _ = write!(
        payload_buf,
        "=== Longan Nano FAT32 Test ===\r\n\
         File: /{}\r\n\
         Test Run: #{}\r\n\
         Timestamp Ticks: {}\r\n\
         Payload: ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\r\n\
         Verification: Pure Rust embedded-sdmmc OK\r\n\
         ==============================\r\n",
        filename, run_counter, ticks
    );

    let payload = payload_buf.as_bytes();
    let _ = writeln!(
        board.uart0,
        "[FS] Writing payload ({} bytes) to /{}...",
        payload.len(),
        filename
    );
    board.lcd.draw_string(4, 28, "Writing file...", lcd_color::YELLOW, lcd_color::BLACK);

    // 5. Create and write file
    match root_dir.open_file_in_dir(filename, Mode::ReadWriteCreateOrTruncate) {
        Ok(file) => {
            if let Err(err) = file.write(payload) {
                let _ = writeln!(board.uart0, "[ERROR] File write failed: {:?}", err);
                board.led_blue.off();
                board.led_red.on();
                board.lcd.draw_string(4, 28, "Write: FAILED!   ", lcd_color::RED, lcd_color::BLACK);
                let _ = file.close();
                return;
            }
            let _ = file.close();
            let _ = writeln!(
                board.uart0,
                "[FS] Successfully wrote {} bytes to /{}!",
                payload.len(),
                filename
            );
            board.lcd.draw_string(4, 28, "Write: OK", lcd_color::GREEN, lcd_color::BLACK);
        }
        Err(err) => {
            let _ = writeln!(board.uart0, "[ERROR] Failed to open file for write: {:?}", err);
            board.led_blue.off();
            board.led_red.on();
            board.lcd.draw_string(4, 28, "Create: FAILED!  ", lcd_color::RED, lcd_color::BLACK);
            return;
        }
    }

    // 6. Re-open file and read back
    let _ = writeln!(
        board.uart0,
        "[FS] Re-opening /{} for read-back verification...",
        filename
    );
    board.lcd.draw_string(4, 38, "Reading file...", lcd_color::YELLOW, lcd_color::BLACK);

    let mut read_buf = [0u8; 384];
    let bytes_read = match root_dir.open_file_in_dir(filename, Mode::ReadOnly) {
        Ok(file) => {
            let res = file.read(&mut read_buf);
            let _ = file.close();
            match res {
                Ok(n) => n,
                Err(err) => {
                    let _ = writeln!(board.uart0, "[ERROR] File read failed: {:?}", err);
                    board.led_blue.off();
                    board.led_red.on();
                    board.lcd.draw_string(4, 38, "Read: FAILED!    ", lcd_color::RED, lcd_color::BLACK);
                    return;
                }
            }
        }
        Err(err) => {
            let _ = writeln!(board.uart0, "[ERROR] Failed to open file for read: {:?}", err);
            board.led_blue.off();
            board.led_red.on();
            board.lcd.draw_string(4, 38, "Open: FAILED!    ", lcd_color::RED, lcd_color::BLACK);
            return;
        }
    };

    let _ = writeln!(
        board.uart0,
        "[FS] Successfully read {} bytes from /{}.",
        bytes_read, filename
    );
    board.lcd.draw_string(4, 38, "Read:  OK", lcd_color::GREEN, lcd_color::BLACK);

    // 7. Print read-back content to UART
    let _ = writeln!(board.uart0, "\r\n---------------- Read-Back File Content ----------------");
    if let Ok(content_str) = core::str::from_utf8(&read_buf[..bytes_read]) {
        let _ = write!(board.uart0, "{}", content_str);
    }
    let _ = writeln!(board.uart0, "--------------------------------------------------------\r\n");

    // 8. Verify byte-for-byte fidelity
    let match_ok = (bytes_read == payload.len()) && (&read_buf[..bytes_read] == payload);

    if match_ok {
        let _ = writeln!(
            board.uart0,
            ">>> [SUCCESS] 100% VERIFIED: Read content matches written payload byte-for-byte! <<<"
        );
        let _ = writeln!(board.uart0, "    File: /{} | Length: {} bytes\r\n", filename, bytes_read);

        board.lcd.draw_string(4, 48, "100% VERIFIED OK!", lcd_color::GREEN, lcd_color::BLACK);

        let mut name_disp = StrBuf::<32>::new();
        let _ = write!(name_disp, "File: /{}", filename);
        board.lcd.draw_string(4, 58, name_disp.as_str(), lcd_color::WHITE, lcd_color::BLACK);

        // 9. List files in root directory
        let _ = writeln!(board.uart0, "--- Root Directory Listing (/) ---");
        let _ = root_dir.iterate_dir(|entry: &DirEntry| {
            let dir_flag = if entry.attributes.is_directory() { "  <DIR>" } else { "" };
            let _ = writeln!(
                board.uart0,
                "  {:<12}  {:6} bytes{}",
                entry.name, entry.size, dir_flag
            );
        });
        let _ = writeln!(board.uart0, "----------------------------------\r\n");

        board.led_blue.off();
        board.led_red.off();
        board.led_green.on(); // Green = SUCCESS!
    } else {
        let _ = writeln!(board.uart0, "[FAIL] Content verification mismatch!");
        board.led_blue.off();
        board.led_red.on();
        board.lcd.draw_string(4, 48, "VERIFY MISMATCH! ", lcd_color::RED, lcd_color::BLACK);
    }

    let _ = root_dir.close();
    let _ = volume.close();
}

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    // Settle delay
    board.delay.delay_ms(100);

    // Title banner on LCD
    board.lcd.init(&mut board.delay);
    board.lcd.clear(lcd_color::BLACK);
    board.lcd.fill_rect(0, 0, LCD_WIDTH, 14, lcd_color::DARK_NAVY);
    board.lcd.draw_string(4, 3, "LONGAN NANO: FAT32", lcd_color::CYAN, lcd_color::DARK_NAVY);

    let _ = writeln!(board.uart0, "\r\n==================================================");
    let _ = writeln!(board.uart0, "Longan Nano -- Pure Rust FAT32 Filesystem Test");
    let _ = writeln!(board.uart0, "CPU: GD32VF103 RV32IMAC @ 108 MHz");
    let _ = writeln!(board.uart0, "Standard: Rust 2021 (#![no_std], -Os, lto)");
    let _ = writeln!(board.uart0, "Filesystem: embedded-sdmmc FAT16/FAT32 Engine");
    let _ = writeln!(board.uart0, "Interface: SPI1 (PB12 CS, PB13 SCK, PB14 MISO, PB15 MOSI)");
    let _ = writeln!(board.uart0, "User Button: PA8 (Press to re-run test at any time)");
    let _ = writeln!(board.uart0, "==================================================");

    let mut test_run_counter: u32 = 1;
    run_filesystem_test(&mut board, test_run_counter);

    let mut loop_counter: u32 = 0;
    let mut prev_button_pressed = false;

    loop {
        board.delay.delay_ms(100);
        loop_counter = loop_counter.wrapping_add(1);

        // Heartbeat on Blue LED and UART every 2 seconds
        if loop_counter % 20 == 0 {
            board.led_blue.toggle();
            board.delay.delay_ms(20);
            board.led_blue.toggle();

            let _ = writeln!(
                board.uart0,
                "[Heartbeat #{:04}] FAT32 Engine alive! Uptime: {} ms",
                loop_counter / 20,
                board.delay.uptime_ms()
            );
        }

        // Live seconds counter on bottom line of LCD
        if loop_counter % 10 == 0 {
            let mut time_buf = StrBuf::<32>::new();
            let _ = write!(time_buf, "Uptime: {:04} s", loop_counter / 10);
            board.lcd.draw_string(4, 68, time_buf.as_str(), lcd_color::GRAY, lcd_color::BLACK);
        }

        // Check PA8 button (Active Low)
        let button_pressed = board.button.is_pressed();
        if button_pressed && !prev_button_pressed {
            test_run_counter = test_run_counter.wrapping_add(1);
            let _ = writeln!(board.uart0, "\r\n[USER] PA8 Button Pressed! Re-running Filesystem Test...");
            run_filesystem_test(&mut board, test_run_counter);
        }
        prev_button_pressed = button_pressed;
    }
}
