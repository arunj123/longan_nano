/*!
    \file    main.c
    \brief   SD card read/write test using the tf_card.c driver.

    \version 2025-02-10
*/
#include "gd32vf103.h"
#include <stdio.h>
#include <string.h>
#include "tf_card.h" // Include the header for our driver


#define TEST_SECTOR   1000  // A safe, high sector number for the test
#define BLOCK_SIZE    512

// A buffer to hold one sector of data
uint8_t sector_buffer[BLOCK_SIZE];

/**
  * @brief  Prints the contents of a buffer in a readable hexdump format.
  * @param  buff: Pointer to the buffer.
  * @param  len: Length of the buffer to print.
  */
void print_buffer(const uint8_t* buff, uint32_t len)
{
    for (uint32_t i = 0; i < len; i++) {
        if (i % 16 == 0) {
            printf("\n  0x%04lX: ", i);
        }
        printf("%02X ", buff[i]);
    }
    printf("\n");
}

/**
  * @brief  Initializes the hardware timer required by tf_card.c
  * @note   This configures TIMER2 to fire an interrupt every 1ms.
  */
void timer_for_tf_card_config(void)
{
    timer_parameter_struct timer_initpara;

    rcu_periph_clock_enable(RCU_TIMER2);
    timer_deinit(TIMER2);
    
    // Configure for a 1ms tick (1000 Hz)
    // Formula: Period = (SystemCoreClock / Prescaler) / 1000 Hz
    // With SystemCoreClock/4 = 24MHz, a prescaler of 24000 gives a 1kHz tick.
    timer_initpara.prescaler         = (rcu_clock_freq_get(CK_APB1) / 1000) - 1;
    timer_initpara.period            = 1 - 1; // Tick on every prescaler overflow
    timer_initpara.alignedmode       = TIMER_COUNTER_EDGE;
    timer_initpara.counterdirection  = TIMER_COUNTER_UP;
    timer_initpara.clockdivision     = TIMER_CKDIV_DIV1;
    timer_initpara.repetitioncounter = 0;
    timer_init(TIMER2, &timer_initpara);

    timer_interrupt_enable(TIMER2, TIMER_INT_UP);
    timer_enable(TIMER2);
}

int main(void)
{
    DSTATUS status;
    DRESULT result;

    /*
     * The system startup code (init.c) has already run, which calls SystemInit()
     * to set up the system clock (96MHz) and eclic_init() to prepare the
     * interrupt controller. The debug UART (USART0) is also initialized.
     */
    printf("\n--- SD Card Read/Write Test using tf_card.c Driver ---\n");

    /* Initialize the 1ms timer required by the SD card driver for timeouts */
    timer_for_tf_card_config();
    eclic_global_interrupt_enable();

    /* --- STEP 1: Initialize the SD Card --- */
    printf("Initializing SD card...\n");
    status = disk_initialize(0);

    if (status & STA_NOINIT) {
        printf("ERROR: SD card initialization failed!\n");
        printf("Check wiring and ensure a card is inserted.\n");
        while(1);
    }
    printf("SUCCESS: SD card initialized.\n\n");

    for(int i = 0; i < 5; i++) {
        sector_buffer[i] = (uint8_t)0x0; // Fill the buffer with test data
            /* --- STEP 2: Read one sector from the card --- */
        printf("--- Phase 1: Reading sector %d ---\n", i);
        result = disk_read(0, sector_buffer, i, 1);

        if (result != RES_OK) {
            printf("ERROR: Failed to read sector %d. Result code: %d\n", i, result);
            while(1);
        }
        printf("SUCCESS: Sector %d read.\n", i);
        printf("Buffer contents after read:");
        print_buffer(sector_buffer, BLOCK_SIZE);
    }

    printf("--- Phase 1: Reading sector %d ---\n", TEST_SECTOR);
    result = disk_read(0, sector_buffer, TEST_SECTOR, 1);

    if (result != RES_OK) {
        printf("ERROR: Failed to read sector %d. Result code: %d\n", TEST_SECTOR, result);
        while(1);
    }
    printf("SUCCESS: Sector %d read.\n", TEST_SECTOR);
    printf("Buffer contents after read:");
    print_buffer(sector_buffer, BLOCK_SIZE);



    /* --- STEP 3: Write the same data back to the sector --- */
    printf("\n--- Phase 2: Writing data back to sector %d ---\n", TEST_SECTOR);
    result = disk_write(0, sector_buffer, TEST_SECTOR, 1);

    if (result != RES_OK) {
        printf("!!! CRITICAL ERROR: Failed to write sector back! Sector %d may be corrupted. Result code: %d\n", TEST_SECTOR, result);
        while(1);
    }
    printf("SUCCESS: Data written back to sector %d.\n", TEST_SECTOR);
    printf("This ensures the original data was preserved.\n");

    printf("\n--- Test Finished Successfully ---\n");

    while(1) {
    }
}