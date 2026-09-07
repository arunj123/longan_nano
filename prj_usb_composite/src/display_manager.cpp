#include "display_manager.h"
#include <cstdio>
#include <cstring>

#include "lcd.h"

namespace display {

// Singleton instance getter
DisplayManager& DisplayManager::getInstance() {
    static DisplayManager instance;
    return instance;
}

void DisplayManager::handleUsbPacket(const uint8_t* data, uint32_t len) {
    if (len < 1) return;

    m_last_rx_time = hal::time::Instant::now();
    const auto command = static_cast<HostCommand>(data[0]);

    switch (command) {
        case HostCommand::DRAW_RECT: {
            if (len < 7) {
                return;
            }

            DrawTask& task = m_draw_tasks[m_usb_head_idx];
            if (task.state != BufferState::EMPTY) {
                if (task.state == BufferState::RECEIVING) {
                    m_dbg_frame_aborted = true;
                    task.bytes_received = 0;
                } else {
                    m_dbg_rect_dropped = true;
                    return;
                }
            }

            Rect r = {data[1], data[2], data[3], data[4]};
            uint16_t seq = data[5] | (data[6] << 8);

            if (seq != m_expected_sequence_num) {
                m_expected_sequence_num = seq; // Resync
            }

            uint32_t total_bytes = static_cast<uint32_t>(r.w) * static_cast<uint32_t>(r.h) * 2;
            if (total_bytes > constants::BufferSizeBytes || total_bytes == 0) {
                return;
            }

            task.state = BufferState::RECEIVING;
            task.region = r;
            task.bytes_received = 0;
            task.total_bytes_expected = total_bytes;
            task.sequence_number = seq;

            m_dbg_last_rect = r;
            m_dbg_last_seq = seq;
            m_dbg_rect_received = true;
            break;
        }

        case HostCommand::IMAGE_DATA: {
            if (len <= 1) return;

            DrawTask& task = m_draw_tasks[m_usb_head_idx];
            if (task.state != BufferState::RECEIVING) {
                m_dbg_dropped_data_pkts = m_dbg_dropped_data_pkts + 1;
                return;
            }

            uint32_t data_len = len - 1;
            if ((task.bytes_received + data_len) > task.total_bytes_expected) {
                data_len = task.total_bytes_expected - task.bytes_received;
            }

            uint8_t* dest_ptr = m_framebuffers[m_usb_head_idx].data() + task.bytes_received;
            memcpy(dest_ptr, &data[1], data_len);
            task.bytes_received += data_len;
            m_dbg_received_data_pkts = m_dbg_received_data_pkts + 1;

            if (task.bytes_received >= task.total_bytes_expected) {
                __asm__ volatile("" ::: "memory");
                task.state = BufferState::READY_TO_DRAW;
                m_usb_head_idx = static_cast<uint8_t>((m_usb_head_idx + 1) % constants::NumBuffers);
                m_expected_sequence_num++;
            }
            break;
        }
        default:
            m_dbg_unknown_cmd = data[0];
            break;
    }
}

void DisplayManager::processDrawTasks() {
    // 1. Report diagnostic events
    if (m_dbg_unknown_cmd != 0) {
        printf("[DM-WARN] Unknown USB Command: 0x%02X\n", m_dbg_unknown_cmd);
        m_dbg_unknown_cmd = 0;
    }
    if (m_dbg_frame_aborted) {
        printf("[DM-WARN] Incomplete frame in slot %d aborted mid-transfer -> Resyncing with new DRAW_RECT\n", m_usb_head_idx);
        m_dbg_frame_aborted = false;
    }
    if (m_dbg_rect_dropped) {
        printf("[DM-ERR] DRAW_RECT dropped! Slot %d is not EMPTY (state=%d)\n",
               m_usb_head_idx, static_cast<int>(m_draw_tasks[m_usb_head_idx].state));
        m_dbg_rect_dropped = false;
    }
    if (m_dbg_dropped_data_pkts > 0) {
        printf("[DM-ERR] Dropped %lu IMAGE_DATA pkts (state not RECEIVING)\n", m_dbg_dropped_data_pkts);
        m_dbg_dropped_data_pkts = 0;
    }
    if (m_dbg_rect_received) {
        printf("[DM] RECV DRAW_RECT #%u: (%d,%d) %dx%d (%lu B) -> Slot %d\n",
               m_dbg_last_seq, m_dbg_last_rect.x, m_dbg_last_rect.y,
               m_dbg_last_rect.w, m_dbg_last_rect.h,
               static_cast<uint32_t>(m_dbg_last_rect.w) * m_dbg_last_rect.h * 2,
               m_usb_head_idx);
        m_dbg_rect_received = false;
    }

    // 1b. Check if slot in RECEIVING timed out (e.g. host script terminated mid-transfer)
    DrawTask& rx_task = m_draw_tasks[m_usb_head_idx];
    if (rx_task.state == BufferState::RECEIVING &&
        m_last_rx_time.elapsed() >= hal::time::Duration::from_ms(5000)) {
        printf("[DM-WARN] Slot %d RECEIVING timed out (%lu/%lu B) -> Reset to EMPTY\n",
               m_usb_head_idx, rx_task.bytes_received, rx_task.total_bytes_expected);
        rx_task.bytes_received = 0;
        rx_task.state = BufferState::EMPTY;
    }

    // 2. Check if currently active DMA transfer finished
    DrawTask& current_task = m_draw_tasks[m_dma_tail_idx];
    if (current_task.state == BufferState::DRAWING) {
        if (lcd_is_dma_busy()) {
            return; // Still transmitting via SPI DMA
        }
        // DMA transmission finished! Free this buffer slot for USB reuse.
        printf("[DM] Slot %d DMA DONE -> EMPTY\n", m_dma_tail_idx);
        __asm__ volatile("" ::: "memory");
        current_task.state = BufferState::EMPTY;
        m_dma_tail_idx = static_cast<uint8_t>((m_dma_tail_idx + 1) % constants::NumBuffers);
    }

    // 3. Check if the buffer at m_dma_tail_idx is ready to draw
    DrawTask& next_task = m_draw_tasks[m_dma_tail_idx];
    if (next_task.state == BufferState::READY_TO_DRAW) {
        if (lcd_is_dma_busy()) {
            return;
        }

        __asm__ volatile("" ::: "memory");
        const Rect r = next_task.region;
        const uint8_t* buf = m_framebuffers[m_dma_tail_idx].data();
        printf("[DM] Slot %d START DRAW (%d,%d) %dx%d (%lu B). Hex dump [0..15]: ",
               m_dma_tail_idx, r.x, r.y, r.w, r.h, next_task.bytes_received);
        for (int i = 0; i < 16; ++i) {
            printf("%02X ", buf[i]);
        }
        printf("\n");

        lcd_write_u16(r.x, r.y, r.w, r.h, buf);
        next_task.state = BufferState::DRAWING;
    }
}

} // namespace display