use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use longan_nano_bsp::CurrentMonitorBoard;

use crate::button::{fmt_hold_progress, BannerType, ButtonController};
use crate::logger::SdDatalogger;
use crate::model::{
    Accumulators, BatteryProfile, BatteryState, HistogramData, InaReading, SampleAggregator,
    SessionStats, WaveformHistory,
};
use crate::telemetry::TelemetryStreamer;
use crate::ui::theme::{COL_AMBER, COL_BG_TOP, COL_CARD_BORDER, COL_CYAN, COL_DIVIDER, COL_MINT};
use crate::ui::{
    BigDigitScreen, GraphScreen, HeroScreen, HistogramScreen, ScreenMode, StatsScreen, TriggerMode,
    UsbDriveScreen, UsbDriveState,
};

pub const BKP_MAGIC_KEY: u16 = 0x5A5A;
pub const BKP_REG_CONFIG: usize = 1; // [15..8] screen_mode, [7..0] battery_profile
pub const BKP_REG_TARE: usize = 2; // tare_offset_tenth (i16)
pub const BKP_REG_LIMIT: usize = 3; // current_limit_ma (u16)
pub const BKP_REG_MAGIC: usize = 4; // BKP_MAGIC_KEY

pub fn save_nvram_config(
    rtc: &mut gd32vf103_hal::Rtc,
    screen_mode: ScreenMode,
    battery_profile: BatteryProfile,
    tare_offset_tenth: i16,
    limit_ma: u16,
) {
    let mode_idx = match screen_mode {
        ScreenMode::Hero => 0u16,
        ScreenMode::Graph => 1u16,
        ScreenMode::Stats => 2u16,
        ScreenMode::Histogram => 3u16,
        ScreenMode::BigDigit => 4u16,
        ScreenMode::UsbDrive => 5u16,
    };
    let config = (mode_idx << 8) | (battery_profile.id() as u16);
    rtc.write_backup_reg(BKP_REG_CONFIG, config);
    rtc.write_backup_reg(BKP_REG_TARE, tare_offset_tenth as u16);
    rtc.write_backup_reg(BKP_REG_LIMIT, limit_ma);
    rtc.write_backup_reg(BKP_REG_MAGIC, BKP_MAGIC_KEY);
}

pub struct App<'a> {
    pub hero_screen: HeroScreen,
    pub graph_screen: GraphScreen,
    pub stats_screen: StatsScreen,
    pub histogram_screen: HistogramScreen,
    pub big_digit_screen: BigDigitScreen,
    pub usb_drive_screen: UsbDriveScreen,
    pub screen_mode: ScreenMode,
    pub current_limit_ma: u16,
    pub over_current: bool,
    pub ina_present: bool,

    pub accum: Accumulators,
    pub history: WaveformHistory,
    pub histogram: HistogramData,
    pub stats: SessionStats,
    pub battery: BatteryState,
    pub telemetry: TelemetryStreamer,
    pub sec_agg: SampleAggregator,
    pub datalogger: SdDatalogger<'a>,
    pub sdcard: &'a core::cell::RefCell<longan_nano_bsp::SdCard>,

    pub last_10hz_ms: u32,
    pub last_1hz_ms: u32,
    pub last_reinit_attempt_ms: u32,
    pub last_heartbeat_ms: u32,
    pub prev_usb_configured: bool,
    pub last_reading: InaReading,

    pub button: ButtonController,
    pub cmd_buf: [u8; 32],
    pub cmd_len: usize,
}

