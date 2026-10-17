use core::fmt::Write;
use gd32vf103_hal::usb::msc::MscStats;
use longan_nano_bsp::lcd::{color as lcd_color, FONT_5X7, Lcd, LCD_WIDTH};

/// Stack-allocated fixed-capacity string buffer for no_std formatting.
#[derive(Copy, Clone)]
pub struct StrBuf<const N: usize> {
    pub buf: [u8; N],
    pub len: usize,
}

impl<const N: usize> StrBuf<N> {
    pub const fn new() -> Self {
        Self { buf: [0; N], len: 0 }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn pad_to(&mut self, target_len: usize) {
        let max_target = core::cmp::min(target_len, N);
        while self.len < max_target {
            self.buf[self.len] = b' ';
            self.len += 1;
        }
    }
}

impl<const N: usize> Default for StrBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for StrBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remain = N.saturating_sub(self.len);
        let copy_len = core::cmp::min(bytes.len(), remain);
        self.buf[self.len..self.len + copy_len].copy_from_slice(&bytes[..copy_len]);
        self.len += copy_len;
        Ok(())
    }
}

/// Tracks cached field text and returns true only when text content has changed.
#[derive(Copy, Clone)]
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

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
    }

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

impl<const N: usize> Default for DirtyField<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats the last LBA accessed as decimal and hex: "LBA: <dec>  (0x<HEX>)  " (26 chars)
pub fn fmt_lba(lba: u32, buf: &mut StrBuf<32>) {
    buf.clear();
    let _ = write!(buf, "LBA: {:<7} (0x{:08X})", lba, lba);
    buf.pad_to(26);
}

/// Formats sector count and human-readable capacity: "RD : <cnt> (<sz>)  " (26 chars)
pub fn fmt_sectors(label: &str, sectors: u32, buf: &mut StrBuf<32>) {
    buf.clear();
    if sectors < 2048 {
        let kb = sectors / 2;
        let _ = write!(buf, "{}: {:<6} ({} KB)", label, sectors, kb);
    } else {
        let val_tenth_mb = (sectors as u64 * 10) / 2048;
        let whole_mb = (val_tenth_mb / 10) as u32;
        let dec_mb = (val_tenth_mb % 10) as u32;
        let _ = write!(buf, "{}: {:<6} ({}.{} MB)", label, sectors, whole_mb, dec_mb);
    }
    buf.pad_to(26);
}

/// Formats cache hit rate percentage and hit/miss counts: "HIT: <pct>%  (H:<h> M:<m>) " (26 chars)
pub fn fmt_cache_hit(hits: u32, misses: u32, buf: &mut StrBuf<32>) {
    buf.clear();
    let total = hits.saturating_add(misses);
    if total == 0 {
        let _ = write!(buf, "HIT: --.-%   (H:0 M:0)");
    } else {
        let permille = (hits as u64 * 1000) / total as u64;
        let whole = (permille / 10) as u32;
        let dec = (permille % 10) as u32;
        let _ = write!(buf, "HIT: {:>3}.{}%  (H:{} M:{})", whole, dec, hits, misses);
    }
    buf.pad_to(26);
}

/// Formats real-time throughput: "SPD: <num> kB/s" or "SPD: <num> MB/s" (26 chars)
pub fn fmt_speed(speed_kbs: u32, buf: &mut StrBuf<32>) {
    buf.clear();
    if speed_kbs == 0 {
        let _ = write!(buf, "SPD: 0 kB/s");
    } else if speed_kbs < 1000 {
        let _ = write!(buf, "SPD: {} kB/s", speed_kbs);
    } else {
        let val_tenth_mbs = (speed_kbs as u64 * 10) / 1000;
        let whole = (val_tenth_mbs / 10) as u32;
        let dec = (val_tenth_mbs % 10) as u32;
        let _ = write!(buf, "SPD: {}.{} MB/s", whole, dec);
    }
    buf.pad_to(26);
}

