#pragma once

#include "drivers/usb/composite_device.hpp"
#include "drivers/usb/drv_usb_core.h"
#include "drivers/usb/usbd_core.h"
#include "drivers/usb/drv_usbd_int.h"
#include "bsp/usb_hw.hpp"
#include "hal/eclic.hpp"
#include "hal/exti.hpp"
#include <cstdio>

namespace usb {

/**
 * @brief Generic, zero-overhead USB Device controller parameterized on CompositeType.
 *
 * Couples hardware peripheral initialization (RCU, ECLIC, Timer), DWC2 core driver,
 * and compile-time composite dispatching with 0 bytes vtable overhead.
 *
 * @tparam CompositeType Composite device dispatcher type (e.g. CompositeDevice<Drivers...>)
 */
template <typename CompositeType>
class GenericUsbDevice {
public:
    using Composite = CompositeType;

    GenericUsbDevice() noexcept {
        s_instance = this;
    }

    static GenericUsbDevice& getInstance() noexcept {
        return *s_instance;
    }

    [[nodiscard]] Composite& composite() noexcept { return m_composite; }
    [[nodiscard]] const Composite& composite() const noexcept { return m_composite; }
    [[nodiscard]] usb_core_driver& core() noexcept { return m_core_driver; }
    [[nodiscard]] const usb_core_driver& core() const noexcept { return m_core_driver; }

    template <typename DriverType>
    [[nodiscard]] DriverType& get_driver() noexcept {
        return m_composite.template get<DriverType>();
    }

    template <typename DriverType>
    [[nodiscard]] const DriverType& get_driver() const noexcept {
        return m_composite.template get<DriverType>();
    }

    template <size_t Index>
    [[nodiscard]] auto& get_driver() noexcept {
        return m_composite.template get<Index>();
    }

    template <size_t Index>
    [[nodiscard]] const auto& get_driver() const noexcept {
        return m_composite.template get<Index>();
    }

    void init(const usb_desc* desc) noexcept {
        m_descriptors = *desc;

        m_class_core.init = &init_cb;
        m_class_core.deinit = &deinit_cb;
        m_class_core.req_proc = &req_proc_cb;
        m_class_core.data_in = &data_in_cb;
        m_class_core.data_out = &data_out_cb;

        hal::eclic::Eclic::set_priority_group(hal::eclic::PriorityGroup::Level2Prio2);
        hal::eclic::Eclic::enable_global_interrupts();
        bsp::usb::rcu_config();
        bsp::usb::timer_init();
        bsp::usb::intr_config();

        usbd_init(&m_core_driver, &m_descriptors, &m_class_core);
    }

    void poll() noexcept {}

    [[nodiscard]] bool is_configured() const noexcept {
        return m_core_driver.dev.cur_status == USBD_CONFIGURED;
    }

    void isr() noexcept {
        usbd_isr(&m_core_driver);
    }

    void wakeup_isr() noexcept {
        if (m_core_driver.bp.low_power) {
            // Wakeup logic if low-power mode active
        }
        hal::exti::Exti::clear_pending(18);
    }

    void timer_isr() noexcept {
        bsp::usb::timer_irq();
    }

private:
    static uint8_t init_cb(usb_dev* udev, uint8_t config_idx) noexcept {
        return s_instance ? s_instance->m_composite.init(udev, config_idx) : static_cast<uint8_t>(USBD_FAIL);
    }

    static uint8_t deinit_cb(usb_dev* udev, uint8_t config_idx) noexcept {
        return s_instance ? s_instance->m_composite.deinit(udev, config_idx) : static_cast<uint8_t>(USBD_FAIL);
    }

    static uint8_t req_proc_cb(usb_dev* udev, usb_req* req) noexcept {
        if (!s_instance) return static_cast<uint8_t>(USBD_FAIL);
        auto status = s_instance->m_composite.handle_request(udev, *reinterpret_cast<SetupPacket*>(req));
        return (status == ReqStatus::Success) ? static_cast<uint8_t>(USBD_OK) : static_cast<uint8_t>(USBD_FAIL);
    }

    static uint8_t data_in_cb(usb_dev* udev, uint8_t ep_num) noexcept {
        if (!s_instance) return static_cast<uint8_t>(USBD_FAIL);
        return s_instance->m_composite.data_in(udev, ep_num) ? static_cast<uint8_t>(USBD_OK) : static_cast<uint8_t>(USBD_FAIL);
    }

    static uint8_t data_out_cb(usb_dev* udev, uint8_t ep_num) noexcept {
        if (!s_instance) return static_cast<uint8_t>(USBD_FAIL);
        return s_instance->m_composite.data_out(udev, ep_num) ? static_cast<uint8_t>(USBD_OK) : static_cast<uint8_t>(USBD_FAIL);
    }

    inline static GenericUsbDevice* s_instance{nullptr};

    usb_core_driver m_core_driver{};
    usb_desc        m_descriptors{};
    usb_class_core  m_class_core{};
    CompositeType   m_composite{};
};

} // namespace usb
