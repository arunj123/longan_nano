#pragma once

#include <tuple>
#include <utility>
#include <concepts>
#include "drivers/usb/class_driver.hpp"

namespace usb {

/**
 * @brief Zero-cost compile-time Composite USB Device dispatcher.
 *
 * Dispatches control requests and endpoint callbacks across multiple class drivers
 * at compile time using fold expressions and std::tuple, completely eliminating
 * virtual tables and runtime indirection.
 *
 * @tparam Drivers Variadic pack of class driver types satisfying usb::ClassDriver
 */
template <ClassDriver... Drivers>
class CompositeDevice {
public:
    std::tuple<Drivers...> drivers;

    constexpr CompositeDevice() = default;
    explicit constexpr CompositeDevice(Drivers... d) : drivers(std::move(d)...) {}

    template <typename T>
    [[nodiscard]] constexpr T& get() noexcept {
        return std::get<T>(drivers);
    }

    template <typename T>
    [[nodiscard]] constexpr const T& get() const noexcept {
        return std::get<T>(drivers);
    }

    template <size_t Index>
    [[nodiscard]] constexpr auto& get() noexcept {
        return std::get<Index>(drivers);
    }

    template <size_t Index>
    [[nodiscard]] constexpr const auto& get() const noexcept {
        return std::get<Index>(drivers);
    }

    uint8_t init(usb_dev* udev, uint8_t config_idx) noexcept {
        uint8_t result = USBD_OK;
        std::apply([&](auto&... d) {
            auto init_one = [&](auto& driver) {
                if (driver.init(udev, config_idx) != USBD_OK) {
                    result = USBD_FAIL;
                }
            };
            (init_one(d), ...);
        }, drivers);
        return result;
    }

    uint8_t deinit(usb_dev* udev, uint8_t config_idx) noexcept {
        uint8_t result = USBD_OK;
        std::apply([&](auto&... d) {
            auto deinit_one = [&](auto& driver) {
                if (driver.deinit(udev, config_idx) != USBD_OK) {
                    result = USBD_FAIL;
                }
            };
            (deinit_one(d), ...);
        }, drivers);
        return result;
    }

    ReqStatus handle_request(usb_dev* udev, const SetupPacket& req) noexcept {
        ReqStatus status = ReqStatus::NotSupported;

        if (req.recipient() == 0x02) { // Recipient is Endpoint
            uint8_t ep_addr = static_cast<uint8_t>(req.wIndex);
            bool is_in = (ep_addr & 0x80) != 0;
            uint8_t ep_num = ep_addr & 0x7F;

            std::apply([&](auto&... d) {
                auto try_ep = [&](auto& driver) -> bool {
                    if ((is_in && driver.owns_ep_in(ep_num)) || (!is_in && driver.owns_ep_out(ep_num))) {
                        status = driver.handle_request(udev, req);
                        return true;
                    }
                    return false;
                };
                (try_ep(d) || ...);
            }, drivers);

            return status;
        }

        uint8_t itf = static_cast<uint8_t>(req.wIndex & 0xFF);
        std::apply([&](auto&... d) {
            auto try_req = [&](auto& driver) -> bool {
                if (driver.owns_interface(itf)) {
                    status = driver.handle_request(udev, req);
                    return true;
                }
                return false;
            };
            (try_req(d) || ...);
        }, drivers);

        return status;
    }

    bool data_in(usb_dev* udev, uint8_t ep_num) noexcept {
        bool handled = false;
        std::apply([&](auto&... d) {
            auto try_in = [&](auto& driver) -> bool {
                if (driver.owns_ep_in(ep_num)) {
                    handled = driver.data_in(udev, ep_num);
                    return true;
                }
                return false;
            };
            (try_in(d) || ...);
        }, drivers);
        return handled;
    }

    bool data_out(usb_dev* udev, uint8_t ep_num) noexcept {
        bool handled = false;
        std::apply([&](auto&... d) {
            auto try_out = [&](auto& driver) -> bool {
                if (driver.owns_ep_out(ep_num)) {
                    handled = driver.data_out(udev, ep_num);
                    return true;
                }
                return false;
            };
            (try_out(d) || ...);
        }, drivers);
        return handled;
    }
};

} // namespace usb