impl<'a> App<'a> {
    pub fn new(
        datalogger: SdDatalogger<'a>,
        sdcard: &'a core::cell::RefCell<longan_nano_bsp::SdCard>,
        now: u32,
    ) -> Self {
        Self {
            hero_screen: HeroScreen::new(),
            graph_screen: GraphScreen::new(),
            stats_screen: StatsScreen::new(),
            histogram_screen: HistogramScreen::new(),
            big_digit_screen: BigDigitScreen::new(),
            usb_drive_screen: UsbDriveScreen::new(),
            screen_mode: ScreenMode::Hero,
            current_limit_ma: 0,
            over_current: false,
            ina_present: false,

            accum: Accumulators::new(),
            history: WaveformHistory::new(),
            histogram: HistogramData::new(),
            stats: SessionStats::new(now),
            battery: BatteryState::new(),
            telemetry: TelemetryStreamer::new(),
            sec_agg: SampleAggregator::new(),
            datalogger,
            sdcard,

            last_10hz_ms: now,
            last_1hz_ms: now,
            last_reinit_attempt_ms: now,
            last_heartbeat_ms: now,
            prev_usb_configured: false,
            last_reading: InaReading::default(),

            button: ButtonController::new(),
            cmd_buf: [0u8; 32],
            cmd_len: 0,
        }
    }

    pub fn save_nvram(&self, board: &mut CurrentMonitorBoard) {
        save_nvram_config(
            &mut board.rtc,
            self.screen_mode,
            self.battery.profile,
            board.ina219.tare_offset_tenth,
            self.current_limit_ma,
        );
    }

    pub fn next_screen_mode(&self) -> ScreenMode {
        match self.screen_mode {
            ScreenMode::Hero => ScreenMode::Graph,
            ScreenMode::Graph => ScreenMode::Stats,
            ScreenMode::Stats => ScreenMode::Histogram,
            ScreenMode::Histogram => ScreenMode::BigDigit,
            ScreenMode::BigDigit => ScreenMode::UsbDrive,
            ScreenMode::UsbDrive => ScreenMode::Hero,
        }
    }

    pub fn prev_screen_mode(&self) -> ScreenMode {
        match self.screen_mode {
            ScreenMode::Hero => ScreenMode::UsbDrive,
            ScreenMode::Graph => ScreenMode::Hero,
            ScreenMode::Stats => ScreenMode::Graph,
            ScreenMode::Histogram => ScreenMode::Stats,
            ScreenMode::BigDigit => ScreenMode::Histogram,
            ScreenMode::UsbDrive => ScreenMode::BigDigit,
        }
    }

    pub fn draw_current_layout(&mut self, board: &mut CurrentMonitorBoard) {
        match self.screen_mode {
            ScreenMode::Hero => self.hero_screen.draw_layout(&mut board.lcd),
            ScreenMode::Graph => {
                self.graph_screen.draw_layout(&mut board.lcd);
                if self.graph_screen.trigger_mode != TriggerMode::Single {
                    self.graph_screen.replot_history(&mut board.lcd, &self.history);
                }
            }
            ScreenMode::Stats => self.stats_screen.draw_layout(&mut board.lcd),
            ScreenMode::Histogram => self.histogram_screen.draw_layout(&mut board.lcd),
            ScreenMode::BigDigit => self.big_digit_screen.draw_layout(&mut board.lcd),
            ScreenMode::UsbDrive => self.usb_drive_screen.draw_layout(&mut board.lcd),
        }
    }

    pub fn set_screen_mode(&mut self, board: &mut CurrentMonitorBoard, new_mode: ScreenMode) {
        if self.screen_mode == ScreenMode::UsbDrive
            && self.usb_drive_screen.state == UsbDriveState::Active
            && new_mode != ScreenMode::UsbDrive
        {
            let now = board.delay.uptime_ms();
            self.exit_usb_drive_active(board, now);
        }
        self.screen_mode = new_mode;
        self.draw_current_layout(board);
        self.save_nvram(board);
        writeln!(
            board.uart0,
            "[UI] Screen mode: {}",
            match self.screen_mode {
                ScreenMode::Hero => "1: HERO",
                ScreenMode::Graph => "2: GRAPH (Oscilloscope)",
                ScreenMode::Stats => "3: STATS (Dashboard)",
                ScreenMode::Histogram => "4: HISTOGRAM",
                ScreenMode::BigDigit => "5: BIG DIGIT",
                ScreenMode::UsbDrive => "6: USB DRIVE (Mass Storage)",
            }
        )
        .ok();
    }

