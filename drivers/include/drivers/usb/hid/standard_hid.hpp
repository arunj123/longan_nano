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
 * @brief Zero-overhead, statically dispatched Standard HID Class Driver.
 *
 * Manages Standard HID interface (Mouse, Keyboard, Consumer media keys),
 * report descriptor requests, and protocol/idle negotiation with 0 bytes vtable overhead.
 *
 * @tparam InterfaceNum Interface number assigned in configuration descriptor
 * @tparam EpInAddr Endpoint IN address (e.g. 0x81)
 */
template <uint8_t InterfaceNum, uint8_t EpInAddr>
class StandardHidDriver {
public:
    static constexpr uint8_t interface_number = InterfaceNum;
    static constexpr uint8_t ep_in = EpInAddr;
    static constexpr uint8_t ep_in_num = EpInAddr & 0x7F;

    struct Config {
        const usb_desc_ep* ep_desc{nullptr};
        const void* hid_desc{nullptr};
        std::span<const uint8_t> report_desc{};
    };

    constexpr StandardHidDriver() = default;
    explicit constexpr StandardHidDriver(Config cfg) : m_cfg(cfg) {}

    void set_config(Config cfg) noexcept { m_cfg = cfg; }

    [[nodiscard]] constexpr bool owns_interface(uint8_t itf) const noexcept {
        return itf == InterfaceNum;
    }
    [[nodiscard]] constexpr bool owns_ep_in(uint8_t ep) const noexcept {
        return ep == ep_in_num;
    }
    [[nodiscard]] constexpr bool owns_ep_out(uint8_t ep) const noexcept {
        (void)ep;
        return false;
    }

    uint8_t init(usb_dev* udev, uint8_t config_idx) noexcept {
        (void)config_idx;
        if (m_cfg.ep_desc) {
            usbd_ep_setup(udev, m_cfg.ep_desc);
        }
        m_in_busy = false;
        return USBD_OK;
    }

    uint8_t deinit(usb_dev* udev, uint8_t config_idx) noexcept {
        (void)config_idx;
        usbd_ep_clear(udev, ep_in);
        m_in_busy = false;
        return USBD_OK;
    }

    ReqStatus handle_request(usb_dev* udev, const SetupPacket& req) noexcept {
        if (req.recipient() == 0x02) {
            if (req.bRequest == static_cast<uint8_t>(StandardRequest::ClearFeature)) {
                m_in_busy = false;
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
        (void)udev;
        (void)ep;
        return false;
    }

    [[nodiscard]] bool is_transfer_complete() const noexcept { return !m_in_busy; }
    [[nodiscard]] bool is_busy() const noexcept { return m_in_busy; }

    bool send_mouse(usb_dev* udev, const MouseReport& report) noexcept {
        if (m_in_busy) return false;
        m_in_busy = true;
        memcpy(m_report_buf, &report, sizeof(MouseReport));
        usbd_ep_send(udev, ep_in, m_report_buf, sizeof(MouseReport));
        return true;
    }

    bool send_mouse(usb_dev* udev, int8_t x, int8_t y, int8_t wheel, uint8_t buttons) noexcept {
        MouseReport report{
            .report_id = 1,
            .buttons = buttons,
            .x = x,
            .y = y,
            .wheel = wheel
        };
        return send_mouse(udev, report);
    }

    bool send_keyboard(usb_dev* udev, const KeyboardReport& report) noexcept {
        if (m_in_busy) return false;
        m_in_busy = true;
        memcpy(m_report_buf, &report, sizeof(KeyboardReport));
        usbd_ep_send(udev, ep_in, m_report_buf, sizeof(KeyboardReport));
        return true;
    }

    bool send_keyboard(usb_dev* udev, uint8_t modifier, uint8_t key) noexcept {
        KeyboardReport report{
            .report_id = 2,
            .modifier = modifier,
            .reserved = 0,
            .keycodes = {key, 0, 0, 0, 0, 0}
        };
        return send_keyboard(udev, report);
    }

    bool send_consumer(usb_dev* udev, const ConsumerReport& report) noexcept {
        if (m_in_busy) return false;
        m_in_busy = true;
        memcpy(m_report_buf, &report, sizeof(ConsumerReport));
        usbd_ep_send(udev, ep_in, m_report_buf, sizeof(ConsumerReport));
        return true;
    }

    bool send_consumer(usb_dev* udev, uint16_t usage_code) noexcept {
        ConsumerReport report{
            .report_id = 3,
            .usage_code = usage_code
        };
        return send_consumer(udev, report);
    }

private:
    Config m_cfg{};
    alignas(4) uint32_t m_idle_state{0};
    alignas(4) uint32_t m_protocol{0};
    volatile bool m_in_busy{false};
    alignas(4) uint8_t m_report_buf[16]{0};
};

static_assert(ClassDriver<StandardHidDriver<0, 0x81>>);

} // namespace usb::hid
