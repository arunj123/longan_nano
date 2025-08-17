/*!
    \file    sd_card.cpp
    \brief   Low-level SD card driver for Longan Nano using polling. (Production Clean)

    \version 2025-02-10, V1.5.3
*/

#include "sd_card.h"
#include "gd32vf103.h"
#include <cstdio>

// Use C-style linkage for ISR
extern "C" void TIMER3_IRQHandler(void);

namespace { // Anonymous namespace for file-local scope

// Define standard integer types for this file
using BYTE  = uint8_t;
using UINT  = uint32_t;
using DWORD = uint32_t;
using WORD  = uint16_t;

// SPI peripheral and pins for Longan Nano SD Card
#define SDCARD_SPI_PORT         SPI1
#define SDCARD_SPI_CLK          RCU_SPI1
#define SDCARD_GPIO_PORT        GPIOB
#define SDCARD_GPIO_CLK         RCU_GPIOB
#define SDCARD_CS_PIN           GPIO_PIN_12
#define SDCARD_SCK_PIN          GPIO_PIN_13
#define SDCARD_MISO_PIN         GPIO_PIN_14
#define SDCARD_MOSI_PIN         GPIO_PIN_15

#define FCLK_SLOW() { SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_256; }
#define FCLK_FAST() { SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_2; }

// Chip Select control
#define CS_HIGH()   gpio_bit_set(SDCARD_GPIO_PORT, SDCARD_CS_PIN)
#define CS_LOW()    gpio_bit_reset(SDCARD_GPIO_PORT, SDCARD_CS_PIN)

// SD command definitions
enum SdCommand : uint8_t {
    CMD0  = 0,    // GO_IDLE_STATE
    CMD1  = 1,    // SEND_OP_COND (MMC)
    CMD8  = 8,    // SEND_IF_COND
    CMD9  = 9,    // SEND_CSD
    CMD12 = 12,   // STOP_TRANSMISSION
    CMD16 = 16,   // SET_BLOCKLEN
    CMD17 = 17,   // READ_SINGLE_BLOCK
    CMD18 = 18,   // READ_MULTIPLE_BLOCK
    ACMD23 = 0x80 + 23, // SET_WR_BLK_ERASE_COUNT (SDC)
    CMD24 = 24,   // WRITE_BLOCK
    CMD25 = 25,   // WRITE_MULTIPLE_BLOCK
    CMD55 = 55,   // APP_CMD
    CMD58 = 58,   // READ_OCR
    ACMD41 = 0x80 + 41 // SEND_OP_COND (SDC)
};

// Card type flags
#define CT_MMC      0x01
#define CT_SD1      0x02
#define CT_SD2      0x04
#define CT_SDC      (CT_SD1 | CT_SD2)
#define CT_BLOCK    0x08

// Module-level variables
volatile DSTATUS Stat = STA_NOINIT;
volatile UINT Timer1, Timer2;
BYTE CardType;

static BYTE xchg_spi(BYTE data) {
    while (RESET == spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TBE));
    spi_i2s_data_transmit(SDCARD_SPI_PORT, data);
    while (RESET == spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RBNE));
    return static_cast<BYTE>(spi_i2s_data_receive(SDCARD_SPI_PORT));
}

static void rcvr_spi_polling(BYTE *buff, UINT btr) {
    do {
        *buff++ = xchg_spi(0xFF);
    } while (--btr);
}

static void xmit_spi_polling(const BYTE *buff, UINT btr) {
    do {
        xchg_spi(*buff++);
    } while (--btr);
}

