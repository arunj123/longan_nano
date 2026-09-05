#pragma once

#include <cstdint>

namespace bsp::usb {

void rcu_config();
void intr_config();
void timer_init();
void timer_irq();

} // namespace bsp::usb
