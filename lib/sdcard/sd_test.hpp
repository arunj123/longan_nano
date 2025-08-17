/*!
    \file    sd_test.hpp
    \brief   Header for a simple, non-destructive SD Card test class.

    \version 2025-02-10
*/

#ifndef SD_TEST_HPP
#define SD_TEST_HPP

#include <cstdint>

/**
 * @class SdCardTest
 * @brief Encapsulates a set of diagnostic tests for an SD card.
 *
 * This class performs a basic read/write verification on a single sector
 * of the SD card to ensure the interface and the card itself are functional.
 * It is designed to be non-destructive by saving and restoring the original
 * contents of the test sector.
 */
class SdCardTest {
public:
    /**
     * @brief Constructor for the SdCardTest class.
     * @param test_sector The logical block address (LBA) of the sector to use for testing.
     *                    It's recommended to use a high-numbered sector that is unlikely
     *                    to contain critical boot or filesystem data.
     */
    explicit SdCardTest(uint32_t test_sector = 1000);

    /**
     * @brief Executes the full suite of SD card tests.
     * @return true if all tests pass, false otherwise.
     */
    bool run_tests();

private:
    /**
     * @brief Verifies that the SD card is present and initialized.
     * @return true if the card is ready, false otherwise.
     */
    bool check_initialization();

    /**
     * @brief Performs a read, write, verify, and restore cycle on the test sector.
     * @return true if the test passes, false otherwise.
     */
    bool run_read_write_test();

    // Member variables
    uint32_t _test_sector;
    bool _is_initialized;

    // A 512-byte buffer is standard for SD card sectors
    static constexpr uint16_t SECTOR_SIZE = 512;
    uint8_t _original_data_buffer[SECTOR_SIZE];
    uint8_t _test_pattern_buffer[SECTOR_SIZE];
};

#endif // SD_TEST_HPP