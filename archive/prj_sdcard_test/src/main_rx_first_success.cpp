/*!
    \file    main.cpp
    \brief   SPI1 command-response transaction with an SD card.
             This is the final, working version implementing the correct SD card init sequence.

    \version 2025-02-10, V4.0 (Final)
*/

#include "gd32vf103.h"
#include <stdio.h>
#include <string.h>

extern "C" {
#include "n200_func.h"
}

/* --- Buffers and State Flags --- */
/* CMD0 (GO_IDLE_STATE) */
uint8_t cmd0_buffer[6] = {0x40 | 0, 0x00, 0x00, 0x00, 0x00, 0x95};
/* CMD8 (SEND_IF_COND) */
uint8_t cmd8_buffer[6] = {0x40 | 8, 0x00, 0x00, 0x01, 0xAA, 0x87};

#define RX_BUFFER_SIZE 8
uint8_t spi_rx_buffer[RX_BUFFER_SIZE];
volatile int dma_tx_complete = 0;

/* --- Interrupt Service Routines --- */
extern "C" void DMA0_Channel4_IRQHandler(void) {
    if(dma_interrupt_flag_get(DMA0, DMA_CH4, DMA_INT_FLAG_FTF)) {
        dma_interrupt_flag_clear(DMA0, DMA_CH4, DMA_INT_FLAG_G);
        dma_interrupt_disable(DMA0, DMA_CH4, DMA_INT_FTF);
        dma_tx_complete = 1;
        printf("\n---> DMA TX Interrupt Fired! <--- \n");
    }
}
extern "C" void DMA0_Channel3_IRQHandler(void) {} // Not used, but must exist

/**
  * @brief  Transmits one byte and receives one byte via SPI polling.
  */
uint8_t spi_xchg_byte(uint32_t spi_periph, uint8_t data) {
    while(RESET == spi_i2s_flag_get(spi_periph, SPI_FLAG_TBE));
    spi_i2s_data_transmit(spi_periph, data);
    while(RESET == spi_i2s_flag_get(spi_periph, SPI_FLAG_RBNE));
    return (uint8_t)spi_i2s_data_receive(spi_periph);
}

/*!
    \brief      main function
*/
int main(void) {
    printf("\n--- Library-based SPI1 DMA TX / Polling RX Test ---\n");
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
    eclic_enable_interrupt(DMA0_Channel4_IRQn);
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

    /* --- STEP 1.5: SD Card SPI Mode Activation --- */
    printf("\n--- Phase 1: Activating SD Card SPI Mode ---\n");
    gpio_bit_set(GPIOB, GPIO_PIN_12); // Ensure CS is HIGH
    for(int i = 0; i < 10; i++) {
        spi_xchg_byte(SPI1, 0xFF); // Send 80 clock cycles
    }
    printf("Step 1.5: Sent 80 init clock cycles.\n");

    /* *** THE FINAL FIX: Send CMD0 to enter Idle State *** */
    gpio_bit_reset(GPIOB, GPIO_PIN_12); // Set CS LOW
    for(int i=0; i<6; i++) {
        spi_xchg_byte(SPI1, cmd0_buffer[i]);
    }
    
    uint8_t response = 0xFF;
    int timeout = 10;
    while (((response = spi_xchg_byte(SPI1, 0xFF)) == 0xFF) && (timeout > 0)) {
        timeout--;
    }
    gpio_bit_set(GPIOB, GPIO_PIN_12); // Set CS HIGH
    spi_xchg_byte(SPI1, 0xFF); // Extra clock cycle
    
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
    dma_init_struct.memory_addr  = (uint32_t)cmd8_buffer; // Use CMD8 buffer
    dma_init_struct.direction    = DMA_MEMORY_TO_PERIPHERAL;
    dma_init_struct.number       = TX_BUFFER_SIZE;
    dma_init(DMA0, DMA_CH4, &dma_init_struct);
    dma_interrupt_enable(DMA0, DMA_CH4, DMA_INT_FTF);
    
    spi_dma_enable(SPI1, SPI_DMA_TRANSMIT);
    gpio_bit_reset(GPIOB, GPIO_PIN_12);
    dma_channel_enable(DMA0, DMA_CH4);
    
    printf("CMD8 sent. Waiting for DMA TX complete...\n");
    while (dma_tx_complete == 0) {}
    while(spi_i2s_flag_get(SPI1, SPI_FLAG_TRANS));
    dma_channel_disable(DMA0, DMA_CH4);
    spi_dma_disable(SPI1, SPI_DMA_TRANSMIT);

    /* --- STEP 3: Receive CMD8 Response using Polling --- */
    printf("\n--- Phase 3: Receiving response via Polling ---\n");
    
    response = 0xFF;
    timeout = 10;
    while (((response = spi_xchg_byte(SPI1, 0xFF)) == 0xFF) && (timeout > 0)) {
        timeout--;
    }

    if (timeout > 0) {
        printf("Found start of response: 0x%02X\n", response);
        spi_rx_buffer[0] = response;
        for (int i = 1; i < 5; i++) {
            spi_rx_buffer[i] = spi_xchg_byte(SPI1, 0xFF);
        }
    } else {
        printf("!!! ERROR: Timed out waiting for CMD8 response. !!!\n");
    }
    
    gpio_bit_set(GPIOB, GPIO_PIN_12);
    spi_disable(SPI1);

    /* --- STEP 4: Print Results --- */
    printf("\n--- Phase 4: Results ---\n");
    printf("Received %d bytes:\n", RX_BUFFER_SIZE);
    for(int i = 0; i < RX_BUFFER_SIZE; i++) {
        printf("  Byte %d: 0x%02X\n", i, spi_rx_buffer[i]);
    }
    printf("\nExpected Response for SDv2 Card: First byte should be 0x01, last 4 should be 00 00 01 AA.\n");
    printf("\nTest finished.\n");

    while(1) {}
}