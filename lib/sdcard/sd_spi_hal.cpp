/*!
    \file    sd_spi_hal.cpp
    \brief   Implementation of the SD Card SPI Hardware Abstraction Layer for GD32VF103.

    \version 2025-02-10, V1.3.0
*/

#include "sd_spi_hal.h"
#include "gd32vf103.h"

extern "C" {
    #include "systick.h"
    void TIMER3_IRQHandler(void);
    void DMA0_Channel1_IRQHandler(void); // RX
    void DMA0_Channel2_IRQHandler(void); // TX
}

// ... (Hardware definitions remain the same) ...
constexpr uint32_t SDCARD_SPI_PORT  = SPI1;
constexpr rcu_periph_enum SDCARD_SPI_CLK = RCU_SPI1;
constexpr rcu_periph_reset_enum SDCARD_SPI_RST = RCU_SPI1RST;
constexpr uint32_t SDCARD_GPIO_PORT = GPIOB;
constexpr rcu_periph_enum SDCARD_GPIO_CLK = RCU_GPIOB;
constexpr uint32_t SDCARD_CS_PIN    = GPIO_PIN_12;
constexpr uint32_t SDCARD_SCK_PIN   = GPIO_PIN_13;
constexpr uint32_t SDCARD_MISO_PIN  = GPIO_PIN_14;
constexpr uint32_t SDCARD_MOSI_PIN  = GPIO_PIN_15;

constexpr uint32_t SDCARD_DMA_PERIPH = DMA0;
constexpr dma_channel_enum SDCARD_DMA_RX_CH = DMA_CH1;
constexpr dma_channel_enum SDCARD_DMA_TX_CH = DMA_CH2;

#define FCLK_SLOW() SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_256
#define FCLK_FAST() SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_2
#define CS_HIGH()   gpio_bit_set(SDCARD_GPIO_PORT, SDCARD_CS_PIN)
#define CS_LOW()    gpio_bit_reset(SDCARD_GPIO_PORT, SDCARD_CS_PIN)

namespace {
volatile uint32_t Timeout_ms = 0;
static uint8_t dummy_tx_ff = 0xFF;
static uint8_t dummy_rx;
volatile HalDmaStatus g_dma_status = HalDmaStatus::IDLE;

static void configure_eclic(void) {
    eclic_global_interrupt_enable();
    eclic_priority_group_set(ECLIC_PRIGROUP_LEVEL3_PRIO1);
    // Enable interrupts for the DMA channels used by SPI1
    eclic_irq_enable(DMA0_Channel1_IRQn, 1, 0); // RX channel
    eclic_irq_enable(DMA0_Channel2_IRQn, 1, 0); // TX channel
}

// ... (configure_rcu, gpio, spi, dma, timer are the same as before) ...
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
    CS_HIGH();
}

static void configure_spi(void) {
    spi_parameter_struct spi_init_struct;
    spi_struct_para_init(&spi_init_struct);
    spi_init_struct.trans_mode           = SPI_TRANSMODE_FULLDUPLEX;
    spi_init_struct.device_mode          = SPI_MASTER;
    spi_init_struct.frame_size           = SPI_FRAMESIZE_8BIT;
    spi_init_struct.clock_polarity_phase = SPI_CK_PL_HIGH_PH_2EDGE; // Mode 3
    spi_init_struct.nss                  = SPI_NSS_SOFT;
    spi_init_struct.prescale             = SPI_PSC_256;
    spi_init_struct.endian               = SPI_ENDIAN_MSB;
    spi_init(SDCARD_SPI_PORT, &spi_init_struct);
    spi_enable(SDCARD_SPI_PORT);
}

static void configure_dma(void) {
    dma_parameter_struct dma_init_struct;
    dma_struct_para_init(&dma_init_struct);
    dma_init_struct.periph_addr  = (uint32_t)&SPI_DATA(SDCARD_SPI_PORT);
    dma_init_struct.memory_width = DMA_MEMORY_WIDTH_8BIT;
    dma_init_struct.periph_width = DMA_PERIPHERAL_WIDTH_8BIT;
    dma_init_struct.periph_inc   = DMA_PERIPH_INCREASE_DISABLE;

    dma_deinit(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_init_struct.direction    = DMA_MEMORY_TO_PERIPHERAL;
    dma_init_struct.memory_inc   = DMA_MEMORY_INCREASE_ENABLE;
    dma_init_struct.priority     = DMA_PRIORITY_MEDIUM;
    dma_circulation_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_init(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, &dma_init_struct);

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
    timer_initpara.prescaler         = static_cast<uint16_t>((rcu_clock_freq_get(CK_APB1) / 1000U) - 1U);
    timer_initpara.period            = 9;
    timer_initpara.alignedmode       = TIMER_COUNTER_EDGE;
    timer_initpara.counterdirection  = TIMER_COUNTER_UP;
    timer_initpara.clockdivision     = TIMER_CKDIV_DIV1;
    timer_initpara.repetitioncounter = 0;
    timer_init(TIMER3, &timer_initpara);
    timer_interrupt_enable(TIMER3, TIMER_INT_UP);
    timer_enable(TIMER3);
}

} // End anonymous namespace

