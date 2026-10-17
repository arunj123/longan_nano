use core::cell::RefCell;
use core::fmt::Write;
use embedded_sdmmc::{
    Block, BlockCount, BlockDevice, BlockIdx, Mode, TimeSource, Timestamp,
    VolumeIdx, VolumeManager,
};
use longan_nano_bsp::{DateTime, SdCard, SdError};
use crate::fmt::BufferCursor;
use crate::model::{Accumulators, InaReading};
use crate::ui::SdStatus;

/// Embedded-SDMMC BlockDevice adapter wrapping BSP SdCard.
pub struct SdCardDevice<'a>(pub &'a RefCell<SdCard>);

impl<'a> Clone for SdCardDevice<'a> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a> Copy for SdCardDevice<'a> {}

impl<'a> BlockDevice for SdCardDevice<'a> {
    type Error = SdError;

    fn read(&self, blocks: &mut [Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut card = self.0.borrow_mut();
        for (i, block) in blocks.iter_mut().enumerate() {
            let lba = start_block_idx.0 + (i as u32);
            card.read_sector(lba, &mut block.contents)?;
        }
        Ok(())
    }

    fn write(&self, blocks: &[Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut card = self.0.borrow_mut();
        for (i, block) in blocks.iter().enumerate() {
            let lba = start_block_idx.0 + (i as u32);
            card.write_sector(lba, &block.contents)?;
        }
        Ok(())
    }

    fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        let card = self.0.borrow();
        Ok(BlockCount(card.sector_count))
    }
}

pub struct DynamicTimeSource(pub Option<DateTime>);
impl TimeSource for DynamicTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        if let Some(dt) = self.0 {
            Timestamp::from_calendar(dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second).unwrap_or(Timestamp {
                year_since_1970: 56,
                zero_indexed_month: 9,
                zero_indexed_day: 9,
                hours: 12,
                minutes: 0,
                seconds: 0,
            })
        } else {
            Timestamp::from_calendar(2026, 10, 10, 12, 0, 0).unwrap_or(Timestamp {
                year_since_1970: 56,
                zero_indexed_month: 9,
                zero_indexed_day: 9,
                hours: 12,
                minutes: 0,
                seconds: 0,
            })
        }
    }
}

/// High-performance buffered datalogger for MicroSD cards.
/// Collects measurements in an internal 512-byte sector buffer to avoid
/// costly per-second filesystem overhead and flash wear.
pub struct SdDatalogger<'a> {
    sdcard: &'a RefCell<SdCard>,
    buffer: [u8; 512],
    buf_len: usize,
    pub status: SdStatus,
    pub total_logged_rows: u32,
    pub committed_rows: u32,
    pub pending_rows: u32,
    pub dropped_rows: u32,
    pub session_file_num: u16,
    pub file_name: [u8; 13],
    file_name_len: usize,
    last_flush_ms: u32,
    last_probe_ms: u32,
    probe_fail_count: u8,
    pub current_dt: Option<DateTime>,
}

impl<'a> SdDatalogger<'a> {
    pub fn new(sdcard: &'a RefCell<SdCard>, now_ms: u32) -> Self {
        Self {
            sdcard,
            buffer: [0; 512],
            buf_len: 0,
            status: SdStatus::NoCard,
            total_logged_rows: 0,
            committed_rows: 0,
            pending_rows: 0,
            dropped_rows: 0,
            session_file_num: 1,
            file_name: *b"LOG_0001.CSV\0",
            file_name_len: 12,
            last_flush_ms: now_ms,
            last_probe_ms: now_ms,
            probe_fail_count: 0,
            current_dt: None,
        }
    }

    /// Returns the number of unflushed bytes currently held in the RAM sector buffer.
    #[inline]
    pub fn buffered_bytes(&self) -> usize {
        self.buf_len
    }

    /// Returns `true` if there are unflushed CSV bytes waiting in the RAM buffer.
    #[inline]
    pub fn has_pending_data(&self) -> bool {
        self.buf_len > 0
    }

    /// Sets active sequential file name based on session number (e.g. `LOG_0001.CSV`).
    /// Clamps at 9999 to prevent 4-digit wrap-around collision with `LOG_0000.CSV`.
    pub fn set_session_num(&mut self, num: u16) {
        let clamped = num.min(9999);
        self.session_file_num = clamped;
        let d0 = (clamped / 1000 % 10) as u8 + b'0';
        let d1 = (clamped / 100 % 10) as u8 + b'0';
        let d2 = (clamped / 10 % 10) as u8 + b'0';
        let d3 = (clamped % 10) as u8 + b'0';
        self.file_name = [
            b'L', b'O', b'G', b'_',
            d0, d1, d2, d3,
            b'.', b'C', b'S', b'V',
            0,
        ];
        self.file_name_len = 12;
    }

