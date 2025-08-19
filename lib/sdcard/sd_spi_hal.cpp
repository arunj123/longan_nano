/*!
    \file    sd_spi_hal.cpp
    \brief   Implementation of the SD Card SPI Hardware Abstraction Layer for GD32VF103.

    \version 2025-02-10, V1.7.0 (Heavy Debug Instrumentation)
*/

#include "sd_spi_hal.h"
#include "gd32vf103.h"
#include <cstdio>

extern "C" {
    #include "systick.h"
    // This header contains the correct eclic_enable_interrupt function
    #include "n200_func.h" 
    void TIMER3_IRQHandler(void);
}

// --- Hardware definitions for SPI1 remain the same ---
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
constexpr dma_channel_enum SDCARD_DMA_RX_CH = DMA_CH3;
constexpr dma_channel_enum SDCARD_DMA_TX_CH = DMA_CH4;

#define FCLK_SLOW() SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_256
#define FCLK_FAST() SPI_CTL0(SDCARD_SPI_PORT) = (SPI_CTL0(SDCARD_SPI_PORT) & ~SPI_CTL0_PSC) | SPI_PSC_2
#define CS_HIGH()   gpio_bit_set(SDCARD_GPIO_PORT, SDCARD_CS_PIN)
#define CS_LOW()    gpio_bit_reset(SDCARD_GPIO_PORT, SDCARD_CS_PIN)

// Definitions for debug printing
#define ECLIC_INTIE_BASE (ECLIC_ADDR_BASE + 0x1000)
#define ECLIC_INTIE_REG(irqn) (*(volatile uint32_t*)(ECLIC_INTIE_BASE + (irqn / 32) * 4))
#define ECLIC_INTIE_BIT(irqn) (1U << (irqn % 32))

// Debug function to print the status of critical registers
void print_debug_regs(const char* stage) {
    printf("\n--- DEBUG REG DUMP (%s) ---\n", stage);
    printf("DMA_INTF: 0x%08lx\n", (unsigned long)DMA_INTF(SDCARD_DMA_PERIPH));
    printf("RX (CH3) CTL: 0x%08lx (EN=%d, FTFIE=%d)\n",
           (unsigned long)DMA_CHCTL(SDCARD_DMA_PERIPH, DMA_CH3),
           (DMA_CHCTL(SDCARD_DMA_PERIPH, DMA_CH3) & DMA_CHXCTL_CHEN) ? 1 : 0,
           (DMA_CHCTL(SDCARD_DMA_PERIPH, DMA_CH3) & DMA_CHXCTL_FTFIE) ? 1 : 0);
    printf("TX (CH4) CTL: 0x%08lx (EN=%d, FTFIE=%d)\n",
           (unsigned long)DMA_CHCTL(SDCARD_DMA_PERIPH, DMA_CH4),
           (DMA_CHCTL(SDCARD_DMA_PERIPH, DMA_CH4) & DMA_CHXCTL_CHEN) ? 1 : 0,
           (DMA_CHCTL(SDCARD_DMA_PERIPH, DMA_CH4) & DMA_CHXCTL_FTFIE) ? 1 : 0);
    printf("SPI1_CTL1: 0x%08lx (DMATEN=%d, DMAREN=%d)\n",
           (unsigned long)SPI_CTL1(SDCARD_SPI_PORT),
           (SPI_CTL1(SDCARD_SPI_PORT) & SPI_CTL1_DMATEN) ? 1 : 0,
           (SPI_CTL1(SDCARD_SPI_PORT) & SPI_CTL1_DMAREN) ? 1 : 0);
    uint32_t irqn_rx = DMA0_Channel3_IRQn;
    uint32_t irqn_tx = DMA0_Channel4_IRQn;
    printf("ECLIC_INTIE: RX_IRQ_EN=%d, TX_IRQ_EN=%d\n",
           (ECLIC_INTIE_REG(irqn_rx) & ECLIC_INTIE_BIT(irqn_rx)) ? 1 : 0,
           (ECLIC_INTIE_REG(irqn_tx) & ECLIC_INTIE_BIT(irqn_tx)) ? 1 : 0);
    printf("---------------------------------------\n");
}

namespace {
volatile uint32_t Timeout_ms = 0;
static uint8_t dummy_tx_ff = 0xFF;
static uint8_t dummy_rx;
volatile HalDmaStatus g_dma_status = HalDmaStatus::IDLE;


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
    spi_init_struct.clock_polarity_phase = SPI_CK_PL_HIGH_PH_2EDGE;
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
    
    timer_deinit(TIMER3);
    timer_initpara.prescaler         = static_cast<uint16_t>((rcu_clock_freq_get(CK_APB1) / 1000U) - 1U);
    timer_initpara.period            = 9;
    timer_initpara.alignedmode       = TIMER_COUNTER_EDGE;
    timer_initpara.counterdirection  = TIMER_COUNTER_UP;
    timer_initpara.clockdivision     = TIMER_CKDIV_DIV1;
    timer_initpara.repetitioncounter = 0;
    timer_init(TIMER3, &timer_initpara);
    
    timer_update_event_enable(TIMER3);
    timer_interrupt_enable(TIMER3,TIMER_INT_UP);
    timer_flag_clear(TIMER3, TIMER_FLAG_UP);
    timer_update_source_config(TIMER3, TIMER_UPDATE_SRC_GLOBAL);

    timer_enable(TIMER3);
}

