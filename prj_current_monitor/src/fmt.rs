use core::fmt::Write;
use crate::model::Accumulators;

/// Zero-allocation buffer cursor for no_std string formatting.
pub struct BufferCursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufferCursor<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
}

impl<'a> Write for BufferCursor<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        let to_copy = core::cmp::min(bytes.len(), remaining);
        self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.pos += to_copy;
        Ok(())
    }
}

/// Dirty field tracker to prevent unnecessary display re-rendering when text hasn't changed.
pub struct DirtyField<const N: usize> {
    buf: [u8; N],
    len: u8,
}

impl<const N: usize> DirtyField<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    /// Returns `true` if text is new and updates cached contents.
    pub fn update(&mut self, text: &str) -> bool {
        let bytes = text.as_bytes();
        if self.len as usize == bytes.len() && &self.buf[..self.len as usize] == bytes {
            return false;
        }
        let copy_len = core::cmp::min(bytes.len(), N);
        self.buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        self.len = copy_len as u8;
        true
    }

    pub fn invalidate(&mut self) {
        self.len = 0;
    }
}

/// Formats hero numeric string with automatic unit selection:
/// Returns `(number_str, unit_str)`.
pub fn fmt_hero_current<'a>(
    cur: &'a mut BufferCursor,
    c_tenth: i16,
) -> (&'a str, &'static str) {
    let is_rev = c_tenth < 0;
    let abs_c = c_tenth.unsigned_abs() as u32;

    if abs_c >= 10_000 {
        // >= 1.000 A: format in Amps
        let whole_a = abs_c / 10_000;
        let frac_a = (abs_c % 10_000) / 10; // 3 decimal digits
        if is_rev {
            write!(cur, "-{}.{:03}", whole_a, frac_a).ok();
        } else {
            write!(cur, "{}.{:03}", whole_a, frac_a).ok();
        }
        (cur.as_str(), " A")
    } else {
        // < 1000.0 mA: format in mA
        let whole_ma = abs_c / 10;
        let frac_ma = abs_c % 10;
        if is_rev {
            write!(cur, "-{}.{}", whole_ma, frac_ma).ok();
        } else {
            write!(cur, "{}.{}", whole_ma, frac_ma).ok();
        }
        (cur.as_str(), "mA")
    }
}

/// Formats bus voltage: `12.345V`.
pub fn fmt_voltage(cur: &mut BufferCursor, v_mv: u16) {
    write!(cur, "{:>2}.{:03}V", v_mv / 1000, v_mv % 1000).ok();
}

/// Formats power with auto mW / W rollover: ` 120.4mW` or `  1.23 W`.
pub fn fmt_power(cur: &mut BufferCursor, p_tenth_mw: u32) {
    if p_tenth_mw < 100_000 {
        write!(cur, "{:>4}.{}mW", p_tenth_mw / 10, p_tenth_mw % 10).ok();
    } else {
        let whole_w = (p_tenth_mw / 10) / 1000;
        let frac_w = ((p_tenth_mw / 10) % 1000) / 10;
        write!(cur, "{:>3}.{:02} W", whole_w, frac_w).ok();
    }
}

/// Formats accumulated energy with auto uWh -> mWh -> Wh rollover.
pub fn fmt_energy_auto(cur: &mut BufferCursor, accum: &Accumulators) {
    let mwh = accum.energy_mwh();
    if mwh == 0 {
        let uwh = accum.energy_uwh();
        write!(cur, "E:{:>3}uWh", uwh).ok();
    } else if mwh < 10_000 {
        let frac = accum.energy_mwh_frac();
        write!(cur, "E:{:>2}.{:02}mWh", mwh, frac).ok();
    } else {
        let wh = mwh / 1000;
        let frac = (mwh % 1000) / 10;
        write!(cur, "E:{:>2}.{:02} Wh", wh, frac).ok();
    }
}

/// Formats accumulated charge with auto uAh -> mAh -> Ah rollover.
#[allow(dead_code)]
pub fn fmt_charge_auto(cur: &mut BufferCursor, accum: &Accumulators) {
    let mah = accum.charge_mah();
    if mah == 0 {
        let uah = accum.charge_uah();
        write!(cur, "Q:{:>3}uAh", uah).ok();
    } else if mah < 10_000 {
        let frac = accum.charge_mah_frac();
        write!(cur, "Q:{:>2}.{:02}mAh", mah, frac).ok();
    } else {
        let ah = mah / 1000;
        let frac = (mah % 1000) / 10;
        write!(cur, "Q:{:>2}.{:02} Ah", ah, frac).ok();
    }
}

