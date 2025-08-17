/*!
    \file    main.cpp
    \brief   Main entry for SD Card library testing and performance benchmark.

    \version 2025-02-10, V1.5.4
*/

// C++ standard library headers
#include <cstdio>

// Project-specific C++ headers
#include "sd_card.h"
#include "sd_test.hpp"

// C library headers for GD32VF103.
extern "C" {
#include "gd32vf103.h"
#include "n200_func.h" // Contains get_cycle_value() and enable_mcycle_minstret()
}
extern "C" void enable_mcycle_minstret(void);

// Performance test configuration
#define POLLING_MODE_TEST
#define DMA_MODE_TEST
#define PERF_TEST_BLOCKS 128 // Number of 512-byte blocks to test (128 blocks = 64 KB)

// Buffer for performance testing
uint8_t perf_buffer[512];

/*!
    \brief      Main function
    \param[in]  none
    \param[out] none
    \retval     none
*/
int main(void)
{
    // The debug UART is automatically initialized by _init() in init.c
    printf("\n\n--- SD Card Library Test and Benchmark for Longan Nano ---\n");
    
    // *** THE FIX: Re-enable the cycle counter which was disabled in init.c ***
    enable_mcycle_minstret();

    uint32_t system_clock = rcu_clock_freq_get(CK_SYS);
    printf("System Clock: %lu Hz\n", system_clock);

    // Initialize the SD Card hardware
    printf("Attempting to initialize SD Card...\n");
    if (sd_init() & STA_NOINIT) {
        printf("ERROR: SD Card initialization failed or card not present.\n");
        printf("Test halted.\n");
        while(1);
    }
    printf("INFO: SD Card initialized successfully.\n");
    
    // Run diagnostic tests
    SdCardTest test_runner(1000);
    bool diagnostics_passed = test_runner.run_tests();
    
    if (!diagnostics_passed) {
        printf("ERROR: SD card failed diagnostic tests. Performance benchmark will not run.\n");
        while(1);
    }
    
    // Run performance benchmark
    printf("\n--- Starting Performance Benchmark (%d blocks, %d KB) ---\n", PERF_TEST_BLOCKS, (PERF_TEST_BLOCKS * 512) / 1024);
    
    uint64_t start_cycle, end_cycle, duration;
    double seconds, speed_kbs;
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
    printf("\nTesting DMA Mode...\n");

    // --- DMA Write Test ---
    start_cycle = get_cycle_value();
    sd_write_blocks_dma(perf_buffer, 1000, PERF_TEST_BLOCKS);
    end_cycle = get_cycle_value();
    duration = end_cycle - start_cycle;

    if (duration > 0) {
        uint32_t duration_ms = static_cast<uint32_t>((duration * 1000) / system_clock);
        uint64_t speed_scaled = (static_cast<uint64_t>(total_bytes) * 1000) / (1024 * duration_ms);
        uint32_t speed_integer = static_cast<uint32_t>(speed_scaled);

        printf(" - DMA Write:     %lu bytes in %lu ms -> %lu KB/s\n", 
               total_bytes, (unsigned long)duration_ms, (unsigned long)speed_integer);
    }
    
    // --- DMA Read Test ---
    start_cycle = get_cycle_value();
    sd_read_blocks_dma(perf_buffer, 1000, PERF_TEST_BLOCKS);
    end_cycle = get_cycle_value();
    duration = end_cycle - start_cycle;

    if (duration > 0) {
        uint32_t duration_ms = static_cast<uint32_t>((duration * 1000) / system_clock);
        uint64_t speed_scaled = (static_cast<uint64_t>(total_bytes) * 1000) / (1024 * duration_ms);
        uint32_t speed_integer = static_cast<uint32_t>(speed_scaled);

        printf(" - DMA Read:      %lu bytes in %lu ms -> %lu KB/s\n", 
               total_bytes, (unsigned long)duration_ms, (unsigned long)speed_integer);
    }
#endif

    printf("\n--- Benchmark Finished ---\n");

    while(1);
}