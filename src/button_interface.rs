use crate::{level_to_bool, EventSet, Events};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{Gpio0, Input, InterruptType, PinDriver, Pull};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{RwLock, RwLockWriteGuard};

#[derive(Copy, Clone)]
pub enum ButtonEventSet {
    None = 0,
    ButtonChange = 1,
}

impl EventSet for ButtonEventSet {
    fn is_none(&self) -> bool {
        *self == ButtonEventSet::None
    }

    fn to_int(&self) -> u32 {
        *self as u32
    }
    fn to_bit(&self) -> u32 {
        1 << self.to_int()
    }
}

impl PartialEq<ButtonEventSet> for ButtonEventSet {
    fn eq(&self, other: &Self) -> bool {
        self.to_bit() == other.to_bit()
    }

    fn ne(&self, other: &Self) -> bool {
        self.to_bit() != other.to_bit()
    }
}

impl PartialEq<u32> for ButtonEventSet {
    fn eq(&self, other: &u32) -> bool {
        self.to_int() == *other
    }
}

struct ButtonEvents(AtomicU32);

static BUTTON_EVENTS: ButtonEvents = ButtonEvents(AtomicU32::new(0));

impl Events<ButtonEventSet> for ButtonEvents {
    fn set(event: ButtonEventSet) {
        let curr_events = BUTTON_EVENTS.0.load(Ordering::Relaxed);
        BUTTON_EVENTS
            .0
            .store(curr_events | 1 << (event.to_int()), Ordering::Relaxed);
    }

    fn is_set(&self, event: ButtonEventSet) -> bool {
        let set_events = BUTTON_EVENTS.0.load(Ordering::Relaxed);
        (set_events & event.to_bit()) != 0
    }

    fn wait_for_any(&self) -> Self {
        while BUTTON_EVENTS.0.load(Ordering::Relaxed) == 0 {
            FreeRtos::delay_ms(1);
        }

        let ret_data = Self(AtomicU32::new(BUTTON_EVENTS.0.load(Ordering::Relaxed)));
        BUTTON_EVENTS.0.store(0, Ordering::Relaxed);

        ret_data
    }
    fn wait_for_all(&self) {
        todo!()
    }
}

pub struct ButtonInterface {
    button_state: AtomicBool,
    toggle_state: AtomicBool,
    
    _has_been_low: bool,
}

static BUTTON_INTERFACE: Lazy<RwLock<ButtonInterface>> = Lazy::new(|| RwLock::new(ButtonInterface::new()));

impl ButtonInterface {
    fn new() -> Self {
        Self {
            button_state: AtomicBool::new(false),
            toggle_state: AtomicBool::new(false),
            _has_been_low: false
        }
    }
    
    fn update_button(button_state: bool) {
        let interface = BUTTON_INTERFACE.write().expect("Failed to gain write lock on led update");
        interface.button_state.store(button_state, Ordering::Relaxed); 
        
        Self::update_toggle(interface);
    } 
    
    fn update_toggle(mut interface: RwLockWriteGuard<ButtonInterface>) {
        if interface.button_state.load(Ordering::Relaxed) && interface._has_been_low {
            interface._has_been_low = false;
            interface.toggle_state.fetch_xor(true, Ordering::Relaxed);
        } 
        else if !interface.button_state.load(Ordering::Relaxed) {
            interface._has_been_low = true;
        }
    }
    
    pub fn get_toggle_state() -> bool {
        let interface = BUTTON_INTERFACE.read().expect("Failed to gain read for button interface");
        interface.toggle_state.load(Ordering::Relaxed)
    }
}


fn button_task(mut button: PinDriver<'static, Gpio0, Input>) -> ! {
    loop {
        match button.enable_interrupt() {
            Ok(_) => {}
            Err(e) => loop {
                log::error!("Error resetting isr for Button, {}", e);
                FreeRtos::delay_ms(100);
                if button.enable_interrupt().is_ok() {
                    break;
                }
            },
        }

        let _ = BUTTON_EVENTS.wait_for_any();

        ButtonInterface::update_button(button.is_high());
    }
}

pub fn button_init(mut button: PinDriver<'static, Gpio0, Input>) {
    button.set_pull(Pull::Up).unwrap();
    button
        .set_interrupt_type(InterruptType::AnyEdge)
        .expect("Failed to set InterruptType");

    unsafe {
        button.subscribe(button_callback).unwrap();
    }

    std::thread::Builder::new()
        .name("button_task".into())
        .stack_size(32 * 60)
        .spawn(move || button_task(button))
        .unwrap();
}

fn button_callback() {
    ButtonEvents::set(ButtonEventSet::ButtonChange);
}
