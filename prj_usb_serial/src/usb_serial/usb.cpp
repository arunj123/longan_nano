#include "usb.hpp"
#include "hal/eclic.hpp"
#include "hal/exti.hpp"

#include "bsp/usb_hw.hpp"

// Define the global USB driver instance here.
// It's now owned by this USB module.
// The ISRs require this to be a global symbol.
usb_core_driver cdc_acm;

namespace usb {

void init() {
    hal::eclic::Eclic::set_priority_group(hal::eclic::PriorityGroup::Level2Prio2);
    hal::eclic::Eclic::enable_global_interrupts();

    bsp::usb::rcu_config();
    bsp::usb::timer_init();
    bsp::usb::intr_config();

    usbd_init(&cdc_acm, &cdc_desc, &cdc_class);
}

void poll() {
    if (is_configured()) {
        if (0U == cdc_acm_check_ready(&cdc_acm)) {
            cdc_acm_data_receive(&cdc_acm);
        } else {
            cdc_acm_data_send(&cdc_acm);
        }
    }
}

bool is_configured() {
    return (USBD_CONFIGURED == cdc_acm.dev.cur_status);
}

} // namespace usb

extern "C" {

void USBFS_IRQHandler(void) {
    usbd_isr(&cdc_acm);
}

void USBFS_WKUP_IRQHandler(void) {
    if (cdc_acm.bp.low_power) {
        bsp::usb::rcu_config();
        usb_clock_active(&cdc_acm);
    }
    hal::exti::Exti::clear_pending(18);
}
}