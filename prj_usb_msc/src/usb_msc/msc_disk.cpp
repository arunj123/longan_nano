#include "msc_disk.hpp"
#include "drivers/sdcard.hpp"
#include <cstdio>

using Sd = drivers::sdcard::SdCard<>;

MscDiskStats g_msc_stats{};

bool msc_disk_init() {
    auto res = Sd::init(true);
    if (res == drivers::sdcard::SdResult::Success) {
        printf("[MSC_DISK] SD Card initialized successfully. Sectors: %lu (~%lu MB)\n",
               static_cast<unsigned long>(Sd::sector_count),
               static_cast<unsigned long>(Sd::sector_count / 2048));
        return true;
    }
    printf("[MSC_DISK] SD Card init failed: %s\n", drivers::sdcard::result_to_string(res));
    return false;
}

bool msc_disk_ready() {
    return Sd::is_initialized;
}

bool msc_disk_get_capacity(uint32_t &block_count, uint32_t &block_size) {
    if (!Sd::is_initialized || Sd::sector_count == 0) {
        return false;
    }
    block_count = Sd::sector_count;
    block_size = 512;
    return true;
}

int8_t msc_disk_read(uint8_t *buf, uint32_t sector_addr, uint32_t sector_count) {
    if (!Sd::is_initialized) return -1;
    g_msc_stats.is_active = true;
    g_msc_stats.last_sector = sector_addr;
    g_msc_stats.last_activity = hal::time::Instant::now();

    auto res = Sd::read_sectors(sector_addr, buf, sector_count);
    if (res == drivers::sdcard::SdResult::Success) {
        g_msc_stats.sectors_read += sector_count;
        return 0;
    }
    printf("[MSC_DISK] Read error at sector %lu (count %lu): %s\n",
           static_cast<unsigned long>(sector_addr),
           static_cast<unsigned long>(sector_count),
           drivers::sdcard::result_to_string(res));
    return -1;
}

int8_t msc_disk_write(const uint8_t *buf, uint32_t sector_addr, uint32_t sector_count) {
    if (!Sd::is_initialized) return -1;
    g_msc_stats.is_active = true;
    g_msc_stats.last_sector = sector_addr;
    g_msc_stats.last_activity = hal::time::Instant::now();

    auto res = Sd::write_sectors(sector_addr, buf, sector_count);
    if (res == drivers::sdcard::SdResult::Success) {
        g_msc_stats.sectors_written += sector_count;
        return 0;
    }
    printf("[MSC_DISK] Write error at sector %lu (count %lu): %s\n",
           static_cast<unsigned long>(sector_addr),
           static_cast<unsigned long>(sector_count),
           drivers::sdcard::result_to_string(res));
    return -1;
}
