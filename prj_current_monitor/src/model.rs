use core::cmp::{max, min};

/// Instantaneous sensor reading snapshot from INA219.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct InaReading {
    pub voltage_mv: u16,
    pub current_tenth_ma: i16, // Signed: negative = reverse flow
    pub power_tenth_mw: u32,
    pub overflow: bool,
    pub is_reverse: bool,
}

impl InaReading {
    #[inline]
    pub fn abs_current_tenth(&self) -> u32 {
        self.current_tenth_ma.unsigned_abs() as u32
    }
}

/// Scale tier configuration for oscilloscope screen.
#[derive(Copy, Clone)]
pub struct ScaleTier {
    pub max_val: i32, // In tenths of mA (0.1 mA)
    pub top_lbl: &'static str,
    pub mid_lbl: &'static str,
    pub badge: &'static str,
}

pub static CURRENT_TIERS: [ScaleTier; 8] = [
    ScaleTier { max_val: 100,   top_lbl: "10m",  mid_lbl: " 5m",  badge: "[10m]" },
    ScaleTier { max_val: 250,   top_lbl: "25m",  mid_lbl: "12m",  badge: "[25m]" },
    ScaleTier { max_val: 500,   top_lbl: "50m",  mid_lbl: "25m",  badge: "[50m]" },
    ScaleTier { max_val: 1000,  top_lbl: "100m", mid_lbl: "50m",  badge: "[100m]"},
    ScaleTier { max_val: 2500,  top_lbl: "250m", mid_lbl: "125m", badge: "[250m]"},
    ScaleTier { max_val: 5000,  top_lbl: "500m", mid_lbl: "250m", badge: "[500m]"},
    ScaleTier { max_val: 10000, top_lbl: " 1A",  mid_lbl: "500m", badge: " [1A] "},
    ScaleTier { max_val: 32000, top_lbl: "3.2A", mid_lbl: "1.6A", badge: "[3.2A]"},
];

/// 137-sample circular buffer for preserving waveform history across autoscale changes.
pub struct WaveformHistory {
    pub samples: [i16; 137],
    pub head: usize,
    pub count: usize,
}

impl WaveformHistory {
    pub const CAPACITY: usize = 137;

    pub const fn new() -> Self {
        Self {
            samples: [0; 137],
            head: 0,
            count: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, val: i16) {
        self.samples[self.head] = val;
        self.head = (self.head + 1) % Self::CAPACITY;
        if self.count < Self::CAPACITY {
            self.count += 1;
        }
    }

    /// Retrieves sample from oldest (0) to newest (count - 1).
    #[inline]
    pub fn get_chronological(&self, index: usize) -> i16 {
        if index >= self.count {
            return 0;
        }
        if self.count < Self::CAPACITY {
            self.samples[index]
        } else {
            let offset = (self.head + index) % Self::CAPACITY;
            self.samples[offset]
        }
    }

    pub fn clear(&mut self) {
        self.samples = [0; 137];
        self.head = 0;
        self.count = 0;
    }
}

/// Drift-free energy and charge integration using exact millisecond delta math.
///
/// Unit definitions:
/// - Power in tenths of mW (10^-4 W).
/// - Current in tenths of mA (10^-4 A).
/// - 1 uWh = 36,000 (tenth_mw * ms).
/// - 1 uAh = 36,000 (tenth_ma * ms).
pub struct Accumulators {
    pub energy_ticks: u64, // (tenth_mw * ms)
    pub charge_ticks: u64, // (tenth_ma * ms)
}

impl Accumulators {
    pub const fn new() -> Self {
        Self {
            energy_ticks: 0,
            charge_ticks: 0,
        }
    }

    #[inline]
    pub fn update(&mut self, p_tenth_mw: u32, abs_c_tenth: u32, dt_ms: u32) {
        let e_inc = (p_tenth_mw as u64) * (dt_ms as u64);
        let q_inc = (abs_c_tenth as u64) * (dt_ms as u64);
        self.energy_ticks = self.energy_ticks.saturating_add(e_inc);
        self.charge_ticks = self.charge_ticks.saturating_add(q_inc);
    }

    #[inline]
    pub fn reset(&mut self) {
        self.energy_ticks = 0;
        self.charge_ticks = 0;
    }

    /// Total accumulated energy in micro-watt-hours (uWh).
    #[inline]
    pub fn energy_uwh(&self) -> u64 {
        self.energy_ticks / 36_000
    }

    /// Total accumulated energy in milli-watt-hours (mWh).
    #[inline]
    pub fn energy_mwh(&self) -> u32 {
        (self.energy_ticks / 36_000_000) as u32
    }

    /// Fractional mWh (0..99).
    #[inline]
    pub fn energy_mwh_frac(&self) -> u32 {
        ((self.energy_ticks % 36_000_000) / 360_000) as u32
    }