static void configure_driver_interrupts(void) {
    // This function only configures the interrupts needed by THIS driver.
    // It assumes the main ECLIC controller has already been initialized by the system startup code.
    
    eclic_enable_interrupt(TIMER3_IRQn);

    // Enable the specific interrupts we need for DMA
    eclic_enable_interrupt(DMA0_Channel3_IRQn);
    eclic_enable_interrupt(DMA0_Channel4_IRQn);

    // Set priorities for these interrupts. Lower number is higher priority.
    eclic_set_irq_priority(DMA0_Channel3_IRQn, 1);
    eclic_set_irq_priority(DMA0_Channel4_IRQn, 1);
}

} // End anonymous namespace

void hal_spi_init(void) {
    rcu_periph_reset_enable(SDCARD_SPI_RST);
    rcu_periph_reset_disable(SDCARD_SPI_RST);
    configure_rcu();
    configure_gpio();
    configure_spi();
    configure_dma();
    configure_timer();
    configure_driver_interrupts(); // <-- Use the new, safer function
}

HalDmaStatus hal_dma_get_status(void) {
    
            printf("DMA Read Wait: INTF=0x%lx, TX_CNT=%lu, RX_CNT=%lu, SPI_STAT=0x%lx\n",
                   (unsigned long)DMA_INTF(SDCARD_DMA_PERIPH),
                   (unsigned long)dma_transfer_number_get(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH),
                   (unsigned long)dma_transfer_number_get(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH),
                   (unsigned long)SPI_STAT(SDCARD_SPI_PORT));

    return g_dma_status;
}

void hal_spi_dma_write_start(const uint8_t *buff, uint32_t count) {
    g_dma_status = HalDmaStatus::BUSY;
    
    spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE | SPI_DMA_TRANSMIT);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, (uint32_t)buff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, count);
    dma_memory_increase_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, (uint32_t)&dummy_rx);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, count);
    dma_memory_increase_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    
    dma_interrupt_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FTF);
    
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_TRANSMIT);

    print_debug_regs("DMA Write Start");
}

void hal_spi_dma_read_start(uint8_t *buff, uint32_t count) {
    g_dma_status = HalDmaStatus::BUSY;
    hal_spi_flush_fifo();
    
    spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE | SPI_DMA_TRANSMIT);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, (uint32_t)&dummy_tx_ff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, count);
    dma_memory_increase_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    
    dma_memory_address_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, (uint32_t)buff);
    dma_transfer_number_config(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, count);
    dma_memory_increase_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    
    dma_interrupt_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FTF);
    
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
    dma_channel_enable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
    
    spi_dma_enable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE);

    print_debug_regs("DMA Read Start");
}

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
    // This robust flush is inspired by your working lcd.c
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TRANS));
    if(spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RXORERR)){
        (void)SPI_DATA(SDCARD_SPI_PORT);
        (void)SPI_STAT(SDCARD_SPI_PORT);
    }
    while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_RBNE)) {
        (void)SPI_DATA(SDCARD_SPI_PORT);
    }
}

void hal_timer_start(uint32_t ms) { Timeout_ms = ms; }
bool hal_timer_is_expired(void) { return (Timeout_ms == 0); }
void hal_delay_ms(uint32_t ms) { delay_1ms(ms); }

// --- Interrupt Service Routines ---
extern "C" {

void DMA0_Channel3_IRQHandler__(void) { // RX Channel for SPI1
    print_debug_regs("DMA TX IRQ");
    if(dma_interrupt_flag_get(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FLAG_FTF)) {
        dma_interrupt_flag_clear(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FLAG_G);
        dma_interrupt_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH, DMA_INT_FTF);
        
        spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE | SPI_DMA_TRANSMIT);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);
        
        g_dma_status = HalDmaStatus::SUCCESS;
    }
}

void DMA0_Channel4_IRQHandler__(void) { // TX Channel for SPI1
    print_debug_regs("DMA TX IRQ");
    if(dma_interrupt_flag_get(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FLAG_FTF)) {
        dma_interrupt_flag_clear(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FLAG_G);
        dma_interrupt_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH, DMA_INT_FTF);

        while (spi_i2s_flag_get(SDCARD_SPI_PORT, SPI_FLAG_TRANS));

        spi_dma_disable(SDCARD_SPI_PORT, SPI_DMA_RECEIVE | SPI_DMA_TRANSMIT);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_RX_CH);
        dma_channel_disable(SDCARD_DMA_PERIPH, SDCARD_DMA_TX_CH);

        hal_spi_flush_fifo();

        g_dma_status = HalDmaStatus::SUCCESS;
    }
}

void TIMER3_IRQHandler(void) {
    {
        static uint32_t my_counter_ms = 0; // Use a static variable to avoid reinitialization
        my_counter_ms++;
        if (my_counter_ms >= 1000) { // 1000 ms = 1 second
            printf("TIMER3 IRQ: 1 second elapsed\n");
            my_counter_ms = 0; // Reset the counter
        }
    }
    if (RESET != timer_interrupt_flag_get(TIMER3, TIMER_INT_UP)) {
        timer_interrupt_flag_clear(TIMER3, TIMER_INT_UP);
        if (Timeout_ms > 0) {
            Timeout_ms = Timeout_ms - 1; // Correct way to decrement volatile
        }
    }
}

void hal_spi_write_polling(const uint8_t *buff, uint32_t count) {
    for (uint32_t i = 0; i < count; i++) {
        hal_spi_xchg(buff[i]);
    }
}

} // extern "C"