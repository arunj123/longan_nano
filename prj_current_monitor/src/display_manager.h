#pragma once

#include <cstdint>
#include "ina219.h"

namespace display {

enum class PlotMode : uint8_t {
    Current = 0,
    Power   = 1,
};

class DisplayManager {
public:
    static DisplayManager& getInstance();
    
    void init();
    void update(const ina219_data_t& data, bool sensor_ok = true);
    void toggle_plot_mode();
    void reset_energy();

    [[nodiscard]] PlotMode plot_mode() const noexcept { return m_plot_mode; }
    [[nodiscard]] uint32_t elapsed_seconds() const noexcept { return m_tick_count / 10; }

private:
    DisplayManager() = default;

    void redraw_grid();
    void update_scale(int32_t val_tenth);

    // Energy & Charge Accumulation (64-bit integer, zero overflow)
    uint64_t m_mw_ticks{0};        // Accumulates (0.1 mW) * 0.1s ticks
    uint64_t m_tenth_ma_ticks{0};  // Accumulates (0.1 mA) * 0.1s ticks
    uint32_t m_tick_count{0};

    PlotMode m_plot_mode{PlotMode::Current};
    uint8_t  m_scale_tier{2};      // Default 50 mA (500 tenths)
    int32_t  m_peak_in_sweep{0};

    // Oscilloscope sweep position (X: 22..158)
    uint8_t  m_sweep_x{22};
    uint8_t  m_prev_y{76};
};

} // namespace display