/// Formats SD card capacity and USB state: "SD: <MB> MB  USB: <state> " (26 chars)
pub fn fmt_sd_usb_status(sd_ok: bool, cap_mb: u32, is_configured: bool, buf: &mut StrBuf<32>) {
    buf.clear();
    let usb_str = if is_configured { "READY" } else { "WAIT" };
    if sd_ok {
        let _ = write!(buf, "SD: {} MB  USB: {}", cap_mb, usb_str);
    } else {
        let _ = write!(buf, "SD: NONE   USB: {}", usb_str);
    }
    buf.pad_to(26);
}

/// Returns header status badge string and color based on connection and activity.
pub fn fmt_header_badge(is_configured: bool, is_active: bool) -> (&'static str, u16) {
    if !is_configured {
        ("[DISC]", lcd_color::RED)
    } else if is_active {
        ("[BUSY]", lcd_color::YELLOW)
    } else {
        ("[IDLE]", lcd_color::GREEN)
    }
}

/// Returns write-protect badge string and color: `"[RO]"` in Yellow vs `"[RW]"` in Green.
pub fn fmt_wp_badge(write_protected: bool) -> (&'static str, u16) {
    if write_protected {
        ("[RO]", lcd_color::YELLOW)
    } else {
        ("[RW]", lcd_color::GREEN)
    }
}

/// Calculates integer throughput in kB/s (1 kB = 1024 bytes) given sector delta and millisecond interval.
#[inline]
pub fn calculate_speed_kbs(delta_sectors: u32, dt_ms: u32) -> u32 {
    if dt_ms == 0 || delta_sectors == 0 {
        0
    } else {
        ((delta_sectors as u64 * 500) / dt_ms as u64) as u32
    }
}

/// Live LCD Telemetry Dashboard for USB Mass Storage.
pub struct TelemetryDashboard {
    pub sd_ok: bool,
    pub cap_mb: u32,
    pub last_speed_calc_time: u32,
    pub last_draw_time: u32,
    pub last_activity_time: u32,
    pub prev_sectors_read: u32,
    pub prev_sectors_written: u32,
    pub current_speed_kbs: u32,
    pub is_active_latch: bool,

    field_wp: DirtyField<8>,
    field_badge: DirtyField<8>,
    field_status: DirtyField<32>,
    field_lba: DirtyField<32>,
    field_read: DirtyField<32>,
    field_write: DirtyField<32>,
    field_hit: DirtyField<32>,
    field_speed: DirtyField<32>,
}

impl TelemetryDashboard {
    pub fn new(sd_ok: bool, cap_mb: u32) -> Self {
        Self {
            sd_ok,
            cap_mb,
            last_speed_calc_time: 0,
            last_draw_time: 0,
            last_activity_time: 0,
            prev_sectors_read: 0,
            prev_sectors_written: 0,
            current_speed_kbs: 0,
            is_active_latch: false,

            field_wp: DirtyField::new(),
            field_badge: DirtyField::new(),
            field_status: DirtyField::new(),
            field_lba: DirtyField::new(),
            field_read: DirtyField::new(),
            field_write: DirtyField::new(),
            field_hit: DirtyField::new(),
            field_speed: DirtyField::new(),
        }
    }

    /// Draws the static dashboard frame, header bar, and divider line.
    pub fn draw_layout(&mut self, lcd: &mut Lcd) {
        lcd.clear(lcd_color::BLACK);
        // Header bar: Y: 0..12
        lcd.fill_rect(0, 0, LCD_WIDTH, 13, lcd_color::DARK_NAVY);
        lcd.draw_string(4, 2, "USB MSC", &FONT_5X7, lcd_color::CYAN, lcd_color::DARK_NAVY);

        // Divider line: Y: 13
        lcd.fill_rect(0, 13, LCD_WIDTH, 1, lcd_color::GRAY);

        // Invalidate all fields to force complete initial draw on next update
        self.field_wp.invalidate();
        self.field_badge.invalidate();
        self.field_status.invalidate();
        self.field_lba.invalidate();
        self.field_read.invalidate();
        self.field_write.invalidate();
        self.field_hit.invalidate();
        self.field_speed.invalidate();
    }

