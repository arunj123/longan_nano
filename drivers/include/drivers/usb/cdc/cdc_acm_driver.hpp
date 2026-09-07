#pragma once

#include <cstdint>
#include <concepts>
#include <span>
#include <algorithm>
#include <cstring>
#include "drivers/usb/class_driver.hpp"

namespace usb::cdc {

#pragma pack(push, 1)
struct LineCoding {
    uint32_t dwDTERate{115200};
    uint8_t  bCharFormat{0}; // 1 stop bit
    uint8_t  bParityType{0}; // None
    uint8_t  bDataBits{8};   // 8 data bits
};
#pragma pack(pop)
static_assert(sizeof(LineCoding) == 7, "LineCoding must be exactly 7 bytes");

inline constexpr uint8_t ReqSetLineCoding        = 0x20U;
inline constexpr uint8_t ReqGetLineCoding        = 0x21U;
inline constexpr uint8_t ReqSetControlLineState = 0x22U;

/**
 * @brief Zero-overhead, statically dispatched CDC-ACM Class Driver.
 *
 * Manages USB CDC Communications (PSTN/ACM) and Data Interfaces, line coding
 * requests, and bidirectional serial streaming with 0 bytes vtable overhead.
 *
 * @tparam CommItf Interface index for Communication Class
 * @tparam DataItf Interface index for Data Class
 * @tparam EpCmdAddr Endpoint IN address for Notification/Command (e.g. 0x82)
 * @tparam EpDataInAddr Endpoint IN address for Data Transmit (e.g. 0x81)
 * @tparam EpDataOutAddr Endpoint OUT address for Data Receive (e.g. 0x03)
 * @tparam PacketSize Maximum packet size in bytes (default 64)
 */
template <uint8_t CommItf, uint8_t DataItf, uint8_t EpCmdAddr, uint8_t EpDataInAddr, uint8_t EpDataOutAddr, uint16_t PacketSize = 64>
class CdcAcmDriver {
public:
    static constexpr uint8_t comm_itf = CommItf;
    static constexpr uint8_t data_itf = DataItf;
    static constexpr uint8_t ep_cmd = EpCmdAddr;
    static constexpr uint8_t ep_cmd_num = EpCmdAddr & 0x7F;
    static constexpr uint8_t ep_data_in = EpDataInAddr;
    static constexpr uint8_t ep_data_in_num = EpDataInAddr & 0x7F;
    static constexpr uint8_t ep_data_out = EpDataOutAddr;
    static constexpr uint8_t ep_data_out_num = EpDataOutAddr & 0x7F;
    static constexpr uint16_t packet_size = PacketSize;

    using RxCallback = void (*)(const uint8_t* data, size_t length);

    struct Config {
        const usb_desc_ep* ep_cmd_desc{nullptr};
        const usb_desc_ep* ep_in_desc{nullptr};
        const usb_desc_ep* ep_out_desc{nullptr};
        RxCallback rx_callback{nullptr};
    };

    constexpr CdcAcmDriver() = default;
    explicit constexpr CdcAcmDriver(Config cfg) : m_cfg(cfg) {}

    void set_config(Config cfg) noexcept { m_cfg = cfg; }
    void set_rx_callback(RxCallback cb) noexcept { m_cfg.rx_callback = cb; }

    [[nodiscard]] constexpr bool owns_interface(uint8_t itf) const noexcept {
        return itf == CommItf || itf == DataItf;
    }

    [[nodiscard]] constexpr bool owns_ep_in(uint8_t ep) const noexcept {
        return ep == ep_cmd_num || ep == ep_data_in_num;
    }

    [[nodiscard]] constexpr bool owns_ep_out(uint8_t ep) const noexcept {
        return ep == ep_data_out_num;
    }

    uint8_t init(usb_dev* udev, uint8_t config_idx) noexcept {
        (void)config_idx;
        if (m_cfg.ep_cmd_desc) usbd_ep_setup(udev, m_cfg.ep_cmd_desc);
        if (m_cfg.ep_in_desc) usbd_ep_setup(udev, m_cfg.ep_in_desc);
        if (m_cfg.ep_out_desc) usbd_ep_setup(udev, m_cfg.ep_out_desc);

        m_tx_busy = false;
        usbd_ep_recev(udev, ep_data_out, m_rx_buf, packet_size);
        return USBD_OK;
    }

    uint8_t deinit(usb_dev* udev, uint8_t config_idx) noexcept {
        (void)config_idx;
        usbd_ep_clear(udev, ep_cmd);
        usbd_ep_clear(udev, ep_data_in);
        usbd_ep_clear(udev, ep_data_out);
        m_tx_busy = false;
        return USBD_OK;
    }

    ReqStatus handle_request(usb_dev* udev, const SetupPacket& req) noexcept {
        switch (req.bRequest) {
            case ReqSetLineCoding:
                udev->dev.transc_out[0].remain_len = req.wLength;
                udev->dev.transc_out[0].xfer_buf = reinterpret_cast<uint8_t*>(&m_line_coding);
                return ReqStatus::Success;

            case ReqGetLineCoding:
                udev->dev.transc_in[0].xfer_buf = reinterpret_cast<uint8_t*>(&m_line_coding);
                udev->dev.transc_in[0].remain_len = std::min(static_cast<uint16_t>(sizeof(LineCoding)), req.wLength);
                return ReqStatus::Success;

            case ReqSetControlLineState:
                m_control_line_state = static_cast<uint16_t>(req.wValue);
                return ReqStatus::Success;

            default:
                return ReqStatus::Fail;
        }
    }

    bool data_in(usb_dev* udev, uint8_t ep) noexcept {
        (void)udev;
        if (ep == ep_data_in_num) {
            m_tx_busy = false;
            return true;
        }
        if (ep == ep_cmd_num) {
            return true;
        }
        return false;
    }

    bool data_out(usb_dev* udev, uint8_t ep) noexcept {
        if (ep == ep_data_out_num) {
            uint32_t count = usbd_rxcount_get(udev, ep_data_out);
            if (count > 0 && m_cfg.rx_callback) {
                m_cfg.rx_callback(m_rx_buf, count);
            }
            usbd_ep_recev(udev, ep_data_out, m_rx_buf, packet_size);
            return true;
        }
        return false;
    }

    [[nodiscard]] bool is_tx_busy() const noexcept { return m_tx_busy; }
    [[nodiscard]] const LineCoding& line_coding() const noexcept { return m_line_coding; }
    [[nodiscard]] uint16_t control_line_state() const noexcept { return m_control_line_state; }

    bool write(usb_dev* udev, const uint8_t* buffer, size_t length) noexcept {
        if (length > packet_size || m_tx_busy) {
            return false;
        }
        m_tx_busy = true;
        memcpy(m_tx_buf, buffer, length);
        usbd_ep_send(udev, ep_data_in, m_tx_buf, static_cast<uint32_t>(length));
        return true;
    }

private:
    Config m_cfg{};
    LineCoding m_line_coding{};
    uint16_t m_control_line_state{0};
    volatile bool m_tx_busy{false};
    alignas(4) uint8_t m_rx_buf[packet_size]{0};
    alignas(4) uint8_t m_tx_buf[packet_size]{0};
};

static_assert(ClassDriver<CdcAcmDriver<0, 1, 0x82, 0x81, 0x03, 64>>);

} // namespace usb::cdc
