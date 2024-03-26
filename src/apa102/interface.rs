use crate::apa102::{LEDState, APA102};
use crate::{EventSet, Events};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyOutputPin, Output, PinDriver};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;

#[derive(Copy, Clone)]
pub enum LEDEventSet {
    None = 0,
    UpdateLed = 1,
}

impl PartialEq<LEDEventSet> for LEDEventSet {
    fn eq(&self, other: &Self) -> bool {
        self.to_bit() == other.to_bit()
    }

    fn ne(&self, other: &Self) -> bool {
        self.to_bit() != other.to_bit()
    }
}

impl EventSet for LEDEventSet {
    fn is_none(&self) -> bool {
        *self == LEDEventSet::None
    }

    fn to_int(&self) -> u32 {
        *self as u32
    }
    fn to_bit(&self) -> u32 {
        1 << self.to_int()
    }
}

struct LEDEvents(AtomicU32);
static LED_EVENTS: LEDEvents = LEDEvents(AtomicU32::new(0));

impl Events<LEDEventSet> for LEDEvents {
    fn set(event: LEDEventSet) {
        let curr_events = LED_EVENTS.0.load(Ordering::Relaxed);
        LED_EVENTS
            .0
            .store(curr_events | 1 << (event.to_int()), Ordering::Relaxed);
    }

    fn is_set(&self, event: LEDEventSet) -> bool {
        let set_events = LED_EVENTS.0.load(Ordering::Relaxed);
        (set_events & event.to_bit()) != 0
    }

    fn wait_for_any(&self) -> Self {
        while LED_EVENTS.0.load(Ordering::Relaxed) == 0 {
            FreeRtos::delay_ms(1);
        }

        let ret_data = Self(AtomicU32::new(LED_EVENTS.0.load(Ordering::Relaxed)));
        LED_EVENTS.0.store(0, Ordering::Relaxed);

        ret_data
    }

    fn wait_for_all(&self) {
        todo!()
    }
}

pub struct LEDInterface(Vec<LEDState>);

static REQUESTED_LED_STATE: Lazy<RwLock<Vec<LEDState>>> = Lazy::new(|| RwLock::new(vec![]));

impl LEDInterface {
    fn init(mut led: Vec<LEDState>) {
        let mut led_data = REQUESTED_LED_STATE.write().expect("Unable it init led vec");
        led_data.clear();
        led_data.append(&mut led);
    }

    pub fn set_led(led: LEDState, position: u8) -> Result<(), String> {
        match REQUESTED_LED_STATE.write() {
            Ok(mut led_data) => {
                if led_data.len() < position as usize {
                    return Err(String::from("Led requested is out of range"));
                };

                led_data.insert(position as usize, led);
                LEDEvents::set(LEDEventSet::UpdateLed);
                Ok(())
            }
            Err(_) => Err(String::from("Failed to gain led write lock")),
        }
    }

    pub fn set_led_vec(mut led: Vec<LEDState>) -> Result<(), String> {
        match REQUESTED_LED_STATE.write() {
            Ok(mut led_data) => {
                if led_data.len() < led.len() {
                    return Err(String::from(
                        "Led vec requested larger then available led's",
                    ));
                };

                led_data.clear();
                led_data.append(&mut led);
                LEDEvents::set(LEDEventSet::UpdateLed);
                Ok(())
            }
            Err(_) => Err(String::from("Failed to gain led write lock")),
        }
    }
}

fn led_task(mut apa: APA102) -> ! {
    loop {
        let _ = LED_EVENTS.wait_for_any();

        let requested_led_state = REQUESTED_LED_STATE
            .read()
            .expect("Main task failed to gain read access to requested led");
        apa.set_led_array(requested_led_state.clone())
            .expect("Failed to set led on apa");
    }
}

pub fn led_init(
    spi_clk: PinDriver<'static, AnyOutputPin, Output>,
    spi_do: PinDriver<'static, AnyOutputPin, Output>,
) {
    let apa = APA102::new(7, spi_clk, spi_do);
    LEDInterface::init(apa.led_states.clone());

    std::thread::Builder::new()
        .name("led_task".into())
        .stack_size(32 * 100)
        .spawn(move || led_task(apa))
        .unwrap();
}
