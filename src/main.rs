#[allow(dead_code)]
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::{InputPin, Pull};
use esp_idf_svc::hal::gpio::{InterruptType, OutputPin};
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use jazagotchi::apa102::interface::{led_init, LEDInterface};
use jazagotchi::apa102::{Brightness, LEDState, APA102};
use jazagotchi::button_interface::{button_init, ButtonInterface};
use jazagotchi::device::{DevicePowerState, PowerToggle};
use jazagotchi::rotary_encoder;
use jazagotchi::rotary_encoder::interface::ROTARY_ENCODER;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    {
        let pin_a = PinDriver::input(peripherals.pins.gpio2.downgrade_input()).unwrap();
        let pin_b = PinDriver::input(peripherals.pins.gpio1.downgrade_input()).unwrap();
        rotary_encoder::interface::init_rotary_encoder(pin_a, pin_b);
    }

    {
        let button = PinDriver::input(peripherals.pins.gpio0).unwrap();
        button_init(button);
    }

    {
        let spi_clk = PinDriver::output(peripherals.pins.gpio45.downgrade_output()).unwrap();
        let spi_do = PinDriver::output(peripherals.pins.gpio42.downgrade_output()).unwrap();
        led_init(spi_clk, spi_do);
    }

    let pwr_pin = PinDriver::output(peripherals.pins.gpio46).unwrap();
    let mut power_controller = DevicePowerState::new(pwr_pin).unwrap();
    power_controller.wake().unwrap();

    loop {
        led_circle_thingy();

        FreeRtos::delay_ms(10);
    }
}

fn led_circle_thingy() {
    let mut vec: Vec<LEDState> = vec![];

    let val = match ROTARY_ENCODER.read() {
        Ok(data) => -data.get_position(),
        Err(err) => {
            log::error!("Failed to gain read lock for encoder data, {}", err);
            0
        }
    };
    
    let state = ButtonInterface::get_toggle_state();
    
    for _ in 0..val {
        vec.push(LEDState {
            brightness: Brightness::MAX,
            blue: if state { 100 } else { 0 },
            red: if state { 0 } else { 50 },
            green: if state { 0 } else { 50 },
        });
    }
    for _ in val..7 {
        vec.push(LEDState {
            brightness: Brightness::MAX,
            blue: 0,
            red: if state { 100 } else { 0 },
            green: if state { 0 } else { 100 },
        });
    }

    match LEDInterface::set_led_vec(vec) {
        Ok(_) => {},
        Err(err) => log::error!("{}", err),
    };
}
