/*!
    \file    main.c
    \brief   SPI1 simplex communication with SD card using DMA0 Channel 4.
             This is a sequential, bare-metal style example using the firmware library.

    \version 2025-02-10
*/

#include "gd32vf103.h"
#include <stdio.h>

/* Use core/ECLIC functions from the provided headers for interrupt control */
#include "n200_func.h"

/*
 * Define a simple command to send to the SD card.
 * CMD0 (GO_IDLE_STATE) is a good choice. It's 6 bytes long.
 * Byte 0: Command index (0x40 | 0)
 * Byte 1-4: Argument (0x00000000)
 * Byte 5: CRC (0x95 for CMD0 with 0 arg)
 */
#define CMD_BUFFER_SIZE 6
uint8_t spi_tx_buffer[CMD_BUFFER_SIZE] = {0x40, 0x00, 0x00, 0x00, 0x00, 0x95};

/*
 * This volatile flag is the communication mechanism between the ISR and main().
 * The ISR will set it to 1 when the transfer is complete.
 */
volatile int dma_transfer_complete = 0;

/**
  * @brief  DMA0 Channel 4 Interrupt Service Routine.
  * @note   This is the handler for SPI1_TX DMA completion.
  */
extern "C" void DMA0_Channel4_IRQHandler(void)
{
    if(dma_interrupt_flag_get(DMA0, DMA_CH4, DMA_INT_FLAG_FTF)) {
        /* Clear all interrupt flags for Channel 4 to prevent re-triggering */
        dma_interrupt_flag_clear(DMA0, DMA_CH4, DMA_INT_FLAG_G);

        /* Signal to the main loop that the transfer is complete */
        dma_transfer_complete = 1;
        
        printf("\n*** SUCCESS: DMA0_Channel4_IRQHandler was called! ***\n");
    }
}