static void timer3_config(void) {
    timer_parameter_struct timer_initpara;
    rcu_periph_clock_enable(RCU_TIMER3);
    eclic_irq_enable(TIMER3_IRQn, 2, 0);
    timer_deinit(TIMER3);
    timer_initpara.prescaler         = static_cast<uint16_t>((rcu_clock_freq_get(CK_APB1) / 1000U) - 1U);
    timer_initpara.period            = 0;
    timer_initpara.alignedmode       = TIMER_COUNTER_EDGE;
    timer_initpara.counterdirection  = TIMER_COUNTER_UP;
    timer_initpara.clockdivision     = TIMER_CKDIV_DIV1;
    timer_initpara.repetitioncounter = 0;
    timer_init(TIMER3, &timer_initpara);
    timer_interrupt_enable(TIMER3, TIMER_INT_UP);
    timer_enable(TIMER3);
}

static int wait_ready(UINT wt) {
    BYTE d;
    Timer2 = wt;
    do {
        d = xchg_spi(0xFF);
    } while (d != 0xFF && Timer2);
    return (d == 0xFF) ? 1 : 0;
}

static void deselect(void) {
    CS_HIGH();
    xchg_spi(0xFF);
}

static int _select(void) {
    CS_LOW();
    if (wait_ready(500)) return 1;
    deselect();
    return 0;
}

static int rcvr_datablock(BYTE *buff, UINT btr) {
    BYTE token;
    Timer1 = 200;
    do {
        token = xchg_spi(0xFF);
    } while ((token == 0xFF) && Timer1);

    if (token != 0xFE) return 0;

    rcvr_spi_polling(buff, btr);
    xchg_spi(0xFF); xchg_spi(0xFF);
    return 1;
}

static int xmit_datablock(const BYTE *buff, BYTE token) {
    BYTE resp;
    if (!wait_ready(500)) return 0;
    xchg_spi(token);
    if (token != 0xFD) {
        xmit_spi_polling(buff, 512);
        xchg_spi(0xFF); xchg_spi(0xFF);
        resp = xchg_spi(0xFF);
        if ((resp & 0x1F) != 0x05) return 0;
    }
    return 1;
}

static BYTE send_cmd(BYTE cmd, DWORD arg) {
    BYTE n, res;
    if (cmd & 0x80) {
        cmd &= 0x7F;
        res = send_cmd(CMD55, 0);
        if (res > 1) return res;
    }
    if (cmd != CMD12) {
        deselect();
        if (!_select()) return 0xFF;
    }
    xchg_spi(0x40 | cmd);
    xchg_spi((BYTE)(arg >> 24)); xchg_spi((BYTE)(arg >> 16));
    xchg_spi((BYTE)(arg >> 8)); xchg_spi((BYTE)arg);
    n = (cmd == CMD0) ? 0x95 : ((cmd == CMD8) ? 0x87 : 0x01);
    xchg_spi(n);
    if (cmd == CMD12) xchg_spi(0xFF);
    n = 10;
    do {
        res = xchg_spi(0xFF);
    } while ((res & 0x80) && --n);
    return res;
}

} // End of anonymous namespace

extern "C" void TIMER3_IRQHandler(void) {
    if (RESET != timer_interrupt_flag_get(TIMER3, TIMER_INT_UP)) {
        timer_interrupt_flag_clear(TIMER3, TIMER_INT_UP);
        UINT t1 = Timer1; if (t1) Timer1 = --t1;
        UINT t2 = Timer2; if (t2) Timer2 = --t2;
    }
}

// =================================================================================
// --- Public API Implementation ---
// =================================================================================