/// Formats estimated load resistance: `12.4R`, ` 470R`, or ` 4.70kR`.
pub fn fmt_resistance(cur: &mut BufferCursor, r_tenth: Option<u32>) {
    if let Some(r_t) = r_tenth {
        if r_t < 1_000 {
            write!(cur, "{:>3}.{}R", r_t / 10, r_t % 10).ok();
        } else if r_t < 100_000 {
            write!(cur, "{:>4}R", r_t / 10).ok();
        } else if r_t < 10_000_000 {
            let r_k = r_t / 10;
            write!(cur, "{:>2}.{:02}kR", r_k / 1000, (r_k % 1000) / 10).ok();
        } else {
            let r_m = r_t / 10_000;
            write!(cur, "{:>2}.{:02}MR", r_m / 100, r_m % 100).ok();
        }
    } else {
        write!(cur, "  --- R").ok();
    }
}

/// Formats elapsed time: `HH:MM:SS`.
pub fn fmt_time(cur: &mut BufferCursor, elapsed_sec: u32) {
    let hrs = elapsed_sec / 3600;
    let mins = (elapsed_sec % 3600) / 60;
    let secs = elapsed_sec % 60;
    write!(cur, "{:02}:{:02}:{:02}", hrs, mins, secs).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_cursor() {
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        write!(cur, "Hello, {}", 123).unwrap();
        assert_eq!(cur.as_str(), "Hello, 123");
        assert_eq!(cur.as_bytes(), b"Hello, 123");

        // Test truncation when exceeding buffer
        let mut small_buf = [0u8; 4];
        let mut cur2 = BufferCursor::new(&mut small_buf);
        write!(cur2, "TooLongString").unwrap();
        assert_eq!(cur2.as_str(), "TooL");
    }

    #[test]
    fn test_dirty_field() {
        let mut df = DirtyField::<16>::new();
        assert!(df.update("12.3V")); // First update is new
        assert!(!df.update("12.3V")); // Identical text is NOT dirty
        assert!(df.update("12.4V")); // Changed text is dirty
        assert!(!df.update("12.4V"));
        df.invalidate();
        assert!(df.update("12.4V")); // Invalidation forces dirty
    }

    #[test]
    fn test_fmt_hero_current() {
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        let (s, unit) = fmt_hero_current(&mut cur, 125);
        assert_eq!(s, "12.5");
        assert_eq!(unit, "mA");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        let (s, unit) = fmt_hero_current(&mut cur, -125);
        assert_eq!(s, "-12.5");
        assert_eq!(unit, "mA");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        let (s, unit) = fmt_hero_current(&mut cur, 15432);
        assert_eq!(s, "1.543");
        assert_eq!(unit, " A");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        let (s, unit) = fmt_hero_current(&mut cur, -25000);
        assert_eq!(s, "-2.500");
        assert_eq!(unit, " A");
    }

    #[test]
    fn test_fmt_voltage() {
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_voltage(&mut cur, 3320);
        assert_eq!(cur.as_str(), " 3.320V");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_voltage(&mut cur, 12050);
        assert_eq!(cur.as_str(), "12.050V");
    }

    #[test]
    fn test_fmt_power() {
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_power(&mut cur, 1234);
        assert_eq!(cur.as_str(), " 123.4mW");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_power(&mut cur, 154321);
        assert_eq!(cur.as_str(), " 15.43 W");
    }

    #[test]
    fn test_fmt_energy_auto() {
        let mut accum = Accumulators::new();
        // 500 uWh
        accum.energy_ticks = 500 * 36_000;
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_energy_auto(&mut cur, &accum);
        assert_eq!(cur.as_str(), "E:500uWh");

        // 12.34 mWh
        accum.energy_ticks = 12 * 36_000_000 + 34 * 360_000;
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_energy_auto(&mut cur, &accum);
        assert_eq!(cur.as_str(), "E:12.34mWh");

        // 15.20 Wh (15,200 mWh)
        accum.energy_ticks = 15_200 * 36_000_000;
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_energy_auto(&mut cur, &accum);
        assert_eq!(cur.as_str(), "E:15.20 Wh");
    }

    #[test]
    fn test_fmt_resistance() {
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_resistance(&mut cur, None);
        assert_eq!(cur.as_str(), "  --- R");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_resistance(&mut cur, Some(124)); // 12.4 ohms
        assert_eq!(cur.as_str(), " 12.4R");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_resistance(&mut cur, Some(4700)); // 470 ohms
        assert_eq!(cur.as_str(), " 470R");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_resistance(&mut cur, Some(47000)); // 4700 ohms
        assert_eq!(cur.as_str(), "4700R");

        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_resistance(&mut cur, Some(470000)); // 47.00 kOhms
        assert_eq!(cur.as_str(), "47.00kR");
    }

    #[test]
    fn test_fmt_time() {
        let mut buf = [0u8; 16];
        let mut cur = BufferCursor::new(&mut buf);
        fmt_time(&mut cur, 3665);
        assert_eq!(cur.as_str(), "01:01:05");
    }
}
