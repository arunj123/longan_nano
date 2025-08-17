/*!
    \file    sd_card.cpp
    \brief   Low-level SD card driver logic with Polling and DMA.

    \version 2025-02-10, V1.5.8
*/

#include "sd_card.h"
#include "sd_spi_hal.h"
#include <cstdio>

// Use a namespace for all internal, file-local symbols and functions
namespace {

// Standard integer types
using BYTE  = uint8_t;
using UINT  = uint32_t;
using DWORD = uint32_t;
using WORD  = uint16_t;

// SD command definitions
enum class SdCommand : uint8_t {
    CMD0   = 0, CMD1   = 1, CMD8   = 8, CMD9   = 9, CMD12  = 12,
    CMD16  = 16, CMD17  = 17, CMD18  = 18, CMD24  = 24, CMD25  = 25,
    CMD55  = 55, CMD58  = 58, ACMD23 = 0x80 + 23, ACMD41 = 0x80 + 41
};

// Card type flags
constexpr BYTE CT_MMC   = 0x01;
constexpr BYTE CT_SD1   = 0x02;
constexpr BYTE CT_SD2   = 0x04;
constexpr BYTE CT_SDC   = (CT_SD1 | CT_SD2);
constexpr BYTE CT_BLOCK = 0x08;

// Module-level variables
volatile DSTATUS Stat = STA_NOINIT;
BYTE CardType;

// --- SD Card Protocol Helper Functions (Internal to this file) ---

int wait_ready(UINT wt) {
    hal_timer_start(wt);
    do {
        if (hal_spi_xchg(0xFF) == 0xFF) return 1;
    } while (!hal_timer_is_expired());
    return 0;
}

void deselect(void) {
    hal_cs_high();
    hal_spi_xchg(0xFF);
}

static int _select(void) {
    hal_cs_low();
    // *** THE FIX: Call the public HAL function ***
    hal_spi_flush_fifo(); 

    if (wait_ready(500)) return 1;
    
    deselect();
    return 0;
}

int rcvr_datablock_polling(BYTE *buff, UINT btr) {
    BYTE token;
    hal_timer_start(200);
    do {
        token = hal_spi_xchg(0xFF);
    } while ((token == 0xFF) && !hal_timer_is_expired());

    if (token != 0xFE) return 0;

    hal_spi_read_polling(buff, btr);
    hal_spi_xchg(0xFF); hal_spi_xchg(0xFF); // Discard CRC
    return 1;
}

int xmit_datablock_polling(const BYTE *buff, BYTE token) {
    BYTE resp;
    if (!wait_ready(500)) return 0;
    hal_spi_xchg(token);
    if (token != 0xFD) {
        for(UINT i = 0; i < 512; i++) hal_spi_xchg(buff[i]);
        hal_spi_xchg(0xFF); hal_spi_xchg(0xFF);
        resp = hal_spi_xchg(0xFF);
        if ((resp & 0x1F) != 0x05) return 0;
    }
    return 1;
}

int rcvr_datablock_dma(BYTE *buff, UINT btr) {
    BYTE token;
    hal_timer_start(200);
    do {
        token = hal_spi_xchg(0xFF);
    } while ((token == 0xFF) && !hal_timer_is_expired());
    if (token != 0xFE) return 0;

    hal_spi_dma_read(buff, btr);
    hal_spi_xchg(0xFF); hal_spi_xchg(0xFF);
    return 1;
}

int xmit_datablock_dma(const BYTE *buff, BYTE token, UINT btr) {
    BYTE resp;
    if (!wait_ready(500)) return 0;
    hal_spi_xchg(token);
    if (token != 0xFD) {
        hal_spi_dma_write(buff, btr);
        hal_spi_xchg(0xFF); hal_spi_xchg(0xFF);
        resp = hal_spi_xchg(0xFF);
        if ((resp & 0x1F) != 0x05) return 0;
    }
    return 1;
}

BYTE send_cmd(SdCommand cmd, DWORD arg) {
    BYTE n, res;
    if (static_cast<uint8_t>(cmd) & 0x80) {
        cmd = static_cast<SdCommand>(static_cast<uint8_t>(cmd) & 0x7F);
        res = send_cmd(SdCommand::CMD55, 0);
        if (res > 1) return res;
    }
    if (cmd != SdCommand::CMD12) {
        deselect();
        if (!_select()) return 0xFF;
    }
    hal_spi_xchg(0x40 | static_cast<uint8_t>(cmd));
    hal_spi_xchg((BYTE)(arg >> 24));
    hal_spi_xchg((BYTE)(arg >> 16));
    hal_spi_xchg((BYTE)(arg >> 8));
    hal_spi_xchg((BYTE)arg);
    n = (cmd == SdCommand::CMD0) ? 0x95 : ((cmd == SdCommand::CMD8) ? 0x87 : 0x01);
    hal_spi_xchg(n);
    if (cmd == SdCommand::CMD12) hal_spi_xchg(0xFF);

    // Use a timer for a more robust timeout instead of a fixed loop
    hal_timer_start(500); // 500ms timeout for command response
    do {
        res = hal_spi_xchg(0xFF);
    } while ((res & 0x80) && !hal_timer_is_expired());
    return res;
}

} // End anonymous namespace

