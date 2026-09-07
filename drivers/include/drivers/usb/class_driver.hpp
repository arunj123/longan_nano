#pragma once

#include <cstdint>
#include <concepts>
#include <span>
#include "drivers/usb/usb_types.hpp"
#include "drivers/usb/drv_usb_core.h"
#include "drivers/usb/usbd_core.h"

namespace usb {

/**
 * @brief Concept defining a static USB Class Driver interface without virtual tables.
 *
 * All class drivers (e.g. StandardHidDriver, CustomHidDriver, CdcAcmDriver) satisfy this concept,
 * enabling 100% inlined compile-time polymorphism with 0 bytes vtable overhead in Flash/RAM.
 */
template <typename T>
concept ClassDriver = requires(T& driver, const T& const_driver,
                               usb_dev* udev, uint8_t config_idx, const SetupPacket& req,
                               uint8_t ep, uint8_t itf) {
    { driver.init(udev, config_idx) } -> std::convertible_to<uint8_t>;
    { driver.deinit(udev, config_idx) } -> std::convertible_to<uint8_t>;
    { driver.handle_request(udev, req) } -> std::same_as<ReqStatus>;
    { driver.data_in(udev, ep) } -> std::same_as<bool>;
    { driver.data_out(udev, ep) } -> std::same_as<bool>;
    { const_driver.owns_interface(itf) } -> std::same_as<bool>;
    { const_driver.owns_ep_in(ep) } -> std::same_as<bool>;
    { const_driver.owns_ep_out(ep) } -> std::same_as<bool>;
};

} // namespace usb
