#include "bsp/usb_hw.hpp"
#include "hal/time.hpp"
#include "hal/rcu.hpp"
#include "hal/exti.hpp"
#include "hal/eclic.hpp"

extern uint32_t SystemCoreClock;

namespace bsp::usb {

uint32_t usbfs_prescaler = static_cast<uint32_t>(hal::rcu::UsbPrescaler::Div2);

void rcu_config() {
    const uint32_t system_clock = SystemCoreClock;
    hal::rcu::UsbPrescaler psc;
    if (system_clock == 48000000) {
        psc = hal::rcu::UsbPrescaler::Div1;
    } else if (system_clock == 72000000) {
        psc = hal::rcu::UsbPrescaler::Div1_5;
    } else { // 96 MHz (default)
        psc = hal::rcu::UsbPrescaler::Div2;
    }

    usbfs_prescaler = static_cast<uint32_t>(psc);
    hal::rcu::set_usb_clock_prescaler(psc);
    hal::rcu::enable(hal::rcu::Peripheral::Usbfs);
}

void intr_config() {
    hal::eclic::Eclic::enable(hal::eclic::Irq::Usbfs, 1, 0);

    // Power management clock for USB wakeup
    hal::rcu::enable(hal::rcu::Peripheral::Pmu);
    hal::exti::Exti::clear_pending(18);
    hal::exti::Exti::enable_line(18, hal::exti::Trigger::Rising);

    hal::eclic::Eclic::enable(hal::eclic::Irq::UsbfsWkup, 3, 0);
}

void timer_init() {
    // Zero-overhead: hardware core mtime is always active
}

void timer_irq() {
    // No-op: mtime needs no periodic timer IRQ
}

} // namespace bsp::usb
