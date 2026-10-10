use longan_nano_bsp::UsbHid;
use crate::model::InaReading;
use crate::ui::SdStatus;

pub struct TelemetryStreamer {
    seq: u8,
}

impl TelemetryStreamer {
    pub const fn new() -> Self {
        Self { seq: 0 }
    }

    /// Packs and transmits high-frequency telemetry report over USB HID.
    ///
    /// Packet layout (9 bytes):
    /// [0]: Report ID = 0x01
    /// [1..2]: Bus Voltage in mV (u16 LE)
    /// [3..4]: Current in 0.1 mA signed (i16 LE) - preserves sub-mA accuracy
    /// [5..6]: Power in mW (u16 LE)
    /// [7]: Flags (bit 0: sensor online, bit 1: reverse, bit 2: overflow, bit 3: SD logging)
    /// [8]: Sequence counter (0..=255)
    pub fn send_reading(
        &mut self,
        usb_hid: &mut UsbHid,
        reading: &InaReading,
        ina_present: bool,
        sd_status: SdStatus,
    ) -> bool {
        let mut flags = 0u8;
        if ina_present {
            flags |= 1 << 0;
        }
        if reading.is_reverse {
            flags |= 1 << 1;
        }
        if reading.overflow {
            flags |= 1 << 2;
        }
        if sd_status == SdStatus::Logging {
            flags |= 1 << 3;
        }

        let report: [u8; 9] = [
            0x01, // Report ID
            (reading.voltage_mv & 0xFF) as u8,
            (reading.voltage_mv >> 8) as u8,
            (reading.current_tenth_ma & 0xFF) as u8,
            ((reading.current_tenth_ma >> 8) & 0xFF) as u8,
            ((reading.power_tenth_mw / 10) & 0xFF) as u8,
            (((reading.power_tenth_mw / 10) >> 8) & 0xFF) as u8,
            flags,
            self.seq,
        ];

        let sent = usb_hid.send_report(&report);
        if sent {
            self.seq = self.seq.wrapping_add(1);
        }
        sent
    }

    /// Polls for incoming host command packets via USB HID OUT report.
    pub fn poll_command(&mut self, usb_hid: &mut UsbHid) -> Option<HidCommand> {
        let mut buf = [0u8; 64];
        if let Some(len) = usb_hid.read_report(&mut buf) {
            if len >= 1 {
                let (cmd_byte, arg_byte) = if buf[0] == 0x02 {
                    if len >= 2 {
                        (buf[1], if len >= 3 { buf[2] } else { 0 })
                    } else {
                        return None;
                    }
                } else {
                    (buf[0], if len >= 2 { buf[1] } else { 0 })
                };

                match cmd_byte {
                    0x01 => Some(HidCommand::SetMode(arg_byte)),
                    0x02 => Some(HidCommand::TareZero),
                    0x03 => Some(HidCommand::FlushSd),
                    0x04 => Some(HidCommand::RotateLog),
                    0x05 => Some(HidCommand::RequestSummary),
                    0x06 => Some(HidCommand::SetBatteryProfile(arg_byte)),
                    other => Some(HidCommand::Unknown(other)),
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidCommand {
    SetMode(u8),
    TareZero,
    FlushSd,
    RotateLog,
    RequestSummary,
    SetBatteryProfile(u8),
    Unknown(u8),
}