extern "C" {

DSTATUS sd_init(void) {
    BYTE n, cmd_int, ty, ocr[4];

    hal_spi_init();
    hal_cs_high();

    // *** THE FIX: Add a power-on delay ***
    // Give the card time to power up and stabilize before sending commands.
    hal_delay_ms(10);

    if (Stat & STA_NODISK) return Stat;

    hal_spi_set_speed(SdHalSpeed::LOW);
    for (n = 10; n; n--) hal_spi_xchg(0xFF);

    // ... (rest of the sd_init function is identical) ...
    ty = 0;
    if (send_cmd(SdCommand::CMD0, 0) == 1) {
        hal_timer_start(1000);
        if (send_cmd(SdCommand::CMD8, 0x1AA) == 1) {
            for (n = 0; n < 4; n++) ocr[n] = hal_spi_xchg(0xFF);
            if (ocr[2] == 0x01 && ocr[3] == 0xAA) {
                while (!hal_timer_is_expired() && send_cmd(SdCommand::ACMD41, 1UL << 30));
                if (!hal_timer_is_expired() && send_cmd(SdCommand::CMD58, 0) == 0) {
                    for (n = 0; n < 4; n++) ocr[n] = hal_spi_xchg(0xFF);
                    ty = (ocr[0] & 0x40) ? CT_SD2 | CT_BLOCK : CT_SD2;
                }
            }
        } else {
            if (send_cmd(SdCommand::ACMD41, 0) <= 1) { 
                ty = CT_SD1; cmd_int = static_cast<uint8_t>(SdCommand::ACMD41); 
            } else { 
                ty = CT_MMC; cmd_int = static_cast<uint8_t>(SdCommand::CMD1); 
            }
            while (!hal_timer_is_expired() && send_cmd(static_cast<SdCommand>(cmd_int), 0));
            if (hal_timer_is_expired() || send_cmd(SdCommand::CMD16, 512) != 0) ty = 0;
        }
    }
    CardType = ty;
    deselect();

    if (ty) {
        printf("[INFO] sd_init: Card type detected: 0x%02X. Initialization successful.\n", ty);
        hal_spi_set_speed(SdHalSpeed::HIGH);
        Stat &= static_cast<DSTATUS>(~STA_NOINIT);
    } else {
        printf("[ERROR] sd_init: Card initialization failed.\n");
        Stat = STA_NOINIT;
    }
    return Stat;
}

DSTATUS sd_status(void) {
    return Stat;
}

DRESULT sd_read_blocks(uint8_t *buff, uint32_t sector, uint32_t count) {
    if (!count) return RES_PARERR;
    if (Stat & STA_NOINIT) return RES_NOTRDY;
    if (!(CardType & CT_BLOCK)) sector *= 512;

    if (count == 1) {
        if ((send_cmd(SdCommand::CMD17, sector) == 0) && rcvr_datablock_polling(buff, 512)) {
            count = 0;
        }
    } else {
        if (send_cmd(SdCommand::CMD18, sector) == 0) {
            do {
                if (!rcvr_datablock_polling(buff, 512)) break;
                buff += 512;
            } while (--count);
            send_cmd(SdCommand::CMD12, 0);
        }
    }
    deselect();
    return count ? RES_ERROR : RES_OK;
}

DRESULT sd_write_blocks(const uint8_t *buff, uint32_t sector, uint32_t count) {
    if (!count) return RES_PARERR;
    if (Stat & STA_NOINIT) return RES_NOTRDY;
    if (Stat & STA_PROTECT) return RES_WRPRT;
    if (!(CardType & CT_BLOCK)) sector *= 512;

    if (count == 1) {
        if ((send_cmd(SdCommand::CMD24, sector) == 0) && xmit_datablock_polling(buff, 0xFE)) {
            count = 0;
        }
    } else {
        if (CardType & CT_SDC) send_cmd(SdCommand::ACMD23, count);
        if (send_cmd(SdCommand::CMD25, sector) == 0) {
            do {
                if (!xmit_datablock_polling(buff, 0xFC)) break;
                buff += 512;
            } while (--count);
            if (!xmit_datablock_polling(nullptr, 0xFD)) count = 1;
        }
    }
    deselect();
    return count ? RES_ERROR : RES_OK;
}

DRESULT sd_read_blocks_dma(uint8_t *buff, uint32_t sector, uint32_t count) {
    if (!count || (Stat & STA_NOINIT)) return RES_NOTRDY;
    if (!(CardType & CT_BLOCK)) sector *= 512;

    DRESULT res = RES_ERROR;
    SdCommand cmd = (count > 1) ? SdCommand::CMD18 : SdCommand::CMD17;

    if (send_cmd(cmd, sector) == 0) {
        if(rcvr_datablock_dma(buff, 512 * count)) {
            res = RES_OK;
        }
    }

    if (count > 1) {
        send_cmd(SdCommand::CMD12, 0);
    }
    
    deselect();
    return res;
}

DRESULT sd_write_blocks_dma(const uint8_t *buff, uint32_t sector, uint32_t count) {
    if (!count || (Stat & STA_NOINIT)) return RES_NOTRDY;
    if (Stat & STA_PROTECT) return RES_WRPRT;
    if (!(CardType & CT_BLOCK)) sector *= 512;

    if (count == 1) {
        if ((send_cmd(SdCommand::CMD24, sector) == 0) && xmit_datablock_dma(buff, 0xFE, 512)) {
            count = 0;
        }
    } else {
        if (CardType & CT_SDC) send_cmd(SdCommand::ACMD23, count);
        if (send_cmd(SdCommand::CMD25, sector) == 0) {
            do {
                if (!xmit_datablock_dma(buff, 0xFC, 512)) break;
                buff += 512;
            } while (--count);
            if (!xmit_datablock_polling(nullptr, 0xFD)) count = 1; // Send stop token with polling
        }
    }

    deselect();
    return count ? RES_ERROR : RES_OK;
}

DRESULT sd_ioctl(uint8_t cmd, void *buff) {
    DRESULT res = RES_ERROR;
    BYTE n, csd[16];
    DWORD csize;

    if (Stat & STA_NOINIT) return RES_NOTRDY;

    switch (cmd) {
        case CTRL_SYNC:
            if (_select()) {
                deselect();
                res = RES_OK;
            }
            break;

        case GET_SECTOR_COUNT:
            if ((send_cmd(SdCommand::CMD9, 0) == 0) && rcvr_datablock_polling(csd, 16)) {
                if ((csd[0] >> 6) == 1) {
                    csize = csd[9] + ((DWORD)csd[8] << 8) + ((DWORD)(csd[7] & 63) << 16) + 1;
                    *(DWORD*)buff = csize << 10;
                } else {
                    n = static_cast<BYTE>((csd[5] & 15) + ((csd[10] & 128) >> 7) + ((csd[9] & 3) << 1) + 2);
                    csize = static_cast<DWORD>((csd[8] >> 6) + ((WORD)csd[7] << 2) + ((WORD)(csd[6] & 3) << 10) + 1);
                    *(DWORD*)buff = csize << (n - 9);
                }
                res = RES_OK;
            }
            break;

        case GET_SECTOR_SIZE:
            *(WORD*)buff = 512;
            res = RES_OK;
            break;

        case GET_BLOCK_SIZE:
            if (CardType & CT_SD2) {
                *(DWORD*)buff = 128;
                res = RES_OK;
            } else {
                if ((send_cmd(SdCommand::CMD9, 0) == 0) && rcvr_datablock_polling(csd, 16)) {
                    if (CardType & CT_SD1) {
                        *(DWORD*)buff = static_cast<DWORD>(((csd[10] & 63) << 1) + ((WORD)(csd[11] & 128) >> 7) + 1) << ((csd[13] >> 6) - 1);
                    } else {
                        *(DWORD*)buff = static_cast<DWORD>(((WORD)((csd[10] & 124) >> 2) + 1) * (((csd[11] & 3) << 3) + ((csd[11] & 224) >> 5) + 1));
                    }
                    res = RES_OK;
                }
            }
            break;

        default:
            res = RES_PARERR;
    }

    deselect();
    return res;
}

} // End of extern "C"