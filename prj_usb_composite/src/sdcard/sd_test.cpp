/*!
    \file    sd_test.cpp
    \brief   Implementation of the SD Card test class.

    \version 2025-02-10
*/

#include "sd_test.hpp"
#include "sd_card.h" // For sd_status, sd_read_blocks, sd_write_blocks
#include <cstdio>      // For printf
#include <cstring>     // For memset and memcmp

SdCardTest::SdCardTest(uint32_t test_sector)
    : _test_sector(test_sector), _is_initialized(false) {
    // Constructor initializes the test sector and state
}

bool SdCardTest::run_tests() {
    printf("\n--- Starting SD Card Diagnostics ---\n");

    if (!check_initialization()) {
        // The check_initialization function will print the specific reason.
        printf("--- SD Card Diagnostics FAILED ---\n");
        return false;
    }

    if (!run_read_write_test()) {
        printf("--- SD Card Diagnostics FAILED ---\n");
        return false;
    }

    printf("--- SD Card Diagnostics PASSED ---\n\n");
    return true;
}

bool SdCardTest::check_initialization() {
    printf("1. Checking card status... ");
    DSTATUS status = sd_status();

    if (status & STA_NODISK) {
        printf("FAILED. No SD card detected.\n");
        return false;
    }

    if (status & STA_NOINIT) {
        printf("FAILED. Card is not initialized.\n");
        return false;
    }
    
    _is_initialized = true;
    printf("OK.\n");
    return true;
}

bool SdCardTest::run_read_write_test() {
    printf("2. Performing Read/Write test on Sector %lu...\n", _test_sector);

    // --- Step 1: Backup the original sector data ---
    printf("   - Backing up original sector data... ");
    if (sd_read_blocks(_original_data_buffer, _test_sector, 1) != RES_OK) {
        printf("FAILED (Read Error).\n");
        return false;
    }
    printf("OK.\n");

    // --- Step 2: Create and write a test pattern ---
    printf("   - Writing test pattern... ");
    // Fill buffer with a non-trivial pattern
    for (uint16_t i = 0; i < SECTOR_SIZE; ++i) {
        _test_pattern_buffer[i] = static_cast<uint8_t>(i % 256);
    }
    if (sd_write_blocks(_test_pattern_buffer, _test_sector, 1) != RES_OK) {
        printf("FAILED (Write Error).\n");
        // Attempt to restore the original data even if the write failed
        printf("   - Attempting to restore original data... ");
        sd_write_blocks(_original_data_buffer, _test_sector, 1);
        printf("Done.\n");
        return false;
    }
    printf("OK.\n");

    // --- Step 3: Read the data back ---
    printf("   - Reading back test pattern... ");
    // Clear the buffer before reading back to ensure a valid test
    uint8_t read_back_buffer[SECTOR_SIZE];
    memset(read_back_buffer, 0, SECTOR_SIZE);
    if (sd_read_blocks(read_back_buffer, _test_sector, 1) != RES_OK) {
        printf("FAILED (Read Error).\n");
        return false;
    }
    printf("OK.\n");

    // --- Step 4: Verify the data ---
    printf("   - Verifying data integrity... ");
    if (memcmp(_test_pattern_buffer, read_back_buffer, SECTOR_SIZE) != 0) {
        printf("FAILED (Data Mismatch).\n");
        // The data is corrupt, but we must still try to restore the original
    } else {
        printf("OK.\n");
    }

    // --- Step 5: CRITICAL - Restore the original sector data ---
    printf("   - Restoring original sector data... ");
    if (sd_write_blocks(_original_data_buffer, _test_sector, 1) != RES_OK) {
        printf("FAILED (Restore Write Error). SECTOR %lu MAY BE CORRUPT!\n", _test_sector);
        // This is a critical failure, but we continue to return the test result
        return false;
    }
    printf("OK.\n");

    return true;
}