#ifndef DISPLAY_MANAGER_H
#define DISPLAY_MANAGER_H

#include <cstdint>
#include <array>
#include <variant>
#include <cstddef>
#include "hal/time.hpp"

namespace display {

/**
 * @brief Contains compile-time constants for the display and buffer management.
 */
namespace constants {
    constexpr size_t LcdWidth = 160;
    constexpr size_t LcdHeight = 80;

    // 3-buffer circular queue for smooth pipelined rendering without starvation
    constexpr size_t NumBuffers = 3;
    constexpr size_t BufferSizeBytes = 4096;
    constexpr size_t MaxPixelsPerBuffer = BufferSizeBytes / 2; // Each pixel is 2 bytes (RGB565)
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
    READY_TO_DRAW,
    DRAWING
};

// Structure to hold all metadata for a single draw task ---
struct DrawTask {
    volatile BufferState state = BufferState::EMPTY;
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

    // The framebuffers, one for each task slot (alignas(4) for DMA 16-bit/32-bit alignment)
    alignas(4) std::array<std::array<uint8_t, constants::BufferSizeBytes>, constants::NumBuffers> m_framebuffers;

    // Volatile indices for safe ISR/main-loop interaction
    volatile uint8_t m_usb_head_idx = 0; // Index for the ISR to write to
    volatile uint8_t m_dma_tail_idx = 0; // Index for the main loop to draw from
    
    // Sequence number tracking ---
    uint16_t m_expected_sequence_num = 0;

    // Diagnostic logging flags for safe reporting from the main loop
    volatile bool     m_dbg_rect_received = false;
    volatile bool     m_dbg_rect_dropped = false;
    volatile bool     m_dbg_frame_aborted = false;
    volatile uint8_t  m_dbg_unknown_cmd = 0;
    volatile uint32_t m_dbg_dropped_data_pkts = 0;
    volatile uint32_t m_dbg_received_data_pkts = 0;
    Rect              m_dbg_last_rect = {0, 0, 0, 0};
    uint16_t          m_dbg_last_seq = 0;

    hal::time::Instant m_last_rx_time = hal::time::Instant::now();
};

} // namespace display

#endif // DISPLAY_MANAGER_H