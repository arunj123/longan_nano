#ifndef ROTARY_ENCODER_H
#define ROTARY_ENCODER_H

#include "gd32vf103.h"

// Public API for the rotary encoder
namespace encoder {

    // Initializes GPIOs and interrupts for the encoder
    void init();

    // Checks if the encoder button has been pressed since the last check.
    // This function is "destructive" - it clears the pressed flag.
    bool is_pressed();

    // Gets the rotation delta since the last check.
    // Returns > 0 for clockwise, < 0 for counter-clockwise, 0 for no rotation.
    // This function is "destructive" - it resets the internal counter.
    int8_t get_rotation();

    // --- Functions below are for internal use by ISRs ---
    void key_isr();
    void rotation_isr();

} // namespace encoder

#endif // ROTARY_ENCODER_H