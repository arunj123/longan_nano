#if __has_include("drv_usb_hw.h")

#include <cstdint>
#include "hal/time.hpp"

extern "C" {
#include "gd32vf103.h"
#include "drv_usb_hw.h"

// Global prescaler variable required by GD32 USB stack
uint32_t usbfs_prescaler = RCU_CKUSB_CKPLL_DIV2;

void usb_rcu_config(void) {
    const uint32_t system_clock = rcu_clock_freq_get(CK_SYS);
    if (system_clock == 48000000) {
        usbfs_prescaler = RCU_CKUSB_CKPLL_DIV1;
    } else if (system_clock == 72000000) {
        usbfs_prescaler = RCU_CKUSB_CKPLL_DIV1_5;
    } else { // 96 MHz (default)
        usbfs_prescaler = RCU_CKUSB_CKPLL_DIV2;
    }

    rcu_usb_clock_config(usbfs_prescaler);
    rcu_periph_clock_enable(RCU_USBFS);
}

void usb_intr_config(void) {
    eclic_irq_enable(static_cast<uint8_t>(USBFS_IRQn), 1, 0);

    // Power management clock for USB wakeup
    rcu_periph_clock_enable(RCU_PMU);
    exti_interrupt_flag_clear(EXTI_18);
    exti_init(EXTI_18, EXTI_INTERRUPT, EXTI_TRIG_RISING);
    exti_interrupt_enable(EXTI_18);

    eclic_irq_enable(static_cast<uint8_t>(USBFS_WKUP_IRQn), 3, 0);
}

void usb_timer_init(void) {
    // Zero-overhead: hardware core mtime is always active
}

void usb_udelay(const uint32_t usec) {
    hal::time::delay_us(usec);
}

void usb_mdelay(const uint32_t msec) {
    hal::time::delay_ms(msec);
}

void usb_timer_irq(void) {
    // No-op: mtime needs no periodic timer IRQ
}

void system_clk_config_stop(void) {
}

} // extern "C"

#endif // __has_include("drv_usb_hw.h")
