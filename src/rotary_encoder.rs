use std::cmp::PartialEq;
use crate::{EventSet, level_into_u8, level_to_bool};
use esp_idf_svc::hal::gpio::{Input, Pin, PinDriver};
use std::time::{Duration, SystemTime, SystemTimeError};

// TODO!: Make it work better cuz dosn't work that well.

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

    position: i8,
    position_ext: i8,
    position_ext_prev: i8,
    position_ext_time: SystemTime,
    position_ext_time_prev: SystemTime,
}

#[derive(PartialEq, Copy)]
pub enum REEventSet {
    None = 0,
    PinChanged = 1, 
}

impl REEventSet {
    fn to_int(&self) -> u32 {
        *self as u32
    }
}

impl EventSet for REEventSet {
    fn is_none(&self) -> bool {
        *self == REEventSet::None
    }
}

impl PartialEq<u32> for REEventSet {
    fn eq(&self, other: &u32) -> bool {
        self.to_int() == *other
    }
}

/// positions: [3] 1 0 2 [3] 1 0 2 [3]
/// [3] is the positions where my rotary switch detends
/// ==> right, count up
/// <== left,  count down
const ENCODER_DIRECTION: [i8; 4 * 4] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];

impl<'d, 'l, P1: Pin, P2: Pin> RotaryEncoder<'d, 'l, P1, P2> {
    fn poll_state(&self) -> u8 {
        level_into_u8(self.pin_a.get_level()) | level_into_u8(self.pin_b.get_level()) << 1
    }

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

            position: 0i8,
            position_ext: 0i8,
            position_ext_prev: 0i8,
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

        self.position += ENCODER_DIRECTION[(self.prev_state | (curr_state << 2)) as usize];
        self.prev_state = curr_state;

        self.position_ext_time_prev = self.position_ext_time;
        self.position_ext_time = SystemTime::now();

        match &self.mode {
            LatchMode::FOUR0 | LatchMode::FOUR3 => self.position_ext = self.position >> 2,
            LatchMode::TWO3 => self.position_ext = self.position >> 1,
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

    pub fn get_rpm(&self) -> f64 {
        let milli_sec = self.get_duration().as_millis() as f64;
        60_000f64 / (milli_sec * 20f64)
    }

    pub fn get_position(&self) -> i8 {
        self.position_ext
    }

    pub fn get_direction(&self) -> Direction {
        let mut result = Direction::NoRotation;

        if self.position_ext_prev > self.position_ext {
            result = Direction::Clockwise;
        } else if self.position_ext_prev < self.position_ext {
            result = Direction::CounterClockwise;
        }

        result
    }

    pub fn get_pin_state(&self) -> (bool, bool) {
        (
            level_to_bool(self.pin_a.get_level()),
            level_to_bool(self.pin_b.get_level()),
        )
    }

    pub fn set_position(&mut self, new_position: i8) {
        match self.mode {
            LatchMode::FOUR0 | LatchMode::FOUR3 => {
                self.position = (new_position << 2) | (self.position & 0x03)
            }
            LatchMode::TWO3 => self.position = (new_position << 1) | (self.position & 0x01),
        };

        self.position_ext = new_position;
        self.position_ext_prev = new_position;
    }
}

pub fn encoder_events<P1: Pin, P2: Pin>(_: RotaryEncoder<P1, P2>) {}
