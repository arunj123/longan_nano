#pragma once
#include <cstdint>
#include "hal/time.hpp"
#include "bsp/usb_hw.hpp"
#define usb_mdelay(x) hal::time::delay_ms(x)
#define usb_udelay(x) hal::time::delay_us(x)
#define usb_rcu_config() bsp::usb::rcu_config()
#define usb_intr_config() bsp::usb::intr_config()
#define usb_timer_init() bsp::usb::timer_init()
