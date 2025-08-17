/*!
    \file    sd_spi_hal.cpp
    \brief   Implementation of the SD Card SPI Hardware Abstraction Layer for GD32VF103.

    \version 2025-02-10, V1.0.1
*/

#include "sd_spi_hal.h"
#include "gd32vf103.h"

// Use C-style linkage for the ISR and the delay function from systick.c
extern "C" {
    #include "systick.h" // For delay_1ms
    void TIMER3_IRQHandler(void);
}

// =================================================================================
// --- File-Scope Hardware Definitions ---
// =================================================================================

// Hardware Configuration
constexpr uint32_t SDCARD_SPI_PORT  = SPI1;
constexpr rcu_periph_enum SDCARD_SPI_CLK = RCU_SPI1;
constexpr uint32_t SDCARD_GPIO_PORT = GPIOB;
constexpr rcu_periph_enum SDCARD_GPIO_CLK = RCU_GPIOB;
constexpr uint32_t SDCARD_CS_PIN    = GPIO_PIN_12;
constexpr uint32_t SDCARD_SCK_PIN   = GPIO_PIN_13;
constexpr uint32_t SDCARD_MISO_PIN  = GPIO_PIN_14;
constexpr uint32_t SDCARD_MOSI_PIN  = GPIO_PIN_15;

// DMA Configuration for SPI1
constexpr uint32_t SDCARD_DMA_PERIPH = DMA0;
constexpr dma_channel_enum SDCARD_DMA_RX_CH = DMA_CH2;
constexpr dma_channel_enum SDCARD_DMA_TX_CH = DMA_CH3;

// SPI Clock Speed Control Macros
#define FCLK_SLOW() { SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_256; }
#define FCLK_FAST() { SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_2; }

// Chip Select Control Macros
#define CS_HIGH()   gpio_bit_set(SDCARD_GPIO_PORT, SDCARD_CS_PIN)
#define CS_LOW()    gpio_bit_reset(SDCARD_GPIO_PORT, SDCARD_CS_PIN)


namespace { // Anonymous namespace for file-local variables and static functions

// Module-level variables
volatile uint32_t Timeout_ms = 0;
static uint8_t dummy_tx_ff = 0xFF;
static uint8_t dummy_rx;

// --- Private Hardware Configuration Functions ---

static void configure_rcu(void) {
    rcu_periph_clock_enable(SDCARD_GPIO_CLK);
    rcu_periph_clock_enable(SDCARD_SPI_CLK);
    rcu_periph_clock_enable(RCU_DMA0);
    rcu_periph_clock_enable(RCU_TIMER3);
}

static void configure_gpio(void) {
    gpio_init(SDCARD_GPIO_PORT, GPIO_MODE_AF_PP, GPIO_OSPEED_50MHZ, SDCARD_SCK_PIN | SDCARD_MOSI_PIN);
    gpio_init(SDCARD_GPIO_PORT, GPIO_MODE_IN_FLOATING, GPIO_OSPEED_50MHZ, SDCARD_MISO_PIN);
    gpio_init(SDCARD_GPIO_PORT, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, SDCARD_CS_PIN);
    hal_cs_high();
}

static void configure_spi(void) {
    spi_parameter_struct spi_init_struct;
    spi_struct_para_init(&spi_init_struct);
    spi_init_struct.trans_mode           = SPI_TRANSMODE_FULLDUPLEX;
    spi_init_struct.device_mode          = SPI_MASTER;
    spi_init_struct.frame_size           = SPI_FRAMESIZE_8BIT;
    spi_init_struct.clock_polarity_phase = SPI_CK_PL_LOW_PH_1EDGE; // SPI Mode 0
    spi_init_struct.nss                  = SPI_NSS_SOFT;
    spi_init_struct.prescale             = SPI_PSC_256; // Start slow
    spi_init_struct.endian               = SPI_ENDIAN_MSB;
    spi_init(SDCARD_SPI_PORT, &spi_init_struct);
    spi_enable(SDCARD_SPI_PORT);
    (void)SPI_DATA(SDCARD_SPI_PORT); // Dummy read to clear RBNE flag
}

static void configure_dma(void) {
    dma_parameter_struct dma_init_struct;
    dma_struct_para_init(&dma_init_struct);

    // Common settings
    dma_init_struct.periph_addr  = (uint32_t)&SPI_DATA(SDCARD_SPI_PORT);
    dma_init_struct.memory_width = DMA_MEMORY_WIDTH_8BIT;
    dma_init_struct.periph_width = DMA_PERIPHERAL_WIDTH_8BIT;
    dma_init_struct.periph_inc   = DMA_PERIPH_INCREASE_DISABLE;

    // SPI_TX channel
    dma_deinit(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_init_struct.direction    = DMA_MEMORY_TO_PERIPHERAL;
    dma_init_struct.memory_inc   = DMA_MEMORY_INCREASE_ENABLE;
    dma_init_struct.priority     = DMA_PRIORITY_MEDIUM;
    dma_circulation_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_init(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, &dma_init_struct);

    // SPI_RX channel
    dma_deinit(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_init_struct.direction    = DMA_PERIPHERAL_TO_MEMORY;
    dma_init_struct.memory_inc   = DMA_MEMORY_INCREASE_ENABLE;
    dma_init_struct.priority     = DMA_PRIORITY_HIGH;
    dma_circulation_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_init(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, &dma_init_struct);
}

static void configure_timer(void) {
    timer_parameter_struct timer_initpara;
    eclic_irq_enable(TIMER3_IRQn, 2, 0);
    timer_deinit(TIMER3);
    timer_initpara.prescaler         = static_cast<uint16_t>((rcu_clock_freq_get(CK_APB1) / 1000U) - 1U); // 1ms tick
    timer_initpara.period            = 9; // 10 ticks per interrupt
    timer_initpara.alignedmode       = TIMER_COUNTER_EDGE;
    timer_initpara.counterdirection  = TIMER_COUNTER_UP;
    timer_initpara.clockdivision     = TIMER_CKDIV_DIV1;
    timer_initpara.repetitioncounter = 0;
    timer_init(TIMER3, &timer_initpara);
    timer_interrupt_enable(TIMER3, TIMER_INT_UP);
    timer_enable(TIMER3);
}

static void spi_flush_fifo(void) {
    // Wait for the SPI to be not busy
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TRANS));
    // Clear the RX buffer by reading from it until it's empty
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RBNE)) {
        (void)SPI_DATA(SDCARD_SPI_PORT);
    }
}

} // End anonymous namespace

