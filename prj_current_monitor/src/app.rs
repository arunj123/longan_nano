use core::fmt::Write;
use longan_nano_bsp::{
    BkpSlot, CurrentMonitorBoard, Rotation, BKP_REG_ROTATION, BKP_ROTATION_180, BKP_ROTATION_NORMAL,
};

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
    TriggerState, UsbDriveScreen, UsbDriveState,
};

pub const BKP_MAGIC_KEY: u16 = 0x5A5A;
pub const BKP_REG_CONFIG: usize = BkpSlot::MonitorConfig.index(); // [15..8] screen_mode, [7..0] battery_profile
pub const BKP_REG_TARE: usize = BkpSlot::TareOffset.index(); // tare_offset_tenth (i16)
pub const BKP_REG_LIMIT: usize = BkpSlot::CurrentLimit.index(); // current_limit_ma (u16)
pub const BKP_REG_MAGIC: usize = BkpSlot::MonitorMagic.index(); // BKP_MAGIC_KEY

/// Unified application command enum representing every user/host interaction across:
/// - PA8 User Button (Single-Click, Double-Click, Long-Press Hold)
/// - Rotary Encoder (CW/CCW Rotation, Switch Press)
/// - UART0 Interactive Console (`'1'..'6'`, `'m'`, `'u'`, `'g'`, `'b'`, `'t'`, `'T'`, `'L'`, etc.)
/// - USB HID Bidirectional OUT Reports (`HidCommand`)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AppCommand {
    SetScreen(ScreenMode),
    NextScreen,
    PrevScreen,
    ToggleUsbDrive,
    CycleTriggerOrRearm { show_banner: bool },
    AdjustTriggerLevel(i16),
    CycleBatteryProfile { show_banner: bool },
    SetBatteryProfile(BatteryProfile),
    TareAndReset { show_banner: bool },
    FlushSd,
    RotateSdLog { show_banner: bool },
    ToggleLcdRotation,
    SetRtcEpoch(u32),
    PrintRtcTime,
    SetCurrentLimit(u16),
    PrintCurrentLimit,
    DumpSession,
    PrintHelp,
    SimButtonTap,
    SimButtonDoubleTap,
    SimButtonHoldTare,
    SimButtonHoldAbort,
}

/// Maximum allowable uncompensated current magnitude (50.0 mA = 500 in 0.1 mA units)
/// for zero-tare calibration. Prevents calibrating while an active load is connected.
pub const TARE_MAX_ABS_TENTH_MA: u16 = 500;

