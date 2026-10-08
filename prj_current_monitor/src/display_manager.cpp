#include "display_manager.h"
#include <cstdio>
#include <cstring>
#include <cstdlib>

#include "lcd.h"

namespace display {

// Compile-time subset fonts
using LabelFont = lcd::font::SubsetFont<
    lcd::font::Font5x7,
    "INA219 MONITORLIVEOKRUNDC VOLTCURRPOWR: "
>;
inline constexpr LabelFont kLabelFont{};

using MetricFont = lcd::font::SubsetFont<
    lcd::font::Font8x16,
    " 0123456789.-+VmAW"
>;
inline constexpr MetricFont kMetricFont{};

DisplayManager& DisplayManager::getInstance() {
    static DisplayManager instance;
    return instance;
}

void DisplayManager::init() {
    lcd_init();
    lcd_clear(lcd::color::Black);

    // 1. Header Bar (Y: 0..14)
    lcd_fill_rect(0, 0, 160, 14, lcd::color::rgb(18, 30, 48)); // Deep Slate Navy
    lcd_fill_rect(0, 14, 160, 1, lcd::color::Cyan);             // Accent rule
    lcd::draw_string(6, 4, "INA219 MONITOR", lcd::color::White, lcd::color::rgb(18, 30, 48), 1, kLabelFont);

    // Live Status Badge (Top-Right)
    lcd_fill_rect(122, 2, 34, 10, lcd::color::rgb(0, 45, 18));
    lcd_rect(122, 2, 34, 10, lcd::color::rgb(0, 180, 60));
    lcd::draw_string(128, 3, "LIVE", lcd::color::Green, lcd::color::rgb(0, 45, 18), 1, kLabelFont);

    // 2. Voltage Row Card Badge (Y: 18..31)
    lcd_fill_rect(4, 18, 34, 14, lcd::color::rgb(0, 40, 15));
    lcd_rect(4, 18, 34, 14, lcd::color::rgb(0, 180, 50));
    lcd::draw_string(8, 21, "VOLT", lcd::color::Green, lcd::color::rgb(0, 40, 15), 1, kLabelFont);

    // Divider 1
    lcd_fill_rect(4, 35, 152, 1, lcd::color::rgb(28, 38, 54));

    // 3. Current Row Card Badge (Y: 39..52)
    lcd_fill_rect(4, 39, 34, 14, lcd::color::rgb(0, 32, 50));
    lcd_rect(4, 39, 34, 14, lcd::color::rgb(0, 160, 220));
    lcd::draw_string(8, 42, "CURR", lcd::color::Cyan, lcd::color::rgb(0, 32, 50), 1, kLabelFont);

    // Divider 2
    lcd_fill_rect(4, 56, 152, 1, lcd::color::rgb(28, 38, 54));

    // 4. Power Row Card Badge (Y: 60..73)
    lcd_fill_rect(4, 60, 34, 14, lcd::color::rgb(50, 28, 0));
    lcd_rect(4, 60, 34, 14, lcd::color::rgb(220, 140, 0));
    lcd::draw_string(8, 63, "POWR", lcd::color::Yellow, lcd::color::rgb(50, 28, 0), 1, kLabelFont);

    // Bottom decorative bar
    lcd_fill_rect(0, 78, 160, 2, lcd::color::rgb(18, 30, 48));
}

void DisplayManager::update(const ina219_data_t& data) {
    char buf[20];

    // 1. Voltage with decimal point (e.g., "  5.042 V  ")
    uint16_t v_mv = data.voltage_mv;
    snprintf(buf, sizeof(buf), "%3u.%03u V  ", v_mv / 1000, v_mv % 1000);
    lcd::draw_string(46, 17, buf, lcd::color::rgb(210, 255, 210), lcd::color::Black, 1, kMetricFont);

    // 2. Current with 0.1 mA resolution (e.g., "   12.4 mA " or "    0.8 mA ")
    int16_t c_tenth = data.current_tenth_ma;
    bool neg = (c_tenth < 0);
    if (neg) c_tenth = -c_tenth;
    int16_t whole_c = c_tenth / 10;
    int16_t frac_c  = c_tenth % 10;

    if (neg) {
        snprintf(buf, sizeof(buf), "-%4d.%1d mA ", whole_c, frac_c);
    } else {
        snprintf(buf, sizeof(buf), " %4d.%1d mA ", whole_c, frac_c);
    }
    lcd::draw_string(46, 38, buf, lcd::color::Cyan, lcd::color::Black, 1, kMetricFont);

    // 3. Power with decimal point (e.g., "   62.5 mW " or "    4.0 mW ")
    uint32_t p_tenth = (static_cast<uint32_t>(v_mv) * static_cast<uint32_t>(c_tenth)) / 1000;
    if (p_tenth < 100000) {
        snprintf(buf, sizeof(buf), " %4lu.%1lu mW ", p_tenth / 10, p_tenth % 10);
    } else {
        snprintf(buf, sizeof(buf), " %4lu.%02lu W  ", (p_tenth / 10) / 1000, ((p_tenth / 10) % 1000) / 10);
    }
    lcd::draw_string(46, 59, buf, lcd::color::Yellow, lcd::color::Black, 1, kMetricFont);
}

} // namespace display