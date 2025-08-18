/*!
    \file    main.cpp
    \brief   Main entry for SD Card library testing and performance benchmark.

    \version 2025-02-10, V1.5.8
*/

#include <cstdio>
#include "sd_card.h"
#include "sd_test.hpp"
#include "gpio.h"

extern "C" {
#include "gd32vf103.h"
#include "n200_func.h"
}
extern "C" void enable_mcycle_minstret(void);

#define POLLING_MODE_TEST
#define DMA_MODE_TEST
#define PERF_TEST_BLOCKS 128
uint8_t perf_buffer[512];

int main(void) {
    printf("\n\n--- SD Card Library Test and Benchmark (Interrupt-driven) ---\n");
    enable_mcycle_minstret();
    uint32_t system_clock = rcu_clock_freq_get(CK_SYS);
    printf("System Clock: %lu Hz\n", system_clock);

    Led activity_led(GPIOA, GPIO_PIN_1, true);

    if (sd_init() & STA_NOINIT) {
        printf("ERROR: SD Card initialization failed.\n");
        while(1);
    }
    printf("INFO: SD Card initialized successfully.\n");
    
    SdCardTest test_runner(1000);
    if (!test_runner.run_tests()) {
        printf("ERROR: SD card failed diagnostic tests.\n");
        while(1);
    }
    
    printf("\n--- Starting Performance Benchmark (%d blocks, %d KB) ---\n", PERF_TEST_BLOCKS, (PERF_TEST_BLOCKS * 512) / 1024);
    
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
        while(sd_dma_transfer_status() == RES_NOTRDY){
            activity_led.toggle();
            for(int i=0; i<50000; i++) __asm__ __volatile__("nop");
        }
    }
    end_cycle = get_cycle_value();
    activity_led.off();
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
    if(sd_read_blocks_dma_start(perf_buffer, 1000, PERF_TEST_BLOCKS) == RES_OK){
        while(sd_dma_transfer_status() == RES_NOTRDY){
            activity_led.toggle();
            for(int i=0; i<50000; i++) __asm__ __volatile__("nop");
        }
    }
    end_cycle = get_cycle_value();
    activity_led.off();
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