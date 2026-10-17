use longan_nano_bsp::UsbHid;
use crate::model::InaReading;
use crate::ui::SdStatus;

pub struct TelemetryStreamer {
    seq: u8,
}

/// Pure helper that packs an `InaReading` and system status flags into a 9-byte HID IN report.
///
/// Packet layout (9 bytes):
/// - `[0]`: Report ID = `0x01`
/// - `[1..2]`: Bus Voltage in mV (`u16` LE)
/// - `[3..4]`: Current in 0.1 mA signed (`i16` LE)
/// - `[5..6]`: Power in mW (`u16` LE, clamped at `65535`)
/// - `[7]`: Flags (`bit 0`: sensor online, `bit 1`: reverse, `bit 2`: overflow/power saturation, `bit 3`: SD logging, `bit 4`: overcurrent alert)
/// - `[8]`: Sequence counter (`0..=255`)
#[inline]
pub fn pack_telemetry_report(
    reading: &InaReading,
    ina_present: bool,
    sd_status: SdStatus,
    alert: bool,
    seq: u8,
) -> [u8; 9] {
    let mut flags = 0u8;
    if ina_present {
        flags |= 1 << 0;
    }
    if reading.is_reverse {
        flags |= 1 << 1;
    }
    if reading.overflow || reading.power_tenth_mw > 655_350 {
        flags |= 1 << 2;
    }
    if sd_status == SdStatus::Logging {
        flags |= 1 << 3;
    }
    if alert {
        flags |= 1 << 4;
    }

    let p_mw = (reading.power_tenth_mw / 10).min(u16::MAX as u32) as u16;

    [
        0x01, // Report ID
        (reading.voltage_mv & 0xFF) as u8,
        (reading.voltage_mv >> 8) as u8,
        (reading.current_tenth_ma & 0xFF) as u8,
        ((reading.current_tenth_ma >> 8) & 0xFF) as u8,
        (p_mw & 0xFF) as u8,
        ((p_mw >> 8) & 0xFF) as u8,
        flags,
        seq,
    ]
}

/// Pure parser for incoming USB HID OUT command reports (`buf[..len]`).
/// Validates packet length and required argument payload size per command opcode.
pub fn decode_hid_command(buf: &[u8]) -> Option<HidCommand> {
    if buf.is_empty() {
        return None;
    }

    let (cmd_byte, payload) = if buf[0] == 0x02 {
        if buf.len() < 2 {
            return None;
        }
        (buf[1], &buf[2..])
    } else {
        (buf[0], &buf[1..])
    };

    match cmd_byte {
        0x00 => None,
        0x01 => {
            if payload.is_empty() {
                None
            } else {
                Some(HidCommand::SetMode(payload[0]))
            }
        }
        0x02 => Some(HidCommand::TareZero),
        0x03 => Some(HidCommand::FlushSd),
        0x04 => Some(HidCommand::RotateLog),
        0x05 => Some(HidCommand::RequestSummary),
        0x06 => {
            if payload.is_empty() {
                None
            } else {
                Some(HidCommand::SetBatteryProfile(payload[0]))
            }
        }
        0x07 => {
            if payload.len() < 4 {
                None
            } else {
                let epoch = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                Some(HidCommand::SetEpoch(epoch))
            }
        }
        0x08 => {
            if payload.len() < 2 {
                None
            } else {
                let limit = u16::from_le_bytes([payload[0], payload[1]]);
                Some(HidCommand::SetCurrentLimit(limit))
            }
        }
        other => Some(HidCommand::Unknown(other)),
    }
}

impl TelemetryStreamer {
    pub const fn new() -> Self {
        Self { seq: 0 }
    }

    #[inline]
    pub const fn seq(&self) -> u8 {
        self.seq
    }

    /// Packs and transmits high-frequency telemetry report over USB HID.
    /// Sequence counter advances unconditionally on every 10 Hz sample so host tools
    /// can detect dropped packets when endpoints are busy or unconfigured.
    pub fn send_reading(
        &mut self,
        usb_hid: &mut UsbHid,
        reading: &InaReading,
        ina_present: bool,
        sd_status: SdStatus,
        alert: bool,
    ) -> bool {
        let cur_seq = self.seq;
        self.seq = self.seq.wrapping_add(1);
        let report = pack_telemetry_report(reading, ina_present, sd_status, alert, cur_seq);
        usb_hid.send_report(&report)
    }

    /// Polls for incoming host command packets via USB HID OUT report.
    pub fn poll_command(&mut self, usb_hid: &mut UsbHid) -> Option<HidCommand> {
        let mut buf = [0u8; 64];
        if let Some(len) = usb_hid.read_report(&mut buf) {
            decode_hid_command(&buf[..len.min(buf.len())])
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
    SetEpoch(u32),
    SetCurrentLimit(u16),
    Unknown(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_telemetry_report_power_saturation_sets_overflow_flag() {
        let reading = InaReading {
            voltage_mv: 30_000,
            current_tenth_ma: 25_000,
            power_tenth_mw: 750_000, // 75,000.0 mW (> 65535 mW u16 max)
            overflow: false,
            is_reverse: false,
        };
        let pkt = pack_telemetry_report(&reading, true, SdStatus::Logging, false, 42);
        assert_eq!(pkt[0], 0x01);
        assert_eq!(u16::from_le_bytes([pkt[5], pkt[6]]), 65535);
        // bit 0 (online) | bit 2 (overflow/power saturation) | bit 3 (logging)
        assert_eq!(pkt[7] & (1 << 2), 1 << 2);
        assert_eq!(pkt[8], 42);
    }

    #[test]
    fn test_decode_hid_command_validates_lengths() {
        assert_eq!(decode_hid_command(&[]), None);
        assert_eq!(decode_hid_command(&[0x02]), None);

        // SetMode requires 1 argument byte
        assert_eq!(decode_hid_command(&[0x02, 0x01]), None);
        assert_eq!(decode_hid_command(&[0x02, 0x01, 0x03]), Some(HidCommand::SetMode(3)));
        assert_eq!(decode_hid_command(&[0x01, 0x04]), Some(HidCommand::SetMode(4)));

        // SetEpoch requires 4 argument bytes
        assert_eq!(decode_hid_command(&[0x02, 0x07, 0x01, 0x02]), None);
        assert_eq!(
            decode_hid_command(&[0x02, 0x07, 0x78, 0x56, 0x34, 0x12]),
            Some(HidCommand::SetEpoch(0x12345678))
        );

        // SetCurrentLimit requires 2 argument bytes
        assert_eq!(decode_hid_command(&[0x02, 0x08, 0xDC]), None);
        assert_eq!(
            decode_hid_command(&[0x02, 0x08, 0xDC, 0x05]),
            Some(HidCommand::SetCurrentLimit(1500))
        );
    }
}

