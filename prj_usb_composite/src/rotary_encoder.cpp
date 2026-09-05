#include "rotary_encoder.h"
#include "hal/gpio.hpp"
#include "hal/exti.hpp"
#include "hal/eclic.hpp"
#include "hal/time.hpp"
#include "hal/core.hpp"

namespace {

using PinS1  = hal::gpio::GpioPin<hal::gpio::Port::B, 10>;
using PinS2  = hal::gpio::GpioPin<hal::gpio::Port::B, 11>;
using PinKey = hal::gpio::GpioPin<hal::gpio::Port::B, 12>;

constexpr uint8_t kExtiLineS1  = 10;
constexpr uint8_t kExtiLineKey = 12;

// Internal state variables
volatile int8_t g_rotation_count = 0;
volatile bool   g_key_pressed_flag = false;
hal::time::Instant g_last_rotation_time{0};
hal::time::Instant g_last_key_time{0};
const auto kRotationDebounce = hal::time::Duration::from_ms(2);
const auto kKeyDebounce      = hal::time::Duration::from_ms(50);

} // anonymous namespace

void encoder::rotation_isr() {
    const auto now = hal::time::Instant::now();
    if ((now - g_last_rotation_time) < kRotationDebounce) { // Reject contact bounce
        hal::exti::Exti::clear_pending(kExtiLineS1);
        return;
    }

    // Modify count atomically
    int8_t current_count = g_rotation_count;
    if (PinS2::read() == hal::gpio::Level::Low) {
        current_count--; // Counter-clockwise
    } else {
        current_count++; // Clockwise
    }
    g_rotation_count = current_count;
    
    g_last_rotation_time = now;
    hal::exti::Exti::clear_pending(kExtiLineS1);
}

void encoder::key_isr() {
    const auto now = hal::time::Instant::now();
    if ((now - g_last_key_time) > kKeyDebounce) {
        g_key_pressed_flag = true;
        g_last_key_time = now;
    }
    hal::exti::Exti::clear_pending(kExtiLineKey);
}

void encoder::init() {
    PinS1::init(hal::gpio::Mode::InputPullUp);
    PinS2::init(hal::gpio::Mode::InputPullUp);
    PinKey::init(hal::gpio::Mode::InputPullUp);

    hal::exti::Exti::map_pin(hal::gpio::Port::B, kExtiLineS1);
    hal::exti::Exti::enable_line(kExtiLineS1, hal::exti::Trigger::Falling);

    hal::exti::Exti::map_pin(hal::gpio::Port::B, kExtiLineKey);
    hal::exti::Exti::enable_line(kExtiLineKey, hal::exti::Trigger::Falling);

    // EXTI Lines 10..15 vector to EXTI10_15
    hal::eclic::Eclic::enable(hal::eclic::Irq::Exti10_15, 1, 0);
}

bool encoder::is_pressed() {
    if (g_key_pressed_flag) {
        g_key_pressed_flag = false; // Clear the flag after reading
        return true;
    }
    return false;
}

int8_t encoder::get_rotation() {
    int8_t count = 0;
    if (g_rotation_count != 0) {
        const hal::core::CriticalSection lock;
        count = g_rotation_count;
        g_rotation_count = 0;
    }
    return count;
}