DSTATUS sd_init(void) {
    BYTE n, cmd, ty, ocr[4];
    rcu_periph_clock_enable(SDCARD_GPIO_CLK);
    rcu_periph_clock_enable(SDCARD_SPI_CLK);
    gpio_init(SDCARD_GPIO_PORT, GPIO_MODE_AF_PP, GPIO_OSPEED_50MHZ, SDCARD_SCK_PIN | SDCARD_MOSI_PIN);
    gpio_init(SDCARD_GPIO_PORT, GPIO_MODE_IN_FLOATING, GPIO_OSPEED_50MHZ, SDCARD_MISO_PIN);
    gpio_init(SDCARD_GPIO_PORT, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, SDCARD_CS_PIN);
    CS_HIGH();

    spi_parameter_struct spi_init_struct;
    spi_struct_para_init(&spi_init_struct);
    spi_init_struct.trans_mode = SPI_TRANSMODE_FULLDUPLEX;
    spi_init_struct.device_mode = SPI_MASTER;
    spi_init_struct.frame_size = SPI_FRAMESIZE_8BIT;
    spi_init_struct.clock_polarity_phase = SPI_CK_PL_HIGH_PH_2EDGE; // SPI Mode 3
    spi_init_struct.nss = SPI_NSS_SOFT;
    spi_init_struct.prescale = SPI_PSC_256;
    spi_init_struct.endian = SPI_ENDIAN_MSB;
    spi_init(SDCARD_SPI_PORT, &spi_init_struct);
    
    spi_enable(SDCARD_SPI_PORT);
    (void)SPI_DATA(SDCARD_SPI_PORT);

    timer3_config();

    if (Stat & STA_NODISK) return Stat;

    FCLK_SLOW();
    for (n = 10; n; n--) xchg_spi(0xFF);

    ty = 0;
    if (send_cmd(CMD0, 0) == 1) {
        Timer1 = 1000;
        if (send_cmd(CMD8, 0x1AA) == 1) {
            for (n = 0; n < 4; n++) ocr[n] = xchg_spi(0xFF);
            if (ocr[2] == 0x01 && ocr[3] == 0xAA) {
                while (Timer1 && send_cmd(ACMD41, 1UL << 30));
                if (Timer1 && send_cmd(CMD58, 0) == 0) {
                    for (n = 0; n < 4; n++) ocr[n] = xchg_spi(0xFF);
                    ty = (ocr[0] & 0x40) ? CT_SD2 | CT_BLOCK : CT_SD2;
                }
            }
        } else {
            if (send_cmd(ACMD41, 0) <= 1) { ty = CT_SD1; cmd = ACMD41; } 
            else { ty = CT_MMC; cmd = CMD1; }
            while (Timer1 && send_cmd(cmd, 0));
            if (!Timer1 || send_cmd(CMD16, 512) != 0) ty = 0;
        }
    }
    CardType = ty;
    deselect();

    if (ty) {
        printf("[INFO] sd_init: Card type detected: 0x%02X. Initialization successful.\n", ty);
        FCLK_FAST();
        Stat &= ~STA_NOINIT;
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
        if ((send_cmd(CMD17, sector) == 0) && rcvr_datablock(buff, 512)) {
            count = 0;
        }
    } else {
        if (send_cmd(CMD18, sector) == 0) {
            do {
                if (!rcvr_datablock(buff, 512)) break;
                buff += 512;
            } while (--count);
            send_cmd(CMD12, 0);
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
        if ((send_cmd(CMD24, sector) == 0) && xmit_datablock(buff, 0xFE)) {
            count = 0;
        }
    } else {
        if (CardType & CT_SDC) send_cmd(ACMD23, count);
        if (send_cmd(CMD25, sector) == 0) {
            do {
                if (!xmit_datablock(buff, 0xFC)) break;
                buff += 512;
            } while (--count);
            if (!xmit_datablock(nullptr, 0xFD)) count = 1;
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
            if ((send_cmd(CMD9, 0) == 0) && rcvr_datablock(csd, 16)) {
                if ((csd[0] >> 6) == 1) { // SDC ver 2.00
                    csize = csd[9] + ((DWORD)csd[8] << 8) + ((DWORD)(csd[7] & 63) << 16) + 1;
                    *(DWORD*)buff = csize << 10;
                } else { // SDC ver 1.XX or MMC
                    // WARNING FIX: Explicitly cast parts of the calculation to avoid sign/conversion warnings
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
                if ((send_cmd(CMD9, 0) == 0) && rcvr_datablock(csd, 16)) {
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