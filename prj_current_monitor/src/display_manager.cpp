#include "display_manager.h"
#include <cstdio>
#include <cstring>

#include "lcd.h"

namespace display {

// Modern compile-time font subset for INA219 monitor:
// Only compiles glyphs for the labels, units, and numeric readings into ROM.
using MonitorFont = lcd::font::SubsetFont<
    lcd::font::Font5x7,
    "INA219 MonitorVoltage: Current: Power: 0123456789- mVmW "
>;
inline constexpr MonitorFont kFont{};

DisplayManager& DisplayManager::getInstance() {
    static DisplayManager instance;
    return instance;
}

void DisplayManager::init() {
    lcd_init();
    lcd_clear(color::Black);
    lcd::draw_string(10, 0,  "INA219 Monitor", color::White,  color::Black, 1, kFont);
    lcd::draw_string(10, 20, "Voltage: ",     color::Green,  color::Black, 1, kFont);
    lcd::draw_string(10, 35, "Current: ",     color::Cyan,   color::Black, 1, kFont);
    lcd::draw_string(10, 50, "Power:   ",     color::Yellow, color::Black, 1, kFont);
}

void DisplayManager::update(const ina219_data_t& data) {
    char buf[32];
    
    // Overdrawing with space padding and black background cleanly updates digits
    snprintf(buf, sizeof(buf), "%5u mV ", data.voltage_mv);
    lcd::draw_string(70, 20, buf, color::White, color::Black, 1, kFont);
    
    snprintf(buf, sizeof(buf), "%5d mA ", data.current_ma);
    lcd::draw_string(70, 35, buf, color::White, color::Black, 1, kFont);
    
    snprintf(buf, sizeof(buf), "%5u mW ", data.power_mw);
    lcd::draw_string(70, 50, buf, color::White, color::Black, 1, kFont);
}

} // namespace display