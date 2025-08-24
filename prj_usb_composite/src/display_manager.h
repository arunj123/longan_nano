#ifndef DISPLAY_MANAGER_H
#define DISPLAY_MANAGER_H

#include <cstdint>
#include <array>
#include <variant>
#include <cstddef>

namespace display {

/**
 * @brief Contains compile-time constants for the display and buffer management.
 */
namespace constants {
    constexpr size_t LcdWidth = 160;
    constexpr size_t LcdHeight = 80;

    // Buffer configuration ---
    constexpr size_t NumBuffers = 4;
    constexpr size_t BufferSizeBytes = 4096;
    constexpr size_t MaxPixelsPerBuffer = BufferSizeBytes / 2; // Each pixel is 2 bytes (RGB555)
}

/**
 * @brief Type-safe enumeration for commands received from the host.
 *        CMD_START_QUADRANT_TRANSFER has been removed.
 */
enum class HostCommand : uint8_t {
    IMAGE_DATA = 0x02,
    DRAW_RECT = 0x06,
};

/**
 * @brief A simple struct to hold rectangle geometry.
 */
struct Rect {
    uint8_t x, y, w, h;
};

// State for each slot in the circular buffer ---
enum class BufferState {
    EMPTY,
    RECEIVING,
    READY_TO_DRAW
};

// Structure to hold all metadata for a single draw task ---
struct DrawTask {
    BufferState state = BufferState::EMPTY;
    Rect region = {0, 0, 0, 0};
    uint32_t bytes_received = 0;
    uint32_t total_bytes_expected = 0;
    uint16_t sequence_number = 0;
};

/**
 * @class DisplayManager
 * @brief Manages the LCD framebuffers, USB data reception, and drawing tasks using a circular buffer.
 */
class DisplayManager {
public:
    DisplayManager(const DisplayManager&) = delete;
    DisplayManager& operator=(const DisplayManager&) = delete;
    static DisplayManager& getInstance();

    void handleUsbPacket(const uint8_t* data, uint32_t len);
    void processDrawTasks();

private:
    DisplayManager() = default;

    // Replaced single task with a circular buffer of tasks and framebuffers ---
    
    // An array of tasks, one for each buffer slot
    std::array<DrawTask, constants::NumBuffers> m_draw_tasks;

    // The framebuffers, one for each task slot
    std::array<std::array<uint8_t, constants::BufferSizeBytes>, constants::NumBuffers> m_framebuffers;

    // Volatile indices for safe ISR/main-loop interaction
    volatile uint8_t m_usb_head_idx = 0; // Index for the ISR to write to
    volatile uint8_t m_dma_tail_idx = 0; // Index for the main loop to draw from
    
    // Sequence number tracking ---
    uint16_t m_expected_sequence_num = 0;
};

} // namespace display

#endif // DISPLAY_MANAGER_H