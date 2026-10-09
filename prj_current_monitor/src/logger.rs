use core::cell::RefCell;
use core::fmt::Write;
use embedded_sdmmc::{
    Block, BlockCount, BlockDevice, BlockIdx, Mode, TimeSource, Timestamp,
    VolumeIdx, VolumeManager,
};
use longan_nano_bsp::{SdCard, SdError};
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

pub struct StaticTimeSource;
impl TimeSource for StaticTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp::from_calendar(2026, 10, 9, 12, 0, 0).unwrap_or(Timestamp {
            year_since_1970: 56,
            zero_indexed_month: 9,
            zero_indexed_day: 8,
            hours: 12,
            minutes: 0,
            seconds: 0,
        })
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
    last_flush_ms: u32,
    last_probe_ms: u32,
    probe_fail_count: u8,
}

impl<'a> SdDatalogger<'a> {
    pub fn new(sdcard: &'a RefCell<SdCard>, now_ms: u32) -> Self {
        Self {
            sdcard,
            buffer: [0; 512],
            buf_len: 0,
            status: SdStatus::NoCard,
            total_logged_rows: 0,
            last_flush_ms: now_ms,
            last_probe_ms: now_ms,
            probe_fail_count: 0,
        }
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
            true
        } else {
            self.status = SdStatus::NoCard;
            self.probe_fail_count = self.probe_fail_count.saturating_add(1);
            false
        }
    }

    /// Buffers a single CSV record. Writes to SD when buffer fills (>= 400 bytes)
    /// or when periodic flush interval (10 seconds) expires.
    pub fn log_sample(
        &mut self,
        now_ms: u32,
        reading: &InaReading,
        accum: &Accumulators,
    ) {
        if self.status != SdStatus::Ready && self.status != SdStatus::Logging {
            return;
        }

        // 1. Format true representative row:
        // Format: Timestamp_ms,Voltage_mV,Current_mA,Power_mW,Energy_mWh,Charge_mAh\r\n
        let mut row_buf = [0u8; 80];
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

        write!(
            row_cur,
            "{},{},{}{}.{},{}.{},{}.{:02},{}.{:02}\r\n",
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

        let row_bytes = row_cur.as_bytes();
        let row_len = row_bytes.len();

        // 2. Check if row fits in buffer
        if self.buf_len + row_len > self.buffer.len() {
            self.flush_buffer();
        }

        if self.buf_len + row_len <= self.buffer.len() {
            self.buffer[self.buf_len..self.buf_len + row_len].copy_from_slice(row_bytes);
            self.buf_len += row_len;
            self.total_logged_rows = self.total_logged_rows.wrapping_add(1);
            self.status = SdStatus::Logging;
        }

        // 3. Periodic flush every 10 seconds or when buffer is >= 400 bytes
        if self.buf_len >= 400 || now_ms.wrapping_sub(self.last_flush_ms) >= 10_000 {
            self.flush_buffer();
            self.last_flush_ms = now_ms;
        }
    }

    /// Flushes internal sector buffer to `MONITOR.CSV` on SD card.
    pub fn flush_buffer(&mut self) {
        if self.buf_len == 0 {
            return;
        }

        let dev = SdCardDevice(self.sdcard);
        let vol_mgr = VolumeManager::new(dev, StaticTimeSource);

        let res: Result<(), ()> = (|| {
            let vol = vol_mgr.open_volume(VolumeIdx(0)).map_err(|_| ())?;
            let root = vol.open_root_dir().map_err(|_| ())?;
            let file = root
                .open_file_in_dir("MONITOR.CSV", Mode::ReadWriteCreateOrAppend)
                .map_err(|_| ())?;

            if file.length() == 0 {
                file.write(b"Timestamp_ms,Voltage_mV,Current_mA,Power_mW,Energy_mWh,Charge_mAh\r\n")
                    .map_err(|_| ())?;
            }

            file.write(&self.buffer[..self.buf_len]).map_err(|_| ())?;
            file.flush().map_err(|_| ())?;
            Ok(())
        })();

        if res.is_ok() {
            self.buf_len = 0;
            self.status = SdStatus::Logging;
        } else {
            self.sdcard.borrow_mut().is_initialized = false;
            self.status = SdStatus::WriteError;
            self.buf_len = 0;
            self.probe_fail_count = 1;
        }
    }
}