    /// Returns the active session filename as a string slice.
    pub fn file_name_str(&self) -> &str {
        core::str::from_utf8(&self.file_name[..self.file_name_len]).unwrap_or("MONITOR.CSV")
    }

    /// Scans the root directory on the SD card to discover existing `LOG_XXXX.CSV`
    /// files and selects the next sequential file number (clamped at 9999).
    pub fn scan_and_set_next_session(&mut self) {
        let dev = SdCardDevice(self.sdcard);
        let vol_mgr = VolumeManager::new(dev, DynamicTimeSource(self.current_dt));
        let mut max_num = 0u16;

        let _ = (|| -> Result<(), ()> {
            let vol = vol_mgr.open_volume(VolumeIdx(0)).map_err(|_| ())?;
            let root = vol.open_root_dir().map_err(|_| ())?;
            let _ = root.iterate_dir(|entry| {
                let name = &entry.name;
                if name.extension() == b"CSV" {
                    let base = name.base_name();
                    if base.len() == 8 && &base[0..4] == b"LOG_" {
                        let mut num = 0u16;
                        let mut valid = true;
                        for &b in &base[4..8] {
                            if b.is_ascii_digit() {
                                num = num * 10 + (b - b'0') as u16;
                            } else {
                                valid = false;
                                break;
                            }
                        }
                        if valid && num > max_num {
                            max_num = num;
                        }
                    }
                }
            });
            Ok(())
        })();

        let next_num = max_num.saturating_add(1).min(9999);
        self.set_session_num(next_num);
    }

    /// Testable helper that flushes using `flush_fn` and rotates to the next sequential
    /// log file ONLY if the old buffer is empty after flushing.
    /// Prevents old-session buffered rows from being written into a new session file.
    pub fn rotate_session_with_flush<F>(&mut self, mut flush_fn: F) -> bool
    where
        F: FnMut(&mut Self) -> Result<(), ()>,
    {
        if flush_fn(self).is_err() || self.buf_len > 0 {
            return false;
        }
        let next_num = self.session_file_num.saturating_add(1).min(9999);
        self.set_session_num(next_num);
        self.total_logged_rows = 0;
        self.committed_rows = 0;
        self.pending_rows = 0;
        true
    }

    /// Flushes current sector buffer and rotates to the next sequential log file.
    /// Returns `true` if rotation succeeded, or `false` if pending data could not be flushed.
    pub fn rotate_session(&mut self) -> bool {
        self.rotate_session_with_flush(|s| s.flush_buffer())
    }

    /// Performs hot-plug probing with exponential backoff to avoid freezing
    /// USB and UI tasks when no card is present.
    pub fn check_probe(&mut self, now_ms: u32, delay: &mut longan_nano_bsp::hal::Delay) -> bool {
        if self.status == SdStatus::Ready || self.status == SdStatus::Logging {
            return true;
        }

        // Exponential backoff: 2.5s, 5s, 10s, 20s, up to 30s max
        let shift = core::cmp::min(self.probe_fail_count, 4);
        let backoff_ms = core::cmp::min(30_000, 2500u32 * (1 << shift));

        if now_ms.wrapping_sub(self.last_probe_ms) < backoff_ms {
            return false;
        }
        self.last_probe_ms = now_ms;

        let res = self.sdcard.borrow_mut().init(delay);
        if res.is_ok() {
            self.status = SdStatus::Ready;
            self.probe_fail_count = 0;
            // Only scan for a new session filename if we don't hold pending unflushed rows
            // bound to the previous session filename.
            if self.buf_len == 0 {
                self.scan_and_set_next_session();
            }
            true
        } else {
            self.status = SdStatus::NoCard;
            self.probe_fail_count = self.probe_fail_count.saturating_add(1);
            false
        }
    }

    /// Testable helper that appends `row_bytes` into the 512-byte buffer, invoking
    /// `flush_fn` first if needed. If `flush_fn` fails and the row still does not fit,
    /// preserves existing buffered bytes and increments `dropped_rows`.
    pub fn push_row_with_flush<F>(&mut self, row_bytes: &[u8], mut flush_fn: F) -> bool
    where
        F: FnMut(&mut Self) -> Result<(), ()>,
    {
        let row_len = row_bytes.len();
        if row_len == 0 || row_len > self.buffer.len() {
            return false;
        }

        if self.buf_len + row_len > self.buffer.len() {
            let _ = flush_fn(self);
            if self.buf_len + row_len > self.buffer.len() {
                // Flush failed and buffer is still full: preserve existing buffered bytes
                // and drop the new sample explicitly rather than wiping the buffer.
                self.dropped_rows = self.dropped_rows.saturating_add(1);
                return false;
            }
        }

        self.buffer[self.buf_len..self.buf_len + row_len].copy_from_slice(row_bytes);
        self.buf_len += row_len;
        self.pending_rows = self.pending_rows.saturating_add(1);
        self.total_logged_rows = self.total_logged_rows.wrapping_add(1);
        self.status = SdStatus::Logging;
        true
    }

