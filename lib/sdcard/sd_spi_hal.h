/*!
    \file    sd_spi_hal.h
    \brief   Header for the SD Card SPI Hardware Abstraction Layer.

    \version 2025-02-10, V1.0.2
*/

#ifndef SD_SPI_HAL_H
#define SD_SPI_HAL_H

#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

// Enum for SPI speed settings
enum class SdHalSpeed {
    LOW,
    HIGH
};

// --- HAL Public API ---

// Initialization
void hal_spi_init(void);
void hal_spi_set_speed(SdHalSpeed speed);
void hal_cs_high(void);
void hal_cs_low(void);

// Data Exchange
uint8_t hal_spi_xchg(uint8_t data);
void hal_spi_read_polling(uint8_t *buff, uint32_t count);
void hal_spi_dma_read(uint8_t *buff, uint32_t count);
void hal_spi_dma_write(const uint8_t *buff, uint32_t count);

// *** NEW/CORRECTED FUNCTION DECLARATION ***
void hal_spi_flush_fifo(void);

// Timer Functions for Timeouts
void hal_timer_start(uint32_t ms);
bool hal_timer_is_expired(void);
void hal_delay_ms(uint32_t ms);

#ifdef __cplusplus
}
#endif

#endif // SD_SPI_HAL_H