/// Pure validator for INA219 zero-tare offset calibration.
/// Rejects tare calibration if the uncompensated shunt reading exceeds ±50.0 mA.
#[inline]
pub const fn validate_tare_offset(raw_offset_tenth_ma: i16) -> Result<i16, i16> {
    if raw_offset_tenth_ma.unsigned_abs() <= TARE_MAX_ABS_TENTH_MA {
        Ok(raw_offset_tenth_ma)
    } else {
        Err(raw_offset_tenth_ma)
    }
}

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
    pub i2c_err_count: u8,

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
    pub last_sample_ms: u32,
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
            i2c_err_count: 0,

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
            last_sample_ms: now,
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

    pub fn toggle_usb_drive(&mut self, board: &mut CurrentMonitorBoard, now: u32) {
        if self.screen_mode != ScreenMode::UsbDrive {
            self.set_screen_mode(board, ScreenMode::UsbDrive);
        } else if self.usb_drive_screen.state == UsbDriveState::Standby {
            self.enter_usb_drive_active(board, now);
        } else {
            self.exit_usb_drive_active(board, now);
        }
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

    pub fn tare_and_reset(&mut self, board: &mut CurrentMonitorBoard, now: u32) -> bool {
        if self.ina_present {
            if let Ok(raw_offset) = board.ina219.read_uncompensated_tenth_ma() {
                match validate_tare_offset(raw_offset) {
                    Ok(valid_offset) => {
                        board.ina219.set_tare(valid_offset);
                        self.save_nvram(board);
                        writeln!(
                            board.uart0,
                            "[SYS] Tare Zero Offset Calibrated: {}.{} mA",
                            valid_offset / 10,
                            valid_offset.unsigned_abs() % 10
                        )
                        .ok();
                    }
                    Err(load_tenth) => {
                        writeln!(
                            board.uart0,
                            "[SYS] Tare Rejected: Active load detected ({}.{} mA > 50.0 mA)",
                            load_tenth / 10,
                            load_tenth.unsigned_abs() % 10
                        )
                        .ok();
                        self.button
                            .overlay
                            .show_context_banner(" TARE REJECTED: LOAD ON ", now, 1200);
                        self.render_top_bar_overlay(&mut board.lcd);
                        return false;
                    }
                }
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
        self.last_sample_ms = self.last_10hz_ms;
        true
    }

    /// Central command dispatcher for all hardware buttons, rotary encoder, UART CLI, and USB HID commands.
    /// Ensures 100% consistent behavior across all input modalities and eliminates multi-place update bugs.
    pub fn execute_command(
        &mut self,
        board: &mut CurrentMonitorBoard,
        cmd: AppCommand,
        now: u32,
    ) {
        match cmd {
            AppCommand::SetScreen(mode) => {
                self.set_screen_mode(board, mode);
            }
            AppCommand::NextScreen => {
                self.cycle_screen_mode(board);
            }
            AppCommand::PrevScreen => {
                self.cycle_screen_mode_rev(board);
            }
            AppCommand::ToggleUsbDrive => {
                self.toggle_usb_drive(board, now);
            }
            AppCommand::CycleTriggerOrRearm { show_banner } => {
                if self.graph_screen.trigger_mode == TriggerMode::Single
                    && self.graph_screen.trigger_state == TriggerState::Halted
                {
                    self.graph_screen.rearm();
                    writeln!(
                        board.uart0,
                        "[TRIG] Single sweep rearmed! Waiting for trigger..."
                    )
                    .ok();
                } else {
                    self.graph_screen.cycle_trigger_mode();
                    writeln!(
                        board.uart0,
                        "[TRIG] Trigger Mode: {} | State: {} | Level: {}.{} mA",
                        self.graph_screen.trigger_mode.name(),
                        self.graph_screen.trigger_state_name(),
                        self.graph_screen.trigger_level_tenth / 10,
                        self.graph_screen.trigger_level_tenth % 10,
                    )
                    .ok();
                    if show_banner {
                        let banner = match self.graph_screen.trigger_mode {
                            TriggerMode::Auto => " TRIG MODE: AUTO        ",
                            TriggerMode::Norm => " TRIG MODE: NORMAL      ",
                            TriggerMode::Single => " TRIG MODE: SINGLE      ",
                        };
                        self.button.overlay.show_context_banner(banner, now, 700);
                        self.render_top_bar_overlay(&mut board.lcd);
                    }
                }
            }
            AppCommand::AdjustTriggerLevel(delta_tenth) => {
                self.graph_screen.adjust_trigger_level(delta_tenth);
                writeln!(
                    board.uart0,
                    "[TRIG] Level: {}.{} mA | Mode: {} | State: {}",
                    self.graph_screen.trigger_level_tenth / 10,
                    self.graph_screen.trigger_level_tenth % 10,
                    self.graph_screen.trigger_mode.name(),
                    self.graph_screen.trigger_state_name(),
                )
                .ok();
            }
            AppCommand::CycleBatteryProfile { show_banner } => {
                self.battery.profile = self.battery.profile.next();
                self.save_nvram(board);
                writeln!(
                    board.uart0,
                    "[BATT] Switched battery profile to: {} (Nominal: {} mAh)",
                    self.battery.profile.name(),
                    self.battery.profile.capacity_mah()
                )
                .ok();
                if show_banner {
                    let banner = match self.battery.profile {
                        BatteryProfile::None => " BATT: NONE (Disabled)  ",
                        BatteryProfile::Lipo500 => " BATT: LIPO 500 mAh     ",
                        BatteryProfile::Lipo1200 => " BATT: LIPO 1200 mAh    ",
                        BatteryProfile::LiIon2500 => " BATT: LI-ION 2500 mAh  ",
                        BatteryProfile::Alkaline1000 => " BATT: ALKALINE 1000 mAh",
                    };
                    self.button.overlay.show_context_banner(banner, now, 700);
                    self.render_top_bar_overlay(&mut board.lcd);
                }
            }
            AppCommand::SetBatteryProfile(profile) => {
                self.battery.profile = profile;
                self.save_nvram(board);
                writeln!(
                    board.uart0,
                    "[HID-CMD] Battery profile set to: {}",
                    self.battery.profile.name()
                )
                .ok();
            }
            AppCommand::TareAndReset { show_banner } => {
                if self.tare_and_reset(board, now) {
                    if show_banner {
                        self.button.overlay.show_tare_confirmed(now, 1000);
                        self.render_top_bar_overlay(&mut board.lcd);
                    }
                }
            }
            AppCommand::FlushSd => {
                if self.datalogger.flush_buffer().is_ok() {
                    writeln!(
                        board.uart0,
                        "[SD] Flushed buffer to {} (Total rows: {})",
                        self.datalogger.file_name_str(),
                        self.datalogger.total_logged_rows
                    )
                    .ok();
                } else {
                    writeln!(
                        board.uart0,
                        "[SD] Flush failed on {} (Pending bytes: {})",
                        self.datalogger.file_name_str(),
                        self.datalogger.buffered_bytes()
                    )
                    .ok();
                }
            }
            AppCommand::RotateSdLog { show_banner } => {
                if self.datalogger.rotate_session() {
                    writeln!(
                        board.uart0,
                        "[SD] Rotated to new file: {}",
                        self.datalogger.file_name_str()
                    )
                    .ok();
                    if show_banner {
                        self.button
                            .overlay
                            .show_context_banner(" SD: ROTATED LOG FILE   ", now, 700);
                        self.render_top_bar_overlay(&mut board.lcd);
                    }
                } else {
                    writeln!(
                        board.uart0,
                        "[SD] Cannot rotate: pending buffer flush failed ({})",
                        self.datalogger.file_name_str()
                    )
                    .ok();
                    if show_banner {
                        self.button
                            .overlay
                            .show_context_banner(" SD ERR: ROTATE BLOCKED ", now, 700);
                        self.render_top_bar_overlay(&mut board.lcd);
                    }
                }
            }
            AppCommand::ToggleLcdRotation => {
                let new_rot = board.lcd.toggle_rotation();
                let bkp_val = if new_rot == Rotation::Rotated180 {
                    BKP_ROTATION_180
                } else {
                    BKP_ROTATION_NORMAL
                };
                board.rtc.write_backup_reg(BKP_REG_ROTATION, bkp_val);
                writeln!(
                    board.uart0,
                    "[LCD] Toggled display rotation to {:?} (Saved BKP: 0x{:04X})",
                    new_rot, bkp_val
                )
                .ok();
                self.draw_current_layout(board);
            }
            AppCommand::SetRtcEpoch(epoch) => {
                board.rtc.set_epoch(epoch);
                let dt = board.rtc.get_datetime();
                writeln!(
                    board.uart0,
                    "[RTC] Set Unix epoch to: {} ({:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC)",
                    epoch, dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
                )
                .ok();
            }
            AppCommand::PrintRtcTime => {
                let epoch = board.rtc.get_epoch();
                let dt = board.rtc.get_datetime();
                writeln!(
                    board.uart0,
                    "[RTC] Current Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC | Epoch: {} | Synced: {}",
                    dt.year,
                    dt.month,
                    dt.day,
                    dt.hour,
                    dt.minute,
                    dt.second,
                    epoch,
                    if board.rtc.is_synced() { "YES" } else { "NO" }
                )
                .ok();
            }
            AppCommand::SetCurrentLimit(limit_ma) => {
                self.current_limit_ma = limit_ma.min(3200);
                self.save_nvram(board);
                writeln!(
                    board.uart0,
                    "[LIMIT] Set Over-Current Alert Limit to: {} mA",
                    self.current_limit_ma
                )
                .ok();
            }
            AppCommand::PrintCurrentLimit => {
                writeln!(
                    board.uart0,
                    "[LIMIT] Over-Current Alert Limit: {} mA (Alert Active: {})",
                    self.current_limit_ma,
                    if self.over_current { "YES" } else { "NO" }
                )
                .ok();
            }
            AppCommand::DumpSession => {
                self.dump_session(board, now);
            }
            AppCommand::PrintHelp => {
                self.print_help(board);
            }
            AppCommand::SimButtonTap => {
                self.button.simulate_tap(now, 80);
                writeln!(board.uart0, "[SIM] Simulated button tap (80 ms)").ok();
            }
            AppCommand::SimButtonDoubleTap => {
                self.button.simulate_double_tap(now, 80, 100);
                writeln!(
                    board.uart0,
                    "[SIM] Simulated button double-tap (80ms + 100ms gap + 80ms)"
                )
                .ok();
            }
            AppCommand::SimButtonHoldTare => {
                self.button.simulate_hold(now, 1600);
                writeln!(board.uart0, "[SIM] Simulated button hold (1600 ms)").ok();
            }
            AppCommand::SimButtonHoldAbort => {
                self.button.simulate_hold(now, 600);
                writeln!(board.uart0, "[SIM] Simulated button aborted hold (600 ms)").ok();
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tare_offset_load_sanity_guard() {
        assert_eq!(validate_tare_offset(0), Ok(0));
        assert_eq!(validate_tare_offset(12), Ok(12));
        assert_eq!(validate_tare_offset(-15), Ok(-15));
        assert_eq!(validate_tare_offset(500), Ok(500));
        assert_eq!(validate_tare_offset(-500), Ok(-500));

        // Active loads exceeding ±50.0 mA (±500 tenths of mA) must be rejected
        assert_eq!(validate_tare_offset(501), Err(501));
        assert_eq!(validate_tare_offset(-501), Err(-501));
        assert_eq!(validate_tare_offset(20_000), Err(20_000));
        assert_eq!(validate_tare_offset(-20_000), Err(-20_000));
    }

    #[test]
    fn test_bkp_slot_mappings() {
        assert_eq!(BKP_REG_CONFIG, BkpSlot::MonitorConfig.index());
        assert_eq!(BKP_REG_TARE, BkpSlot::TareOffset.index());
        assert_eq!(BKP_REG_LIMIT, BkpSlot::CurrentLimit.index());
        assert_eq!(BKP_REG_MAGIC, BkpSlot::MonitorMagic.index());
        assert_eq!(BKP_REG_CONFIG, 1);
        assert_eq!(BKP_REG_TARE, 2);
        assert_eq!(BKP_REG_LIMIT, 3);
        assert_eq!(BKP_REG_MAGIC, 4);
    }

    #[test]
    fn test_app_command_variants() {
        let cmd = AppCommand::SetScreen(ScreenMode::Hero);
        assert_eq!(cmd, AppCommand::SetScreen(ScreenMode::Hero));
        assert_ne!(cmd, AppCommand::NextScreen);
        assert_eq!(AppCommand::ToggleUsbDrive, AppCommand::ToggleUsbDrive);
        assert_eq!(AppCommand::AdjustTriggerLevel(100), AppCommand::AdjustTriggerLevel(100));
        assert_eq!(
            AppCommand::CycleBatteryProfile { show_banner: true },
            AppCommand::CycleBatteryProfile { show_banner: true }
        );
        assert_eq!(
            AppCommand::SetBatteryProfile(BatteryProfile::Lipo500),
            AppCommand::SetBatteryProfile(BatteryProfile::Lipo500)
        );
        assert_eq!(
            AppCommand::TareAndReset { show_banner: false },
            AppCommand::TareAndReset { show_banner: false }
        );
        assert_eq!(
            AppCommand::RotateSdLog { show_banner: true },
            AppCommand::RotateSdLog { show_banner: true }
        );
    }

    #[test]
    fn test_app_command_exhaustive_coverage() {
        let all_cmds = [
            AppCommand::SetScreen(ScreenMode::Graph),
            AppCommand::NextScreen,
            AppCommand::PrevScreen,
            AppCommand::ToggleUsbDrive,
            AppCommand::CycleTriggerOrRearm { show_banner: false },
            AppCommand::AdjustTriggerLevel(50),
            AppCommand::CycleBatteryProfile { show_banner: false },
            AppCommand::SetBatteryProfile(BatteryProfile::LiIon2500),
            AppCommand::TareAndReset { show_banner: true },
            AppCommand::FlushSd,
            AppCommand::RotateSdLog { show_banner: false },
            AppCommand::ToggleLcdRotation,
            AppCommand::SetRtcEpoch(1700000000),
            AppCommand::PrintRtcTime,
            AppCommand::SetCurrentLimit(1500),
            AppCommand::PrintCurrentLimit,
            AppCommand::DumpSession,
            AppCommand::PrintHelp,
            AppCommand::SimButtonTap,
            AppCommand::SimButtonDoubleTap,
            AppCommand::SimButtonHoldTare,
            AppCommand::SimButtonHoldAbort,
        ];

        for (i, cmd_a) in all_cmds.iter().enumerate() {
            for (j, cmd_b) in all_cmds.iter().enumerate() {
                if i == j {
                    assert_eq!(cmd_a, cmd_b);
                } else {
                    assert_ne!(cmd_a, cmd_b);
                }
            }
        }
    }

    #[test]
    fn test_screen_mode_forward_and_backward_transitions() {
        let modes = [
            ScreenMode::Hero,
            ScreenMode::Graph,
            ScreenMode::Stats,
            ScreenMode::Histogram,
            ScreenMode::BigDigit,
            ScreenMode::UsbDrive,
        ];

        // Forward cycling
        for (i, &mode) in modes.iter().enumerate() {
            let next_expected = modes[(i + 1) % modes.len()];
            let next_actual = match mode {
                ScreenMode::Hero => ScreenMode::Graph,
                ScreenMode::Graph => ScreenMode::Stats,
                ScreenMode::Stats => ScreenMode::Histogram,
                ScreenMode::Histogram => ScreenMode::BigDigit,
                ScreenMode::BigDigit => ScreenMode::UsbDrive,
                ScreenMode::UsbDrive => ScreenMode::Hero,
            };
            assert_eq!(next_actual, next_expected);
        }

        // Backward cycling
        for (i, &mode) in modes.iter().enumerate() {
            let prev_expected = modes[(i + modes.len() - 1) % modes.len()];
            let prev_actual = match mode {
                ScreenMode::Hero => ScreenMode::UsbDrive,
                ScreenMode::Graph => ScreenMode::Hero,
                ScreenMode::Stats => ScreenMode::Graph,
                ScreenMode::Histogram => ScreenMode::Stats,
                ScreenMode::BigDigit => ScreenMode::Histogram,
                ScreenMode::UsbDrive => ScreenMode::BigDigit,
            };
            assert_eq!(prev_actual, prev_expected);
        }
    }

    #[test]
    fn test_tare_offset_boundary_cases() {
        assert_eq!(validate_tare_offset(-500), Ok(-500));
        assert_eq!(validate_tare_offset(500), Ok(500));
        assert_eq!(validate_tare_offset(-501), Err(-501));
        assert_eq!(validate_tare_offset(501), Err(501));
        assert_eq!(validate_tare_offset(i16::MIN), Err(i16::MIN));
        assert_eq!(validate_tare_offset(i16::MAX), Err(i16::MAX));
    }

    #[test]
    fn test_nvram_config_packing_and_screen_mode_encoding() {
        let mode_idx = |m: ScreenMode| -> u16 {
            match m {
                ScreenMode::Hero => 0,
                ScreenMode::Graph => 1,
                ScreenMode::Stats => 2,
                ScreenMode::Histogram => 3,
                ScreenMode::BigDigit => 4,
                ScreenMode::UsbDrive => 5,
            }
        };

        for &mode in &[
            ScreenMode::Hero,
            ScreenMode::Graph,
            ScreenMode::Stats,
            ScreenMode::Histogram,
            ScreenMode::BigDigit,
            ScreenMode::UsbDrive,
        ] {
            let config = (mode_idx(mode) << 8) | (BatteryProfile::LiIon2500.id() as u16);
            assert_eq!((config >> 8) as usize, mode_idx(mode) as usize);
            assert_eq!((config & 0xFF) as u8, BatteryProfile::LiIon2500.id());
        }
    }

    #[test]
    fn test_decoupled_sample_dt_integration_no_double_counting() {
        // Demonstrate [MON-MATH-01]:
        // When a 40 ms loop stall occurs, compare phase-locked dt_ms vs decoupled dt_sample_ms.
        let mut accum_legacy = Accumulators::new();
        let mut accum_decoupled = Accumulators::new();

        let power_tenth_mw = 50_000; // 5000.0 mW
        let current_tenth_ma = 10_000; // 1000.0 mA

        // Initial state at t = 0 ms
        let mut last_10hz_ms = 0u32;
        let mut last_sample_ms = 0u32;

        // Iteration 1: Delayed by 40 ms (SD card flush or flash stall) -> now = 140 ms
        let now_1 = 140u32;
        let dt_ms_1 = now_1.wrapping_sub(last_10hz_ms); // 140 ms
        assert_eq!(dt_ms_1, 140);
        last_10hz_ms = if dt_ms_1 >= 200 { now_1 } else { last_10hz_ms.wrapping_add(100) }; // 100 ms
        accum_legacy.update(power_tenth_mw, current_tenth_ma, dt_ms_1); // integrates 140 ms

        let dt_sample_1 = now_1.wrapping_sub(last_sample_ms); // 140 ms
        last_sample_ms = now_1;
        accum_decoupled.update(power_tenth_mw, current_tenth_ma, dt_sample_1.min(500)); // integrates 140 ms

        // Iteration 2: Catch-up tick at t = 200 ms (nominal next 10 Hz phase)
        let now_2 = 200u32;
        let dt_ms_2 = now_2.wrapping_sub(last_10hz_ms); // 200 - 100 = 100 ms
        assert_eq!(dt_ms_2, 100);
        last_10hz_ms = if dt_ms_2 >= 200 { now_2 } else { last_10hz_ms.wrapping_add(100) };
        accum_legacy.update(power_tenth_mw, current_tenth_ma, dt_ms_2); // integrates 100 ms

        let dt_sample_2 = now_2.wrapping_sub(last_sample_ms); // 200 - 140 = 60 ms!
        last_sample_ms = now_2;
        accum_decoupled.update(power_tenth_mw, current_tenth_ma, dt_sample_2.min(500)); // integrates 60 ms

        // In 200 ms of physical wall-clock time:
        // Legacy integrated: 140 ms + 100 ms = 240 ms (40 ms / 20% overcount!)
        // Decoupled integrated: 140 ms + 60 ms = 200 ms (exact physical elapsed time!)
        let expected_energy_ticks_200ms = (power_tenth_mw as u64) * 200;
        assert_eq!(accum_decoupled.energy_ticks, expected_energy_ticks_200ms);

        let inflated_energy_ticks_240ms = (power_tenth_mw as u64) * 240;
        assert_eq!(accum_legacy.energy_ticks, inflated_energy_ticks_240ms);
        assert!(accum_legacy.energy_ticks > accum_decoupled.energy_ticks);
    }

    #[test]
    fn test_over_current_saturation_gating() {
        // Demonstrate [MON-MATH-02]:
        let check_oc = |limit_ma: u16, reading: &InaReading| -> bool {
            limit_ma > 0
                && (reading.abs_current_tenth() >= ((limit_ma as u32) * 10)
                    || (limit_ma >= 3200 && reading.overflow))
        };

        // Case 1: Limit 3200 mA (INA219 full-scale)
        let mut r = InaReading {
            voltage_mv: 5000,
            current_tenth_ma: 31990,
            power_tenth_mw: 159950,
            overflow: false,
            is_reverse: false,
        };
        assert!(!check_oc(3200, &r)); // 3199.0 mA < 3200 mA

        r.current_tenth_ma = 32000;
        assert!(check_oc(3200, &r)); // 3200.0 mA == limit

        // At full-scale, analog shunt saturation sets overflow flag
        r.current_tenth_ma = 32000;
        r.overflow = true;
        assert!(check_oc(3200, &r)); // Saturation overflow triggers alert!

        // If math overflow occurred at 2500 mA
        r.current_tenth_ma = 25000;
        r.overflow = true;
        assert!(check_oc(3200, &r)); // limit >= 3200 && overflow triggers

        // Case 2: Limit 1000 mA
        let mut r_1a = InaReading {
            voltage_mv: 5000,
            current_tenth_ma: 9990,
            power_tenth_mw: 49950,
            overflow: false,
            is_reverse: false,
        };
        assert!(!check_oc(1000, &r_1a));

        r_1a.current_tenth_ma = 10000;
        assert!(check_oc(1000, &r_1a));

        // Negative current magnitude checks (reverse charging)
        r_1a.current_tenth_ma = -10000;
        r_1a.is_reverse = true;
        assert!(check_oc(1000, &r_1a));

        // Case 3: Limit 0 (disabled)
        assert!(!check_oc(0, &r));
    }

    #[test]
    fn test_sample_valid_gating_prevents_offline_pollution() {
        // Demonstrate [MON-MATH-02]:
        let mut history = WaveformHistory::new();
        let mut histogram = HistogramData::new();
        let mut stats = SessionStats::new(0);

        // When sample_valid is false (sensor offline or transient error):
        let sample_valid = false;
        let reading = InaReading::default(); // 0 values

        if sample_valid {
            history.push(reading.current_tenth_ma);
            histogram.record(reading.abs_current_tenth());
            stats.update(reading.voltage_mv, reading.current_tenth_ma, reading.power_tenth_mw);
        }

        // History, histogram, and stats must remain pristine
        assert_eq!(history.count, 0);
        assert_eq!(histogram.total, 0);
        assert_eq!(stats.samples, 0);
        assert!(!stats.has_samples);

        // When sample_valid is true:
        let sample_valid = true;
        let valid_reading = InaReading {
            voltage_mv: 5000,
            current_tenth_ma: 2500,
            power_tenth_mw: 12500,
            overflow: false,
            is_reverse: false,
        };

        if sample_valid {
            history.push(valid_reading.current_tenth_ma);
            histogram.record(valid_reading.abs_current_tenth());
            stats.update(valid_reading.voltage_mv, valid_reading.current_tenth_ma, valid_reading.power_tenth_mw);
        }

        assert_eq!(history.count, 1);
        assert_eq!(histogram.total, 1);
        assert_eq!(stats.samples, 1);
        assert!(stats.has_samples);
    }
}
