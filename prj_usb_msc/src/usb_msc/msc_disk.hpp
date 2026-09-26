#ifndef MSC_DISK_HPP
#define MSC_DISK_HPP

#include <cstdint>
#include <cstddef>

#include "hal/time.hpp"

struct MscDiskStats {
    uint32_t sectors_read{0};
    uint32_t sectors_written{0};
    uint32_t last_sector{0};
    bool is_active{false};
    hal::time::Instant last_activity{hal::time::Instant::now()};
    uint32_t last_sd_result{0};
    uint32_t last_sd_error_lba{0};
};

extern MscDiskStats g_msc_stats;

bool msc_disk_init();
bool msc_disk_ready();
bool msc_disk_get_capacity(uint32_t &block_count, uint32_t &block_size);
int8_t msc_disk_read(uint8_t *buf, uint32_t sector_addr, uint32_t sector_count);
int8_t msc_disk_write(const uint8_t *buf, uint32_t sector_addr, uint32_t sector_count);

#endif /* MSC_DISK_HPP */
