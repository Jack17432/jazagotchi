use crate::rotary_encoder::{EncoderData, LatchMode, RotaryEncoder};
use crate::{EventSet, Events};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyInputPin, Input, InterruptType, PinDriver};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;

#[derive(Copy, Clone)]
pub enum REEventSet {
    None = 0,
    PinChanged = 1,
}

impl PartialEq<REEventSet> for REEventSet {
    fn eq(&self, other: &Self) -> bool {
        self.to_bit() == other.to_bit()
    }

    fn ne(&self, other: &Self) -> bool {
        self.to_bit() != other.to_bit()
    }
}

static ROTARY_EVENTS: REEvents = REEvents(AtomicU32::new(0));

impl EventSet for REEventSet {
    fn is_none(&self) -> bool {
        *self == REEventSet::None
    }

    fn to_int(&self) -> u32 {
        *self as u32
    }
    fn to_bit(&self) -> u32 {
        1 << self.to_int()
    }
}

struct REEvents(AtomicU32);

impl Events<REEventSet> for REEvents {
    // TODO!: Turn this into a macro to setup
    fn set(event: REEventSet) {
        let curr_events = ROTARY_EVENTS.0.load(Ordering::Relaxed);
        ROTARY_EVENTS
            .0
            .store(curr_events | 1 << (event.to_int()), Ordering::Relaxed);
    }

    fn is_set(&self, event: REEventSet) -> bool {
        let set_events = ROTARY_EVENTS.0.load(Ordering::Relaxed);
        (set_events & event.to_bit()) != 0
    }

    fn wait_for_any(&self) -> Self {
        while ROTARY_EVENTS.0.load(Ordering::Relaxed) == 0 {
            FreeRtos::delay_ms(1);
        }

        let ret_data = Self(AtomicU32::new(ROTARY_EVENTS.0.load(Ordering::Relaxed)));
        ROTARY_EVENTS.0.store(0, Ordering::Relaxed);

        ret_data
    }
    fn wait_for_all(&self) {
        todo!()
    }
}

impl PartialEq<u32> for REEventSet {
    fn eq(&self, other: &u32) -> bool {
        self.to_int() == *other
    }
}

fn encoder_task(mut encoder: RotaryEncoder) -> ! {
    loop {
        match encoder.restart_isr() {
            Ok(_) => {}
            Err(e) => loop {
                log::error!("Error resetting isr for Rotary Encoder, {}", e);
                FreeRtos::delay_ms(100);
                if encoder.restart_isr().is_ok() {
                    break;
                }
            },
        }

        let _ = ROTARY_EVENTS.wait_for_any();

        encoder.update();

        match ROTARY_ENCODER.write() {
            Ok(mut data) => data.set(&encoder.data),
            Err(err) => log::error!("Failed to gain encoder data write lock, {}", err),
        };
    }
}

static ROTARY_ENCODER: Lazy<RwLock<EncoderData>> =
    Lazy::new(|| RwLock::new(EncoderData::default()));

pub mod rotary_interface {
    use super::ROTARY_ENCODER;

    pub fn get_position() -> Result<i8, String> {
        match ROTARY_ENCODER.read() {
            Ok(data) => Ok(data.position_ext),
            Err(_) => Err(String::from("Failed to gain read lock for rotary encoder")),
        }
    }
}

fn on_pin_trigger() {
    REEvents::set(REEventSet::PinChanged);
}

pub fn rotary_encoder_init(
    mut pin_a: PinDriver<'static, AnyInputPin, Input>,
    mut pin_b: PinDriver<'static, AnyInputPin, Input>,
) {
    pin_a.set_interrupt_type(InterruptType::AnyEdge).unwrap();
    pin_b.set_interrupt_type(InterruptType::AnyEdge).unwrap();

    unsafe {
        pin_a.subscribe(on_pin_trigger).unwrap();
        pin_b.subscribe(on_pin_trigger).unwrap();
    }

    let encoder = RotaryEncoder::new(pin_a, pin_b, LatchMode::TWO3, (-7, 0));

    std::thread::Builder::new()
        .name("encoder_task".into())
        .stack_size(32 * 100)
        .spawn(move || encoder_task(encoder))
        .unwrap();
}
