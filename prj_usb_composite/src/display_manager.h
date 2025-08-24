#ifndef DISPLAY_MANAGER_H
#define DISPLAY_MANAGER_H

#include <cstdint>
#include <array>
#include <variant>

namespace display {

/**
 * @brief Contains compile-time constants related to the LCD geometry.
 */
namespace constants {
    constexpr int LcdWidth = 160;
    constexpr int LcdHeight = 80;
    constexpr int NumQuadrants = 4;
    constexpr int QuadrantHeight = LcdHeight / NumQuadrants;
    constexpr int QuadrantPixels = LcdWidth * QuadrantHeight;
    constexpr int QuadrantBytes = QuadrantPixels * 2;
}

/**
 * @brief Type-safe enumeration for commands received from the host.
 */
enum class HostCommand : uint8_t {
    START_QUADRANT_TRANSFER = 0x05,
    IMAGE_DATA = 0x02,
    DRAW_RECT = 0x06,
};

/**
 * @brief A simple struct to hold rectangle geometry.
 */
struct Rect {
    uint8_t x, y, w, h;
};

/**
 * @brief Represents a task to draw a full quadrant.
 */
struct QuadrantUpdate {
    int quadrant_index;
};

/**
 * @brief Represents a task to draw a partial rectangle.
 */
struct PartialUpdate {
    Rect region;
};

/**
 * @class DisplayManager
 * @brief Manages the LCD framebuffers, USB data reception, and drawing tasks.
 */
class DisplayManager {
public:
    DisplayManager(const DisplayManager&) = delete;
    DisplayManager& operator=(const DisplayManager&) = delete;
    static DisplayManager& getInstance();

    void handleUsbPacket(const uint8_t* data, uint32_t len);
    void processDrawTasks();

private:
    // --- FIX: All member variables are now correctly placed INSIDE the class definition ---
    DisplayManager() = default;

    // The single-element queue for the next draw task.
    std::variant<std::monostate, QuadrantUpdate, PartialUpdate> m_draw_task;

    // Synchronization flag between ISR and main loop.
    volatile bool m_task_is_ready = false;

    // Double framebuffers.
    std::array<std::array<uint8_t, constants::QuadrantBytes>, 2> m_framebuffers;
    volatile int m_usb_buffer_idx = 0;
    volatile int m_dma_buffer_idx = 0;

    // State for the current data reception process (only accessed by ISR).
    uint32_t m_bytes_received = 0;
    std::variant<std::monostate, QuadrantUpdate, PartialUpdate> m_current_rx_task;
};

} // namespace display

#endif // DISPLAY_MANAGER_H