/*!
    \brief      main function
    \param[in]  none
    \param[out] none
    \retval     none
*/
int main(void)
{
    /*
     * The system startup code (init.c) has already run, which calls SystemInit()
     * to set up the system clock (96MHz) and eclic_init() to prepare the
     * interrupt controller. The debug UART (USART0) is also initialized.
     */
    printf("\n--- Library-based SPI1 DMA Transmit Test ---\n");

    /*
     * STEP 1: Enable peripheral clocks (RCU configuration)
     */
    rcu_periph_clock_enable(RCU_GPIOB);
    rcu_periph_clock_enable(RCU_AF);
    rcu_periph_clock_enable(RCU_SPI1);
    rcu_periph_clock_enable(RCU_DMA0);
    printf("Step 1: RCU clocks enabled.\n");

    /*
     * STEP 2: Configure GPIO pins for SPI1
     * - PB12 (NSS):  Software-controlled GPIO Output Push-Pull.
     * - PB13 (SCK):  Alternate Function Push-Pull.
     * - PB14 (MISO): Input Floating.
     * - PB15 (MOSI): Alternate Function Push-Pull.
     */
    gpio_init(GPIOB, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_12);
    gpio_init(GPIOB, GPIO_MODE_AF_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_13 | GPIO_PIN_15);
    gpio_init(GPIOB, GPIO_MODE_IN_FLOATING, GPIO_OSPEED_50MHZ, GPIO_PIN_14);

    /* Set NSS high (inactive) initially */
    gpio_bit_set(GPIOB, GPIO_PIN_12);
    printf("Step 2: GPIO pins for SPI1 configured.\n");

    /*
     * STEP 3: Configure Interrupt Controller (ECLIC) for DMA
     * The ECLIC was initialized in _init(), but we must enable our specific interrupt.
     */
    eclic_enable_interrupt(DMA0_Channel4_IRQn);
    eclic_set_irq_priority(DMA0_Channel4_IRQn, 1);
    eclic_global_interrupt_enable();; // Globally enable machine-level interrupts (sets mstatus.MIE)
    printf("Step 3: ECLIC interrupt for DMA0_CH4 enabled.\n");

    /*
     * STEP 4: Configure SPI1 peripheral for SD Card communication
     */
    spi_parameter_struct spi_init_struct;
    spi_i2s_deinit(SPI1); // Reset SPI1 to a known state
    spi_struct_para_init(&spi_init_struct);

    spi_init_struct.device_mode          = SPI_MASTER;
    spi_init_struct.trans_mode           = SPI_TRANSMODE_FULLDUPLEX;
    spi_init_struct.frame_size           = SPI_FRAMESIZE_8BIT;
    spi_init_struct.nss                  = SPI_NSS_SOFT;
    spi_init_struct.endian               = SPI_ENDIAN_MSB;
    // SD cards often use Mode 0 or Mode 3. We use Mode 3 as in the original driver.
    spi_init_struct.clock_polarity_phase = SPI_CK_PL_HIGH_PH_2EDGE; 
    // Start with a slow clock for initialization commands
    spi_init_struct.prescale             = SPI_PSC_256;
    spi_init(SPI1, &spi_init_struct);
    
    printf("Step 4: SPI1 peripheral configured as master.\n");
    
    /*
     * STEP 5: Configure DMA0 Channel 4 for Memory-to-Peripheral transfer
     */
    dma_parameter_struct dma_init_struct;
    dma_deinit(DMA0, DMA_CH4); // Reset DMA channel to a known state
    dma_struct_para_init(&dma_init_struct);

    dma_init_struct.periph_addr  = (uint32_t)&SPI_DATA(SPI1);
    dma_init_struct.memory_addr  = (uint32_t)spi_tx_buffer;
    dma_init_struct.direction    = DMA_MEMORY_TO_PERIPHERAL;
    dma_init_struct.memory_width = DMA_MEMORY_WIDTH_8BIT;
    dma_init_struct.periph_width = DMA_PERIPHERAL_WIDTH_8BIT;
    dma_init_struct.priority     = DMA_PRIORITY_HIGH;
    dma_init_struct.number       = CMD_BUFFER_SIZE;
    dma_init_struct.periph_inc   = DMA_PERIPH_INCREASE_DISABLE;
    dma_init_struct.memory_inc   = DMA_MEMORY_INCREASE_ENABLE;
    dma_init(DMA0, DMA_CH4, &dma_init_struct);

    /* Configure DMA circulation mode */
    dma_circulation_disable(DMA0, DMA_CH4);
    dma_memory_to_memory_disable(DMA0, DMA_CH4);

    /* Enable the Full Transfer Finish interrupt for the channel */
    dma_interrupt_enable(DMA0, DMA_CH4, DMA_INT_FTF);
    
    printf("Step 5: DMA0_CH4 configured for SPI1_TX.\n");

    /*
     * STEP 6: Enable SPI and start the DMA transfer
     */
    /* Enable the SPI DMA requests for transmission */
    spi_dma_enable(SPI1, SPI_DMA_TRANSMIT);

    /* Enable the SPI peripheral itself */
    spi_enable(SPI1);
    
    /* Pull NSS line low to select the SD card */
    gpio_bit_reset(GPIOB, GPIO_PIN_12);

    /* Enable DMA Channel 4, which starts the transfer immediately */
    dma_channel_enable(DMA0, DMA_CH4);
    
    printf("Step 6: SPI1 enabled, NSS low, DMA transfer started. Waiting for interrupt...\n");

    /*
     * STEP 7: Wait for the transfer to complete
     * The program will loop here until the ISR sets dma_transfer_complete to 1.
     */
    while (dma_transfer_complete == 0) {
        // The CPU is free to do other things here.
    }

    /*
     * STEP 8: Cleanup and finish
     */
    
    /* Wait for the last byte to finish shifting out of the SPI peripheral */
    while(spi_i2s_flag_get(SPI1, SPI_FLAG_TRANS));

    /* Pull NSS line high to de-select the slave */
    gpio_bit_set(GPIOB, GPIO_PIN_12);
    
    /* Disable peripherals */
    spi_disable(SPI1);
    dma_channel_disable(DMA0, DMA_CH4);
    spi_dma_disable(SPI1, SPI_DMA_TRANSMIT);

    printf("Step 8: Transfer complete and peripherals disabled.\n\n");
    printf("Test finished successfully.\n");

    /* Infinite loop */
    while(1) {
    }
}