/*!
    \file    sd_card.h
    \brief   Header for the low-level SD card driver for Longan Nano

    \version 2025-02-10, V1.5.2
*/

#ifndef SD_CARD_H
#define SD_CARD_H

#include <cstdint>

// This wrapper is essential for linking C++ code with this C-style API.
// It tells the C++ compiler to use C linkage for these function declarations.
#ifdef __cplusplus
extern "C" {
#endif

// FatFs-compatible status definitions
typedef uint8_t DSTATUS;
#define STA_NOINIT		0x01	/* Drive not initialized */
#define STA_NODISK		0x02	/* No medium in the drive */
#define STA_PROTECT		0x04	/* Write protected */

typedef enum {
	RES_OK = 0,		/* 0: Successful */
	RES_ERROR,		/* 1: R/W Error */
	RES_WRPRT,		/* 2: Write Protected */
	RES_NOTRDY,		/* 3: Not Ready */
	RES_PARERR		/* 4: Invalid Parameter */
} DRESULT;

/* Command codes for disk_ioctl function */
#define CTRL_SYNC			0	/* Complete pending write process */
#define GET_SECTOR_COUNT	1	/* Get media size in sectors */
#define GET_SECTOR_SIZE		2	/* Get sector size in bytes */
#define GET_BLOCK_SIZE		3	/* Get erase block size in sectors */

/* --- Public API --- */

DSTATUS sd_init(void);
DSTATUS sd_status(void);
DRESULT sd_read_blocks(uint8_t *buff, uint32_t sector, uint32_t count);
DRESULT sd_write_blocks(const uint8_t *buff, uint32_t sector, uint32_t count);
DRESULT sd_read_blocks_dma(uint8_t *buff, uint32_t sector, uint32_t count);
DRESULT sd_write_blocks_dma(const uint8_t *buff, uint32_t sector, uint32_t count);
DRESULT sd_ioctl(uint8_t cmd, void *buff);

#ifdef __cplusplus
}
#endif

#endif // SD_CARD_H