    /// Latches active transfer state and updates activity timestamp.
    #[inline]
    pub fn mark_active(&mut self, now: u32) {
        self.is_active_latch = true;
        self.last_activity_time = now;
    }

    /// Dynamically updates SD card presence and capacity on hot-swap events.
    pub fn update_sd_status(&mut self, sd_ok: bool, cap_mb: u32) {
        if self.sd_ok != sd_ok || self.cap_mb != cap_mb {
            self.sd_ok = sd_ok;
            self.cap_mb = cap_mb;
            self.field_status.invalidate();
        }
    }

    /// Updates dirty fields on the LCD display.
    /// MUST ONLY BE INVOKED WHEN THE USB BUS IS IDLE (`board.usb_msc.is_idle()`).
    pub fn update(&mut self, lcd: &mut Lcd, stats: &MscStats, configured: bool, write_protected: bool, now: u32) {
        // Enforce rate limiting: update LCD at most every 50 ms
        let dt_draw = now.wrapping_sub(self.last_draw_time);
        if dt_draw < 50 && self.last_draw_time != 0 {
            return;
        }

        // Enforce settle guard: wait at least 20 ms after wire activity before drawing
        if self.is_active_latch && now.wrapping_sub(self.last_activity_time) < 20 {
            return;
        }

        // Active latch timeout: expire after 300 ms of idle
        let is_active = self.is_active_latch && now.wrapping_sub(self.last_activity_time) < 300;
        if !is_active {
            self.is_active_latch = false;
        }

        // Throughput computation window: evaluate speed every >= 300 ms
        let dt_speed = now.wrapping_sub(self.last_speed_calc_time);
        if dt_speed >= 300 || self.last_speed_calc_time == 0 {
            let delta_read = stats.sectors_read.saturating_sub(self.prev_sectors_read);
            let delta_write = stats.sectors_written.saturating_sub(self.prev_sectors_written);
            let delta_sectors = delta_read.saturating_add(delta_write);

            if delta_sectors > 0 && dt_speed > 0 {
                self.current_speed_kbs = calculate_speed_kbs(delta_sectors, dt_speed);
            } else if now.wrapping_sub(self.last_activity_time) >= 1200 {
                self.current_speed_kbs = 0;
            }

            self.prev_sectors_read = stats.sectors_read;
            self.prev_sectors_written = stats.sectors_written;
            self.last_speed_calc_time = now;
        }

        let mut buf = StrBuf::<32>::new();

        // 1a. WP Badge: X: 90, Y: 2
        let (wp_str, wp_color) = fmt_wp_badge(write_protected);
        if self.field_wp.update(wp_str) {
            lcd.draw_string(90, 2, self.field_wp.as_str(), &FONT_5X7, wp_color, lcd_color::DARK_NAVY);
        }

        // 1b. Header Status Badge: X: 118, Y: 2
        let (badge_str, badge_color) = fmt_header_badge(configured, is_active);
        if self.field_badge.update(badge_str) {
            lcd.draw_string(118, 2, self.field_badge.as_str(), &FONT_5X7, badge_color, lcd_color::DARK_NAVY);
        }

        // 2. SD & USB Status: X: 2, Y: 15
        fmt_sd_usb_status(self.sd_ok, self.cap_mb, configured, &mut buf);
        if self.field_status.update(buf.as_str()) {
            lcd.draw_string(2, 15, self.field_status.as_str(), &FONT_5X7, lcd_color::WHITE, lcd_color::BLACK);
        }

        // 3. Last LBA: X: 2, Y: 26
        fmt_lba(stats.last_lba, &mut buf);
        if self.field_lba.update(buf.as_str()) {
            lcd.draw_string(2, 26, self.field_lba.as_str(), &FONT_5X7, lcd_color::YELLOW, lcd_color::BLACK);
        }

        // 4. Sectors Read: X: 2, Y: 37
        fmt_sectors("RD ", stats.sectors_read, &mut buf);
        if self.field_read.update(buf.as_str()) {
            lcd.draw_string(2, 37, self.field_read.as_str(), &FONT_5X7, lcd_color::CYAN, lcd_color::BLACK);
        }

        // 5. Sectors Written: X: 2, Y: 48
        fmt_sectors("WR ", stats.sectors_written, &mut buf);
        if self.field_write.update(buf.as_str()) {
            lcd.draw_string(2, 48, self.field_write.as_str(), &FONT_5X7, lcd_color::MAGENTA, lcd_color::BLACK);
        }

        // 6. Cache Hit Rate: X: 2, Y: 59
        fmt_cache_hit(stats.cache_hits, stats.cache_misses, &mut buf);
        if self.field_hit.update(buf.as_str()) {
            lcd.draw_string(2, 59, self.field_hit.as_str(), &FONT_5X7, lcd_color::GREEN, lcd_color::BLACK);
        }

        // 7. Real-Time Speed: X: 2, Y: 70
        fmt_speed(self.current_speed_kbs, &mut buf);
        if self.field_speed.update(buf.as_str()) {
            let spd_color = if self.current_speed_kbs > 0 { lcd_color::GREEN } else { lcd_color::GRAY };
            lcd.draw_string(2, 70, self.field_speed.as_str(), &FONT_5X7, spd_color, lcd_color::BLACK);
        }

        self.last_draw_time = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_buf_write_and_padding() {
        let mut buf = StrBuf::<32>::new();
        let _ = write!(buf, "Hello {}", 123);
        assert_eq!(buf.as_str(), "Hello 123");
        assert_eq!(buf.len, 9);

        buf.pad_to(16);
        assert_eq!(buf.as_str(), "Hello 123       ");
        assert_eq!(buf.len, 16);
    }

    #[test]
    fn test_dirty_field_detection() {
        let mut field = DirtyField::<32>::new();
        // First update must report true (dirty)
        assert!(field.update("Test 1"));
        assert_eq!(field.as_str(), "Test 1");

        // Same text must report false (clean)
        assert!(!field.update("Test 1"));

        // Changed text must report true (dirty)
        assert!(field.update("Test 2"));
        assert_eq!(field.as_str(), "Test 2");

        // Invalidation must cause next update to report true
        field.invalidate();
        assert!(field.update("Test 2"));
    }

    #[test]
    fn test_fmt_lba_formatting() {
        let mut buf = StrBuf::<32>::new();
        fmt_lba(0, &mut buf);
        assert_eq!(buf.as_str(), "LBA: 0       (0x00000000) ");
        assert_eq!(buf.len, 26);

        fmt_lba(1024, &mut buf);
        assert_eq!(buf.as_str(), "LBA: 1024    (0x00000400) ");
        assert_eq!(buf.len, 26);

        fmt_lba(983040, &mut buf);
        assert_eq!(buf.as_str(), "LBA: 983040  (0x000F0000) ");
        assert_eq!(buf.len, 26);
    }

    #[test]
    fn test_fmt_sectors_kb_and_mb() {
        let mut buf = StrBuf::<32>::new();
        // Zero sectors (< 2048) -> KB
        fmt_sectors("RD ", 0, &mut buf);
        assert_eq!(buf.as_str(), "RD : 0      (0 KB)        ");
        assert_eq!(buf.len, 26);

        // 100 sectors -> 50 KB
        fmt_sectors("WR ", 100, &mut buf);
        assert_eq!(buf.as_str(), "WR : 100    (50 KB)       ");
        assert_eq!(buf.len, 26);

        // 8192 sectors -> 4.0 MB
        fmt_sectors("RD ", 8192, &mut buf);
        assert_eq!(buf.as_str(), "RD : 8192   (4.0 MB)      ");
        assert_eq!(buf.len, 26);

        // 983040 sectors -> 480.0 MB
        fmt_sectors("RD ", 983040, &mut buf);
        assert_eq!(buf.as_str(), "RD : 983040 (480.0 MB)    ");
        assert_eq!(buf.len, 26);
    }

    #[test]
    fn test_fmt_cache_hit_formatting() {
        let mut buf = StrBuf::<32>::new();
        // Zero total accesses
        fmt_cache_hit(0, 0, &mut buf);
        assert_eq!(buf.as_str(), "HIT: --.-%   (H:0 M:0)    ");
        assert_eq!(buf.len, 26);

        // 100.0% hits
        fmt_cache_hit(16, 0, &mut buf);
        assert_eq!(buf.as_str(), "HIT: 100.0%  (H:16 M:0)   ");
        assert_eq!(buf.len, 26);

        // 50.0% hits
        fmt_cache_hit(10, 10, &mut buf);
        assert_eq!(buf.as_str(), "HIT:  50.0%  (H:10 M:10)  ");
        assert_eq!(buf.len, 26);

        // 98.4% hits
        fmt_cache_hit(984, 16, &mut buf);
        assert_eq!(buf.as_str(), "HIT:  98.4%  (H:984 M:16) ");
        assert_eq!(buf.len, 26);
    }

    #[test]
    fn test_fmt_speed_formatting() {
        let mut buf = StrBuf::<32>::new();
        fmt_speed(0, &mut buf);
        assert_eq!(buf.as_str(), "SPD: 0 kB/s               ");
        assert_eq!(buf.len, 26);

        fmt_speed(832, &mut buf);
        assert_eq!(buf.as_str(), "SPD: 832 kB/s             ");
        assert_eq!(buf.len, 26);

        fmt_speed(1250, &mut buf);
        assert_eq!(buf.as_str(), "SPD: 1.2 MB/s             ");
        assert_eq!(buf.len, 26);
    }

    #[test]
    fn test_fmt_sd_usb_status() {
        let mut buf = StrBuf::<32>::new();
        fmt_sd_usb_status(true, 480, true, &mut buf);
        assert_eq!(buf.as_str(), "SD: 480 MB  USB: READY    ");
        assert_eq!(buf.len, 26);

        fmt_sd_usb_status(true, 480, false, &mut buf);
        assert_eq!(buf.as_str(), "SD: 480 MB  USB: WAIT     ");
        assert_eq!(buf.len, 26);

        fmt_sd_usb_status(false, 0, false, &mut buf);
        assert_eq!(buf.as_str(), "SD: NONE   USB: WAIT      ");
        assert_eq!(buf.len, 26);
    }

    #[test]
    fn test_fmt_header_badge() {
        let (text, color) = fmt_header_badge(false, false);
        assert_eq!(text, "[DISC]");
        assert_eq!(color, lcd_color::RED);

        let (text, color) = fmt_header_badge(true, true);
        assert_eq!(text, "[BUSY]");
        assert_eq!(color, lcd_color::YELLOW);

        let (text, color) = fmt_header_badge(true, false);
        assert_eq!(text, "[IDLE]");
        assert_eq!(color, lcd_color::GREEN);
    }

    #[test]
    fn test_calculate_speed_kbs() {
        assert_eq!(calculate_speed_kbs(0, 1000), 0);
        assert_eq!(calculate_speed_kbs(100, 0), 0);
        // 1600 sectors in 1000 ms -> 800 kB/s
        assert_eq!(calculate_speed_kbs(1600, 1000), 800);
        // 8192 sectors in 5000 ms -> 819 kB/s
        assert_eq!(calculate_speed_kbs(8192, 5000), 819);
    }

    #[test]
    fn test_dashboard_update_sd_status() {
        let mut dashboard = TelemetryDashboard::new(true, 480);
        assert!(dashboard.sd_ok);
        assert_eq!(dashboard.cap_mb, 480);

        // Update when card is removed
        dashboard.update_sd_status(false, 0);
        assert!(!dashboard.sd_ok);
        assert_eq!(dashboard.cap_mb, 0);

        // Update when card is re-inserted
        dashboard.update_sd_status(true, 480);
        assert!(dashboard.sd_ok);
        assert_eq!(dashboard.cap_mb, 480);
    }

    #[test]
    fn test_fmt_wp_badge() {
        let (text, color) = fmt_wp_badge(false);
        assert_eq!(text, "[RW]");
        assert_eq!(color, lcd_color::GREEN);

        let (text, color) = fmt_wp_badge(true);
        assert_eq!(text, "[RO]");
        assert_eq!(color, lcd_color::YELLOW);
    }
}
