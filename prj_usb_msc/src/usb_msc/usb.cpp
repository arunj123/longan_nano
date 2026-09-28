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
#include "hal/rcu.hpp"

usb_core_driver msc_udev;
static usb_desc msc_desc;

extern "C" void enable_debug_uart_interrupt(void);

namespace usb {

void init() {
    hal::eclic::Eclic::set_priority_group(hal::eclic::PriorityGroup::Level2Prio2);
    hal::eclic::Eclic::enable_global_interrupts();
    enable_debug_uart_interrupt();

    // Hardware SE0 pulse: drive PA11 (D-) and PA12 (D+) LOW for 200 ms
    // to guarantee the upstream host hub detects a physical detachment across resets.
    using DmPin = hal::gpio::GpioPin<hal::gpio::Port::A, 11>;
    using DpPin = hal::gpio::GpioPin<hal::gpio::Port::A, 12>;
    DmPin::init(hal::gpio::Mode::OutputPushPull);
    DpPin::init(hal::gpio::Mode::OutputPushPull);
    DmPin::reset();
    DpPin::reset();
    hal::time::delay_ms(200);
    DmPin::init(hal::gpio::Mode::InputFloating);
    DpPin::init(hal::gpio::Mode::InputFloating);
    hal::time::delay_ms(50);

    bsp::usb::rcu_config();
    bsp::usb::timer_init();
    bsp::usb::intr_config();

    msc_desc.dev_desc = reinterpret_cast<uint8_t*>(&msc_dev_desc);
    msc_desc.config_desc = reinterpret_cast<uint8_t*>(&msc_config_desc);
    msc_desc.strings = usbd_msc_strings;

    // Set clean fresh serial number before usbd_init initializes descriptors
    set_custom_serial_string("LNMSC0000079");

    usbd_init(&msc_udev, &msc_desc, &msc::msc_class);
}

void poll() {
    msc::poll(&msc_udev);
}

bool is_configured() {
    // USB 2.0 §9.1.1.6: a suspended device retains its configuration.
    // DWC2 sets cur_status = USBD_SUSPENDED after ~3 ms of bus idle; backup_status
    // preserves the pre-suspend state. Treat both as "configured" to prevent main()
    // from calling draw_string() on every idle interval (which would block for ~20 ms
    // and cause the device to miss host resume tokens, creating a ping-pong freeze).
    return (msc_udev.dev.cur_status == USBD_CONFIGURED) ||
           (msc_udev.dev.cur_status == USBD_SUSPENDED &&
            msc_udev.dev.backup_status == USBD_CONFIGURED);
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
