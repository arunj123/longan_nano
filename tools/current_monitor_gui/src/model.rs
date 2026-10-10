use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetryPacket {
    pub timestamp: Instant,
    pub relative_secs: f64,
    pub voltage_mv: u16,
    pub current_tenth_ma: i16,
    pub power_mw: u16,
    pub flags: u8,
    pub seq: u8,
}

impl TelemetryPacket {
    pub fn voltage_v(&self) -> f64 {
        self.voltage_mv as f64 / 1000.0
    }

    pub fn current_ma(&self) -> f64 {
        self.current_tenth_ma as f64 / 10.0
    }

    pub fn power_mw_f64(&self) -> f64 {
        self.power_mw as f64
    }

    pub fn is_online(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    pub fn is_reverse(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    pub fn is_overflow(&self) -> bool {
        (self.flags & 0x04) != 0
    }

    pub fn is_sd_logging(&self) -> bool {
        (self.flags & 0x08) != 0
    }

    pub fn is_alert(&self) -> bool {
        (self.flags & 0x10) != 0
    }

    pub fn resistance_ohms(&self) -> Option<f64> {
        let v = self.voltage_v();
        let i = self.current_ma().abs() / 1000.0;
        if v > 0.5 && i > 0.002 {
            Some(v / i)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryProfile {
    None = 0,
    Lipo500 = 1,
    Lipo1200 = 2,
    LiIon2500 = 3,
    Alkaline1000 = 4,
}

impl BatteryProfile {
    pub fn name(&self) -> &'static str {
        match self {
            BatteryProfile::None => "None / Direct VDD",
            BatteryProfile::Lipo500 => "LiPo 500 mAh (1S)",
            BatteryProfile::Lipo1200 => "LiPo 1200 mAh (1S)",
            BatteryProfile::LiIon2500 => "Li-Ion 18650 2500 mAh",
            BatteryProfile::Alkaline1000 => "Alkaline 2xAA 1000 mAh",
        }
    }

    pub fn id(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnDeviceScreen {
    Hero = 0,
    Graph = 1,
    Stats = 2,
    Histogram = 3,
    BigDigit = 4,
    Cycle = 0xFF,
}

#[derive(Debug, Clone)]
pub enum HidCommand {
    SetMode(OnDeviceScreen),
    TareZero,
    FlushSd,
    RotateLog,
    RequestSummary,
    SetBatteryProfile(BatteryProfile),
    SetEpoch(u32),
    SetCurrentLimit(u16),
}

impl HidCommand {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            HidCommand::SetMode(m) => vec![0x02, 0x01, *m as u8],
            HidCommand::TareZero => vec![0x02, 0x02],
            HidCommand::FlushSd => vec![0x02, 0x03],
            HidCommand::RotateLog => vec![0x02, 0x04],
            HidCommand::RequestSummary => vec![0x02, 0x05],
            HidCommand::SetBatteryProfile(p) => vec![0x02, 0x06, p.id()],
            HidCommand::SetEpoch(epoch) => {
                let mut b = vec![0x02, 0x07];
                b.extend_from_slice(&epoch.to_le_bytes());
                b
            }
            HidCommand::SetCurrentLimit(limit) => {
                let mut b = vec![0x02, 0x08];
                b.extend_from_slice(&limit.to_le_bytes());
                b
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LiveStats {
    pub total_samples: u64,
    pub v_min_mv: u16,
    pub v_max_mv: u16,
    pub v_sum_mv: f64,
    pub c_min_ma: f64,
    pub c_max_ma: f64,
    pub c_sum_ma: f64,
    pub p_max_mw: u16,
    pub p_sum_mw: f64,
    pub energy_mwh: f64,
    pub charge_mah: f64,
    pub start_time: Instant,
    pub last_sample_time: Option<Instant>,
}

impl LiveStats {
    pub fn new() -> Self {
        Self {
            total_samples: 0,
            v_min_mv: u16::MAX,
            v_max_mv: 0,
            v_sum_mv: 0.0,
            c_min_ma: f64::INFINITY,
            c_max_ma: f64::NEG_INFINITY,
            c_sum_ma: 0.0,
            p_max_mw: 0,
            p_sum_mw: 0.0,
            energy_mwh: 0.0,
            charge_mah: 0.0,
            start_time: Instant::now(),
            last_sample_time: None,
        }
    }

    pub fn update(&mut self, pkt: &TelemetryPacket) {
        let now = pkt.timestamp;
        if let Some(last) = self.last_sample_time {
            let dt_hrs = (now.duration_since(last).as_secs_f64()).max(0.0) / 3600.0;
            if dt_hrs < 1.0 {
                self.energy_mwh += pkt.power_mw as f64 * dt_hrs;
                self.charge_mah += pkt.current_ma().abs() * dt_hrs;
            }
        }
        self.last_sample_time = Some(now);

        self.total_samples += 1;
        self.v_min_mv = self.v_min_mv.min(pkt.voltage_mv);
        self.v_max_mv = self.v_max_mv.max(pkt.voltage_mv);
        self.v_sum_mv += pkt.voltage_mv as f64;

        let cur = pkt.current_ma();
        self.c_min_ma = self.c_min_ma.min(cur);
        self.c_max_ma = self.c_max_ma.max(cur);
        self.c_sum_ma += cur;

        self.p_max_mw = self.p_max_mw.max(pkt.power_mw);
        self.p_sum_mw += pkt.power_mw as f64;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn avg_voltage_mv(&self) -> f64 {
        if self.total_samples > 0 {
            self.v_sum_mv / self.total_samples as f64
        } else {
            0.0
        }
    }

    pub fn avg_current_ma(&self) -> f64 {
        if self.total_samples > 0 {
            self.c_sum_ma / self.total_samples as f64
        } else {
            0.0
        }
    }

    pub fn avg_power_mw(&self) -> f64 {
        if self.total_samples > 0 {
            self.p_sum_mw / self.total_samples as f64
        } else {
            0.0
        }
    }
}
