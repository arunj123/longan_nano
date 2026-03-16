#ifndef DISPLAY_MANAGER_H
#define DISPLAY_MANAGER_H

#include <stdint.h>
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

#endif // DISPLAY_MANAGER_H