    /// Buffers a single CSV record. Writes to SD when buffer fills (>= 400 bytes)
    /// or when periodic flush interval (10 seconds) expires.
    pub fn log_sample(
        &mut self,
        now_ms: u32,
        reading: &InaReading,
        accum: &Accumulators,
        dt: Option<DateTime>,
    ) {
        if self.status != SdStatus::Ready && self.status != SdStatus::Logging {
            return;
        }
        self.current_dt = dt;

        // 1. Format true representative row:
        // Format: Uptime_ms,DateTime_UTC,Voltage_mV,Current_mA,Power_mW,Energy_mWh,Charge_mAh\r\n
        let mut row_buf = [0u8; 96];
        let mut row_cur = BufferCursor::new(&mut row_buf);

        let abs_c = reading.abs_current_tenth();
        let whole_c = abs_c / 10;
        let frac_c = abs_c % 10;

        let whole_p = reading.power_tenth_mw / 10;
        let frac_p = reading.power_tenth_mw % 10;

        let mwh = accum.energy_mwh();
        let mwh_frac = accum.energy_mwh_frac();

        let mah = accum.charge_mah();
        let mah_frac = accum.charge_mah_frac();

        if let Some(d) = dt {
            write!(
                row_cur,
                "{},{:04}-{:02}-{:02} {:02}:{:02}:{:02},{},{}{}.{},{}.{},{}.{:02},{}.{:02}\r\n",
                now_ms,
                d.year,
                d.month,
                d.day,
                d.hour,
                d.minute,
                d.second,
                reading.voltage_mv,
                if reading.is_reverse { "-" } else { "" },
                whole_c,
                frac_c,
                whole_p,
                frac_p,
                mwh,
                mwh_frac,
                mah,
                mah_frac
            )
            .ok();
        } else {
            write!(
                row_cur,
                "{},-,{},{}{}.{},{}.{},{}.{:02},{}.{:02}\r\n",
                now_ms,
                reading.voltage_mv,
                if reading.is_reverse { "-" } else { "" },
                whole_c,
                frac_c,
                whole_p,
                frac_p,
                mwh,
                mwh_frac,
                mah,
                mah_frac
            )
            .ok();
        }

        let row_len = row_cur.as_bytes().len();
        let mut local_row = [0u8; 96];
        local_row[..row_len].copy_from_slice(row_cur.as_bytes());

        // 2. Append row to buffer (flushing first if buffer is near capacity)
        if !self.push_row_with_flush(&local_row[..row_len], |s| s.flush_buffer()) {
            return;
        }

        // 3. Periodic flush every 10 seconds or when buffer is >= 400 bytes
        if self.buf_len >= 400 || now_ms.wrapping_sub(self.last_flush_ms) >= 10_000 {
            let _ = self.flush_buffer();
            self.last_flush_ms = now_ms;
        }
    }

