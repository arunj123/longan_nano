use embedded_hal::digital::InputPin;

pub struct RotaryEncoder<CLK, DT, SW> {
    clk: CLK,
    dt: DT,
    sw: SW,
    last_clk: bool,
    last_sw: bool,
}

impl<CLK: InputPin, DT: InputPin, SW: InputPin> RotaryEncoder<CLK, DT, SW> {
    pub fn new(clk: CLK, dt: DT, sw: SW) -> Self {
        Self {
            clk,
            dt,
            sw,
            last_clk: true,
            last_sw: true,
        }
    }

    /// Polls encoder and returns (rotation, switch_pressed)
    /// rotation: +1 for clockwise, -1 for counter-clockwise, 0 for none.
    /// switch_pressed: true on falling edge (press event).
    pub fn poll(&mut self) -> (i8, bool) {
        let clk_high = self.clk.is_high().unwrap_or(true);
        let dt_high = self.dt.is_high().unwrap_or(true);
        let sw_high = self.sw.is_high().unwrap_or(true);

        let mut rotation = 0;
        // Detect falling edge on CLK
        if self.last_clk && !clk_high {
            if dt_high {
                rotation = 1; // Clockwise
            } else {
                rotation = -1; // Counter-Clockwise
            }
        }
        self.last_clk = clk_high;

        let mut pressed = false;
        if self.last_sw && !sw_high {
            pressed = true;
        }
        self.last_sw = sw_high;

        (rotation, pressed)
    }
}
