//! Interactive Button Controller for Longan Nano PA8 / BOOT0.
//!
//! Provides:
//! - Hardware debouncing (>= 40 ms threshold)
//! - Single-click detection (cycles screen mode or rearms halted single-sweep)
//! - Double-click detection (<= 280 ms gap between releases) for screen context actions:
//!   * GraphScreen: cycle TriggerMode (AUTO -> NORM -> SINGLE)
//!   * StatsScreen / BigDigitScreen: cycle BatteryProfile
//!   * HeroScreen / HistogramScreen: flush & rotate SD log file
//! - Long-press visual progress bar in LCD top bar (300 ms .. 1500 ms)
//! - Immediate Tare Zero & Reset trigger at 1500 ms without waiting for button release
//! - Tare confirmation banner and context action banners
//! - Test injection hooks for automated remote validation

pub const MIN_PRESS_MS: u32 = 40;        // Minimum duration for valid press (debounce)
pub const DOUBLE_CLICK_GAP_MS: u32 = 280; // Max gap between first release and second press
pub const HOLD_START_MS: u32 = 300;      // Hold start threshold (progress bar begins)
pub const HOLD_TRIGGER_MS: u32 = 1500;   // Long press threshold: Tare & Reset triggers
pub const PROGRESS_STEPS: u8 = 10;       // 10 steps in [==========]
pub const HOLD_SPAN_MS: u32 = HOLD_TRIGGER_MS - HOLD_START_MS; // 1200 ms
pub const STEP_SPAN_MS: u32 = HOLD_SPAN_MS / (PROGRESS_STEPS as u32); // 120 ms per step

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    /// Button is physically or virtually held down for the first time.
    Pressed { start_ms: u32 },
    /// Button has been held >= 300 ms. Progress bar is active.
    Holding { start_ms: u32, last_step: u8 },
    /// Button held >= 1500 ms. Tare was triggered; waiting for button release.
    TareExecuted,
    /// First tap released (< 300 ms). Waiting up to 280 ms for second tap.
    WaitingForSecond { release_ms: u32 },
    /// Second tap is held down. Double click already fired; waiting for release.
    SecondPressed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ButtonEvent {
    None,
    /// Single click confirmed (after 280 ms expired without second tap).
    SingleClick,
    /// Double click confirmed (second press began within 280 ms).
    DoubleClick,
    /// Progress bar update while holding between 300 ms and 1500 ms (step 0..10).
    HoldProgress { step: u8, duration_ms: u32 },
    /// Button was released while in Holding (< 1500 ms). Progress aborted.
    HoldAborted,
    /// Button reached 1500 ms. Trigger Tare Zero & Reset immediately!
    TareTrigger,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BannerType {
    None,
    TareConfirmed,
    ContextAction(&'static str),
}

pub struct TopBarOverlay {
    pub is_active: bool,
    pub banner_type: BannerType,
    pub until_ms: u32,
    pub hold_step: u8,
}

impl TopBarOverlay {
    pub const fn new() -> Self {
        Self {
            is_active: false,
            banner_type: BannerType::None,
            until_ms: 0,
            hold_step: 0,
        }
    }

    pub fn show_hold_progress(&mut self, step: u8) {
        self.is_active = true;
        self.banner_type = BannerType::None;
        self.hold_step = step;
    }

    pub fn show_tare_confirmed(&mut self, now: u32, duration_ms: u32) {
        self.is_active = true;
        self.banner_type = BannerType::TareConfirmed;
        self.until_ms = now.saturating_add(duration_ms);
    }

    pub fn show_context_banner(&mut self, text: &'static str, now: u32, duration_ms: u32) {
        self.is_active = true;
        self.banner_type = BannerType::ContextAction(text);
        self.until_ms = now.saturating_add(duration_ms);
    }

    pub fn clear(&mut self) {
        self.is_active = false;
        self.banner_type = BannerType::None;
        self.until_ms = 0;
        self.hold_step = 0;
    }

    pub fn is_banner_expired(&self, now: u32) -> bool {
        self.is_active && self.banner_type != BannerType::None && now >= self.until_ms
    }
}

pub struct ButtonController {
    pub state: ButtonState,
    pub overlay: TopBarOverlay,
    sim_start_ms: u32,
    sim_until_ms: u32,
    sim_second_start_ms: u32,
    sim_second_until_ms: u32,
}

impl ButtonController {
    pub const fn new() -> Self {
        Self {
            state: ButtonState::Idle,
            overlay: TopBarOverlay::new(),
            sim_start_ms: 0,
            sim_until_ms: 0,
            sim_second_start_ms: 0,
            sim_second_until_ms: 0,
        }
    }

    /// Injects a simulated tap: press for duration_ms
    pub fn simulate_tap(&mut self, now: u32, duration_ms: u32) {
        self.state = ButtonState::Idle;
        self.sim_second_start_ms = 0;
        self.sim_second_until_ms = 0;
        self.sim_start_ms = now;
        self.sim_until_ms = now.saturating_add(duration_ms);
    }

    /// Injects a simulated double-tap:
    /// tap 1: tap_ms (e.g. 80ms)
    /// gap: gap_ms (e.g. 100ms)
    /// tap 2: tap_ms (e.g. 80ms)
    pub fn simulate_double_tap(&mut self, now: u32, tap_ms: u32, gap_ms: u32) {
        self.state = ButtonState::Idle;
        self.sim_start_ms = now;
        self.sim_until_ms = now.saturating_add(tap_ms);
        self.sim_second_start_ms = self.sim_until_ms.saturating_add(gap_ms);
        self.sim_second_until_ms = self.sim_second_start_ms.saturating_add(tap_ms);
    }

    /// Injects a simulated hold: press for hold_ms (e.g. 1600ms)
    pub fn simulate_hold(&mut self, now: u32, hold_ms: u32) {
        self.state = ButtonState::Idle;
        self.sim_second_start_ms = 0;
        self.sim_second_until_ms = 0;
        self.sim_start_ms = now;
        self.sim_until_ms = now.saturating_add(hold_ms);
    }

    /// Checks if a simulated press is currently active.
    pub fn is_simulated_pressed(&self, now: u32) -> bool {
        (now >= self.sim_start_ms && now < self.sim_until_ms && self.sim_until_ms > 0)
            || (now >= self.sim_second_start_ms
                && now < self.sim_second_until_ms
                && self.sim_second_until_ms > 0)
    }

    /// Whether the button is effectively down (physical or simulated).
    pub fn is_down(&self, raw_pressed: bool, now: u32) -> bool {
        raw_pressed || self.is_simulated_pressed(now)
    }

    /// Primary state machine update, invoked on every main loop iteration.
    pub fn update(&mut self, raw_pressed: bool, now: u32) -> ButtonEvent {
        let is_pressed = self.is_down(raw_pressed, now);

        match self.state {
            ButtonState::Idle => {
                if is_pressed {
                    self.state = ButtonState::Pressed { start_ms: now };
                }
                ButtonEvent::None
            }

            ButtonState::Pressed { start_ms } => {
                let duration = now.wrapping_sub(start_ms);
                if is_pressed {
                    if duration >= HOLD_TRIGGER_MS {
                        self.state = ButtonState::TareExecuted;
                        ButtonEvent::TareTrigger
                    } else if duration >= HOLD_START_MS {
                        let step = (((duration - HOLD_START_MS) / STEP_SPAN_MS) as u8).min(PROGRESS_STEPS);
                        self.state = ButtonState::Holding { start_ms, last_step: step };
                        ButtonEvent::HoldProgress { step, duration_ms: duration }
                    } else {
                        ButtonEvent::None
                    }
                } else {
                    // Released before HOLD_START_MS
                    if duration >= MIN_PRESS_MS {
                        self.state = ButtonState::WaitingForSecond { release_ms: now };
                        ButtonEvent::None
                    } else {
                        // Glitch / debounce noise (< 40 ms)
                        self.state = ButtonState::Idle;
                        ButtonEvent::None
                    }
                }
            }

            ButtonState::Holding { start_ms, ref mut last_step } => {
                let duration = now.wrapping_sub(start_ms);
                if is_pressed {
                    if duration >= HOLD_TRIGGER_MS {
                        self.state = ButtonState::TareExecuted;
                        ButtonEvent::TareTrigger
                    } else {
                        let step = (((duration - HOLD_START_MS) / STEP_SPAN_MS) as u8).min(PROGRESS_STEPS);
                        if step != *last_step {
                            *last_step = step;
                            ButtonEvent::HoldProgress { step, duration_ms: duration }
                        } else {
                            ButtonEvent::None
                        }
                    }
                } else {
                    // User released before 1500 ms: hold aborted!
                    self.state = ButtonState::Idle;
                    ButtonEvent::HoldAborted
                }
            }

            ButtonState::TareExecuted => {
                if !is_pressed {
                    self.state = ButtonState::Idle;
                }
                ButtonEvent::None
            }

            ButtonState::WaitingForSecond { release_ms } => {
                let gap = now.wrapping_sub(release_ms);
                if is_pressed {
                    if gap <= DOUBLE_CLICK_GAP_MS {
                        self.state = ButtonState::SecondPressed;
                        ButtonEvent::DoubleClick
                    } else {
                        // Gap exceeded before second press: first click confirms as single click,
                        // and second press starts a new Pressed sequence.
                        self.state = ButtonState::Pressed { start_ms: now };
                        ButtonEvent::SingleClick
                    }
                } else if gap > DOUBLE_CLICK_GAP_MS {
                    // Timeout! Single click confirmed
                    self.state = ButtonState::Idle;
                    ButtonEvent::SingleClick
                } else {
                    ButtonEvent::None
                }
            }

            ButtonState::SecondPressed => {
                if !is_pressed {
                    self.state = ButtonState::Idle;
                }
                ButtonEvent::None
            }
        }
    }
}

/// Formats the hold progress bar string into a 25-character ASCII buffer:
/// `"HOLD TO TARE [====      ]"`
pub fn fmt_hold_progress(buf: &mut [u8; 25], step: u8) {
    buf[0..14].copy_from_slice(b"HOLD TO TARE [");
    let filled = step.min(10) as usize;
    for i in 0..filled {
        buf[14 + i] = b'=';
    }
    for i in filled..10 {
        buf[14 + i] = b' ';
    }
    buf[24] = b']';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_debounce_glitch() {
        let mut btn = ButtonController::new();
        // Press for 20 ms (< 40 ms debounce threshold)
        assert_eq!(btn.update(true, 100), ButtonEvent::None);
        assert_eq!(btn.update(true, 120), ButtonEvent::None);
        assert_eq!(btn.update(false, 125), ButtonEvent::None);
        assert_eq!(btn.state, ButtonState::Idle);
    }

    #[test]
    fn test_button_single_click() {
        let mut btn = ButtonController::new();
        // Tap down at t=1000
        assert_eq!(btn.update(true, 1000), ButtonEvent::None);
        // Release at t=1080 (duration 80ms)
        assert_eq!(btn.update(false, 1080), ButtonEvent::None);
        assert!(matches!(btn.state, ButtonState::WaitingForSecond { release_ms: 1080 }));

        // Still waiting at t=1350 (gap 270 ms <= 280 ms)
        assert_eq!(btn.update(false, 1350), ButtonEvent::None);

        // Timeout fires at t=1361 (gap 281 ms > 280 ms)
        assert_eq!(btn.update(false, 1361), ButtonEvent::SingleClick);
        assert_eq!(btn.state, ButtonState::Idle);
    }

    #[test]
    fn test_button_double_click() {
        let mut btn = ButtonController::new();
        // Tap 1 down at t=1000, up at t=1080
        btn.update(true, 1000);
        btn.update(false, 1080);

        // Tap 2 down at t=1200 (gap 120 ms <= 280 ms)
        assert_eq!(btn.update(true, 1200), ButtonEvent::DoubleClick);
        assert_eq!(btn.state, ButtonState::SecondPressed);

        // Tap 2 released at t=1280
        assert_eq!(btn.update(false, 1280), ButtonEvent::None);
        assert_eq!(btn.state, ButtonState::Idle);
    }

    #[test]
    fn test_button_double_click_held_does_not_tare() {
        let mut btn = ButtonController::new();
        // Tap 1 down at t=1000, up at t=1080
        btn.update(true, 1000);
        btn.update(false, 1080);

        // Tap 2 down at t=1200
        assert_eq!(btn.update(true, 1200), ButtonEvent::DoubleClick);

        // Keep held past 1500 ms (t=2800)
        assert_eq!(btn.update(true, 2800), ButtonEvent::None);
        assert_eq!(btn.state, ButtonState::SecondPressed);

        // Finally released
        assert_eq!(btn.update(false, 2900), ButtonEvent::None);
        assert_eq!(btn.state, ButtonState::Idle);
    }

    #[test]
    fn test_button_hold_progress_and_tare_trigger() {
        let mut btn = ButtonController::new();
        // Press down at t=1000
        assert_eq!(btn.update(true, 1000), ButtonEvent::None);

        // Reaches 300 ms at t=1300: Hold progress step 0
        assert_eq!(
            btn.update(true, 1300),
            ButtonEvent::HoldProgress { step: 0, duration_ms: 300 }
        );

        // Reaches 420 ms at t=1420: Hold progress step 1
        assert_eq!(
            btn.update(true, 1420),
            ButtonEvent::HoldProgress { step: 1, duration_ms: 420 }
        );

        // Step doesn't change at t=1450 (still step 1)
        assert_eq!(btn.update(true, 1450), ButtonEvent::None);

        // Advance to 1500 ms at t=2500: Tare triggers immediately!
        assert_eq!(btn.update(true, 2500), ButtonEvent::TareTrigger);
        assert_eq!(btn.state, ButtonState::TareExecuted);

        // Keep holding to t=3000: Tare does not re-trigger
        assert_eq!(btn.update(true, 3000), ButtonEvent::None);

        // Released at t=3100: returns to Idle without click
        assert_eq!(btn.update(false, 3100), ButtonEvent::None);
        assert_eq!(btn.state, ButtonState::Idle);
    }

    #[test]
    fn test_button_hold_abort() {
        let mut btn = ButtonController::new();
        btn.update(true, 1000);
        btn.update(true, 1300); // Step 0
        btn.update(true, 1600); // Step 2 (duration 600 ms)

        // Released before 1500 ms: hold aborted!
        assert_eq!(btn.update(false, 1700), ButtonEvent::HoldAborted);
        assert_eq!(btn.state, ButtonState::Idle);
    }

    #[test]
    fn test_fmt_hold_progress() {
        let mut buf = [0u8; 25];
        fmt_hold_progress(&mut buf, 0);
        assert_eq!(&buf, b"HOLD TO TARE [          ]");

        fmt_hold_progress(&mut buf, 4);
        assert_eq!(&buf, b"HOLD TO TARE [====      ]");

        fmt_hold_progress(&mut buf, 10);
        assert_eq!(&buf, b"HOLD TO TARE [==========]");
    }

    #[test]
    fn test_button_simulation_tap() {
        let mut btn = ButtonController::new();
        btn.simulate_tap(100, 80); // Pressed 100..180
        assert!(!btn.is_down(false, 99));
        assert!(btn.is_down(false, 100));
        assert!(btn.is_down(false, 179));
        assert!(!btn.is_down(false, 180));
    }

    #[test]
    fn test_button_simulation_double_tap() {
        let mut btn = ButtonController::new();
        btn.simulate_double_tap(100, 80, 100); // Tap 1: 100..180, Tap 2: 280..360
        assert!(btn.is_down(false, 100));
        assert!(btn.is_down(false, 150));
        assert!(!btn.is_down(false, 180));
        assert!(!btn.is_down(false, 250));
        assert!(btn.is_down(false, 280));
        assert!(btn.is_down(false, 350));
        assert!(!btn.is_down(false, 360));
    }
}