// --- Public HAL Function Implementations ---

// This function is now public, part of the HAL.
void hal_spi_flush_fifo(void) {
    // Wait for the SPI to be not busy (transmission complete)
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TRANS));
    // Clear the RX buffer by reading from it until it's empty
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RBNE)) {
        (void)SPI_DATA(SDCARD_SPI_PORT);
    }
}

void hal_spi_init(void) {
    configure_rcu();
    configure_gpio();
    configure_spi();
    configure_dma();
    configure_timer();
}

void hal_spi_set_speed(SdHalSpeed speed) {
    if (speed == SdHalSpeed::HIGH) {
        FCLK_FAST();
    } else {
        FCLK_SLOW();
    }
}

void hal_cs_high(void) { CS_HIGH(); }
void hal_cs_low(void)  { CS_LOW(); }

uint8_t hal_spi_xchg(uint8_t data) {
    // Wait for transmit buffer to be empty
    while (RESET == spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TBE));
    spi_i2s_data_transmit(SDCARD_SPI_PORT, data);

    // Wait for receive buffer to be not empty
    while (RESET == spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RBNE));
    return static_cast<uint8_t>(spi_i2s_data_receive(SDCARD_SPI_PORT));
}

void hal_spi_read_polling(uint8_t *buff, uint32_t count) {
    for (uint32_t i = 0; i < count; i++) {
        buff[i] = hal_spi_xchg(0xFF);
    }
}

void hal_spi_dma_read(uint8_t *buff, uint32_t count) {
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);

    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, (uint32_t)&dummy_tx_ff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, count);
    dma_memory_increase_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);

    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, (uint32_t)buff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, count);
    dma_memory_increase_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);

    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    
    // *** THE FIX: Call spi_dma_enable separately for each direction ***
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);

    while (!dma_flag_get(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_FLAG_FTF));

    spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
    spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
}

void hal_spi_dma_write(const uint8_t *buff, uint32_t count) {
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);

    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, (uint32_t)buff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, count);
    dma_memory_increase_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);

    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, (uint32_t)&dummy_rx);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, count);
    dma_memory_increase_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);

    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);

    // *** THE FIX: Call spi_dma_enable separately for each direction ***
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);
    
    while (!dma_flag_get(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_FLAG_FTF));

    spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
    spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
}

void hal_timer_start(uint32_t ms) {
    Timeout_ms = ms;
}

bool hal_timer_is_expired(void) {
    return (Timeout_ms == 0);
}

// New function implementation
void hal_delay_ms(uint32_t ms) {
    delay_1ms(ms);
}

// ISR to decrement the timeout counter
extern "C" void TIMER3_IRQHandler(void) {
    if (RESET != timer_interrupt_flag_get(TIMER3, TIMER_INT_UP)) {
        timer_interrupt_flag_clear(TIMER3, TIMER_INT_UP);
        uint32_t temp = Timeout_ms;
        if (temp) {
            Timeout_ms = temp - 1;
        }
    }
}