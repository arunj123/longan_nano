#include "fatfs/diskio.h"
#include "drivers/sdcard.hpp"
#include "hal/time.hpp"

using Sd = drivers::sdcard::SdCard<>;

extern "C" {

DSTATUS disk_initialize(BYTE pdrv) {
    if (pdrv != 0) return STA_NOINIT;

    if (Sd::is_initialized) {
        return 0; // Already initialized
    }

    auto res = Sd::init(true);
    if (res == drivers::sdcard::SdResult::Success) {
        return 0;
    }
    return STA_NOINIT;
}

DSTATUS disk_status(BYTE pdrv) {
    if (pdrv != 0) return STA_NOINIT;
    return Sd::is_initialized ? 0 : STA_NOINIT;
}

DRESULT disk_read(BYTE pdrv, BYTE* buff, DWORD sector, UINT count) {
    if (pdrv != 0 || count == 0) return RES_PARERR;
    if (!Sd::is_initialized) return RES_NOTRDY;

    auto res = Sd::read_sectors(sector, buff, count);
    return (res == drivers::sdcard::SdResult::Success) ? RES_OK : RES_ERROR;
}

DRESULT disk_write(BYTE pdrv, const BYTE* buff, DWORD sector, UINT count) {
    if (pdrv != 0 || count == 0) return RES_PARERR;
    if (!Sd::is_initialized) return RES_NOTRDY;

    auto res = Sd::write_sectors(sector, buff, count);
    return (res == drivers::sdcard::SdResult::Success) ? RES_OK : RES_ERROR;
}

DRESULT disk_ioctl(BYTE pdrv, BYTE cmd, void* buff) {
    if (pdrv != 0) return RES_PARERR;
    if (!Sd::is_initialized) return RES_NOTRDY;

    switch (cmd) {
        case CTRL_SYNC:
            return Sd::wait_ready(500) ? RES_OK : RES_ERROR;

        case GET_SECTOR_COUNT:
            if (!buff) return RES_PARERR;
            *reinterpret_cast<DWORD*>(buff) = Sd::sector_count;
            return RES_OK;

        case GET_SECTOR_SIZE:
            if (!buff) return RES_PARERR;
            *reinterpret_cast<WORD*>(buff) = 512;
            return RES_OK;

        case GET_BLOCK_SIZE:
            if (!buff) return RES_PARERR;
            *reinterpret_cast<DWORD*>(buff) = 1; // 1 sector erase block granularity
            return RES_OK;

        default:
            return RES_PARERR;
    }
}

DWORD get_fattime(void) {
    // Return packed FAT timestamp: 2026-09-05 12:00:00
    // Bit 31:25 - Year origin from 1980 (2026 - 1980 = 46)
    // Bit 24:21 - Month (1..12 = 9)
    // Bit 20:16 - Day (1..31 = 5)
    // Bit 15:11 - Hour (0..23 = 12)
    // Bit 10:5  - Minute (0..59 = 0)
    // Bit 4:0   - Second / 2 (0..29 = 0)
    constexpr DWORD year = 2026 - 1980;
    constexpr DWORD month = 9;
    constexpr DWORD day = 5;
    constexpr DWORD hour = 12;
    constexpr DWORD minute = 0;
    constexpr DWORD second = 0;

    return (year << 25) | (month << 21) | (day << 16) | (hour << 11) | (minute << 5) | (second >> 1);
}

} // extern "C"
