#pragma once

#include <cstdint>
#include "ina219.h"

namespace display {

class DisplayManager {
public:
    static DisplayManager& getInstance();
    
    void init();
    void update(const ina219_data_t& data);

private:
    DisplayManager() = default;
};

} // namespace display