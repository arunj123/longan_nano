/*!
    \file    usbd_msc_mem.cpp
    \brief   USB MSC memory access layer implementation for an SD Card

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

#include "usbd_conf.h"
#include "sd_card.h"
#include "usbd_msc_mem.h"

// --- Function Prototypes for the callback structure ---
static int8_t mem_init (uint8_t lun);
static int8_t mem_ready (uint8_t lun);
static int8_t mem_protected (uint8_t lun);
static int8_t mem_read (uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len);
static int8_t mem_write (uint8_t lun, const uint8_t *buf, uint32_t block_addr, uint16_t block_len);
static int8_t mem_maxlun (void);

/* USB mass storage inquiry data */
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

// We will populate these with the actual card size during initialization.
static uint32_t card_block_size = 512;
static uint32_t card_block_count = 0;

// This structure connects your functions and storage parameters to the USB MSC class.
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

usbd_mem_cb& get_msc_mem_fops()
{
    return usbd_storage_fops;
}

// --- IMPLEMENTATION OF THE CALLBACK FUNCTIONS ---

/*!
    \brief      initialize the memory media and get its properties
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for OK, -1 for fail)
*/
static int8_t mem_init (uint8_t lun) {
    (void)lun; // Unused parameter

    DSTATUS status = sd_init();

    if (status & STA_NOINIT) {
        return -1; // Initialization failed
    }

    // Get the actual card capacity and block size
    sd_ioctl(GET_SECTOR_COUNT, &card_block_count);
    sd_ioctl(GET_SECTOR_SIZE, &card_block_size);

    // Update the fops structure with the correct values
    usbd_storage_fops.mem_block_len[lun] = card_block_count;
    usbd_storage_fops.mem_block_size[lun] = card_block_size;

    return 0;
}

/*!
    \brief      check whether the memory media is ready
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for OK, -1 for not ready)
*/
static int8_t mem_ready (uint8_t lun) {
    (void)lun;
    return (sd_status() & STA_NOINIT) ? -1 : 0;
}

/*!
    \brief      check whether the memory media is write-protected
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for not protected, 1 for protected)
*/
static int8_t mem_protected (uint8_t lun) {
    (void)lun;
    return (sd_status() & STA_PROTECT) ? 1 : 0;
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
    return (sd_read_blocks(buf, block_addr, block_len) == RES_OK) ? 0 : -1;
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
    return (sd_write_blocks(buf, block_addr, block_len) == RES_OK) ? 0 : -1;
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