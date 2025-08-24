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
    if (len < 1) return; // Must have at least a command byte

    // The data buffer STARTS with the command byte (data[0]).
    const auto command = static_cast<HostCommand>(data[0]);

    switch (command) {
        case HostCommand::DRAW_RECT: {
            // Packet format received: [CMD, x, y, w, h, seq_lsb, seq_msb]
            if (len < 7) return;

            uint8_t next_head = (m_usb_head_idx + 1) % constants::NumBuffers;
            if (next_head == m_dma_tail_idx) return; // Buffers are full

            DrawTask& task = m_draw_tasks[m_usb_head_idx];
            if (task.state != BufferState::EMPTY) return; // Head not ready

            // Parameters start at index 1, immediately after the command.
            Rect r = {data[1], data[2], data[3], data[4]};
            uint16_t seq = data[5] | (data[6] << 8);

            if (seq != m_expected_sequence_num) {
                m_expected_sequence_num = seq; // Resync
            }

            uint32_t total_bytes = static_cast<uint32_t>(r.w) * static_cast<uint32_t>(r.h) * 2;
            if (total_bytes > constants::BufferSizeBytes || total_bytes == 0) return;

            task.state = BufferState::RECEIVING;
            task.region = r;
            task.bytes_received = 0;
            task.total_bytes_expected = total_bytes;
            task.sequence_number = seq;
            break;
        }

        case HostCommand::IMAGE_DATA: {
            DrawTask& task = m_draw_tasks[m_usb_head_idx];
            if (task.state != BufferState::RECEIVING) return;

            // Payload length is total length minus the 1-byte command.
            uint32_t data_len = len - 1;

            if ((task.bytes_received + data_len) > task.total_bytes_expected) {
                data_len = task.total_bytes_expected - task.bytes_received;
            }

            // Pixel data starts at index 1, after the command.
            uint8_t* dest_ptr = m_framebuffers[m_usb_head_idx].data() + task.bytes_received;
            memcpy(dest_ptr, &data[1], data_len);
            task.bytes_received += data_len;

            if (task.bytes_received >= task.total_bytes_expected) {
                task.state = BufferState::READY_TO_DRAW;
                m_usb_head_idx = (m_usb_head_idx + 1) % constants::NumBuffers;
                m_expected_sequence_num++;
            }
            break;
        }
        default:
            // This case handles any garbage data if synchronization is lost.
            break;
    }
}

void DisplayManager::processDrawTasks() {
    // This function is correct and does not need changes.
    if (m_dma_tail_idx == m_usb_head_idx) return;

    DrawTask& task = m_draw_tasks[m_dma_tail_idx];

    if (task.state == BufferState::READY_TO_DRAW) {
        const Rect& r = task.region;
        lcd_write_u16(r.x, r.y, r.w, r.h, m_framebuffers[m_dma_tail_idx].data());
        
        task.state = BufferState::EMPTY;
        m_dma_tail_idx = (m_dma_tail_idx + 1) % constants::NumBuffers;
    }
}

} // namespace display