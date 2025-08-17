/*!
    \file    usbd_msc_mem.cpp
    \brief   USB MSC memory access layer that gracefully handles SD card failures.

    \version 2025-02-10, V1.5.1
*/

#include "usbd_conf.h"
#include "usbd_msc_mem.h"
#include <cstdio> // For printf

// Conditionally include the SD card driver
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    #include "sd_card.h"
#endif

// --- State and Storage Properties ---
static bool is_media_present = false;
static uint32_t card_block_size = 512;
static uint32_t card_block_count = 0;

// --- Forward Declarations ---
static int8_t mem_init (uint8_t lun);
static int8_t mem_ready (uint8_t lun);
static int8_t mem_protected (uint8_t lun);
static int8_t mem_read (uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len);
static int8_t mem_write (uint8_t lun, const uint8_t *buf, uint32_t block_addr, uint16_t block_len);
static int8_t mem_maxlun (void);

/* USB mass storage inquiry data (remains the same) */
const uint8_t msc_inquiry_data[] = {
    0x00, /* Direct-access device */
    0x80, /* Removable media */
    0x02, /* ISO/ECMA, ANSI version */
    0x02, /* Response data format */
    (USBD_STD_INQUIRY_LENGTH - 5),
    0x00, 0x00, 0x00,
    'G', 'D', '3', '2', ' ', ' ', ' ', ' ', /* Manufacturer: 8 bytes */
    'S', 'D', ' ', 'C', 'a', 'r', 'd', ' ', /* Product: 16 bytes */
    '1', '.', '0', '0',                     /* Version: 4 bytes */
};

// This structure is now populated by msc_mem_pre_init()
usbd_mem_cb usbd_storage_fops = {
    .mem_init      = mem_init,
    .mem_ready     = mem_ready,
    .mem_protected = mem_protected,
    .mem_read      = mem_read,
    .mem_write     = (int8_t (*)(uint8_t, const uint8_t*, uint32_t, uint16_t))mem_write,
    .mem_maxlun    = mem_maxlun,
    .mem_inquiry_data = {(uint8_t*)msc_inquiry_data},
    .mem_block_size   = {card_block_size},
    .mem_block_len    = {card_block_count}
};

usbd_mem_cb& get_msc_mem_fops() {
    return usbd_storage_fops;
}

// --- Public Initialization Function ---

void msc_mem_pre_init() {
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    printf("Pre-caching MSC drive properties...\n");
    // This is called once from main() before USB starts.
    if ((sd_status() & STA_NOINIT) || (sd_status() & STA_NODISK)) {
        printf("WARN: SD Card not ready for MSC.\n");
        is_media_present = false;
        return;
    }

    if (sd_ioctl(GET_SECTOR_COUNT, &card_block_count) != RES_OK || card_block_count == 0) {
        printf("WARN: Failed to get SD card sector count.\n");
        is_media_present = false;
        return;
    }
    
    sd_ioctl(GET_SECTOR_SIZE, &card_block_size);

    // Success! Update the fops structure with the correct, pre-cached values.
    usbd_storage_fops.mem_block_len[0] = card_block_count;
    usbd_storage_fops.mem_block_size[0] = card_block_size;
    is_media_present = true;
    printf("INFO: MSC properties cached successfully. Block count: %lu\n", card_block_count);
#else
    // If SD card is disabled in the build, ensure we always report no media.
    is_media_present = false;
    printf("INFO: MSC is disabled in this build.\n");
#endif
}

// --- IMPLEMENTATION OF THE CALLBACKS ---

/*!
    \brief      Initialize the memory media (NOW A FAST, NON-BLOCKING STUB)
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for OK, -1 for fail)
*/
static int8_t mem_init (uint8_t lun) {
    (void)lun;
    return is_media_present ? 0 : -1;
}

/*!
    \brief      check whether the memory media is ready
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for OK, -1 for not ready)
*/
static int8_t mem_ready (uint8_t lun) {
    (void)lun;
    return is_media_present ? 0 : -1;
}

/*!
    \brief      check whether the memory media is write-protected
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for not protected, 1 for protected)
*/
static int8_t mem_protected (uint8_t lun) {
    (void)lun;
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    if (!is_media_present) return 1;
    return (sd_status() & STA_PROTECT) ? 1 : 0;
#else
    return 1;
#endif
}

/*!
    \brief      read data from the memory media
    \param[in]  lun: logical unit number
    \param[in]  buf: pointer to the buffer to save data
    \param[in]  block_addr: block address
    \param[in]  block_len: number of blocks to read
    \param[out] none
    \retval     status (0 for OK, -1 for fail)
*/
static int8_t mem_read (uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len) {
    (void)lun;
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    if (!is_media_present) return -1;
    return (sd_read_blocks(buf, block_addr, block_len) == RES_OK) ? 0 : -1;
#else
    (void)buf; (void)block_addr; (void)block_len;
    return -1;
#endif
}

/*!
    \brief      write data to the memory media
    \param[in]  lun: logical unit number
    \param[in]  buf: pointer to the buffer to write data
    \param[in]  block_addr: block address
    \param[in]  block_len: number of blocks to write
    \param[out] none
    \retval     status (0 for OK, -1 for fail)
*/
static int8_t mem_write (uint8_t lun, const uint8_t *buf, uint32_t block_addr, uint16_t block_len) {
    (void)lun;
#if defined(USE_SD_CARD_MSC) && (USE_SD_CARD_MSC == 1)
    if (!is_media_present) return -1;
    return (sd_write_blocks(buf, block_addr, block_len) == RES_OK) ? 0 : -1;
#else
    (void)buf; (void)block_addr; (void)block_len;
    return -1;
#endif
}

/*!
    \brief      get number of supported logical unit
    \param[in]  none
    \param[out] none
    \retval     number of logical unit
*/
static int8_t mem_maxlun (void) {
    return (MEM_LUN_NUM - 1);
}