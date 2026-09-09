#include "usb.hpp"
#include "msc_core.hpp"
#include "usbd_descriptors.hpp"
#include "drivers/usb/usb_core.hpp"
#include "drivers/usb/usbd_core.h"
#include "drivers/usb/usbd_enum.h"
#include "drivers/usb/drv_usb_hw.h"
#include "bsp/usb_hw.hpp"
#include "hal/eclic.hpp"
#include "hal/exti.hpp"
#include "hal/time.hpp"
#include "hal/gpio.hpp"

usb_core_driver msc_udev;
static usb_desc msc_desc;

namespace usb {

void init() {
    // 1. Reset and disable USBFS core so GPIO completely takes over PA11/PA12
    hal::rcu::RegAHBRST::set_bits(1U << 12);
    hal::time::delay_ms(10);
    hal::rcu::RegAHBRST::clear_bits(1U << 12);
    hal::rcu::disable(hal::rcu::Peripheral::Usbfs);

    // 2. Drive PA11 and PA12 LOW (SE0 condition: Disconnect) for 500 ms
    hal::gpio::GpioPin<hal::gpio::Port::A, 12>::init(hal::gpio::Mode::OutputPushPull);
    hal::gpio::GpioPin<hal::gpio::Port::A, 11>::init(hal::gpio::Mode::OutputPushPull);
    hal::gpio::GpioPin<hal::gpio::Port::A, 12>::reset();
    hal::gpio::GpioPin<hal::gpio::Port::A, 11>::reset();
    hal::time::delay_ms(500);

    hal::eclic::Eclic::set_priority_group(hal::eclic::PriorityGroup::Level2Prio2);
    hal::eclic::Eclic::enable_global_interrupts();

    bsp::usb::rcu_config();
    bsp::usb::timer_init();
    bsp::usb::intr_config();

    msc_desc.dev_desc = reinterpret_cast<uint8_t*>(&msc_dev_desc);
    msc_desc.config_desc = reinterpret_cast<uint8_t*>(&msc_config_desc);
    msc_desc.strings = usbd_msc_strings;

    usbd_init(&msc_udev, &msc_desc, &msc::msc_class);

    // Set clean fresh serial number to prevent Windows using stale corrupted registry cache
    set_custom_serial_string("LNMSC0000003");

    // Force explicit 1200ms soft disconnect so upstream hub reliably registers disconnect
    usbd_disconnect(&msc_udev);
    hal::time::delay_ms(1200);
    usbd_connect(&msc_udev);
}

void poll() {
    msc::poll(&msc_udev);
}

bool is_configured() {
    return (USBD_CONFIGURED == msc_udev.dev.cur_status);
}

} // namespace usb

extern "C" {

void USBFS_IRQHandler(void) {
    usbd_isr(&msc_udev);
}

void USBFS_WKUP_IRQHandler(void) {
    if (msc_udev.bp.low_power) {
        bsp::usb::rcu_config();
        usb_clock_active(&msc_udev);
    }
    hal::exti::Exti::clear_pending(18);
}

}
