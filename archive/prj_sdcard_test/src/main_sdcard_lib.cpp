/*!
    \file    main.cpp
    \brief   Asynchronous SPI1 command-response transaction with an SD card using DMA.
             This is the final, working version with all helper functions and correct protocol.

    \version 2025-02-10, V5.1 (Final)
*/

#include "gd32vf103.h"
#include <stdio.h>
#include <string.h>

extern "C" {
#include "n200_func.h"
}

/* --- Buffers and State Flags --- */
/* CMD0 (GO_IDLE_STATE) for initialization */
uint8_t cmd0_buffer[6] = {0x40 | 0, 0x00, 0x00, 0x00, 0x00, 0x95};
/* CMD8 (SEND_IF_COND) for the main test */
#define TX_BUFFER_SIZE 6
uint8_t cmd8_buffer[TX_BUFFER_SIZE] = {0x40 | 8, 0x00, 0x00, 0x01, 0xAA, 0x87};

#define RX_BUFFER_SIZE 8
uint8_t spi_rx_buffer[RX_BUFFER_SIZE];

/* Dummy byte to transmit during a receive operation */
const uint8_t DUMMY_BYTE = 0xFF;

/* Volatile flags to signal completion from ISRs */
volatile int dma_tx_complete = 0;
volatile int dma_rx_complete = 0;


/*
 * =============================================================================
 * --- Helper Function Implementations ---
 * =============================================================================
 */

/**
  * @brief  Prints the contents of a buffer in a readable hexdump format.
  */
void print_buffer(const char* title, const uint8_t* buff, uint32_t len) {
    printf("%s (%lu bytes):\n", title, len);
    for (uint32_t i = 0; i < len; i++) {
        if (i % 16 == 0) {
            printf("  0x%04lX: ", i);
        }
        printf("%02X ", buff[i]);
    }
    printf("\n");
}

/**
  * @brief  Prints the state of key DMA and SPI registers for debugging.
  */
void print_debug_regs(const char* stage) {
    printf("\n--- DEBUG REG DUMP (%s) ---\n", stage);
    printf("  DMA_INTF: 0x%08lX\n", (unsigned long)DMA_INTF(DMA0));
    printf("  RX(CH3) CTL: 0x%08lX, CNT: %lu\n", (unsigned long)DMA_CHCTL(DMA0, DMA_CH3), (unsigned long)dma_transfer_number_get(DMA0, DMA_CH3));
    printf("  TX(CH4) CTL: 0x%08lX, CNT: %lu\n", (unsigned long)DMA_CHCTL(DMA0, DMA_CH4), (unsigned long)dma_transfer_number_get(DMA0, DMA_CH4));
    printf("  SPI1->CTL0=0x%08lX, CTL1=0x%08lX, STAT=0x%08lX\n", 
           (unsigned long)SPI_CTL0(SPI1), (unsigned long)SPI_CTL1(SPI1), (unsigned long)SPI_STAT(SPI1));
    printf("---------------------------------------\n");
}

/**
  * @brief  Robustly flushes the SPI FIFO and clears any overrun errors.
  */
void spi_fifo_flush(uint32_t spi_periph) {
    while(spi_i2s_flag_get(spi_periph, SPI_FLAG_TRANS));
    if(spi_i2s_flag_get(spi_periph, SPI_FLAG_RXORERR)) {
        (void)SPI_DATA(spi_periph);
        (void)SPI_STAT(spi_periph);
    }
    while(spi_i2s_flag_get(spi_periph, SPI_FLAG_RBNE)) {
        (void)SPI_DATA(spi_periph);
    }
}

/**
  * @brief  Transmits one byte and receives one byte via SPI polling.
  */
uint8_t spi_xchg_byte(uint32_t spi_periph, uint8_t data) {
    while(RESET == spi_i2s_flag_get(spi_periph, SPI_FLAG_TBE));
    spi_i2s_data_transmit(spi_periph, data);
    while(RESET == spi_i2s_flag_get(spi_periph, SPI_FLAG_RBNE));
    return (uint8_t)spi_i2s_data_receive(spi_periph);
}

/*
 * =============================================================================
 * --- Interrupt Service Routines ---
 * =============================================================================
 */

