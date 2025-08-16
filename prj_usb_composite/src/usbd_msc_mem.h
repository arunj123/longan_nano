/*!
    \file    usbd_msc_mem.h
    \brief   Header file for the MSC memory interface

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef USBD_MSC_MEM_H
#define USBD_MSC_MEM_H

#include <cstdint>

#define USBD_STD_INQUIRY_LENGTH          36U

// This structure defines the function pointers that your storage driver must provide.
typedef struct {
    int8_t (*mem_init)(uint8_t lun);
    int8_t (*mem_ready)(uint8_t lun);
    int8_t (*mem_protected)(uint8_t lun);
    int8_t (*mem_read)(uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len);
    int8_t (*mem_write)(uint8_t lun, uint8_t *buf, uint32_t block_addr, uint16_t block_len);
    int8_t (*mem_maxlun)(void);

    // Pointers to constant data describing the storage medium
    uint8_t *mem_inquiry_data[MEM_LUN_NUM];
    uint32_t mem_block_size[MEM_LUN_NUM];
    uint32_t mem_block_len[MEM_LUN_NUM];
} usbd_mem_cb;

// The UsbDevice class will access your implemented functions through this pointer.
// The definition of this pointer is in usbd_msc_mem.cpp
extern usbd_mem_cb *usbd_mem_fops;

#endif /* USBD_MSC_MEM_H */