    /// Flushes internal sector buffer to `LOG_XXXX.CSV` on SD card.
    /// Returns `Ok(())` if the buffer is empty or was successfully committed to SD,
    /// or `Err(())` if the write failed (retaining buffered bytes in RAM for retry).
    pub fn flush_buffer(&mut self) -> Result<(), ()> {
        if self.buf_len == 0 {
            return Ok(());
        }

        let dev = SdCardDevice(self.sdcard);
        let vol_mgr = VolumeManager::new(dev, DynamicTimeSource(self.current_dt));

        let res: Result<(), ()> = (|| {
            let vol = vol_mgr.open_volume(VolumeIdx(0)).map_err(|_| ())?;
            let root = vol.open_root_dir().map_err(|_| ())?;
            let filename = self.file_name_str();
            let file = root
                .open_file_in_dir(filename, Mode::ReadWriteCreateOrAppend)
                .map_err(|_| ())?;

            if file.length() == 0 {
                file.write(b"Uptime_ms,DateTime_UTC,Voltage_mV,Current_mA,Power_mW,Energy_mWh,Charge_mAh\r\n")
                    .map_err(|_| ())?;
            }

            file.write(&self.buffer[..self.buf_len]).map_err(|_| ())?;
            file.flush().map_err(|_| ())?;
            Ok(())
        })();

        if res.is_ok() {
            self.committed_rows = self.committed_rows.saturating_add(self.pending_rows);
            self.pending_rows = 0;
            self.buf_len = 0;
            self.status = SdStatus::Logging;
            Ok(())
        } else {
            self.sdcard.borrow_mut().is_initialized = false;
            self.status = SdStatus::WriteError;
            // Retain self.buf_len and self.pending_rows so buffered data is preserved for retry
            self.probe_fail_count = 1;
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dummy_sdcard() -> RefCell<SdCard> {
        // Safe in host tests because pure buffer/session tests never invoke MMIO methods on SdCard
        RefCell::new(unsafe { core::mem::zeroed() })
    }

    #[test]
    fn test_session_num_formatting_and_9999_clamping() {
        let sd = make_dummy_sdcard();
        let mut logger = SdDatalogger::new(&sd, 0);
        assert_eq!(logger.file_name_str(), "LOG_0001.CSV");

        logger.set_session_num(42);
        assert_eq!(logger.session_file_num, 42);
        assert_eq!(logger.file_name_str(), "LOG_0042.CSV");

        logger.set_session_num(9999);
        assert_eq!(logger.session_file_num, 9999);
        assert_eq!(logger.file_name_str(), "LOG_9999.CSV");

        // Verify 10000 clamps at 9999 instead of wrapping to LOG_0000.CSV
        logger.set_session_num(10000);
        assert_eq!(logger.session_file_num, 9999);
        assert_eq!(logger.file_name_str(), "LOG_9999.CSV");
    }

    #[test]
    fn test_push_row_preserves_buffer_and_counts_drops_on_flush_failure() {
        let sd = make_dummy_sdcard();
        let mut logger = SdDatalogger::new(&sd, 0);
        logger.status = SdStatus::Ready;

        // Fill buffer with 5 chunks of 100 bytes (= 500 bytes)
        let row = [b'A'; 100];
        for _ in 0..5 {
            assert!(logger.push_row_with_flush(&row, |_| Ok(())));
        }
        assert_eq!(logger.buffered_bytes(), 500);
        assert_eq!(logger.pending_rows, 5);
        assert_eq!(logger.total_logged_rows, 5);
        assert_eq!(logger.dropped_rows, 0);

        // Attempt to push a 6th 100-byte row while flush fails
        let row_new = [b'B'; 100];
        let ok = logger.push_row_with_flush(&row_new, |s| {
            s.status = SdStatus::WriteError;
            Err(())
        });
        assert!(!ok);
        // Existing 500 bytes MUST be preserved, not wiped!
        assert_eq!(logger.buffered_bytes(), 500);
        assert_eq!(&logger.buffer[..500], &[b'A'; 500]);
        assert_eq!(logger.pending_rows, 5);
        assert_eq!(logger.total_logged_rows, 5);
        assert_eq!(logger.dropped_rows, 1);

        // Now simulate card recovery: flush succeeds, freeing buffer for the new row
        let ok_recovered = logger.push_row_with_flush(&row_new, |s| {
            s.committed_rows = s.committed_rows.saturating_add(s.pending_rows);
            s.pending_rows = 0;
            s.buf_len = 0;
            s.status = SdStatus::Logging;
            Ok(())
        });
        assert!(ok_recovered);
        assert_eq!(logger.committed_rows, 5);
        assert_eq!(logger.pending_rows, 1);
        assert_eq!(logger.buffered_bytes(), 100);
        assert_eq!(&logger.buffer[..100], &[b'B'; 100]);
    }

    #[test]
    fn test_rotate_session_blocks_when_flush_fails_and_rotates_when_empty() {
        let sd = make_dummy_sdcard();
        let mut logger = SdDatalogger::new(&sd, 0);
        logger.set_session_num(10);
        assert_eq!(logger.file_name_str(), "LOG_0010.CSV");

        // Add 1 buffered row
        assert!(logger.push_row_with_flush(b"100,-,5000,10.0,50.0,0.00,0.00\r\n", |_| Ok(())));
        assert!(logger.has_pending_data());

        // Attempt to rotate while flush fails -> MUST refuse rotation and keep LOG_0010.CSV
        let rotated = logger.rotate_session_with_flush(|_| Err(()));
        assert!(!rotated);
        assert_eq!(logger.session_file_num, 10);
        assert_eq!(logger.file_name_str(), "LOG_0010.CSV");
        assert!(logger.has_pending_data());

        // Attempt to rotate when flush succeeds and clears buffer -> MUST advance to LOG_0011.CSV
        let rotated_ok = logger.rotate_session_with_flush(|s| {
            s.committed_rows = s.committed_rows.saturating_add(s.pending_rows);
            s.pending_rows = 0;
            s.buf_len = 0;
            Ok(())
        });
        assert!(rotated_ok);
        assert_eq!(logger.session_file_num, 11);
        assert_eq!(logger.file_name_str(), "LOG_0011.CSV");
        assert!(!logger.has_pending_data());
        assert_eq!(logger.total_logged_rows, 0);
        assert_eq!(logger.committed_rows, 0);
        assert_eq!(logger.pending_rows, 0);
    }
}

