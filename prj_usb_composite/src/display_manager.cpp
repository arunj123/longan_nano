#include "display_manager.h"
#include <cstdio>
#include <cstring>

// This block correctly links the C functions from the LCD library.
extern "C" {
    #include "lcd.h"
}

namespace display {

// Singleton instance getter
DisplayManager& DisplayManager::getInstance() {
    static DisplayManager instance;
    return instance;
}

void DisplayManager::handleUsbPacket(const uint8_t* data, uint32_t len) {
    if (len == 0) return;

    const auto command = static_cast<HostCommand>(data[0]);

    switch (command) {
        case HostCommand::START_QUADRANT_TRANSFER: {
            if (len > 1) {
                m_current_rx_task = QuadrantUpdate{data[1]};
                m_bytes_received = 0;
            }
            break;
        }
        case HostCommand::DRAW_RECT: {
            if (len > 4) {
                m_current_rx_task = PartialUpdate{Rect{data[1], data[2], data[3], data[4]}};
                m_bytes_received = 0;
            }
            break;
        }
        case HostCommand::IMAGE_DATA: {
            uint32_t data_len = len - 2;
            uint32_t total_bytes_expected = 0;

            std::visit([&](auto&& task) {
                using T = std::decay_t<decltype(task)>;
                if constexpr (std::is_same_v<T, QuadrantUpdate>) {
                    total_bytes_expected = constants::QuadrantBytes;
                } else if constexpr (std::is_same_v<T, PartialUpdate>) {
                    total_bytes_expected = static_cast<uint32_t>(task.region.w) * static_cast<uint32_t>(task.region.h) * 2;
                }
            }, m_current_rx_task);

            if (total_bytes_expected == 0) break;

            if((total_bytes_expected - m_bytes_received) < data_len) {
                // Overflow, ignore this packet
                data_len = total_bytes_expected - m_bytes_received;
            }

            if ((m_bytes_received + data_len) <= total_bytes_expected) {
                uint8_t* dest_ptr = m_framebuffers[static_cast<size_t>(m_usb_buffer_idx)].data() + m_bytes_received;
                memcpy(dest_ptr, &data[2], data_len);
                m_bytes_received += data_len;
            }

            if (m_bytes_received >= total_bytes_expected) {
                if (!m_task_is_ready) {
                    m_dma_buffer_idx = m_usb_buffer_idx;
                    m_draw_task = m_current_rx_task; 
                    m_task_is_ready = true;
                }

                m_usb_buffer_idx = 1 - m_usb_buffer_idx;
                m_current_rx_task = std::monostate{};
                m_bytes_received = 0;
            }
            break;
        }
    }
}

void DisplayManager::processDrawTasks() {
    if (!m_task_is_ready /* || lcd_is_dma_busy()*/) {
        return;
    }
    
    std::visit([&](auto&& task) {
        using T = std::decay_t<decltype(task)>;
        if constexpr (std::is_same_v<T, QuadrantUpdate>) {
            printf("INFO: Drawing Quadrant %d\n", task.quadrant_index);
            const int y_pos = task.quadrant_index * constants::QuadrantHeight;
            lcd_write_u16(0, y_pos, constants::LcdWidth, constants::QuadrantHeight, m_framebuffers[static_cast<size_t>(m_dma_buffer_idx)].data());
        } 
        else if constexpr (std::is_same_v<T, PartialUpdate>) {
            const Rect& r = task.region;
            printf("INFO: Drawing partial update at (%d,%d) size %dx%d\n", r.x, r.y, r.w, r.h);
            lcd_write_u16(r.x, r.y, r.w, r.h, m_framebuffers[static_cast<size_t>(m_dma_buffer_idx)].data());
        }
    }, m_draw_task);

    m_task_is_ready = false;
}

} // namespace display