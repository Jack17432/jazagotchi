use crate::level_into_u8;
use esp_idf_svc::hal::gpio::{Input, Pin, PinDriver};
use std::time::{Duration, SystemTime, SystemTimeError};

pub enum LatchMode {
    FOUR3 = 1, // 4 steps, Latch at position 3 only (compatible to older versions)
    FOUR0 = 2, // 4 steps, Latch at position 0 (reverse wiring's)
    TWO3 = 3,  // 2 steps, Latch at position 0 and 3
}

#[derive(Default)]
pub enum Direction {
    CounterClockwise = -1,
    #[default]
    NoRotation = 0,
    Clockwise = 1,
}

pub struct RotaryEncoder<'d, 'l, P1: Pin, P2: Pin> {
    pin_a: PinDriver<'d, P1, Input>,
    pin_b: PinDriver<'l, P2, Input>,

    mode: LatchMode,
    prev_state: u8,

    position_int: i8,
    position_ext: f64,
    position_ext_prev: f64,
    position_ext_time: SystemTime,
    position_ext_time_prev: SystemTime,
}

/// positions: [3] 1 0 2 [3] 1 0 2 [3]
/// [3] is the positions where my rotary switch detends
/// ==> right, count up
/// <== left,  count down
const ENCODER_DIRECTION: [i8; 4 * 4] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];

impl<'d, 'l, P1: Pin, P2: Pin> RotaryEncoder<'d, 'l, P1, P2> {
    pub fn new(
        pin_a: PinDriver<'d, P1, Input>,
        pin_b: PinDriver<'l, P2, Input>,
        mode: LatchMode,
    ) -> Self {
        let mut encoder = Self {
            pin_a,
            pin_b,
            mode,
            prev_state: 0,

            position_int: 0i8,
            position_ext: 0f64,
            position_ext_prev: 0f64,
            position_ext_time: SystemTime::now(),
            position_ext_time_prev: SystemTime::now(),
        };
        encoder.prev_state = encoder.poll_state();

        encoder
    }

    pub fn update(&mut self) {
        let curr_state = self.poll_state();

        if self.prev_state == curr_state {
            return;
        }

        self.position_int += ENCODER_DIRECTION[(self.prev_state | (curr_state << 2)) as usize];
        self.prev_state = curr_state;

        self.position_ext_time_prev = self.position_ext_time;
        self.position_ext_time = SystemTime::now();

        match &self.mode {
            LatchMode::FOUR0 => self.position_ext = (self.position_int >> 2) as f64,
            LatchMode::FOUR3 => self.position_ext = (self.position_int >> 2) as f64,
            LatchMode::TWO3 => self.position_ext = (self.position_int >> 1) as f64,
        };
    }

    pub fn get_duration(&self) -> Duration {
        self.position_ext_time
            .duration_since(self.position_ext_time_prev)
            .unwrap_or_else(|err| {
                log::error!("Failed to get duration between rotations, {}", err);
                Duration::from_secs(0)
            })
    }

    fn poll_state(&self) -> u8 {
        level_into_u8(self.pin_a.get_level()) | level_into_u8(self.pin_b.get_level()) << 1
    }
}
