use esp_idf_svc::hal::gpio::{Input, Pin, PinDriver};
use std::time::Duration;
use crate::level_into_u8;

enum LatchMode {
    FOUR3 = 1, // 4 steps, Latch at position 3 only (compatible to older versions)
    FOUR0 = 2, // 4 steps, Latch at position 0 (reverse wirings)
    TWO3 = 3,  // 2 steps, Latch at position 0 and 3
}

enum Direction {
    CounterClockwise = -1,
    NoRotation = 0,
    Clockwise = 1,
}

pub struct RotaryEncoder<'d, 'l, P1: Pin, P2: Pin> {
    pin_a: PinDriver<'d, P1, Input>,
    pin_b: PinDriver<'l, P2, Input>,

    mode: LatchMode,
    previous_state: u8,

    position_int: f64,
    position_ext: f64,
    position_ext_prev: f64,
    position_ext_time: Duration,
    position_ext_time_prev: Duration,
}

impl<'d, 'l, P1: Pin, P2: Pin> RotaryEncoder<'d, 'l, P1, P2> {
    pub fn new(
        pin_a: PinDriver<'d, P1, Input>, 
        pin_b: PinDriver<'l, P2, Input>,
        mode: LatchMode) {
        let previous_state = level_into_u8(pin_a.get_level()) | level_into_u8(pin_b.get_level()) << 1;
        
        todo!()
    }
}
