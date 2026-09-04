/*!
    \file    main.cpp
    \brief   Main entry for SD Card library testing and performance benchmark.

    \version 2025-02-10, V1.6.0
*/

#include <cstdio>
#include "sd_card.h"
#include "sd_test.hpp"
#include "gpio.h" // Using your Gpio/Led class header

extern "C" {
#include "gd32vf103.h"
#include "n200_func.h"
#include "systick.h" // For delay functions
extern void enable_mcycle_minstret(void);
}
void print_debug_regs(const char* stage);

// Performance test configuration
#define POLLING_MODE_TEST
#define DMA_MODE_TEST
#define PERF_TEST_BLOCKS 127 

// Buffer for performance testing
uint8_t perf_buffer[512];

#include "sd_spi_hal.h" // Include the SD SPI HAL header


void hw_time_set(uint8_t unit)
{
    timer_parameter_struct timer_initpara;

    rcu_periph_clock_enable(RCU_TIMER2);
    timer_deinit(TIMER2);
  
    timer_initpara.period = 11999;

    timer_initpara.prescaler         = 7; // Adjust based on your system clock
    timer_initpara.alignedmode       = TIMER_COUNTER_EDGE;
    timer_initpara.counterdirection  = TIMER_COUNTER_UP;
    timer_initpara.clockdivision     = TIMER_CKDIV_DIV1;
    timer_initpara.repetitioncounter = 0;
    timer_init(TIMER2, &timer_initpara);
    
    timer_update_event_enable(TIMER2);
    timer_interrupt_enable(TIMER2,TIMER_INT_UP);
    timer_flag_clear(TIMER2, TIMER_FLAG_UP);
    timer_update_source_config(TIMER2, TIMER_UPDATE_SRC_GLOBAL);
  
    /* TIMER2 counter enable */
    timer_enable(TIMER2);
}

extern "C" void TIMER2_IRQHandler(void) {
    if (RESET != timer_interrupt_flag_get(TIMER3, TIMER_INT_UP)) {
        timer_interrupt_flag_clear(TIMER3, TIMER_INT_UP);
        printf("TIMER2 IRQ: delay_time");
    }
}


/*!
    \brief      Main function
    \param[in]  none
    \param[out] none
    \retval     none
*/
int main(void)
{
    printf("\n\n--- SD Card Library Test and Benchmark ---\n");
    //hal_spi_init();
    printf("System Clock: %lu Hz\n", rcu_clock_freq_get(CK_SYS));

    eclic_global_interrupt_enable();
    eclic_priority_group_set(ECLIC_PRIGROUP_LEVEL2_PRIO2);

    //printf("Configuring TIMER2 for delay...\n");
    //hw_time_set(1000); // Set timer for 1000 ms


    #if 1

    // The debug UART is automatically initialized by _init() in init.c
    printf("\n\n--- SD Card Library Test and Benchmark (Interrupt-driven) ---\n");
    
    // Re-enable the cycle counter which was disabled in init.c
    enable_mcycle_minstret();

    uint32_t system_clock = rcu_clock_freq_get(CK_SYS);
    printf("System Clock: %lu Hz\n", system_clock);

    // Initialize an LED to show that the CPU is not blocked during DMA
    Led activity_led(GPIOA, GPIO_PIN_1, true); // Green LED on PA1 is Active Low on some dev boards

    // Initialize the SD Card hardware
    printf("Attempting to initialize SD Card...\n");
    if (sd_init() & STA_NOINIT) {
        printf("ERROR: SD Card initialization failed or card not present.\n");
        printf("Test halted.\n");
        while(1);
    }
    printf("INFO: SD Card initialized successfully.\n");
    
    // Run diagnostic tests to ensure card is working
    SdCardTest test_runner(1000); // Use a safe, high sector number for tests
    bool diagnostics_passed = test_runner.run_tests();
    
    if (!diagnostics_passed) {
        printf("ERROR: SD card failed diagnostic tests. Performance benchmark will not run.\n");
        while(1);
    }
    
    // Run performance benchmark
    printf("\n--- Starting Performance Benchmark (%d blocks, %d KB) ---\n", PERF_TEST_BLOCKS, (PERF_TEST_BLOCKS * 512) / 1024);
    
    // Declare variables in the correct scope
    uint64_t start_cycle, end_cycle, duration;
    uint32_t total_bytes = PERF_TEST_BLOCKS * 512;

#ifdef POLLING_MODE_TEST
    printf("\nTesting Polling Mode...\n");
    
    // --- Polling Write Test ---
    start_cycle = get_cycle_value();
    for(uint32_t i = 0; i < PERF_TEST_BLOCKS; ++i) {
        sd_write_blocks(perf_buffer, 1000 + i, 1);
    }
    end_cycle = get_cycle_value();
    duration = end_cycle - start_cycle;

    if (duration > 0) {
        uint32_t duration_ms = static_cast<uint32_t>((duration * 1000) / system_clock);
        uint64_t speed_scaled = (static_cast<uint64_t>(total_bytes) * 1000) / (1024 * duration_ms);
        uint32_t speed_integer = static_cast<uint32_t>(speed_scaled);
        
        printf(" - Polling Write: %lu bytes in %lu ms -> %lu KB/s\n", 
               total_bytes, (unsigned long)duration_ms, (unsigned long)speed_integer);
    }

    // --- Polling Read Test ---
    start_cycle = get_cycle_value();
    for(uint32_t i = 0; i < PERF_TEST_BLOCKS; ++i) {
        sd_read_blocks(perf_buffer, 1000 + i, 1);
    }
    end_cycle = get_cycle_value();
    duration = end_cycle - start_cycle;

    if (duration > 0) {
        uint32_t duration_ms = static_cast<uint32_t>((duration * 1000) / system_clock);
        uint64_t speed_scaled = (static_cast<uint64_t>(total_bytes) * 1000) / (1024 * duration_ms);
        uint32_t speed_integer = static_cast<uint32_t>(speed_scaled);

        printf(" - Polling Read:  %lu bytes in %lu ms -> %lu KB/s\n", 
               total_bytes, (unsigned long)duration_ms, (unsigned long)speed_integer);
    }
#endif

#ifdef DMA_MODE_TEST
    printf("\nTesting DMA Mode (CPU will blink LED during transfer)...\n");

    // --- DMA Write Test ---
    start_cycle = get_cycle_value();
    if(sd_write_blocks_dma_start(perf_buffer, 1000, PERF_TEST_BLOCKS) == RES_OK){
        // Wait for completion while doing other work
        while(sd_dma_transfer_status() == RES_NOTRDY){
            activity_led.toggle();
            delay_1ms(500); // Use a small delay to avoid busy-waiting
        }
    }
    end_cycle = get_cycle_value();
    activity_led.off(); // Turn LED off when done
    // ... (rest of write test calculation) ...
    
    delay_1ms(100); // Small delay between tests

    // --- DMA Read Test ---
    start_cycle = get_cycle_value();
    if(sd_read_blocks_dma_start(perf_buffer, 1000, PERF_TEST_BLOCKS) == RES_OK){
        while(sd_dma_transfer_status() == RES_NOTRDY){
            activity_led.toggle();
            delay_1ms(50); // Use a small delay to avoid busy-waiting
        }
    }
    end_cycle = get_cycle_value();
    activity_led.off();
    // ... (rest of read test calculation) ...
#endif

    printf("\n--- Benchmark Finished ---\n");
#endif
    while(1);
}