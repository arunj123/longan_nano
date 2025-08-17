#ifndef GPIO_H_OPTIMIZED
#define GPIO_H_OPTIMIZED

#include <stdint.h> // For uint32_t, uint16_t

// Wrap C peripheral headers in extern "C" to ensure C linkage
extern "C" {
#include "gd32vf103_gpio.h" // For GPIO functions like gpio_init, gpio_bit_set, etc., and types like bit_status, SET, RESET
#include "gd32vf103_rcu.h" // For RCU functions like rcu_periph_clock_enable and type rcu_periph_enum
}

/**
 * @brief Gpio class provides an object-oriented interface for a single GPIO pin.
 * It encapsulates the low-level GPIO operations provided by the GD32VF103 library.
 */
class Gpio {
  public:
    /**
     * @brief Constructor for the Gpio class.
     * Initializes the GPIO peripheral and enables its clock.
     * @param gpio_periph The GPIO peripheral (e.g., GPIOA, GPIOB, GPIOC).
     * @param pin The specific pin number (e.g., GPIO_PIN_0, GPIO_PIN_13).
     */
    Gpio(uint32_t gpio_periph, uint16_t pin) : _gpio_periph(gpio_periph), _pin(pin) {
        // Enable the clock for the GPIO peripheral.
        rcu_periph_clock_enable(static_cast<rcu_periph_enum>(_gpio_periph));
    }

    /**
     * @brief Sets the direction of the GPIO pin (input or output).
     * @param direction The direction mode (e.g., GPIO_MODE_IN_FLOATING, GPIO_MODE_OUT_PP).
     */
    inline void setDirection(uint32_t direction) { // Marked as inline
        gpio_init(_gpio_periph, direction, GPIO_OSPEED_50MHZ, _pin);
    }

    /**
     * @brief Sets the output mode of the GPIO pin (push-pull, open-drain).
     * @param mode The output mode (e.g., GPIO_MODE_OUT_PP, GPIO_MODE_OUT_OD).
     */
    inline void setOutputMode(uint32_t mode) { // Marked as inline
        gpio_init(_gpio_periph, mode, GPIO_OSPEED_50MHZ, _pin);
    }

    /**
     * @brief Configures the pull-up/pull-down resistor for the GPIO pin.
     * @param pull_up_down The pull-up/pull-down configuration (e.g., GPIO_PUPD_NONE, GPIO_PUPD_PULLUP).
     */
    inline void setPullUpPullDown(uint32_t pull_up_down) { // Marked as inline
        gpio_init(_gpio_periph, GPIO_MODE_IN_FLOATING, pull_up_down, _pin);
    }

    /**
     * @brief Sets the state of the GPIO pin (high or low).
     * @param state True for high, false for low.
     */
    inline void set(bool state) {
        if (state) {
            gpio_bit_set(_gpio_periph, _pin);
        } else {
            gpio_bit_reset(_gpio_periph, _pin);
        }
    }

    /**
     * @brief Toggles the current state of the GPIO pin.
     * This directly flips the physical pin state.
     */
    inline void toggle() {
        gpio_bit_write(_gpio_periph, _pin, static_cast<bit_status>(1 - gpio_input_bit_get(_gpio_periph, _pin)));
    }

    /**
     * @brief Gets the current state of the GPIO pin.
     * @return True if the pin is high, false if low.
     */
    inline bool get() const { return gpio_input_bit_get(_gpio_periph, _pin) == SET; }

  private:
    uint32_t _gpio_periph; ///< Stores the GPIO peripheral base address.
    uint16_t _pin;         ///< Stores the specific pin number.
};

/**
 * @brief Led class provides a simplified interface for controlling an LED.
 * It encapsulates a Gpio object and handles the active-low/active-high logic internally.
 */
class Led {
  public:
    /**
     * @brief Constructor for the Led class.
     * @param gpio_periph The GPIO peripheral the LED is connected to.
     * @param pin The specific pin number the LED is connected to.
     * @param active_low True if the LED is active low (lights up when pin is low), false otherwise.
     */
    Led(uint32_t gpio_periph, uint16_t pin, bool active_low = false)
        : _gpio(gpio_periph, pin), _active_low(active_low) {
        _gpio.setOutputMode(GPIO_MODE_OUT_PP);
        off(); // Initialize LED to be off
    }

    /**
     * @brief Turns the LED on.
     */
    inline void on() { _gpio.set(!_active_low); }

    /**
     * @brief Turns the LED off.
     */
    inline void off() { _gpio.set(_active_low); }

    /**
     * @brief Toggles the state of the LED.
     * This implementation explicitly manages the logical state to ensure correct toggling
     * regardless of the underlying GPIO's `toggle()` behavior or `_active_low` setting.
     */
    inline void toggle() {
        // Read the current physical state of the pin
        bool current_physical_state = _gpio.get();

        // Determine the current logical state of the LED (true if ON, false if OFF)
        bool current_logical_state = (current_physical_state != _active_low);

        // Determine the next logical state (flip it)
        bool next_logical_state = !current_logical_state;

        // Set the physical pin based on the next logical state and active_low
        _gpio.set(next_logical_state != _active_low);
    }

    /**
     * @brief Checks if the LED is currently on.
     * @return True if the LED is on, false otherwise.
     */
    inline bool isOn() const { return _gpio.get() != _active_low; }

  private:
    Gpio _gpio;       ///< The underlying Gpio object for the LED.
    bool _active_low; ///< Flag to indicate if the LED is active low.
};

#endif // GPIO_H_OPTIMIZED
