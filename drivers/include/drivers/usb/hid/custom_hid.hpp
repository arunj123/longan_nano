#pragma once

#include <cstdint>
#include <concepts>
#include <span>
#include <algorithm>
#include <cstring>
#include "drivers/usb/class_driver.hpp"
#include "drivers/usb/hid/hid_types.hpp"

namespace usb::hid {

/**
 * @brief Zero-overhead, statically dispatched Custom HID Class Driver.
 *
 * Manages vendor-specific raw packet transfers (e.g. 64-byte IN/OUT reports)
 * with direct callback routing and 0 bytes vtable overhead.
 *
 * @tparam InterfaceNum Interface index in configuration descriptor
 * @tparam EpInAddr Endpoint IN address (e.g. 0x82)
 * @tparam EpOutAddr Endpoint OUT address (e.g. 0x02)
 * @tparam PacketSize Maximum packet size in bytes (default 64)
 */
template <uint8_t InterfaceNum, uint8_t EpInAddr, uint8_t EpOutAddr, uint16_t PacketSize = 64>
class CustomHidDriver {
public:
    static constexpr uint8_t interface_number = InterfaceNum;
    static constexpr uint8_t ep_in = EpInAddr;
    static constexpr uint8_t ep_in_num = EpInAddr & 0x7F;
    static constexpr uint8_t ep_out = EpOutAddr;
    static constexpr uint8_t ep_out_num = EpOutAddr & 0x7F;
    static constexpr uint16_t packet_size = PacketSize;

    using RxCallback = void (*)(const uint8_t* data, size_t length);

    struct Config {
        const usb_desc_ep* ep_in_desc{nullptr};
        const usb_desc_ep* ep_out_desc{nullptr};
        const void* hid_desc{nullptr};
        std::span<const uint8_t> report_desc{};
        RxCallback rx_callback{nullptr};
    };

    constexpr CustomHidDriver() = default;
    explicit constexpr CustomHidDriver(Config cfg) : m_cfg(cfg) {}

    void set_config(Config cfg) noexcept { m_cfg = cfg; }
    void set_rx_callback(RxCallback cb) noexcept { m_cfg.rx_callback = cb; }

    [[nodiscard]] constexpr bool owns_interface(uint8_t itf) const noexcept {
        return itf == InterfaceNum;
    }
    [[nodiscard]] constexpr bool owns_ep_in(uint8_t ep) const noexcept {
        return ep == ep_in_num;
    }
    [[nodiscard]] constexpr bool owns_ep_out(uint8_t ep) const noexcept {
        return ep == ep_out_num;
    }

    uint8_t init(usb_dev* udev, uint8_t config_idx) noexcept {
        (void)config_idx;
        if (m_cfg.ep_in_desc) {
            usbd_ep_setup(udev, m_cfg.ep_in_desc);
        }
        if (m_cfg.ep_out_desc) {
            usbd_ep_setup(udev, m_cfg.ep_out_desc);
        }
        m_in_busy = false;
        // Arm OUT endpoint to receive first packet
        usbd_ep_recev(udev, ep_out, m_rx_buf, packet_size);
        return USBD_OK;
    }

    uint8_t deinit(usb_dev* udev, uint8_t config_idx) noexcept {
        (void)config_idx;
        usbd_ep_clear(udev, ep_in);
        usbd_ep_clear(udev, ep_out);
        m_in_busy = false;
        return USBD_OK;
    }

    ReqStatus handle_request(usb_dev* udev, const SetupPacket& req) noexcept {
        if (req.recipient() == 0x02) {
            if (req.bRequest == static_cast<uint8_t>(StandardRequest::ClearFeature)) {
                if ((req.wIndex & 0x80) != 0) {
                    m_in_busy = false;
                }
                return ReqStatus::Success;
            }
            return ReqStatus::Fail;
        }

        usb_transc* transc = &udev->dev.transc_in[0];

        switch (static_cast<Request>(req.bRequest)) {
            case Request::GetReport:
                return ReqStatus::Fail;

            case Request::GetIdle:
                transc->xfer_buf = reinterpret_cast<uint8_t*>(&m_idle_state);
                transc->remain_len = 1U;
                return ReqStatus::Success;

            case Request::GetProtocol:
                transc->xfer_buf = reinterpret_cast<uint8_t*>(&m_protocol);
                transc->remain_len = 1U;
                return ReqStatus::Success;

            case Request::SetReport:
                return ReqStatus::Fail;

            case Request::SetIdle:
                m_idle_state = static_cast<uint8_t>(req.wValue >> 8);
                return ReqStatus::Success;

            case Request::SetProtocol:
                m_protocol = static_cast<uint8_t>(req.wValue);
                return ReqStatus::Success;

            default:
                if (req.bRequest == static_cast<uint8_t>(StandardRequest::GetDescriptor)) {
                    auto desc_type = static_cast<uint8_t>(req.wValue >> 8);
                    if (desc_type == static_cast<uint8_t>(DescriptorType::Report)) {
                        if (!m_cfg.report_desc.empty()) {
                            transc->remain_len = std::min(static_cast<uint16_t>(m_cfg.report_desc.size()), req.wLength);
                            transc->xfer_buf = const_cast<uint8_t*>(m_cfg.report_desc.data());
                            return ReqStatus::Success;
                        }
                    } else if (desc_type == static_cast<uint8_t>(DescriptorType::Hid)) {
                        if (m_cfg.hid_desc) {
                            transc->remain_len = std::min(static_cast<uint16_t>(sizeof(HidDescriptor)), req.wLength);
                            transc->xfer_buf = const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(m_cfg.hid_desc));
                            return ReqStatus::Success;
                        }
                    }
                }
                return ReqStatus::Fail;
        }
    }

    bool data_in(usb_dev* udev, uint8_t ep) noexcept {
        (void)udev;
        if (ep == ep_in_num) {
            m_in_busy = false;
            return true;
        }
        return false;
    }

    bool data_out(usb_dev* udev, uint8_t ep) noexcept {
        if (ep == ep_out_num) {
            uint32_t count = usbd_rxcount_get(udev, ep_out);
            if (count > 0 && m_cfg.rx_callback) {
                m_cfg.rx_callback(m_rx_buf, count);
            }
            // Re-arm endpoint for next packet
            usbd_ep_recev(udev, ep_out, m_rx_buf, packet_size);
            return true;
        }
        return false;
    }

    [[nodiscard]] bool is_transfer_complete() const noexcept { return !m_in_busy; }
    [[nodiscard]] bool is_busy() const noexcept { return m_in_busy; }

    bool send_report(usb_dev* udev, const uint8_t* buffer, size_t length) noexcept {
        if (length > packet_size || m_in_busy) {
            return false;
        }
        m_in_busy = true;
        memcpy(m_tx_buf, buffer, length);
        if (length < packet_size) {
            memset(m_tx_buf + length, 0, packet_size - length);
        }
        usbd_ep_send(udev, ep_in, m_tx_buf, packet_size);
        return true;
    }

private:
    Config m_cfg{};
    alignas(4) uint32_t m_idle_state{0};
    alignas(4) uint32_t m_protocol{0};
    volatile bool m_in_busy{false};
    alignas(4) uint8_t m_rx_buf[packet_size]{0};
    alignas(4) uint8_t m_tx_buf[packet_size]{0};
};

static_assert(ClassDriver<CustomHidDriver<1, 0x82, 0x02, 64>>);

} // namespace usb::hid