extern "C" void DMA0_Channel4_IRQHandler(void) {
    if(dma_interrupt_flag_get(DMA0, DMA_CH4, DMA_INT_FLAG_FTF)) {
        dma_interrupt_flag_clear(DMA0, DMA_CH4, DMA_INT_FLAG_G);
        dma_interrupt_disable(DMA0, DMA_CH4, DMA_INT_FTF);
        dma_tx_complete = 1;
        printf("\n---> DMA TX Interrupt Fired! <--- \n");
    }
}
extern "C" void DMA0_Channel3_IRQHandler(void) {
    if(dma_interrupt_flag_get(DMA0, DMA_CH3, DMA_INT_FLAG_FTF)) {
        dma_interrupt_flag_clear(DMA0, DMA_CH3, DMA_INT_FLAG_G);
        dma_interrupt_disable(DMA0, DMA_CH3, DMA_INT_FTF);
        dma_rx_complete = 1;
        printf("\n---> DMA RX Interrupt Fired! <--- \n");
    }
}

/*!
    \brief      main function
*/
int main(void) {
    printf("\n--- Asynchronous SD Card Test using DMA ---\n");
    memset(spi_rx_buffer, 0x55, RX_BUFFER_SIZE);

    /* --- STEP 1: Initialization --- */
    rcu_periph_clock_enable(RCU_GPIOB);
    rcu_periph_clock_enable(RCU_AF);
    rcu_periph_clock_enable(RCU_SPI1);
    rcu_periph_clock_enable(RCU_DMA0);
    gpio_init(GPIOB, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_12);
    gpio_init(GPIOB, GPIO_MODE_AF_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_13 | GPIO_PIN_15);
    gpio_init(GPIOB, GPIO_MODE_IPU, GPIO_OSPEED_50MHZ, GPIO_PIN_14);
    gpio_bit_set(GPIOB, GPIO_PIN_12);
    eclic_enable_interrupt(DMA0_Channel3_IRQn);
    eclic_enable_interrupt(DMA0_Channel4_IRQn);
    eclic_set_irq_priority(DMA0_Channel3_IRQn, 1);
    eclic_set_irq_priority(DMA0_Channel4_IRQn, 1);
    eclic_global_interrupt_enable();
    spi_parameter_struct spi_init_struct;
    spi_i2s_deinit(SPI1);
    spi_struct_para_init(&spi_init_struct);
    spi_init_struct.device_mode = SPI_MASTER;
    spi_init_struct.trans_mode = SPI_TRANSMODE_FULLDUPLEX;
    spi_init_struct.frame_size = SPI_FRAMESIZE_8BIT;
    spi_init_struct.nss = SPI_NSS_SOFT;
    spi_init_struct.endian = SPI_ENDIAN_MSB;
    spi_init_struct.clock_polarity_phase = SPI_CK_PL_HIGH_PH_2EDGE;
    spi_init_struct.prescale = SPI_PSC_256;
    spi_init(SPI1, &spi_init_struct);
    spi_enable(SPI1);
    printf("Step 1: All peripherals initialized.\n");

    /* --- STEP 1.5: SD Card SPI Mode Activation (Polling) --- */
    printf("\n--- Phase 1: Activating SD Card SPI Mode ---\n");
    gpio_bit_set(GPIOB, GPIO_PIN_12);
    for(int i = 0; i < 10; i++) {
        spi_xchg_byte(SPI1, 0xFF);
    }
    printf("Step 1.5: Sent 80 init clock cycles.\n");

    gpio_bit_reset(GPIOB, GPIO_PIN_12);
    for(int i=0; i<6; i++) {
        spi_xchg_byte(SPI1, cmd0_buffer[i]);
    }
    uint8_t response = 0xFF;
    int timeout = 10;
    while (((response = spi_xchg_byte(SPI1, 0xFF)) == 0xFF) && (timeout > 0)) {
        timeout--;
    }
    gpio_bit_set(GPIOB, GPIO_PIN_12);
    if (response != 0x01) {
        printf("!!! ERROR: CMD0 failed. Card did not enter idle state. Response: 0x%02X\n", response);
        while(1);
    }
    printf("Step 1.6: CMD0 successful. Card is in idle state.\n");
    
    /* --- STEP 2: Transmit CMD8 using DMA --- */
    printf("\n--- Phase 2: Transmitting CMD8 ---\n");
    dma_parameter_struct dma_init_struct;
    dma_deinit(DMA0, DMA_CH4);
    dma_struct_para_init(&dma_init_struct);
    dma_init_struct.periph_addr  = (uint32_t)&SPI_DATA(SPI1);
    dma_init_struct.memory_addr  = (uint32_t)cmd8_buffer;
    dma_init_struct.direction    = DMA_MEMORY_TO_PERIPHERAL;
    dma_init_struct.number       = TX_BUFFER_SIZE;
    dma_init_struct.periph_inc   = DMA_PERIPH_INCREASE_DISABLE;
    dma_init_struct.memory_inc   = DMA_MEMORY_INCREASE_ENABLE;
    dma_init(DMA0, DMA_CH4, &dma_init_struct);
    dma_interrupt_enable(DMA0, DMA_CH4, DMA_INT_FTF);
    
    spi_dma_enable(SPI1, SPI_DMA_TRANSMIT);
    print_debug_regs("Pre-TX");
    gpio_bit_reset(GPIOB, GPIO_PIN_12);
    dma_channel_enable(DMA0, DMA_CH4);
    
    printf("CMD8 sent. Waiting for DMA TX complete...\n");
    while (dma_tx_complete == 0) {}
    spi_fifo_flush(SPI1);
    dma_channel_disable(DMA0, DMA_CH4);
    spi_dma_disable(SPI1, SPI_DMA_TRANSMIT);
    print_debug_regs("Post-TX");

    /* --- STEP 3: Receive CMD8 Response using DMA --- */
    printf("\n--- Phase 3: Receiving response ---\n");
    dma_deinit(DMA0, DMA_CH3);
    dma_struct_para_init(&dma_init_struct);
    dma_init_struct.periph_addr  = (uint32_t)&SPI_DATA(SPI1);
    dma_init_struct.memory_addr  = (uint32_t)spi_rx_buffer;
    dma_init_struct.direction    = DMA_PERIPHERAL_TO_MEMORY;
    dma_init_struct.number       = RX_BUFFER_SIZE;
    dma_init_struct.priority     = DMA_PRIORITY_ULTRA_HIGH;
    dma_init_struct.memory_inc   = DMA_MEMORY_INCREASE_ENABLE;
    dma_init(DMA0, DMA_CH3, &dma_init_struct);
    dma_interrupt_enable(DMA0, DMA_CH3, DMA_INT_FTF);
    
    dma_deinit(DMA0, DMA_CH4);
    dma_init_struct.memory_addr = (uint32_t)&DUMMY_BYTE;
    dma_init_struct.number      = RX_BUFFER_SIZE;
    dma_init_struct.memory_inc  = DMA_MEMORY_INCREASE_DISABLE;
    dma_init(DMA0, DMA_CH4, &dma_init_struct);
    
    spi_dma_enable(SPI1, SPI_DMA_RECEIVE | SPI_DMA_TRANSMIT);
    dma_channel_enable(DMA0, DMA_CH3);
    dma_channel_enable(DMA0, DMA_CH4);
    
    printf("DMA receive started. Waiting for DMA RX complete or timeout...\n");
    print_debug_regs("Pre-RX");
    uint64_t start_time = get_timer_value();
    while (dma_rx_complete == 0) {
        if ((get_timer_value() - start_time) > (SystemCoreClock / 4)) {
            printf("\n!!! RX TIMEOUT !!!\n");
            print_debug_regs("RX HANG");
            break;
        }
    }
    
    gpio_bit_set(GPIOB, GPIO_PIN_12);
    spi_disable(SPI1);
    dma_channel_disable(DMA0, DMA_CH3);
    dma_channel_disable(DMA0, DMA_CH4);

    /* --- STEP 4: Print Results --- */
    printf("\n--- Phase 4: Results ---\n");
    print_buffer("Received Data", spi_rx_buffer, RX_BUFFER_SIZE);
    printf("\nExpected Response for SDv2 Card: First non-FF byte is 0x01, last 4 should be 00 00 01 AA.\n");
    printf("\nTest finished.\n");

    while(1) {}
}