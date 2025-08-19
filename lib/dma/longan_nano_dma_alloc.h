
#include "gd32vf103.h"

#ifndef LONGAN_NANO_DMA_ALLOC_H
#define LONGAN_NANO_DMA_ALLOC_H

// DMA0, DMA_CH1 and DMA_CH2 are used for SPI0 transfers by the LCD driver.

// Device and channel definitions. This cannot be changed. This is hardware limitation
// Refere README.md for more information.

#define DMA0_SPI0_RX_CH  DMA_CH1
#define DMA0_SPI0_TX_CH  DMA_CH2

#define DMA0_SPI1_RX_CH  DMA_CH3
#define DMA0_SPI1_TX_CH  DMA_CH4

#endif // LONGAN_NANO_DMA_ALLOC_H