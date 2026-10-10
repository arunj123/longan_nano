use embedded_hal::digital::InputPin;

pub struct RotaryEncoder<CLK, DT, SW> {
    clk: CLK,
    dt: DT,
    sw: SW,
    state: u8,
    accum: i8,
    sw_debounce: u8,
    last_sw_pressed: bool,
}

impl<CLK: InputPin, DT: InputPin, SW: InputPin> RotaryEncoder<CLK, DT, SW> {
    pub fn new(clk: CLK, dt: DT, sw: SW) -> Self {
        Self {
            clk,
            dt,
            sw,
            state: 0,
            accum: 0,
            sw_debounce: 0,
            last_sw_pressed: false,
        }
    }

    /// Polls encoder using a 4-state quadrature transition table and switch debouncing.
    /// Returns (rotation, switch_pressed):
    /// - rotation: +1 for clockwise detent, -1 for counter-clockwise detent, 0 for none.
    /// - switch_pressed: true once on debounced press event.
    pub fn poll(&mut self) -> (i8, bool) {
        let clk_high = self.clk.is_high().unwrap_or(true);
        let dt_high = self.dt.is_high().unwrap_or(true);
        let sw_low = self.sw.is_low().unwrap_or(false);

        // 1. Quadrature State Table Transition Decoding
        // Read 2-bit pin state: [CLK, DT]
        let pin_state = ((clk_high as u8) << 1) | (dt_high as u8);
        // Combine previous state and current state into 4-bit index
        let transition = ((self.state & 0x03) << 2) | pin_state;
        self.state = pin_state;

        // Gray code state table transitions:
        // Clockwise: 00->01 (1), 01->11 (7), 11->10 (14), 10->00 (8)
        // Counter-Clockwise: 00->10 (2), 10->11 (11), 11->01 (13), 01->00 (4)
        let step: i8 = match transition {
            0b00_01 | 0b01_11 | 0b11_10 | 0b10_00 => 1,
            0b00_10 | 0b10_11 | 0b11_01 | 0b01_00 => -1,
            _ => 0,
        };

        let mut rotation = 0;
        if step != 0 {
            self.accum = self.accum.saturating_add(step);
            // Standard EC11 detent produces 2 or 4 quadrature transitions per click.
            // Using a threshold of ±2 provides crisp, responsive 1-click-to-1-action navigation.
            if self.accum >= 2 {
                rotation = 1;
                self.accum = 0;
            } else if self.accum <= -2 {
                rotation = -1;
                self.accum = 0;
            }
        }

        // 2. Switch Debouncing
        // Require stable low state before firing single pressed event
        let mut pressed = false;
        if sw_low {
            if self.sw_debounce < 5 {
                self.sw_debounce += 1;
                if self.sw_debounce == 5 && !self.last_sw_pressed {
                    pressed = true;
                    self.last_sw_pressed = true;
                }
            }
        } else {
            self.sw_debounce = 0;
            self.last_sw_pressed = false;
        }

        (rotation, pressed)
    }
}
