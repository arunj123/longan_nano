/*!
    \file    usbd_msc_mem.cpp
    \brief   USB MSC memory access layer implementation

    \version 2025-02-10, firmware for GD32VF103
*/

#include "usbd_conf.h"

// This file must be C-compatible as it's included by C++ code,
// but the functions it calls might be from C libraries.
extern "C" {
    #include "usbd_msc_mem.h"
}

// --- User Implementation Section ---
// TODO: Include your low-level storage driver header here (e.g., "spi_flash.h" or "sd_card.h")

// --- Function Prototypes for your storage driver ---
// These are the functions you will write to control the hardware.
static int8_t mem_init (uint8_t lun);
static int8_t mem_ready (uint8_t lun);
static int8_t mem_protected (uint8_t lun);
static int8_t mem_read (uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len);
static int8_t mem_write (uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len);
static int8_t mem_maxlun (void);

/* USB mass storage inquiry data */
const uint8_t msc_inquiry_data[] = {
    0x00, 0x80, 0x02, 0x02,
    (USBD_STD_INQUIRY_LENGTH - 5),
    0x00, 0x00, 0x00,
    'G', 'D', '3', '2', ' ', ' ', ' ', ' ', /* Manufacturer: 8 bytes */
    'U', 'S', 'B', ' ', 'D', 'i', 's', 'k', /* Product: 16 bytes */
    '1', '.', '0', '0',                     /* Version: 4 bytes */
};

// This structure connects your functions and storage parameters to the USB MSC class.
usbd_mem_cb usbd_storage_fops = {
    .mem_init      = mem_init,
    .mem_ready     = mem_ready,
    .mem_protected = mem_protected,
    .mem_read      = mem_read,
    .mem_write     = mem_write,
    .mem_maxlun    = mem_maxlun,

    .mem_inquiry_data = {(uint8_t*)msc_inquiry_data},
    .mem_block_size   = {512}, // TODO: Set your actual sector/block size (e.g., 512 for SD, 4096 for flash)
    .mem_block_len    = {1024} // TODO: Set your actual number of blocks (e.g., for a 512KB disk with 512B sectors, this is 1024)
};

usbd_mem_cb *usbd_mem_fops = &usbd_storage_fops;

// --- IMPLEMENT THE FOLLOWING FUNCTIONS ---

/*!
    \brief      initialize the memory media
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for OK, -1 for fail)
*/
static int8_t mem_init (uint8_t lun)
{
    // TODO: Initialize your storage medium (e.g., SD card init, SPI flash init)
    // Return 0 if successful, -1 if failed.
    return 0;
}

/*!
    \brief      check whether the memory media is ready
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for OK, -1 for not ready)
*/
static int8_t mem_ready (uint8_t lun)
{
    // TODO: Check if your storage medium is present and ready for operations.
    // Return 0 if ready, -1 if not.
    return 0;
}

/*!
    \brief      check whether the memory media is write-protected
    \param[in]  lun: logical unit number
    \param[out] none
    \retval     status (0 for not protected, 1 for protected)
*/
static int8_t mem_protected (uint8_t lun)
{
    // TODO: Check if your storage medium is write-protected.
    // Return 0 if writable, 1 if protected.
    return 0;
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
static int8_t mem_read (uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len)
{
    // TODO: Implement the read operation for your storage medium.
    // Example: sd_card_read_blocks(buf, block_addr, block_len);
    // Return 0 if successful, -1 if failed.
    return 0;
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
static int8_t mem_write (uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len)
{
    // TODO: Implement the write operation for your storage medium.
    // Example: sd_card_write_blocks(buf, block_addr, block_len);
    // Return 0 if successful, -1 if failed.
    return 0;
}

/*!
    \brief      get number of supported logical unit
    \param[in]  none
    \param[out] none
    \retval     number of logical unit
*/
static int8_t mem_maxlun (void)
{
    return (MEM_LUN_NUM - 1);
}