    /// Total accumulated charge in micro-amp-hours (uAh).
    #[inline]
    pub fn charge_uah(&self) -> u64 {
        self.charge_ticks / 36_000
    }

    /// Total accumulated charge in milli-amp-hours (mAh).
    #[inline]
    pub fn charge_mah(&self) -> u32 {
        (self.charge_ticks / 36_000_000) as u32
    }

    /// Fractional mAh (0..99).
    #[inline]
    pub fn charge_mah_frac(&self) -> u32 {
        ((self.charge_ticks % 36_000_000) / 360_000) as u32
    }
}

/// Comprehensive session analytics and min/max/average tracking.
pub struct SessionStats {
    pub v_min: u16,
    pub v_max: u16,
    pub c_min: i16,
    pub c_max: i16,
    pub p_max: u32, // In tenths of mW
    pub v_sum: u64,
    pub c_sum_abs: u64,
    pub samples: u32,
    pub start_time_ms: u32,
    pub has_samples: bool,
}

impl SessionStats {
    pub fn new(now_ms: u32) -> Self {
        Self {
            v_min: u16::MAX,
            v_max: 0,
            c_min: i16::MAX,
            c_max: i16::MIN,
            p_max: 0,
            v_sum: 0,
            c_sum_abs: 0,
            samples: 0,
            start_time_ms: now_ms,
            has_samples: false,
        }
    }

    pub fn update(&mut self, v_mv: u16, c_tenth: i16, p_tenth: u32) {
        if v_mv > 500 {
            self.has_samples = true;
            self.samples = self.samples.saturating_add(1);
            self.v_sum = self.v_sum.saturating_add(v_mv as u64);
            self.c_sum_abs = self.c_sum_abs.saturating_add(c_tenth.unsigned_abs() as u64);

            self.v_min = min(self.v_min, v_mv);
            self.v_max = max(self.v_max, v_mv);
            self.c_min = min(self.c_min, c_tenth);
            self.c_max = max(self.c_max, c_tenth);
            self.p_max = max(self.p_max, p_tenth);
        }
    }

    pub fn reset(&mut self, now_ms: u32) {
        self.v_min = u16::MAX;
        self.v_max = 0;
        self.c_min = i16::MAX;
        self.c_max = i16::MIN;
        self.p_max = 0;
        self.v_sum = 0;
        self.c_sum_abs = 0;
        self.samples = 0;
        self.start_time_ms = now_ms;
        self.has_samples = false;
    }

    #[allow(dead_code)]
    pub fn avg_voltage_mv(&self) -> u16 {
        if self.samples > 0 {
            (self.v_sum / (self.samples as u64)) as u16
        } else {
            0
        }
    }

    #[allow(dead_code)]
    pub fn avg_current_tenth_ma(&self) -> u16 {
        if self.samples > 0 {
            (self.c_sum_abs / (self.samples as u64)) as u16
        } else {
            0
        }
    }

    /// Computes load resistance in tenths of ohms: R = (V_mV * 100) / c_tenth.
    pub fn load_resistance_tenth_ohms(&self, v_mv: u16, c_tenth: i16) -> Option<u32> {
        let c_abs = c_tenth.unsigned_abs() as u32;
        if v_mv > 500 && c_abs > 20 { // > 2.0 mA
            Some((v_mv as u32 * 100) / c_abs)
        } else {
            None
        }
    }
}

/// 7-bin logarithmic current distribution histogram for sleep/active power profiling.
pub struct HistogramData {
    pub bins: [u32; 7],
    pub total: u32,
}

impl HistogramData {
    pub const fn new() -> Self {
        Self {
            bins: [0; 7],
            total: 0,
        }
    }

    pub fn clear(&mut self) {
        self.bins = [0; 7];
        self.total = 0;
    }

    pub fn record(&mut self, abs_current_tenth: u32) {
        let bin_idx = match abs_current_tenth {
            0..=9 => 0,       // < 1.0 mA
            10..=49 => 1,     // 1.0 .. 5.0 mA
            50..=199 => 2,    // 5.0 .. 20.0 mA
            200..=499 => 3,   // 20.0 .. 50.0 mA
            500..=1499 => 4,  // 50.0 .. 150.0 mA
            1500..=4999 => 5, // 150.0 .. 500.0 mA
            _ => 6,           // >= 500.0 mA
        };
        self.bins[bin_idx] = self.bins[bin_idx].saturating_add(1);
        self.total = self.total.saturating_add(1);
    }

    /// Returns integer percentage (0..=100) for a given bin.
    pub fn pct(&self, bin_idx: usize) -> u8 {
        if self.total == 0 || bin_idx >= 7 {
            0
        } else {
            let pct = (self.bins[bin_idx] as u64 * 100) / (self.total as u64);
            pct.min(100) as u8
        }
    }
}
