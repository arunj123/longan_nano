/*!
    \file    sd_spi_hal.h
    \brief   Header for the SD Card SPI Hardware Abstraction Layer.

    \version 2025-02-10, V1.3.0
*/

#ifndef SD_SPI_HAL_H
#define SD_SPI_HAL_H

#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

enum class SdHalSpeed { LOW, HIGH };

// Represents the current state of a DMA transfer
enum class HalDmaStatus { IDLE, BUSY, SUCCESS, ERROR };

// --- HAL Public API ---
void hal_spi_init(void);
void hal_spi_set_speed(SdHalSpeed speed);
void hal_cs_high(void);
void hal_cs_low(void);
uint8_t hal_spi_xchg(uint8_t data);
void hal_spi_read_polling(uint8_t *buff, uint32_t count);

// Non-blocking DMA functions
void hal_spi_dma_read_start(uint8_t *buff, uint32_t count);
void hal_spi_dma_write_start(const uint8_t *buff, uint32_t count);
HalDmaStatus hal_dma_get_status(void);

void hal_spi_flush_fifo(void);
void hal_timer_start(uint32_t ms);
bool hal_timer_is_expired(void);
void hal_delay_ms(uint32_t ms);

#ifdef __cplusplus
}
#endif

#endif // SD_SPI_HAL_H