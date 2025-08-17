/*!
    \file    usbd_msc_mem.h
    \brief   Header file for the MSC memory interface

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

#ifndef USBD_MSC_MEM_H
#define USBD_MSC_MEM_H

#include "usbd_conf.h"
#include <cstdint>

#define USBD_STD_INQUIRY_LENGTH          36U

// This structure defines the function pointers that your storage driver must provide.
typedef struct {
    int8_t (*mem_init)(uint8_t lun);
    int8_t (*mem_ready)(uint8_t lun);
    int8_t (*mem_protected)(uint8_t lun);
    int8_t (*mem_read)(uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len);
    int8_t (*mem_write)(uint8_t lun, const uint8_t *buf, uint32_t block_addr, uint16_t block_len);
    int8_t (*mem_maxlun)(void);

    // Pointers to constant data describing the storage medium
    uint8_t *mem_inquiry_data[MEM_LUN_NUM];
    uint32_t mem_block_size[MEM_LUN_NUM];
    uint32_t mem_block_len[MEM_LUN_NUM];
} usbd_mem_cb;

// Use a function to get the fops structure, preventing static init order issues.
usbd_mem_cb& get_msc_mem_fops();

/**
 * @brief Performs the one-time, slow initialization of the SD card properties.
 * This must be called from main() before the USB stack is initialized.
 * @return void
 */
void msc_mem_pre_init();

#endif /* USBD_MSC_MEM_H */