#include "display_manager.h"
#include <cstdio>
#include <cstring>
#include <cstdlib>

#include "lcd.h"

namespace display {

struct ScaleTier {
    int32_t max_val;       // Value in tenths (0.1 mA or 0.1 mW)
    const char* top_lbl;
    const char* mid_lbl;
    const char* badge;
};

static constexpr ScaleTier kCurrentTiers[] = {
    { 100,   "10m",  " 5m", "[10m]" },
    { 250,   "25m",  "12m", "[25m]" },
    { 500,   "50m",  "25m", "[50m]" },
    { 1000,  "100m", "50m", "[100m]"},
    { 2500,  "250m", "125m","[250m]"},
    { 5000,  "500m", "250m","[500m]"},
    { 10000, " 1A",  "500m"," [1A] "},
    { 32000, "3.2A", "1.6A","[3.2A]"}
};
static constexpr size_t kNumCurrentTiers = sizeof(kCurrentTiers) / sizeof(kCurrentTiers[0]);

static constexpr ScaleTier kPowerTiers[] = {
    { 100,   "10m",  " 5m", "[10mW]" },
    { 500,   "50m",  "25m", "[50mW]" },
    { 1000,  "100m", "50m", "[100m]"},
    { 5000,  "500m", "250m","[500m]"},
    { 10000, " 1W",  "500m"," [1W] "},
    { 50000, " 5W",  "2.5W"," [5W] "},
    { 100000,"10W",  " 5W", "[10W] "}
};
static constexpr size_t kNumPowerTiers = sizeof(kPowerTiers) / sizeof(kPowerTiers[0]);

// Graph canvas boundaries
static constexpr uint8_t kGraphXMin = 22;
static constexpr uint8_t kGraphXMax = 158;
static constexpr uint8_t kGraphYTop = 22;
static constexpr uint8_t kGraphYBase = 76;
static constexpr uint8_t kGraphHeight = kGraphYBase - kGraphYTop; // 54 pixels

// Color Palette
static constexpr uint16_t kColBgTop      = lcd::color::rgb(14, 22, 36);  // Deep Slate Navy
static constexpr uint16_t kColDivider    = lcd::color::rgb(35, 60, 90);  // Header divider
static constexpr uint16_t kColGraphBg    = lcd::color::rgb(2, 4, 8);     // Near pitch black
static constexpr uint16_t kColGrid       = lcd::color::rgb(22, 34, 48);  // Subtle grid dot
static constexpr uint16_t kColAxis       = lcd::color::rgb(45, 65, 95);  // Axis border
static constexpr uint16_t kColTraceI     = lcd::color::rgb(0, 240, 255); // Neon Cyan
static constexpr uint16_t kColFillI      = lcd::color::rgb(0, 24, 38);   // Dark Cyan glow fill
static constexpr uint16_t kColTraceP     = lcd::color::rgb(255, 215, 0); // Electric Amber
static constexpr uint16_t kColFillP      = lcd::color::rgb(42, 30, 0);   // Dark Amber glow fill

DisplayManager& DisplayManager::getInstance() {
    static DisplayManager instance;
    return instance;
}

void DisplayManager::init() {
    lcd_init();
    if (m_screen_mode == ScreenMode::Text) {
        draw_text_layout();
    } else {
        lcd_clear(lcd::color::Black);
        lcd_fill_rect(0, 0, 160, 19, kColBgTop);
        lcd_fill_rect(0, 19, 160, 1, kColDivider);
        m_sweep_x = kGraphXMin;
        m_prev_y = kGraphYBase;
        m_scale_tier = 2; // Default 50mA tier
        m_peak_in_sweep = 0;
        redraw_grid();
    }
}

void DisplayManager::redraw_grid() {
    // 1. Left Y-Axis Labels
    const ScaleTier& tier = (m_plot_mode == PlotMode::Current)
        ? kCurrentTiers[m_scale_tier]
        : kPowerTiers[m_scale_tier];

    lcd_fill_rect(0, 20, 21, 60, lcd::color::Black);
    lcd::draw_string(1, 22, tier.top_lbl, lcd::color::rgb(140, 160, 185), lcd::color::Black);
    lcd::draw_string(1, 46, tier.mid_lbl, lcd::color::rgb(110, 130, 155), lcd::color::Black);
    lcd::draw_string(6, 70, " 0",         lcd::color::rgb(90, 110, 135),  lcd::color::Black);

    // 2. Vertical Axis Line
    lcd_fill_rect(21, 20, 1, 60, kColAxis);

    // 3. Clear Graph Canvas & Draw Base/Grid
    lcd_fill_rect(kGraphXMin, 20, (kGraphXMax - kGraphXMin + 1), 60, kColGraphBg);
    lcd_fill_rect(kGraphXMin, kGraphYBase, (kGraphXMax - kGraphXMin + 1), 1, kColAxis);

    // Dotted grid lines across graph area
    for (int x = kGraphXMin; x <= kGraphXMax; x += 4) {
        lcd_setpixel(x, 36, kColGrid); // 75%
        lcd_setpixel(x, 49, kColGrid); // 50%
        lcd_setpixel(x, 63, kColGrid); // 25%
    }
}

void DisplayManager::toggle_screen_mode() {
    m_screen_mode = (m_screen_mode == ScreenMode::Graph) ? ScreenMode::Text : ScreenMode::Graph;
    if (m_screen_mode == ScreenMode::Graph) {
        lcd_clear(lcd::color::Black);
        lcd_fill_rect(0, 0, 160, 19, kColBgTop);
        lcd_fill_rect(0, 19, 160, 1, kColDivider);
        m_sweep_x = kGraphXMin;
        m_prev_y = kGraphYBase;
        m_scale_tier = 2;
        m_peak_in_sweep = 0;
        redraw_grid();
    } else {
        draw_text_layout();
    }
}

void DisplayManager::draw_text_layout() {
    lcd_clear(lcd::color::Black);

    // 1. Top Header Bar (Y: 0..13)
    lcd_fill_rect(0, 0, 160, 13, 0x11E6); // Slate Navy
    lcd_fill_rect(0, 13, 160, 1, 0x1AE7); // Hairline separator
    lcd::draw_string(5, 3, "INA219", lcd::color::White, 0x11E6);

    // 2. Bottom Weather Widget Tiles (Y: 51..78)
    // Left Tile: VOLTAGE
    lcd_fill_rect(3, 51, 76, 27, 0x0944); // Deep Navy Slate tile
    lcd_fill_rect(3, 51, 76, 1, 0x1AE7);  // Top border
    lcd_fill_rect(3, 77, 76, 1, 0x1AE7);  // Bottom border
    lcd_fill_rect(3, 51, 1, 27, 0x1AE7);  // Left border
    lcd_fill_rect(78, 51, 1, 27, 0x1AE7); // Right border
    lcd::draw_string(7, 53, "VOLTAGE", 0x7FE0, 0x0944); // Mint label

    // Right Tile: POWER
    lcd_fill_rect(81, 51, 76, 27, 0x0944); // Deep Navy Slate tile
    lcd_fill_rect(81, 51, 76, 1, 0x1AE7);  // Top border
    lcd_fill_rect(81, 77, 76, 1, 0x1AE7);  // Bottom border
    lcd_fill_rect(81, 51, 1, 27, 0x1AE7);  // Left border
    lcd_fill_rect(156, 51, 1, 27, 0x1AE7); // Right border
    lcd::draw_string(85, 53, "POWER", 0xFEA0, 0x0944); // Amber label

    // Baseline
    lcd_fill_rect(0, 79, 160, 1, lcd::color::Black);
}

void DisplayManager::toggle_plot_mode() {
    m_plot_mode = (m_plot_mode == PlotMode::Current) ? PlotMode::Power : PlotMode::Current;
    m_scale_tier = 2;
    m_peak_in_sweep = 0;
    m_sweep_x = kGraphXMin;
    m_prev_y = kGraphYBase;
    redraw_grid();
}

void DisplayManager::reset_energy() {
    m_mw_ticks = 0;
    m_tenth_ma_ticks = 0;
    m_tick_count = 0;
}

void DisplayManager::update_scale(int32_t val_tenth) {
    const size_t num_tiers = (m_plot_mode == PlotMode::Current) ? kNumCurrentTiers : kNumPowerTiers;
    const auto* tiers = (m_plot_mode == PlotMode::Current) ? kCurrentTiers : kPowerTiers;

    if (val_tenth > m_peak_in_sweep) {
        m_peak_in_sweep = val_tenth;
    }

    // Step UP immediately if value exceeds current ceiling
    if (val_tenth > tiers[m_scale_tier].max_val && (static_cast<size_t>(m_scale_tier) + 1) < num_tiers) {
        while ((static_cast<size_t>(m_scale_tier) + 1) < num_tiers && val_tenth > tiers[m_scale_tier].max_val) {
            m_scale_tier++;
        }
        redraw_grid();
        return;
    }

    // When sweep wraps around to start: evaluate hysteresis downscale
    if (m_sweep_x == kGraphXMin) {
        if (m_scale_tier > 0) {
            int32_t lower_ceiling = tiers[m_scale_tier - 1].max_val;
            // Downscale only if peak was below 65% of lower ceiling
            if (m_peak_in_sweep < (lower_ceiling * 65 / 100)) {
                m_scale_tier--;
                redraw_grid();
            }
        }
        m_peak_in_sweep = 0;
    }
}

void DisplayManager::update(const ina219_data_t& data, bool sensor_ok) {
    m_tick_count++;

    // 1. Data parsing & signed handling with near-zero deadband to eliminate ADC jitter
    int16_t raw_c_tenth = data.current_tenth_ma;
    int16_t c_tenth = (raw_c_tenth >= -1 && raw_c_tenth <= 1) ? 0 : raw_c_tenth;
    bool is_reverse = (c_tenth < 0);
    int16_t abs_c_tenth = is_reverse ? -c_tenth : c_tenth;

    uint16_t v_mv = data.voltage_mv;
    // Power in tenths of mW: (mV * tenths_of_mA) / 1000
    uint32_t p_tenth = (static_cast<uint32_t>(v_mv) * static_cast<uint32_t>(abs_c_tenth)) / 1000;

    // 2. Accumulate Energy (mWh) and Charge (mAh)
    if (sensor_ok && v_mv > 500 && abs_c_tenth > 2) {
        m_mw_ticks += static_cast<uint64_t>(p_tenth / 10);
        m_tenth_ma_ticks += static_cast<uint64_t>(abs_c_tenth);
    }

    if (m_screen_mode == ScreenMode::Text) {
        char buf[24];
        uint32_t total_mwh = static_cast<uint32_t>(m_mw_ticks / 36000ULL);
        uint32_t frac_mwh  = static_cast<uint32_t>((m_mw_ticks % 36000ULL) / 360ULL);
        if (total_mwh == 0) {
            uint32_t uwh = static_cast<uint32_t>((m_mw_ticks * 100ULL) / 360ULL);
            snprintf(buf, sizeof(buf), "E:%3luuWh", uwh);
        } else {
            snprintf(buf, sizeof(buf), "E:%2lu.%02lumWh", total_mwh, frac_mwh);
        }
        lcd::draw_string(54, 3, buf, lcd::color::rgb(255, 180, 50), 0x11E6);

        // Status Badge in text header
        if (!sensor_ok) {
            lcd_fill_rect(122, 2, 34, 10, lcd::color::rgb(50, 0, 0));
            lcd::draw_string(128, 3, "ERR ", lcd::color::Red, lcd::color::rgb(50, 0, 0));
        } else if (is_reverse) {
            lcd_fill_rect(122, 2, 34, 10, lcd::color::rgb(50, 0, 0));
            lcd::draw_string(128, 3, "REV ", lcd::color::Red, lcd::color::rgb(50, 0, 0));
        } else {
            lcd_fill_rect(122, 2, 34, 10, 0x0162);
            lcd::draw_string(128, 3, "LIVE", lcd::color::Green, 0x0162);
        }

        // 1. HERO CURRENT DISPLAY (Font 16x24 hero number, unit small at right edge)
        char c_buf[16];
        int16_t whole_c = abs_c_tenth / 10;
        int16_t frac_c = abs_c_tenth % 10;
        const char* unit_str = "mA";
        if (is_reverse) {
            snprintf(c_buf, sizeof(c_buf), "-%d.%1d", whole_c, frac_c);
        } else {
            snprintf(c_buf, sizeof(c_buf), "%d.%1d", whole_c, frac_c);
        }
        uint16_t curr_col = is_reverse ? lcd::color::rgb(255, 80, 80) : lcd::color::Cyan;

        size_t len = strlen(c_buf);
        int str_w = static_cast<int>(len * 16);
        int x = (str_w < 136) ? (136 - str_w) / 2 : 0;

        if (x > 0) {
            lcd_fill_rect(0, 14, x, 37, lcd::color::Black);
        }
        if (x + str_w < 138) {
            lcd_fill_rect(x + str_w, 14, 138 - (x + str_w), 37, lcd::color::Black);
        }
        lcd_fill_rect(x, 14, str_w, 6, lcd::color::Black);
        lcd_fill_rect(x, 44, str_w, 7, lcd::color::Black);

        // Render smooth 16x24 hero digits
        lcd::draw_string<lcd::font::Font16x24>(x, 20, c_buf, curr_col, lcd::color::Black);

        // Render small unit attached to the right edge (X: 140..156, Y: 27)
        lcd_fill_rect(138, 14, 2, 37, lcd::color::Black);
        lcd_fill_rect(140, 14, 20, 13, lcd::color::Black);
        lcd_fill_rect(140, 43, 20, 8, lcd::color::Black);
        lcd_fill_rect(156, 27, 4, 16, lcd::color::Black);
        lcd::draw_string<lcd::font::Font8x16>(140, 27, unit_str, curr_col, lcd::color::Black);

        // 2. BOTTOM TILE: VOLTAGE (Font 8x16)
        char v_buf[16];
        snprintf(v_buf, sizeof(v_buf), "%2u.%03uV", v_mv / 1000, v_mv % 1000);
        lcd::draw_string<lcd::font::Font8x16>(13, 62, v_buf, 0xD7FA, 0x0944);

        // 3. BOTTOM TILE: POWER (Font 8x16)
        char p_buf[16];
        if (p_tenth < 100000) {
            snprintf(p_buf, sizeof(p_buf), "%4lu.%1lumW", p_tenth / 10, p_tenth % 10);
        } else {
            snprintf(p_buf, sizeof(p_buf), "%3lu.%02lu W", (p_tenth / 10) / 1000, ((p_tenth / 10) % 1000) / 10);
        }
        lcd::draw_string<lcd::font::Font8x16>(91, 62, p_buf, lcd::color::Yellow, 0x0944);
        return;
    }

    // 3. Render Top Line 1 (Instantaneous V, I, P)
    char buf[24];
    // Voltage: "3.32V" (5 chars)
    snprintf(buf, sizeof(buf), "%2u.%02uV ", v_mv / 1000, (v_mv % 1000) / 10);
    lcd::draw_string(2, 1, buf, lcd::color::Green, kColBgTop);

    // Current: " 8.9mA" (7 chars)
    if (is_reverse) {
        snprintf(buf, sizeof(buf), "-%3d.%1dmA", abs_c_tenth / 10, abs_c_tenth % 10);
    } else {
        snprintf(buf, sizeof(buf), " %3d.%1dmA", abs_c_tenth / 10, abs_c_tenth % 10);
    }
    lcd::draw_string(42, 1, buf, lcd::color::Cyan, kColBgTop);

    // Power: " 29.5mW" or " 1.25W " (7 chars)
    if (p_tenth < 100000) {
        snprintf(buf, sizeof(buf), " %3lu.%1lu mW", p_tenth / 10, p_tenth % 10);
    } else {
        snprintf(buf, sizeof(buf), " %3lu.%02lu W", (p_tenth / 10) / 1000, ((p_tenth / 10) % 1000) / 10);
    }
    lcd::draw_string(88, 1, buf, lcd::color::Yellow, kColBgTop);

    // Status Badge (Top-Right): [LIVE], [REV], or [ERR]
    if (!sensor_ok) {
        lcd::draw_string(132, 1, "[ERR] ", lcd::color::Red, kColBgTop);
    } else if (is_reverse) {
        lcd::draw_string(132, 1, "[REV] ", lcd::color::Red, kColBgTop);
    } else {
        lcd::draw_string(132, 1, "[LIVE]", lcd::color::Green, kColBgTop);
    }

    // 4. Render Top Line 2 (Energy Consumed, Charge, Scale Tier)
    uint32_t total_mwh = static_cast<uint32_t>(m_mw_ticks / 36000ULL);
    uint32_t frac_mwh  = static_cast<uint32_t>((m_mw_ticks % 36000ULL) / 360ULL);
    if (total_mwh == 0) {
        uint32_t uwh = static_cast<uint32_t>((m_mw_ticks * 100ULL) / 360ULL);
        snprintf(buf, sizeof(buf), "E:%3lu uWh ", uwh);
    } else {
        snprintf(buf, sizeof(buf), "E:%2lu.%02lumWh", total_mwh, frac_mwh);
    }
    lcd::draw_string(2, 10, buf, lcd::color::rgb(255, 160, 40), kColBgTop);

    uint32_t total_mah = static_cast<uint32_t>(m_tenth_ma_ticks / 360000ULL);
    uint32_t frac_mah  = static_cast<uint32_t>((m_tenth_ma_ticks % 360000ULL) / 3600ULL);
    snprintf(buf, sizeof(buf), "Q:%2lu.%02lumAh", total_mah, frac_mah);
    lcd::draw_string(64, 10, buf, lcd::color::rgb(100, 255, 140), kColBgTop);

    // Current Scale Badge
    const ScaleTier& tier = (m_plot_mode == PlotMode::Current)
        ? kCurrentTiers[m_scale_tier]
        : kPowerTiers[m_scale_tier];
    lcd::draw_string(126, 10, tier.badge, lcd::color::White, kColBgTop);

    // 5. Update Oscilloscope Waveform Sweep
    int32_t plot_val = (m_plot_mode == PlotMode::Current) ? abs_c_tenth : static_cast<int32_t>(p_tenth);
    update_scale(plot_val);

    // Calculate Y on the 54-pixel canvas
    int32_t scale_max = (m_plot_mode == PlotMode::Current)
        ? kCurrentTiers[m_scale_tier].max_val
        : kPowerTiers[m_scale_tier].max_val;

    int32_t y_curr = kGraphYBase - (plot_val * kGraphHeight) / scale_max;
    if (y_curr < kGraphYTop)  y_curr = kGraphYTop;
    if (y_curr > kGraphYBase) y_curr = kGraphYBase;

    uint16_t trace_col = (m_plot_mode == PlotMode::Current) ? kColTraceI : kColTraceP;
    uint16_t fill_col  = (m_plot_mode == PlotMode::Current) ? kColFillI  : kColFillP;

    // Erase ahead (sweep head) to create clean oscilloscope trace clearing
    uint8_t erase_x1 = (m_sweep_x + 1 > kGraphXMax) ? kGraphXMin : (m_sweep_x + 1);
    uint8_t erase_x2 = (m_sweep_x + 2 > kGraphXMax) ? (kGraphXMin + 1) : (m_sweep_x + 2);

    for (uint8_t ex : {erase_x1, erase_x2}) {
        lcd_fill_rect(ex, kGraphYTop, 1, kGraphHeight + 1, kColGraphBg);
        lcd_setpixel(ex, kGraphYBase, kColAxis);
        if (ex % 4 == 0) {
            lcd_setpixel(ex, 36, kColGrid);
            lcd_setpixel(ex, 49, kColGrid);
            lcd_setpixel(ex, 63, kColGrid);
        }
    }

    // Render single column buffer for high-performance DMA transfer
    uint16_t col_buf[56];
    int y_min = (m_prev_y < y_curr) ? m_prev_y : y_curr;
    int y_max = (m_prev_y > y_curr) ? m_prev_y : y_curr;

    for (int y = kGraphYTop; y <= kGraphYBase; ++y) {
        int idx = y - kGraphYTop;
        if (y >= y_min && y <= y_max) {
            col_buf[idx] = trace_col; // Vibrant trace segment
        } else if (y > y_max && y < kGraphYBase) {
            col_buf[idx] = fill_col;  // Subtle glowing area under curve
        } else if (y == kGraphYBase) {
            col_buf[idx] = kColAxis;  // Baseline
        } else if ((y == 36 || y == 49 || y == 63) && (m_sweep_x % 4 == 0)) {
            col_buf[idx] = kColGrid;  // Dotted grid
        } else {
            col_buf[idx] = kColGraphBg;
        }
    }
    lcd_write_u16(m_sweep_x, kGraphYTop, 1, kGraphHeight + 1, col_buf);

    m_prev_y = static_cast<uint8_t>(y_curr);
    m_sweep_x++;
    if (m_sweep_x > kGraphXMax) {
        m_sweep_x = kGraphXMin;
    }
}

} // namespace display