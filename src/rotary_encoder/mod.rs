pub mod interface;

use super::{level_into_u8, level_to_bool};
use esp_idf_svc::hal::gpio::{AnyInputPin, Input, Pin, PinDriver};
use esp_idf_svc::sys::EspError;
use std::time::{Duration, SystemTime};

#[derive(Copy, Clone, Default)]
pub enum LatchMode {
    #[default]
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

#[derive(Copy, Clone)]
pub struct EncoderData {
    mode: LatchMode,

    range: (i8, i8),
    position: i8,
    position_ext: i8,
    position_ext_prev: i8,
    position_ext_time: SystemTime,
    position_ext_time_prev: SystemTime,
}

impl EncoderData {
    pub(super) fn set(&mut self, other: &EncoderData) {
        *self = other.clone();
    }
}

impl Default for EncoderData {
    fn default() -> Self {
        Self {
            mode: Default::default(),

            range: (-100, 100),
            position: Default::default(),
            position_ext: Default::default(),
            position_ext_prev: Default::default(),
            position_ext_time: SystemTime::now(),
            position_ext_time_prev: SystemTime::now(),
        }
    }
}

pub(self) struct RotaryEncoder {
    pin_a: PinDriver<'static, AnyInputPin, Input>,
    pin_b: PinDriver<'static, AnyInputPin, Input>,
    prev_state: u8,

    data: EncoderData,
}

/// positions: [3] 1 0 2 [3] 1 0 2 [3]
/// positions pins: [11] 01 00 10 [11] 01 00 10 [11]
/// [3] is the positions where my rotary switch detends
/// ==> right, count up
/// <== left,  count down
const ENCODER_DIRECTION: [i8; 4 * 4] = [
    0, -1, 1, 0, 
    1, 0, 0, -1, 
    -1, 0, 0, 1, 
    0, 1, -1, 0];

impl RotaryEncoder {
    fn poll_state(&self) -> u8 {
        let val = level_into_u8(self.pin_a.get_level()) | level_into_u8(self.pin_b.get_level()) << 1;
        val
    }

    fn new(
        pin_a: PinDriver<'static, AnyInputPin, Input>,
        pin_b: PinDriver<'static, AnyInputPin, Input>,
        mode: LatchMode,
        range: (i8, i8),
    ) -> Self {
        let mut encoder = Self {
            pin_a,
            pin_b,
            prev_state: 0,

            data: EncoderData {
                mode,
                range,
                position: (range.0 + range.1) / 2i8,
                position_ext: 0i8,
                position_ext_prev: 0i8,
                position_ext_time: SystemTime::now(),
                position_ext_time_prev: SystemTime::now(),
            },
        };
        encoder.prev_state = encoder.poll_state();

        encoder
    }

    fn update(&mut self) {
        let curr_state = self.poll_state();

        if self.prev_state == curr_state {
            return;
        }

        self.data.position += ENCODER_DIRECTION[(self.prev_state | (curr_state << 2)) as usize];
        
        if self.data.range.0 > self.data.position >> 1 {
            self.data.position = self.data.range.1 << 2;
        }
        else if self.data.range.1 < self.data.position >> 2 {
            self.data.position = self.data.range.0 << 1;
        }

        self.prev_state = curr_state;

        self.data.position_ext_time_prev = self.data.position_ext_time;
        self.data.position_ext_time = SystemTime::now();

        match &self.data.mode {
            LatchMode::FOUR0 | LatchMode::FOUR3 => self.data.position_ext = self.data.position >> 2,
            LatchMode::TWO3 => self.data.position_ext = self.data.position >> 1,
        };
    }

    fn get_pin_state(&self) -> (bool, bool) {
        (
            level_to_bool(self.pin_a.get_level()),
            level_to_bool(self.pin_b.get_level()),
        )
    }

    fn restart_isr(&mut self) -> Result<(), EspError> {
        self.pin_a.enable_interrupt()?;
        self.pin_b.enable_interrupt()?;

        Ok(())
    }
}

impl EncoderData {
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
}
