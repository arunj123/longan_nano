#include "usb.hpp"

// Include necessary C headers for implementation details.
extern "C" {
#include "gd32vf103.h"
#include "drv_usb_hw.h"
}

// Define the global USB driver instance here.
// It's now owned by this USB module.
// The C-based ISRs require this to be a global symbol.
usb_core_driver cdc_acm;

namespace usb {

void init() {
    eclic_global_interrupt_enable();
    eclic_priority_group_set(ECLIC_PRIGROUP_LEVEL2_PRIO2);

    usb_rcu_config();
    usb_timer_init();
    usb_intr_config();

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