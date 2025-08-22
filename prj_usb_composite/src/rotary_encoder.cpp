#include "rotary_encoder.h"
#include "systick.h" // For the millisecond timer

// Pin Definitions (These are correct and stay the same)
#define ENCODER_S1_PIN      GPIO_PIN_10
#define ENCODER_S1_PORT     GPIOB
#define ENCODER_S1_CLK      RCU_GPIOB
#define ENCODER_S1_EXTI     EXTI_10
#define ENCODER_S1_IRQn     EXTI10_15_IRQn

#define ENCODER_S2_PIN      GPIO_PIN_11
#define ENCODER_S2_PORT     GPIOB
#define ENCODER_S2_CLK      RCU_GPIOB

#define ENCODER_KEY_PIN     GPIO_PIN_12
#define ENCODER_KEY_PORT    GPIOB
#define ENCODER_KEY_CLK     RCU_GPIOB
#define ENCODER_KEY_EXTI    EXTI_12
#define ENCODER_KEY_IRQn    EXTI10_15_IRQn

// Internal state variables
static volatile int8_t rotation_count = 0;
static volatile bool key_pressed_flag = false;
static volatile uint32_t last_interrupt_time = 0;
const uint32_t DEBOUNCE_TIME_MS = 50;
const uint32_t ROTATION_DEBOUNCE_MS = 2; // Debounce for rotation interrupts

// Update the rotation ISR to include a micro-debounce
void encoder::rotation_isr() {
    uint32_t now = get_timer_value();
    if ((now - last_interrupt_time) < ROTATION_DEBOUNCE_MS) { // Reject noise
        exti_interrupt_flag_clear(ENCODER_S1_EXTI);
        return;
    }

    if (gpio_input_bit_get(ENCODER_S2_PORT, ENCODER_S2_PIN) == RESET) {
        rotation_count--; // This reports counter-clockwise / left turn
    } else {
        rotation_count++; // This reports clockwise / right turn
    }
    
    last_interrupt_time = now;
    exti_interrupt_flag_clear(ENCODER_S1_EXTI);
}

// Update the key ISR to use the shared timer variable
void encoder::key_isr() {
    uint32_t now = get_timer_value();
    if ((now - last_interrupt_time) > DEBOUNCE_TIME_MS) {
        key_pressed_flag = true;
        last_interrupt_time = now;
    }
    exti_interrupt_flag_clear(ENCODER_KEY_EXTI);
}

// Public function to initialize the hardware
void encoder::init() {
    rcu_periph_clock_enable(ENCODER_S1_CLK);
    rcu_periph_clock_enable(RCU_AF);

    // Configure S1 (PB10) as interrupt source, S2 (PB11) as simple input
    gpio_init(ENCODER_S1_PORT, GPIO_MODE_IPU, GPIO_OSPEED_50MHZ, ENCODER_S1_PIN);
    gpio_init(ENCODER_S2_PORT, GPIO_MODE_IPU, GPIO_OSPEED_50MHZ, ENCODER_S2_PIN);

    // *** CRITICAL CHANGE: Trigger only on FALLING edge, not BOTH ***
    gpio_exti_source_select(GPIO_PORT_SOURCE_GPIOB, GPIO_PIN_SOURCE_10);
    exti_init(ENCODER_S1_EXTI, EXTI_INTERRUPT, EXTI_TRIG_FALLING); 
    exti_interrupt_flag_clear(ENCODER_S1_EXTI);
    eclic_irq_enable(ENCODER_S1_IRQn, 1, 0);

    // Configure KEY (PB0) for button press
    gpio_init(ENCODER_KEY_PORT, GPIO_MODE_IPU, GPIO_OSPEED_50MHZ, ENCODER_KEY_PIN);
    
    gpio_exti_source_select(GPIO_PORT_SOURCE_GPIOB, GPIO_PIN_SOURCE_12);
    exti_init(ENCODER_KEY_EXTI, EXTI_INTERRUPT, EXTI_TRIG_FALLING);
    exti_interrupt_flag_clear(ENCODER_KEY_EXTI);
    eclic_irq_enable(ENCODER_KEY_IRQn, 1, 0);
}

// Public function to check for a button press
bool encoder::is_pressed() {
    if (key_pressed_flag) {
        key_pressed_flag = false; // Clear the flag after reading
        return true;
    }
    return false;
}

// Public function to get the net rotation count
int8_t encoder::get_rotation() {
    int8_t count = 0;
    if (rotation_count != 0) {
        // Atomically read and reset the count
        eclic_global_interrupt_disable();
        count = rotation_count;
        rotation_count = 0;
        eclic_global_interrupt_enable();
    }
    return count;
}