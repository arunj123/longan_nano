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
/// Tracks bi-directional flow:
/// - charge_out: energy/charge delivered to load (discharging)
/// - charge_in: energy/charge absorbed from source (charging)
///
/// Unit definitions:
/// - Power in tenths of mW (10^-4 W).
/// - Current in tenths of mA (10^-4 A).
/// - 1 uWh = 36,000 (tenth_mw * ms).
/// - 1 uAh = 36,000 (tenth_ma * ms).
pub struct Accumulators {
    pub energy_ticks: u64,     // Total energy throughput (tenth_mw * ms)
    pub charge_ticks: u64,     // Total charge throughput (tenth_ma * ms)
    pub charge_in_ticks: u64,  // Charging into source/battery (negative current)
    pub charge_out_ticks: u64, // Discharging from source/battery (positive current)
}

impl Accumulators {
    pub const fn new() -> Self {
        Self {
            energy_ticks: 0,
            charge_ticks: 0,
            charge_in_ticks: 0,
            charge_out_ticks: 0,
        }
    }

    #[inline]
    pub fn update(&mut self, p_tenth_mw: u32, c_tenth_signed: i16, dt_ms: u32) {
        let abs_c = c_tenth_signed.unsigned_abs() as u64;
        let e_inc = (p_tenth_mw as u64) * (dt_ms as u64);
        let q_inc = abs_c * (dt_ms as u64);

        self.energy_ticks = self.energy_ticks.saturating_add(e_inc);
        self.charge_ticks = self.charge_ticks.saturating_add(q_inc);

        if c_tenth_signed >= 0 {
            self.charge_out_ticks = self.charge_out_ticks.saturating_add(q_inc);
        } else {
            self.charge_in_ticks = self.charge_in_ticks.saturating_add(q_inc);
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.energy_ticks = 0;
        self.charge_ticks = 0;
        self.charge_in_ticks = 0;
        self.charge_out_ticks = 0;
    }

    /// Net charge consumed in mAh: (discharge - charge).
    /// Positive = net discharge, Negative = net recharge.
    #[inline]
    pub fn net_charge_mah(&self) -> i32 {
        let out_mah = (self.charge_out_ticks / 36_000_000) as i32;
        let in_mah = (self.charge_in_ticks / 36_000_000) as i32;
        out_mah.saturating_sub(in_mah)
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
    pub mcu_temp_tenth_c: i16,
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
            mcu_temp_tenth_c: 250,
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

    pub fn update_temp(&mut self, temp_tenth: i16) {
        self.mcu_temp_tenth_c = temp_tenth;
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

/// Selectable battery chemistry and capacity profiles for fuel gauge tracking.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BatteryProfile {
    None,
    Lipo500,      // 500 mAh, nominal 3.7V, cutoff 3.2V
    Lipo1200,     // 1200 mAh, nominal 3.7V, cutoff 3.2V
    LiIon2500,    // 2500 mAh, nominal 3.7V, cutoff 3.0V
    Alkaline1000, // 1000 mAh, nominal 1.5V, cutoff 1.0V
}

impl BatteryProfile {
    pub fn capacity_mah(&self) -> u32 {
        match self {
            BatteryProfile::None => 0,
            BatteryProfile::Lipo500 => 500,
            BatteryProfile::Lipo1200 => 1200,
            BatteryProfile::LiIon2500 => 2500,
            BatteryProfile::Alkaline1000 => 1000,
        }
    }

    pub fn cutoff_mv(&self) -> u16 {
        match self {
            BatteryProfile::None => 0,
            BatteryProfile::Lipo500 => 3200,
            BatteryProfile::Lipo1200 => 3200,
            BatteryProfile::LiIon2500 => 3000,
            BatteryProfile::Alkaline1000 => 1000,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BatteryProfile::None => "NONE",
            BatteryProfile::Lipo500 => "500mAh",
            BatteryProfile::Lipo1200 => "1200mAh",
            BatteryProfile::LiIon2500 => "2500mAh",
            BatteryProfile::Alkaline1000 => "1000mAh",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            BatteryProfile::None => BatteryProfile::Lipo500,
            BatteryProfile::Lipo500 => BatteryProfile::Lipo1200,
            BatteryProfile::Lipo1200 => BatteryProfile::LiIon2500,
            BatteryProfile::LiIon2500 => BatteryProfile::Alkaline1000,
            BatteryProfile::Alkaline1000 => BatteryProfile::None,
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            BatteryProfile::None => 0,
            BatteryProfile::Lipo500 => 1,
            BatteryProfile::Lipo1200 => 2,
            BatteryProfile::LiIon2500 => 3,
            BatteryProfile::Alkaline1000 => 4,
        }
    }

    pub fn from_id(id: u8) -> Self {
        match id {
            1 => BatteryProfile::Lipo500,
            2 => BatteryProfile::Lipo1200,
            3 => BatteryProfile::LiIon2500,
            4 => BatteryProfile::Alkaline1000,
            _ => BatteryProfile::None,
        }
    }
}

/// Dynamic battery fuel gauge and state-of-charge estimator.
#[derive(Copy, Clone, Debug)]
pub struct BatteryState {
    pub profile: BatteryProfile,
    pub soc_pct: u8,
    pub rem_mah: u32,
    pub time_rem_mins: u32,
    pub is_low_voltage: bool,
}

impl BatteryState {
    pub const fn new() -> Self {
        Self {
            profile: BatteryProfile::None,
            soc_pct: 100,
            rem_mah: 0,
            time_rem_mins: 0,
            is_low_voltage: false,
        }
    }

    pub fn update(&mut self, v_mv: u16, accum: &Accumulators, avg_c_tenth: u16) {
        let cap = self.profile.capacity_mah();
        if cap == 0 {
            self.soc_pct = 100;
            self.rem_mah = 0;
            self.time_rem_mins = 0;
            self.is_low_voltage = false;
            return;
        }

        let net_used_mah = accum.net_charge_mah();
        let rem = if net_used_mah <= 0 {
            cap
        } else if (net_used_mah as u32) >= cap {
            0
        } else {
            cap - (net_used_mah as u32)
        };
        self.rem_mah = rem;
        self.soc_pct = (((rem as u64) * 100) / (cap as u64)).min(100) as u8;

        if avg_c_tenth > 10 {
            self.time_rem_mins = ((rem as u64 * 600) / (avg_c_tenth as u64)).min(9999) as u32;
        } else {
            self.time_rem_mins = 9999;
        }

        self.is_low_voltage = v_mv > 500 && v_mv <= self.profile.cutoff_mv();
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

/// Fixed-point Exponential Moving Average filter with 4 fractional bits.
/// Eliminates integer division truncation stalls and guarantees exact steady-state convergence.
#[derive(Copy, Clone, Debug)]
pub struct EmaFilter {
    acc: i32,
    initialized: bool,
}

impl EmaFilter {
    pub const fn new() -> Self {
        Self {
            acc: 0,
            initialized: false,
        }
    }

    pub fn reset(&mut self) {
        self.initialized = false;
    }

    pub fn update(&mut self, raw: i16) -> i16 {
        let x = (raw as i32) << 4;
        if !self.initialized || (x - self.acc).abs() > (100 << 4) {
            // Step jump > 10.0 mA: bypass filter for instant response
            self.acc = x;
            self.initialized = true;
        } else {
            let delta = x - self.acc;
            // Ensure step moves by at least 1 in the direction of delta
            // to completely eliminate integer division truncation stalls
            let step = if delta > 0 {
                core::cmp::max(1, delta >> 2)
            } else if delta < 0 {
                core::cmp::min(-1, delta >> 2)
            } else {
                0
            };
            self.acc += step;
        }
        // Round to nearest integer: (acc + 8) >> 4
        ((self.acc + 8) >> 4) as i16
    }
}

/// 1-second sample aggregator for calculating true arithmetic mean, min, and max
/// across high-frequency conversion samples for MicroSD logging.
#[derive(Copy, Clone, Debug, Default)]
pub struct SampleAggregator {
    pub v_sum: u32,
    pub c_sum: i32,
    pub p_sum: u32,
    pub count: u16,
    pub c_min: i16,
    pub c_max: i16,
    pub last_overflow: bool,
}

impl SampleAggregator {
    pub const fn new() -> Self {
        Self {
            v_sum: 0,
            c_sum: 0,
            p_sum: 0,
            count: 0,
            c_min: i16::MAX,
            c_max: i16::MIN,
            last_overflow: false,
        }
    }

    #[inline]
    pub fn record(&mut self, reading: &InaReading) {
        self.v_sum = self.v_sum.saturating_add(reading.voltage_mv as u32);
        self.c_sum = self.c_sum.saturating_add(reading.current_tenth_ma as i32);
        self.p_sum = self.p_sum.saturating_add(reading.power_tenth_mw);
        self.count = self.count.saturating_add(1);
        if reading.current_tenth_ma < self.c_min {
            self.c_min = reading.current_tenth_ma;
        }
        if reading.current_tenth_ma > self.c_max {
            self.c_max = reading.current_tenth_ma;
        }
        self.last_overflow = reading.overflow;
    }

    pub fn finish_and_reset(&mut self, fallback: &InaReading) -> InaReading {
        if self.count == 0 {
            return *fallback;
        }
        let avg_v = (self.v_sum / (self.count as u32)) as u16;
        let avg_c = (self.c_sum / (self.count as i32)) as i16;
        let avg_p = self.p_sum / (self.count as u32);

        let res = InaReading {
            voltage_mv: avg_v,
            current_tenth_ma: avg_c,
            power_tenth_mw: avg_p,
            overflow: self.last_overflow,
            is_reverse: avg_c < 0,
        };

        self.v_sum = 0;
        self.c_sum = 0;
        self.p_sum = 0;
        self.count = 0;
        self.c_min = i16::MAX;
        self.c_max = i16::MIN;
        self.last_overflow = false;

        res
    }
}
