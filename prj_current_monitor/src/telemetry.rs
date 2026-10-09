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
}