// --- Public HAL Function Implementations ---
void hal_spi_init(void) {
    rcu_periph_reset_enable(SDCARD_SPI_RST);
    rcu_periph_reset_disable(SDCARD_SPI_RST);
    configure_rcu();
    configure_gpio();
    configure_spi();
    configure_dma();
    configure_timer();
    configure_eclic(); // Configure interrupts
}

HalDmaStatus hal_dma_get_status(void) {
    return g_dma_status;
}

void hal_spi_dma_write_start(const uint8_t *buff, uint32_t count) {
    g_dma_status = HalDmaStatus::BUSY;
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, (uint32_t)buff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, count);
    dma_memory_increase_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, (uint32_t)&dummy_rx);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, count);
    dma_memory_increase_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_interrupt_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FTF);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);
}

void hal_spi_dma_read_start(uint8_t *buff, uint32_t count) {
    g_dma_status = HalDmaStatus::BUSY;
    hal_spi_flush_fifo();
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, (uint32_t)&dummy_tx_ff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, count);
    dma_memory_increase_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, (uint32_t)buff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, count);
    dma_memory_increase_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_interrupt_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FTF);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);
}

// ... (polling functions, cs, speed, etc. remain the same) ...
void hal_spi_set_speed(SdHalSpeed speed) { (speed == SdHalSpeed::HIGH) ? FCLK_FAST() : FCLK_SLOW(); }
void hal_cs_high(void) { CS_HIGH(); }
void hal_cs_low(void)  { CS_LOW(); }
uint8_t hal_spi_xchg(uint8_t data) {
    while (RESET == spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TBE));
    spi_i2s_data_transmit(SDCARD_SPI_PORT, data);
    while (RESET == spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RBNE));
    return static_cast<uint8_t>(spi_i2s_data_receive(SDCARD_SPI_PORT));
}
void hal_spi_read_polling(uint8_t *buff, uint32_t count) { for (uint32_t i = 0; i < count; i++) buff[i] = hal_spi_xchg(0xFF); }
void hal_spi_flush_fifo(void) {
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TRANS));
    if(spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RXORERR)){ (void)SPI_DATA(SDCARD_SPI_PORT); (void)SPI_STAT(SDCARD_SPI_PORT); }
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RBNE)) { (void)SPI_DATA(SDCARD_SPI_PORT); }
}
void hal_timer_start(uint32_t ms) { Timeout_ms = ms; }
bool hal_timer_is_expired(void) { return (Timeout_ms == 0); }
void hal_delay_ms(uint32_t ms) { delay_1ms(ms); }

// --- Interrupt Service Routines ---
extern "C" {

void DMA0_Channel1_IRQHandler(void) { // RX Channel
    if(dma_interrupt_flag_get(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FLAG_FTF)) {
        dma_interrupt_flag_clear(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FLAG_G);
        dma_interrupt_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FTF);
        
        spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
        spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
        
        g_dma_status = HalDmaStatus::SUCCESS;
    }
}

void DMA0_Channel2_IRQHandler(void) { // TX Channel
    if(dma_interrupt_flag_get(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FLAG_FTF)) {
        dma_interrupt_flag_clear(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FLAG_G);
        dma_interrupt_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FTF);

        // Wait for SPI to finish last byte
        while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TRANS));

        spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);
        spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);

        // Clear the expected overrun error
        hal_spi_flush_fifo();

        g_dma_status = HalDmaStatus::SUCCESS;
    }
}

void TIMER3_IRQHandler(void) {
    if (RESET != timer_interrupt_flag_get(TIMER3, TIMER_INT_UP)) {
        timer_interrupt_flag_clear(TIMER3, TIMER_INT_UP);
        uint32_t temp = Timeout_ms;
        if (temp) {
            Timeout_ms = temp - 1;
        }
    }
}

} // extern "C"