    pub fn cycle_screen_mode(&mut self, board: &mut CurrentMonitorBoard) {
        let next = self.next_screen_mode();
        self.set_screen_mode(board, next);
    }

    pub fn cycle_screen_mode_rev(&mut self, board: &mut CurrentMonitorBoard) {
        let prev = self.prev_screen_mode();
        self.set_screen_mode(board, prev);
    }

    pub fn screen_mode_name(&self) -> &'static str {
        match self.screen_mode {
            ScreenMode::Hero => "HERO",
            ScreenMode::Graph => "GRAPH",
            ScreenMode::Stats => "STATS",
            ScreenMode::Histogram => "HISTO",
            ScreenMode::BigDigit => "BIG DIGIT",
            ScreenMode::UsbDrive => "USB DRIVE",
        }
    }

    pub fn enter_usb_drive_active(
        &mut self,
        board: &mut CurrentMonitorBoard,
        now: u32,
    ) -> bool {
        if self.datalogger.status == crate::ui::SdStatus::NoCard
            || self.datalogger.status == crate::ui::SdStatus::WriteError
        {
            writeln!(board.uart0, "[USB-DRIVE] Cannot mount: SD card not ready").ok();
            self.button.overlay.show_context_banner(" NO SD CARD INSERTED  ", now, 1000);
            self.render_top_bar_overlay(&mut board.lcd);
            return false;
        }

        // 1. Flush any pending buffered log records to SD before handing sector ownership to host
        if self.datalogger.flush_buffer().is_err() || self.datalogger.has_pending_data() {
            writeln!(
                board.uart0,
                "[USB-DRIVE] Cannot mount: failed to flush pending log buffer to SD"
            )
            .ok();
            self.button
                .overlay
                .show_context_banner(" SD FLUSH ERR: NO MOUNT", now, 1000);
            self.render_top_bar_overlay(&mut board.lcd);
            return false;
        }

        // 2. Terminate any SPI streaming states in SdCard
        {
            let mut sd = self.sdcard.borrow_mut();
            sd.end_multi_read();
            sd.end_multi_write();
        }

        // 3. Switch USB peripheral hardware to MSC
        writeln!(board.uart0, "[USB-DRIVE] Detaching USB HID and initializing USB MSC...").ok();
        if !board.switch_to_msc() {
            writeln!(board.uart0, "[USB-DRIVE] Failed to switch to MSC").ok();
            return false;
        }

        // 4. Update MSC media readiness and default to Read-Only for safe log extraction
        if let Some(msc) = board.usb_msc() {
            let sectors = self.sdcard.borrow().sector_count;
            msc.set_media_present(sectors > 0);
            msc.set_write_protected(true);
        }

        // 5. Update state & UI
        self.usb_drive_screen.state = UsbDriveState::Active;
        self.draw_current_layout(board);
        self.button.overlay.show_context_banner(" USB DRIVE: PC MOUNTED ", now, 1000);
        self.render_top_bar_overlay(&mut board.lcd);
        writeln!(
            board.uart0,
            "[USB-DRIVE] >>> Screen 6: USB Drive {} (PC Mounted, Logging Paused) <<<",
            self.usb_drive_screen.state.name()
        )
        .ok();
        true
    }

    pub fn exit_usb_drive_active(
        &mut self,
        board: &mut CurrentMonitorBoard,
        now: u32,
    ) -> bool {
        if self.usb_drive_screen.state != UsbDriveState::Active {
            return false;
        }

        writeln!(board.uart0, "[USB-DRIVE] Detaching USB MSC and restoring USB HID...").ok();

        // 1. Terminate any active MSC read/write stream in SdCard
        {
            let mut sd = self.sdcard.borrow_mut();
            sd.end_multi_read();
            sd.end_multi_write();
        }

        // 2. Switch USB peripheral hardware to HID (calls msc.into_inner() with soft disconnect)
        board.switch_to_hid();

        // 3. Mark state as Standby
        self.usb_drive_screen.state = UsbDriveState::Standby;

        // 4. Re-probe SD card and rotate session to new file
        self.datalogger.status = crate::ui::SdStatus::NoCard;
        if self.datalogger.check_probe(now, &mut board.delay) {
            self.datalogger.rotate_session();
            writeln!(
                board.uart0,
                "[USB-DRIVE] SD card re-probed. Resuming logging in new file: {}",
                self.datalogger.file_name_str()
            )
            .ok();
        } else {
            writeln!(board.uart0, "[USB-DRIVE] SD card probe failed after unmount").ok();
        }

        self.draw_current_layout(board);
        self.button.overlay.show_context_banner(" USB DRIVE: DISMOUNTED ", now, 1000);
        self.render_top_bar_overlay(&mut board.lcd);
        writeln!(
            board.uart0,
            "[USB-DRIVE] >>> Screen 6: USB Drive {} (HID Restored, Logging Resumed) <<<",
            self.usb_drive_screen.state.name()
        )
        .ok();
        true
    }

    pub fn render_top_bar_overlay(&mut self, lcd: &mut longan_nano_bsp::Lcd) {
        if !self.button.overlay.is_active {
            return;
        }
        match self.button.overlay.banner_type {
            BannerType::None => {
                let mut buf = [0u8; 25];
                fmt_hold_progress(&mut buf, self.button.overlay.hold_step);
                if let Ok(s) = core::str::from_utf8(&buf) {
                    lcd.fill_rect(0, 0, 160, 13, COL_BG_TOP);
                    lcd.fill_rect(0, 13, 160, 1, COL_DIVIDER);
                    lcd.draw_string(5, 3, s, &longan_nano_bsp::lcd::FONT_5X7, COL_AMBER, COL_BG_TOP);
                }
            }
            BannerType::TareConfirmed => {
                lcd.fill_rect(0, 0, 160, 13, 0x0162);
                lcd.fill_rect(0, 13, 160, 1, COL_MINT);
                lcd.draw_string(11, 3, "  TARE ZERO RESET OK!  ", &longan_nano_bsp::lcd::FONT_5X7, COL_MINT, 0x0162);
            }
            BannerType::ContextAction(msg) => {
                lcd.fill_rect(0, 0, 160, 13, 0x0944);
                lcd.fill_rect(0, 13, 160, 1, COL_CARD_BORDER);
                lcd.draw_string(8, 3, msg, &longan_nano_bsp::lcd::FONT_5X7, COL_CYAN, 0x0944);
            }
        }
    }

    pub fn tare_and_reset(&mut self, board: &mut CurrentMonitorBoard, now: u32) {
        if self.ina_present {
            let mut sum_raw = 0i32;
            let mut valid_samples = 0i32;
            let cur_tare = board.ina219.tare_offset_tenth as i32;
            for _ in 0..8 {
                if let Ok(raw_data) = board.ina219.read_all() {
                    let true_raw = (raw_data.current_tenth_ma as i32) + cur_tare;
                    sum_raw += true_raw;
                    valid_samples += 1;
                }
                board.delay.delay_ms(2);
            }
            if valid_samples > 0 {
                let avg_offset = (sum_raw / valid_samples) as i16;
                board.ina219.set_tare(avg_offset);
                self.save_nvram(board);
                writeln!(
                    board.uart0,
                    "[SYS] Tare Zero Offset Calibrated: {}.{} mA",
                    avg_offset / 10,
                    avg_offset.unsigned_abs() % 10
                )
                .ok();
            }
        }
        let _ = self.datalogger.flush_buffer();
        let _ = self.datalogger.rotate_session();
        self.accum.reset();
        self.sec_agg.finish_and_reset(&InaReading::default());
        self.history.clear();
        self.histogram.clear();
        self.stats.reset(now);
        writeln!(
            board.uart0,
            "[SYS] Session Reset! Rotated to: {}",
            self.datalogger.file_name_str()
        )
        .ok();

        self.draw_current_layout(board);
        self.last_10hz_ms = board.delay.uptime_ms();
    }

    pub fn dump_session(&self, board: &mut CurrentMonitorBoard, now: u32) {
        let uptime_s = now / 1000;
        let mode_name = match self.screen_mode {
            ScreenMode::Hero => "HERO",
            ScreenMode::Graph => "GRAPH",
            ScreenMode::Stats => "STATS",
            ScreenMode::Histogram => "HISTO",
            ScreenMode::BigDigit => "BIG",
            ScreenMode::UsbDrive => "USB_DRIVE",
        };
        let c_min = if self.stats.has_samples {
            self.stats.c_min
        } else {
            0
        };
        let c_max = if self.stats.has_samples {
            self.stats.c_max
        } else {
            0
        };
        let v_min = if self.stats.has_samples {
            self.stats.v_min
        } else {
            0
        };
        let v_max = if self.stats.has_samples {
            self.stats.v_max
        } else {
            0
        };
        let p_peak_mw = self.stats.p_max / 10;
        let avg_c = self.stats.avg_current_tenth_ma();
        let mcu_temp_tenth = self.stats.mcu_temp_tenth_c;
        let rtc_epoch = board.rtc.get_epoch();
        let rtc_dt = board.rtc.get_datetime();
        let rtc_synced = board.rtc.is_synced();
        let vdda_mv = board.mcu_temp.read_vdda_mv();

        writeln!(board.uart0, "\r\n--- SESSION TELEMETRY SUMMARY ---").ok();
        writeln!(
            board.uart0,
            "Uptime: {} s | Mode: {} | MCU Temp: {}.{} C (VDDA: {} mV)",
            uptime_s,
            mode_name,
            mcu_temp_tenth / 10,
            mcu_temp_tenth.unsigned_abs() % 10,
            vdda_mv
        )
        .ok();
        writeln!(
            board.uart0,
            "Date/Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC | Epoch: {} | Synced: {}",
            rtc_dt.year,
            rtc_dt.month,
            rtc_dt.day,
            rtc_dt.hour,
            rtc_dt.minute,
            rtc_dt.second,
            rtc_epoch,
            if rtc_synced { "YES" } else { "NO" }
        )
        .ok();
        writeln!(
            board.uart0,
            "Voltage: Min {} mV, Max {} mV",
            v_min, v_max
        )
        .ok();
        writeln!(
            board.uart0,
            "Current: Min {}.{} mA, Max {}.{} mA, Avg {}.{} mA",
            c_min / 10,
            c_min.unsigned_abs() % 10,
            c_max / 10,
            c_max.unsigned_abs() % 10,
            avg_c / 10,
            avg_c % 10
        )
        .ok();
        writeln!(board.uart0, "Peak Power: {} mW", p_peak_mw).ok();
        writeln!(
            board.uart0,
            "Energy: {}.{:02} mWh | Charge: {}.{:02} mAh",
            self.accum.energy_mwh(),
            self.accum.energy_mwh_frac(),
            self.accum.charge_mah(),
            self.accum.charge_mah_frac()
        )
        .ok();
        let is_over = self.over_current;
        let is_low_bat = self.battery.is_low_voltage;
        let is_alert = is_over || is_low_bat;
        writeln!(
            board.uart0,
            "Battery: Profile {} | SoC: {}% | Rem: {} mAh ({} mins)",
            self.battery.profile.name(),
            self.battery.soc_pct,
            self.battery.rem_mah,
            self.battery.time_rem_mins
        )
        .ok();
        writeln!(
            board.uart0,
            "Alert Limit: {} mA | Over-Current: {} | Low-Voltage: {}",
            self.current_limit_ma,
            if is_over {
                "YES (OVERCURRENT)"
            } else {
                "NO"
            },
            if is_low_bat {
                "YES (LOW BATTERY)"
            } else {
                "NO"
            }
        )
        .ok();
        writeln!(
            board.uart0,
            "SD File: {} (Rows: {}, Committed: {}, Dropped: {}, Status: {:?})",
            self.datalogger.file_name_str(),
            self.datalogger.total_logged_rows,
            self.datalogger.committed_rows,
            self.datalogger.dropped_rows,
            self.datalogger.status
        )
        .ok();
        writeln!(
            board.uart0,
            "[JSON] {{\"uptime_s\":{},\"mode\":\"{}\",\"epoch\":{},\"time\":\"{:04}-{:02}-{:02} {:02}:{:02}:{:02}\",\"rtc_synced\":{},\"v_min\":{},\"v_max\":{},\"c_min\":{},\"c_max\":{},\"c_avg\":{},\"p_peak_mw\":{},\"mwh\":{},\"mah\":{},\"mcu_temp_c\":{}.{},\"vdda_mv\":{},\"bat_soc\":{},\"sd_file\":\"{}\",\"sd_rows\":{},\"sd_committed\":{},\"sd_dropped\":{},\"limit_ma\":{},\"alert\":{},\"over_current\":{},\"low_voltage\":{}}}",
            uptime_s,
            mode_name,
            rtc_epoch,
            rtc_dt.year, rtc_dt.month, rtc_dt.day, rtc_dt.hour, rtc_dt.minute, rtc_dt.second,
            rtc_synced,
            v_min,
            v_max,
            c_min,
            c_max,
            avg_c,
            p_peak_mw,
            self.accum.energy_mwh(),
            self.accum.charge_mah(),
            mcu_temp_tenth / 10,
            mcu_temp_tenth.unsigned_abs() % 10,
            vdda_mv,
            self.battery.soc_pct,
            self.datalogger.file_name_str(),
            self.datalogger.total_logged_rows,
            self.datalogger.committed_rows,
            self.datalogger.dropped_rows,
            self.current_limit_ma,
            is_alert,
            is_over,
            is_low_bat
        ).ok();
        writeln!(board.uart0, "---------------------------------").ok();
    }

    pub fn print_help(&self, board: &mut CurrentMonitorBoard) {
        writeln!(board.uart0, "\r\n=== REMOTE COMMAND CONSOLE ===").ok();
        writeln!(
            board.uart0,
            " '1'..'6' : Set Screen (1:Hero, 2:Graph, 3:Stats, 4:Histo, 5:BigDigit, 6:UsbDrive)"
        )
        .ok();
        writeln!(board.uart0, " 'm'      : Cycle next screen mode").ok();
        writeln!(
            board.uart0,
            " 'u'      : Toggle USB Drive Mount/Unmount (Screen 6)"
        )
        .ok();
        writeln!(
            board.uart0,
            " 'g'      : Cycle Scope Trigger (AUTO/NORM/SINGLE) / Rearm"
        )
        .ok();
        writeln!(board.uart0, " 'b'      : Cycle battery capacity profile").ok();
        writeln!(
            board.uart0,
            " 't'      : Zero-Tare calibration & reset session"
        )
        .ok();
        writeln!(
            board.uart0,
            " 'c'      : Print current RTC time & sync status"
        )
        .ok();
        writeln!(board.uart0, " 'T <ep>' : Set RTC Unix epoch seconds").ok();
        writeln!(
            board.uart0,
            " 'l' / 'L': Set overcurrent alert limit (0 = disable)"
        )
        .ok();
        writeln!(
            board.uart0,
            " 'r' / 'n': Rotate SD log file to next session"
        )
        .ok();
        writeln!(
            board.uart0,
            " 'o'      : Toggle LCD 180° rotation (persisted in BKP)"
        )
        .ok();
        writeln!(
            board.uart0,
            " 'f'      : Force flush MicroSD sector buffer"
        )
        .ok();
        writeln!(
            board.uart0,
            " 's'      : Dump full telemetry summary (JSON)"
        )
        .ok();
        writeln!(
            board.uart0,
            " 'k' / 'K': Sim PA8 Button (k: Click, K: Double-Click)"
        )
        .ok();
        writeln!(
            board.uart0,
            " 'P' / 'p': Sim PA8 Hold (P: 1.6s Tare, p: Abort Hold)"
        )
        .ok();
        writeln!(board.uart0, " '?' / 'h': Print this command menu").ok();
        writeln!(board.uart0, "==============================").ok();
    }
}
